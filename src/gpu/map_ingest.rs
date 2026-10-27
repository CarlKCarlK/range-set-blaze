//! [`RangeMapBlaze::from_shuffled_slices`]: a map from key and value columns whose keys are in no
//! useful order, built on several CPU threads or on the GPU.
//!
//! Both engines produce the same thing: each range's start and end keys, and the input position
//! of its first surviving pair. (A pair survives if it is the last pair with its key, which is
//! the precedence `from_iter` and `from_slices` use.) Each range's value is
//! `values[position]`, so no engine ever decodes a value.
//!
//! Type dispatch is by `castaway`, which works for any type, including borrowed ones:
//!
//! - Keys of the crate's types of at most 32 bits become order-preserving `u32` words. The CPU
//!   packs them with their positions into `u64`s, and the GPU sorts them. Any other key type
//!   runs on the CPU, sorting `(key, position)` pairs.
//! - Values of the crate's types of at most 64 bits become one or two `u32` words, used only to
//!   test equality, on any thread or on the GPU. Any other value type runs on the CPU, comparing
//!   values with `==` on one thread (so `V` needn't be `Sync`).

use core::ops::Range;
use std::borrow::Cow;
use std::vec::Vec;

use lampshade::{Compactor, Context, KeyValueSoaSorter};
use rayon::ThreadPool;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::slice::{ParallelSlice, ParallelSliceMut};

use super::{
    BoundaryBuffers, BoundaryKernel, Engine, GpuElement, all_threads, buffers_fit, encoder,
    gpu_context, groups_x, high_word, low_word, map_u32, read_endpoints_32, readback_buffer,
    resource_or_panic, slice_storage_buffer, storage_buffer, submit, thread_pool,
    uses_mappable_primary_buffers, with_error_scopes,
};
use crate::{CheckSortedDisjointMap, Integer, NotNanF32, NotNanF64, RangeMapBlaze};

// SAFETY: `NotNan<T>` holds only a `T`, and `f32` and `f64` have no lifetime parameters.
unsafe impl castaway::LifetimeFree for NotNanF32 {}
// SAFETY: as above.
unsafe impl castaway::LifetimeFree for NotNanF64 {}

impl<T: Integer, V: Eq + Clone> RangeMapBlaze<T, V> {
    /// Creates a [`RangeMapBlaze`] from a slice of keys and a slice of their values, using
    /// several CPU threads or the GPU, as `engine` says. Where a key repeats, the last value
    /// wins, as with [`RangeMapBlaze::from_slices`].
    ///
    /// *Experimental; requires the `shuffled_slices` feature.* It pays off for large slices
    /// whose keys are in no useful order, such as shuffled data. If nearby keys are already next
    /// to each other (sorted, or clumpy data in order), [`RangeMapBlaze::from_slices`] may be
    /// faster.
    ///
    /// Any key and value types work with [`Engine::Cpu`] and [`Engine::WgpuOrCpu`]. The GPU
    /// handles keys of type `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, `char`, or `NotNanF32`, with
    /// values of those types or of `u64`, `i64`, `usize`, `isize`, or `NotNanF64`. For other types,
    /// [`Engine::WgpuOrCpu`] uses the CPU and [`Engine::Wgpu`] panics. On the CPU, values of
    /// other types are compared on one thread, so they needn't be [`Sync`].
    ///
    /// # Panics
    ///
    /// Panics if `keys` and `values` have different lengths, and as [`Engine::Wgpu`] says.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::{Engine, prelude::*};
    ///
    /// let keys = [3_u32, 1, 2, 9, 2];
    /// let values = ["c", "a", "x", "z", "a"];
    /// let map = RangeMapBlaze::from_shuffled_slices(&keys, &values, Engine::WgpuOrCpu);
    /// assert_eq!(map.to_string(), r#"(1..=2, "a"), (3..=3, "c"), (9..=9, "z")"#);
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "shuffled_slices")))]
    #[must_use]
    pub fn from_shuffled_slices(keys: &[T], values: &[V], engine: Engine) -> Self {
        from_shuffled_slices(keys, values, engine)
    }
}

fn from_shuffled_slices<K: Integer, V: Eq + Clone>(
    keys: &[K],
    values: &[V],
    engine: Engine,
) -> RangeMapBlaze<K, V> {
    assert_eq!(
        keys.len(),
        values.len(),
        "keys and values must have the same length"
    );
    if keys.is_empty() {
        return RangeMapBlaze::new();
    }
    // Positions are `u32`s. Beyond that, `from_slices` (rather than Engine::Wgpu's panic).
    if u32::try_from(keys.len() - 1).is_err() {
        return RangeMapBlaze::from_slices(keys, values);
    }
    let threads = match engine {
        Engine::Cpu(threads) => threads,
        Engine::Wgpu | Engine::WgpuOrCpu => all_threads(),
        #[cfg(feature = "cuda")]
        Engine::Cuda | Engine::CudaOrCpu => all_threads(),
        #[cfg(feature = "cutile")]
        Engine::CuTile => all_threads(),
    };
    // Only work that needs no `V: Sync` runs on the pool: keys, and values once they are words.
    let pool = thread_pool(threads);
    let key_words = key_words(&pool, keys);
    let value_words = value_words(&pool, values);
    let found = match engine {
        Engine::Cpu(_) => ranges_cpu(
            &pool,
            keys,
            values,
            key_words.as_ref(),
            value_words.as_ref(),
        ),
        Engine::Wgpu => {
            let (Some(key_words), Some(value_words)) = (&key_words, &value_words) else {
                panic!("Engine::Wgpu: the key or value type has no GPU representation");
            };
            let context = gpu_context().expect("Engine::Wgpu: no hardware GPU is available");
            ranges_gpu(context, key_words, value_words)
                .expect("Engine::Wgpu: the input exceeds the GPU's limits or memory")
        }
        Engine::WgpuOrCpu => key_words
            .as_ref()
            .zip(value_words.as_ref())
            .zip(gpu_context())
            .and_then(|((key_words, value_words), context)| {
                ranges_gpu(context, key_words, value_words)
            })
            .unwrap_or_else(|| {
                ranges_cpu(
                    &pool,
                    keys,
                    values,
                    key_words.as_ref(),
                    value_words.as_ref(),
                )
            }),
        #[cfg(feature = "cuda")]
        Engine::Cuda => ranges_cuda(key_words.as_ref(), value_words.as_ref())
            .unwrap_or_else(|unavailable| panic!("Engine::Cuda: {unavailable}")),
        #[cfg(feature = "cuda")]
        Engine::CudaOrCpu => {
            ranges_cuda(key_words.as_ref(), value_words.as_ref()).unwrap_or_else(|_| {
                ranges_cpu(
                    &pool,
                    keys,
                    values,
                    key_words.as_ref(),
                    value_words.as_ref(),
                )
            })
        }
        #[cfg(feature = "cutile")]
        Engine::CuTile => {
            let (Some(key_words), Some(value_words)) = (&key_words, &value_words) else {
                panic!("Engine::CuTile: the key or value type has no GPU representation");
            };
            ranges_cutile(&pool, key_words, value_words)
        }
    };
    build_map(&pool, values, found)
}

/// The ranges, from CUDA, if the key and value types have words and CUDA can run.
#[cfg(feature = "cuda")]
fn ranges_cuda<K>(
    key_words: Option<&KeyWords<'_>>,
    value_words: Option<&ValueWords<'_>>,
) -> Result<Found<K>, super::CudaUnavailable> {
    let (Some(key_words), Some(value_words)) = (key_words, value_words) else {
        return Err(super::CudaUnavailable::UnsupportedType);
    };
    let cuda = super::cuda::cuda_context().ok_or(super::CudaUnavailable::NoGpu)?;
    let (bounds, firsts) = super::cuda::map_ranges(
        cuda,
        &key_words.words,
        key_words.gap,
        &value_words.lo,
        value_words.hi.as_deref(),
    )
    .ok_or(super::CudaUnavailable::OutOfMemory)?;
    Ok(Found::Words(Ranges {
        bounds,
        firsts: firsts.into_iter().map(|first| first as usize).collect(),
    }))
}

#[cfg(feature = "cutile")]
fn ranges_cutile<K>(
    pool: &ThreadPool,
    key_words: &KeyWords<'_>,
    value_words: &ValueWords<'_>,
) -> Found<K> {
    // cuTile uploads from owned vectors. Copying 100M keys and values on one thread took about
    // 300 ms (mostly page faults on the fresh memory), on every thread about 60 ms.
    let (keys, value_lo) = pool.install(|| {
        (
            key_words.words.par_iter().copied().collect(),
            value_words.lo.par_iter().copied().collect(),
        )
    });
    let (bounds, firsts) =
        super::cutile::map_ranges(keys, key_words.gap, value_lo, value_words.hi.clone())
            .unwrap_or_else(|error| panic!("Engine::CuTile: {error}"));
    Found::Words(Ranges {
        bounds,
        firsts: firsts.into_iter().map(|first| first as usize).collect(),
    })
}

/// The map from the engine's ranges.
fn build_map<K: Integer, V: Eq + Clone>(
    pool: &ThreadPool,
    values: &[V],
    found: Found<K>,
) -> RangeMapBlaze<K, V> {
    let Ranges { bounds, firsts } = match found {
        Found::Keys(ranges) => ranges,
        Found::Words(Ranges { bounds, firsts }) => Ranges {
            bounds: decode_bounds(pool, &bounds),
            firsts,
        },
    };
    // The values are scattered through the input, so looking them up is much of the work when
    // there are many ranges: do it on every thread where the type allows.
    let range_values = gather_values(pool, values, &firsts);
    RangeMapBlaze::from_sorted_disjoint_map(CheckSortedDisjointMap::new(
        bounds
            .into_iter()
            .zip(&range_values)
            .map(|((start, end), value)| (start..=end, value)),
    ))
}

/// The map's ranges in key order: each one's start and end keys, and the input position of its
/// first surviving pair, whose value is the range's.
struct Ranges<Key> {
    bounds: Vec<(Key, Key)>,
    firsts: Vec<usize>,
}

/// The ranges, with their keys as the keys themselves or as their `u32` words.
enum Found<K> {
    Keys(Ranges<K>),
    Words(Ranges<u32>),
}

/// The keys of these words. `K` must be one of the types [`key_words`] makes words of.
fn decode_bounds<K>(pool: &ThreadPool, words: &[(u32, u32)]) -> Vec<(K, K)> {
    let mut bounds: Vec<(K, K)> = Vec::new();
    macro_rules! decode_as {
        ($($type:ty),+) => {$(
            if let Ok(typed) = castaway::cast!(&mut bounds, &mut Vec<($type, $type)>) {
                let key = |word: u32| <$type as GpuElement>::decode(u64::from(word));
                *typed = pool.install(|| {
                    words.par_iter().map(|&(start, end)| (key(start), key(end))).collect()
                });
                return bounds;
            }
        )+};
    }
    decode_as!(u32, u8, u16, i8, i16, i32, char, NotNanF32);
    unreachable!("only key_words's types have key words")
}

/// `values[position]` for each of `positions`: on every thread if `V` is one of the crate's
/// integer types (so certainly `Sync`), and otherwise on this thread.
fn gather_values<V: Clone>(pool: &ThreadPool, values: &[V], positions: &[usize]) -> Vec<V> {
    let mut gathered: Vec<V> = Vec::new();
    macro_rules! gather_as {
        ($($type:ty),+) => {$(
            if let Ok(values) = castaway::cast!(values, &[$type]) {
                // `V` is `$type`, so this cast succeeds too.
                if let Ok(typed) = castaway::cast!(&mut gathered, &mut Vec<$type>) {
                    *typed = pool.install(|| {
                        positions.par_iter().map(|&position| values[position]).collect()
                    });
                }
                return gathered;
            }
        )+};
    }
    gather_as!(
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, char, NotNanF32, NotNanF64
    );
    positions
        .iter()
        .map(|&position| values[position].clone())
        .collect()
}

/// Each key as an order-preserving `u32` word, with the one place (if any) where the next key
/// after `left` is `right` rather than `left + 1`.
struct KeyWords<'a> {
    words: Cow<'a, [u32]>,
    gap: Option<(u32, u32)>,
}

/// `None` if `K` isn't one of the crate's key types of at most 32 bits.
fn key_words<'a, K>(pool: &ThreadPool, keys: &'a [K]) -> Option<KeyWords<'a>> {
    if let Ok(words) = castaway::cast!(keys, &[u32]) {
        return Some(KeyWords {
            words: Cow::Borrowed(words),
            gap: None,
        });
    }
    macro_rules! encode_as {
        ($($type:ty),+) => {$(
            if let Ok(keys) = castaway::cast!(keys, &[$type]) {
                let word = |key: &$type| u32::try_from(key.encode()).expect("32-bit GPU key");
                return Some(KeyWords {
                    words: Cow::Owned(pool.install(|| keys.par_iter().map(word).collect())),
                    gap: <$type as GpuElement>::GAP.map(|(left, right)| {
                        (
                            u32::try_from(left).expect("32-bit gap"),
                            u32::try_from(right).expect("32-bit gap"),
                        )
                    }),
                });
            }
        )+};
    }
    // Keep in step with `decode_bounds`.
    encode_as!(u8, u16, i8, i16, i32, char, NotNanF32);
    None
}

/// Each value as one or two `u32` words: equal words exactly when the values are equal.
struct ValueWords<'a> {
    lo: Cow<'a, [u32]>,
    hi: Option<Vec<u32>>,
}

impl ValueWords<'_> {
    fn same(&self, left: usize, right: usize) -> bool {
        self.lo[left] == self.lo[right] && self.hi.as_ref().is_none_or(|hi| hi[left] == hi[right])
    }
}

/// `None` if `V` isn't one of the crate's integer types of at most 64 bits.
fn value_words<'a, V>(pool: &ThreadPool, values: &'a [V]) -> Option<ValueWords<'a>> {
    if let Ok(words) = castaway::cast!(values, &[u32]) {
        return Some(ValueWords {
            lo: Cow::Borrowed(words),
            hi: None,
        });
    }
    macro_rules! encode_as {
        ($($type:ty),+) => {$(
            if let Ok(values) = castaway::cast!(values, &[$type]) {
                return Some(pool.install(|| {
                    if <$type as GpuElement>::KEY_WORDS == 1 {
                        ValueWords {
                            lo: Cow::Owned(
                                values.par_iter().map(|value| low_word(value.encode())).collect(),
                            ),
                            hi: None,
                        }
                    } else {
                        let (lo, hi) = values
                            .par_iter()
                            .map(|value| {
                                let bits = value.encode();
                                (low_word(bits), high_word(bits))
                            })
                            .unzip();
                        ValueWords {
                            lo: Cow::Owned(lo),
                            hi: Some(hi),
                        }
                    }
                }));
            }
        )+};
    }
    encode_as!(
        u8, u16, u64, usize, i8, i16, i32, i64, isize, char, NotNanF32, NotNanF64
    );
    None
}

/// The ranges, on `pool`'s threads. Values without words are compared on this thread.
fn ranges_cpu<K: Integer, V: Eq>(
    pool: &ThreadPool,
    keys: &[K],
    values: &[V],
    key_words: Option<&KeyWords<'_>>,
    value_words: Option<&ValueWords<'_>>,
) -> Found<K> {
    fn run<S: Sorted, V: Eq>(
        pool: &ThreadPool,
        sorted: &S,
        values: &[V],
        value_words: Option<&ValueWords<'_>>,
    ) -> Ranges<S::Key> {
        let ranges = value_words.map_or_else(
            || ranges_sequential(sorted, |left, right| values[left] == values[right]),
            |value_words| {
                pool.install(|| {
                    ranges_parallel(sorted, |left, right| value_words.same(left, right))
                })
            },
        );
        // The entries are in key order, so these reads are too.
        let (bounds, firsts) = pool.install(|| {
            ranges
                .par_iter()
                .map(|&(first, last)| {
                    (
                        (sorted.key(first), sorted.key(last)),
                        sorted.position(first),
                    )
                })
                .unzip()
        });
        Ranges { bounds, firsts }
    }
    key_words.map_or_else(
        || {
            Found::Keys(run(
                pool,
                &pool.install(|| Pairs::new(keys)),
                values,
                value_words,
            ))
        },
        |key_words| {
            Found::Words(run(
                pool,
                &pool.install(|| Packed::new(key_words)),
                values,
                value_words,
            ))
        },
    )
}

/// Sorted (key, position) entries, keys ascending and, within a key, positions ascending.
trait Sorted: Sync {
    type Key: Copy + Send + Sync;
    fn len(&self) -> usize;
    fn key(&self, index: usize) -> Self::Key;
    fn position(&self, index: usize) -> usize;
    /// Whether entries `index` and `index + 1` have the same key.
    fn same_key_as_next(&self, index: usize) -> bool;
    /// Whether the key of entry `right` comes right after the key of entry `left`.
    fn adjacent(&self, left: usize, right: usize) -> bool;
}

/// `u32` key words packed with positions as `key << 32 | position`. Each packed value is
/// distinct, so an unstable sort still keeps each key's entries in input order.
struct Packed {
    entries: Vec<u64>,
    gap: Option<(u32, u32)>,
}

impl Packed {
    fn new(key_words: &KeyWords<'_>) -> Self {
        let mut entries: Vec<u64> = key_words
            .words
            .par_iter()
            .enumerate()
            .map(|(position, &word)| u64::from(word) << 32 | position as u64)
            .collect();
        entries.par_sort_unstable();
        Self {
            entries,
            gap: key_words.gap,
        }
    }
}

impl Sorted for Packed {
    type Key = u32;

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn key(&self, index: usize) -> u32 {
        high_word(self.entries[index])
    }

    fn position(&self, index: usize) -> usize {
        low_word(self.entries[index]) as usize
    }

    fn same_key_as_next(&self, index: usize) -> bool {
        self.key(index) == self.key(index + 1)
    }

    fn adjacent(&self, left: usize, right: usize) -> bool {
        let (left, right) = (self.key(left), self.key(right));
        left.checked_add(1) == Some(right) || self.gap == Some((left, right))
    }
}

/// `(key, position)` pairs, for key types with no `u32` word.
struct Pairs<K>(Vec<(K, u32)>);

impl<K: Integer> Pairs<K> {
    fn new(keys: &[K]) -> Self {
        let mut pairs: Vec<(K, u32)> = keys
            .par_iter()
            .enumerate()
            .map(|(position, &key)| (key, u32::try_from(position).expect("positions fit in u32")))
            .collect();
        pairs.par_sort_unstable();
        Self(pairs)
    }
}

impl<K: Integer> Sorted for Pairs<K> {
    type Key = K;

    fn len(&self) -> usize {
        self.0.len()
    }

    fn key(&self, index: usize) -> K {
        self.0[index].0
    }

    fn position(&self, index: usize) -> usize {
        self.0[index].1 as usize
    }

    fn same_key_as_next(&self, index: usize) -> bool {
        self.0[index].0 == self.0[index + 1].0
    }

    fn adjacent(&self, left: usize, right: usize) -> bool {
        self.0[left].0.checked_add_one() == Some(self.0[right].0)
    }
}

/// The ranges among the sorted entries, as (first, last) entry indices, found in parallel
/// chunks. `same_value` compares the values at two input positions.
fn ranges_parallel(
    sorted: &impl Sorted,
    same_value: impl Fn(usize, usize) -> bool + Sync,
) -> Vec<(usize, usize)> {
    let len = sorted.len();
    // Chunk boundaries move forward to key boundaries, so no key spans two chunks.
    let chunk_count = (rayon::current_num_threads() * 4).min(len);
    let mut bounds = std::vec![0];
    for chunk in 1..chunk_count {
        let previous = *bounds.last().expect("bounds start with 0");
        let mut bound = (chunk * len / chunk_count).max(previous);
        while bound > 0 && bound < len && sorted.same_key_as_next(bound - 1) {
            bound += 1;
        }
        bounds.push(bound);
    }
    bounds.push(len);
    let chunks: Vec<Vec<(usize, usize)>> = bounds
        .par_windows(2)
        .map(|window| chunk_ranges(sorted, window[0]..window[1], &same_value))
        .collect();
    // Join the chunks. Only a chunk's first range can continue the one before it.
    let mut ranges = Vec::with_capacity(chunks.iter().map(Vec::len).sum());
    for chunk in chunks {
        let mut chunk = chunk.into_iter();
        if let Some((first, last)) = chunk.next() {
            extend(sorted, &same_value, &mut ranges, first, last);
        }
        ranges.extend(chunk);
    }
    ranges
}

fn ranges_sequential(
    sorted: &impl Sorted,
    same_value: impl Fn(usize, usize) -> bool,
) -> Vec<(usize, usize)> {
    chunk_ranges(sorted, 0..sorted.len(), &same_value)
}

/// The ranges among the entries in `chunk`, which no key spans out of.
fn chunk_ranges(
    sorted: &impl Sorted,
    chunk: Range<usize>,
    same_value: &impl Fn(usize, usize) -> bool,
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let end = chunk.end;
    for index in chunk {
        // Only the last entry of each key survives.
        if index + 1 < end && sorted.same_key_as_next(index) {
            continue;
        }
        extend(sorted, same_value, &mut ranges, index, index);
    }
    ranges
}

/// Appends range `first..=last` (entry indices), merging it into the previous range if their
/// keys are adjacent and their values equal.
fn extend(
    sorted: &impl Sorted,
    same_value: &impl Fn(usize, usize) -> bool,
    ranges: &mut Vec<(usize, usize)>,
    first: usize,
    last: usize,
) {
    if let Some((previous_first, previous_last)) = ranges.last_mut()
        && sorted.adjacent(*previous_last, first)
        && same_value(sorted.position(*previous_first), sorted.position(first))
    {
        *previous_last = last;
    } else {
        ranges.push((first, last));
    }
}

/// Fills `positions` with `0..len`.
const POSITIONS_SHADER: &str = r"
struct Parameters {
    len: u32,
    groups_x: u32,
    _padding0: u32,
    _padding1: u32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<u32>;
@group(0) @binding(1) var<uniform> parameters: Parameters;

@compute @workgroup_size(256)
fn main(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(local_invocation_id) local: vec3<u32>,
) {
    let index = (workgroup.y * parameters.groups_x + workgroup.x) * 256u + local.x;
    if index < parameters.len {
        positions[index] = index;
    }
}
";

/// Keeps sorted entry `index` if it is the last of its key.
const SURVIVOR_SHADER: &str = r"
struct Parameters {
    len: u32,
    groups_x: u32,
    _padding0: u32,
    _padding1: u32,
}

@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read_write> keep: array<u32>;
@group(0) @binding(2) var<uniform> parameters: Parameters;

@compute @workgroup_size(256)
fn main(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(local_invocation_id) local: vec3<u32>,
) {
    let index = (workgroup.y * parameters.groups_x + workgroup.x) * 256u + local.x;
    let len = parameters.len;
    if index >= len {
        return;
    }
    let last_of_key = index + 1u == len || keys[index + 1u] != keys[index];
    keep[index] = select(0u, 1u, last_of_key);
}
";

/// Marks range starts and ends among the survivors. Their count is read on the GPU, so the
/// survivor compaction never round-trips to the CPU; entries at or past it get no marks.
const MAP_BOUNDARY_SHADER: &str = r"
struct Parameters {
    len: u32,
    groups_x: u32,
    gap_left: u32,
    gap_right: u32,
    two_words: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read> positions: array<u32>;
@group(0) @binding(2) var<storage, read> value_lo: array<u32>;
@group(0) @binding(3) var<storage, read> value_hi: array<u32>;
@group(0) @binding(4) var<storage, read> count: array<u32>;
@group(0) @binding(5) var<storage, read_write> start_mask: array<u32>;
@group(0) @binding(6) var<storage, read_write> end_mask: array<u32>;
@group(0) @binding(7) var<uniform> parameters: Parameters;

// Survivors `left` and `left + 1` share a range. Test `!= MAX` first so `+ 1u` can't overflow.
fn joined(left: u32) -> bool {
    let key = keys[left];
    let next = keys[left + 1u];
    let ordinary = key != 0xffffffffu && next == key + 1u;
    let exceptional = key == parameters.gap_left && next == parameters.gap_right;
    if !(ordinary || exceptional) {
        return false;
    }
    let a = positions[left];
    let b = positions[left + 1u];
    return value_lo[a] == value_lo[b] && (parameters.two_words == 0u || value_hi[a] == value_hi[b]);
}

@compute @workgroup_size(256)
fn main(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(local_invocation_id) local: vec3<u32>,
) {
    let index = (workgroup.y * parameters.groups_x + workgroup.x) * 256u + local.x;
    if index >= parameters.len {
        return;
    }
    let survivors = count[0];
    if index >= survivors {
        start_mask[index] = 0u;
        end_mask[index] = 0u;
        return;
    }
    let is_start = index == 0u || !joined(index - 1u);
    let is_end = index + 1u == survivors || !joined(index);
    start_mask[index] = select(0u, 1u, is_start);
    end_mask[index] = select(0u, 1u, is_end);
}
";

/// The ranges, from the GPU: sort the keys carrying their positions, keep the last of each key,
/// mark where keys stop being adjacent or values change, and compact each range's start and end
/// keys and first position. Only those come back. `None` if the input exceeds the GPU's limits
/// or memory.
fn ranges_gpu<K>(
    context: &Context,
    key_words: &KeyWords<'_>,
    value_words: &ValueWords<'_>,
) -> Option<Found<K>> {
    let len = u32::try_from(key_words.words.len()).ok()?;
    with_error_scopes(context, || {
        let byte_len = u64::from(len) * 4;
        if !buffers_fit(context, byte_len, byte_len * 2) {
            return None;
        }
        let (keys, value_lo, value_hi) = upload(context, key_words, value_words)?;
        let positions = column(context, "GPU map positions", byte_len);
        let survivors = Survivors::new(context, byte_len);
        let buffers = BoundaryBuffers::new(context, byte_len);

        let groups_x = groups_x(context, len)?;
        let (gap_left, gap_right) = key_words.gap.unwrap_or((0, 0));
        let fill_positions = BoundaryKernel::new(
            context,
            POSITIONS_SHADER,
            &[(&positions, false)],
            &[len, groups_x, 0, 0],
        );
        let mark_survivors = BoundaryKernel::new(
            context,
            SURVIVOR_SHADER,
            &[(&keys, true), (&survivors.keep, false)],
            &[len, groups_x, 0, 0],
        );
        let boundaries = survivors.boundary_kernel(
            context,
            (&value_lo, &value_hi),
            &buffers,
            &[
                len,
                groups_x,
                gap_left,
                gap_right,
                u32::from(value_words.hi.is_some()),
                0,
                0,
                0,
            ],
        );
        let mut sorter = KeyValueSoaSorter::from_context(context);
        resource_or_panic(sorter.prepare_sort(&keys, &positions, len))?;
        let mut compactor = Compactor::from_context(context);

        let mut encoder = encoder(context, "GPU map ingestion");
        fill_positions.record(&mut encoder);
        resource_or_panic(sorter.record_reserved_sort(&mut encoder, &keys, &positions, len))?;
        mark_survivors.record(&mut encoder);
        survivors.record_compactions(&mut compactor, &mut encoder, &keys, &positions, len)?;
        boundaries.record(&mut encoder);
        let firsts = storage_buffer(
            context,
            "GPU map first positions",
            byte_len,
            readback_usage(context),
        );
        let first_count = counter(context, "GPU map first position count");
        for (input, mask, output, count) in [
            (
                &survivors.keys,
                &buffers.start_mask,
                &buffers.starts,
                &buffers.start_count,
            ),
            (
                &survivors.keys,
                &buffers.end_mask,
                &buffers.ends,
                &buffers.end_count,
            ),
            (
                &survivors.positions,
                &buffers.start_mask,
                &firsts,
                &first_count,
            ),
        ] {
            resource_or_panic(compactor.record_compact(
                &mut encoder,
                input,
                mask,
                output,
                count,
                len,
            ))?;
        }
        submit(context, encoder.finish())?;
        let bounds = read_endpoints_32(context, &buffers, len)?;
        let firsts = read_words(context, &firsts, bounds.len())?;
        Some(Found::Words(Ranges {
            bounds: bounds
                .into_iter()
                .map(|(start, end)| (low_word(start), low_word(end)))
                .collect(),
            firsts: firsts.into_iter().map(|first| first as usize).collect(),
        }))
    })
}

/// How the GPU's output buffers are read back, as in [`BoundaryBuffers`].
fn readback_usage(context: &Context) -> wgpu::BufferUsages {
    if uses_mappable_primary_buffers(context) {
        wgpu::BufferUsages::MAP_READ
    } else {
        wgpu::BufferUsages::COPY_SRC
    }
}

/// The first `count` words of `buffer`, which has [`readback_usage`].
fn read_words(context: &Context, buffer: &wgpu::Buffer, count: usize) -> Option<Vec<u32>> {
    if count == 0 {
        return Some(Vec::new());
    }
    if uses_mappable_primary_buffers(context) {
        return map_u32(context, buffer, count);
    }
    let bytes = count as u64 * 4;
    let readback = readback_buffer(context, "GPU map first position readback", bytes);
    let mut encoder = encoder(context, "GPU map first position copy");
    encoder.copy_buffer_to_buffer(buffer, 0, &readback, 0, bytes);
    submit(context, encoder.finish())?;
    map_u32(context, &readback, count)
}

/// The surviving entries (the last of each key), compacted on the GPU.
struct Survivors {
    keep: wgpu::Buffer,
    keys: wgpu::Buffer,
    positions: wgpu::Buffer,
    count: wgpu::Buffer,
    position_count: wgpu::Buffer,
}

impl Survivors {
    fn new(context: &Context, byte_len: u64) -> Self {
        Self {
            keep: column(context, "GPU map keep mask", byte_len),
            keys: column(context, "GPU map survivor keys", byte_len),
            positions: column(context, "GPU map survivor positions", byte_len),
            count: counter(context, "GPU map survivor count"),
            position_count: counter(context, "GPU map survivor position count"),
        }
    }

    /// Marks range starts and ends among the survivors, comparing the values' words.
    fn boundary_kernel(
        &self,
        context: &Context,
        (value_lo, value_hi): (&wgpu::Buffer, &wgpu::Buffer),
        buffers: &BoundaryBuffers,
        parameters: &[u32; 8],
    ) -> BoundaryKernel {
        BoundaryKernel::new(
            context,
            MAP_BOUNDARY_SHADER,
            &[
                (&self.keys, true),
                (&self.positions, true),
                (value_lo, true),
                (value_hi, true),
                (&self.count, true),
                (&buffers.start_mask, false),
                (&buffers.end_mask, false),
            ],
            parameters,
        )
    }

    /// Compacts the sorted `keys` and `positions` by the keep mask.
    fn record_compactions(
        &self,
        compactor: &mut Compactor,
        encoder: &mut wgpu::CommandEncoder,
        keys: &wgpu::Buffer,
        positions: &wgpu::Buffer,
        len: u32,
    ) -> Option<()> {
        for (input, output, count) in [
            (keys, &self.keys, &self.count),
            (positions, &self.positions, &self.position_count),
        ] {
            resource_or_panic(
                compactor.record_compact(encoder, input, &self.keep, output, count, len),
            )?;
        }
        Some(())
    }
}

fn column(context: &Context, label: &'static str, byte_len: u64) -> wgpu::Buffer {
    storage_buffer(context, label, byte_len, wgpu::BufferUsages::COPY_SRC)
}

fn counter(context: &Context, label: &'static str) -> wgpu::Buffer {
    storage_buffer(
        context,
        label,
        4,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    )
}

/// The keys, and the values' low and high words (a stand-in if they fit in one word).
fn upload(
    context: &Context,
    key_words: &KeyWords<'_>,
    value_words: &ValueWords<'_>,
) -> Option<(wgpu::Buffer, wgpu::Buffer, wgpu::Buffer)> {
    let keys = slice_storage_buffer(context, "GPU map keys", &key_words.words)?;
    let value_lo = slice_storage_buffer(context, "GPU map value low words", &value_words.lo)?;
    let value_hi = match &value_words.hi {
        Some(hi) => slice_storage_buffer(context, "GPU map value high words", hi)?,
        None => storage_buffer(
            context,
            "GPU map value high words",
            4,
            wgpu::BufferUsages::empty(),
        ),
    };
    Some((keys, value_lo, value_hi))
}

#[cfg(test)]
mod tests {
    use core::net::Ipv4Addr;
    use std::string::{String, ToString};
    use std::vec;

    use core::num::NonZeroUsize;

    use super::super::test_context;
    use super::*;
    use crate::not_nan::{nnf32, nnf64};

    fn has_hardware_gpu() -> bool {
        gpu_context()
            .is_some_and(|context| context.adapter_info.device_type != wgpu::DeviceType::Cpu)
    }

    fn engines(gpu_capable: bool) -> Vec<Engine> {
        let mut engines = vec![
            Engine::Cpu(NonZeroUsize::MIN),
            Engine::Cpu(NonZeroUsize::new(3).expect("3 is nonzero")),
            Engine::WgpuOrCpu,
        ];
        if gpu_capable && has_hardware_gpu() {
            engines.push(Engine::Wgpu);
        }
        #[cfg(feature = "cuda")]
        if gpu_capable && super::super::cuda::cuda_context().is_some() {
            engines.push(Engine::Cuda);
        }
        #[cfg(feature = "cuda")]
        engines.push(Engine::CudaOrCpu);
        #[cfg(feature = "cutile")]
        if gpu_capable && super::super::cutile::available() {
            engines.push(Engine::CuTile);
        }
        engines
    }

    /// Every engine, and the GPU pipeline on the test GPU (which may be a software Vulkan
    /// driver), must match `from_slices`. `gpu_capable` says whether the GPU handles `K` and `V`.
    fn check<K: Integer, V: Eq + Clone + core::fmt::Debug>(
        keys: &[K],
        values: &[V],
        gpu_capable: bool,
    ) {
        let expected = RangeMapBlaze::from_slices(keys, values);
        for engine in engines(gpu_capable) {
            assert_eq!(
                RangeMapBlaze::from_shuffled_slices(keys, values, engine),
                expected,
                "{engine:?}"
            );
        }
        let pool = thread_pool(all_threads());
        let (Some(key_words), Some(value_words)) =
            (key_words(&pool, keys), value_words(&pool, values))
        else {
            assert!(!gpu_capable, "expected a GPU representation");
            return;
        };
        assert!(gpu_capable, "unexpected GPU representation");
        let Some(context) = test_context() else {
            return;
        };
        if keys.is_empty() {
            return;
        }
        let found = ranges_gpu(context, &key_words, &value_words).expect("GPU ingestion");
        assert_eq!(build_map(&pool, values, found), expected, "GPU pipeline");
    }

    /// Seeded xorshift, so failures reproduce.
    fn random(seed: u64) -> impl FnMut() -> u64 {
        let mut state = seed;
        move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        }
    }

    #[test]
    fn small_cases() {
        check::<u32, u32>(&[], &[], true);
        check(&[7_u32], &[1_u32], true);
        // Unsorted, equal values merge.
        check(&[3_u32, 1, 2], &[5_u32, 5, 5], true);
        // Changing values split a run of consecutive keys.
        check(&[1_u32, 2, 3, 4], &[5_u32, 5, 6, 6], true);
        // Gaps split equal values.
        check(&[1_u32, 3, 5], &[5_u32, 5, 5], true);
        // Duplicates with conflicting values: the last wins, and can join or split ranges.
        check(&[2_u32, 1, 2, 3, 2], &[8_u32, 7, 9, 7, 7], true);
        check(&[1_u32, 1, 1], &[1_u32, 2, 3], true);
    }

    /// The largest key is not adjacent to the smallest.
    #[test]
    fn key_boundaries() {
        check(&[u32::MAX, 0], &[4_u32, 4], true);
        check(&[u32::MAX - 1, u32::MAX, 0, 1], &[4_u32, 4, 4, 4], true);
        check(&[u32::MAX, u32::MAX - 1, u32::MAX], &[1_u32, 2, 2], true);
        check(&[i8::MAX, i8::MIN, 0, -1], &[1_u8, 1, 1, 1], true);
        check(&[i32::MIN, i32::MAX, -1, 0, 1], &[1_u8, 1, 1, 1, 1], true);
    }

    /// The key gaps: `char` skips the surrogates, and `NotNanF32` skips `-0.0`.
    #[test]
    fn key_gaps() {
        let chars = ['\u{d7ff}', '\u{e000}', '\u{e001}', '\u{0}', '\u{10ffff}'];
        check(&chars, &[1_u32; 5], true);
        check(&chars, &[1_u32, 2, 2, 2, 2], true);
        let floats = [
            nnf32(f32::from_bits(1)),
            nnf32(-f32::from_bits(1)),
            nnf32(0.0),
            nnf32(1.0),
            nnf32(f32::NEG_INFINITY),
        ];
        check(&floats, &[3_i64; 5], true);
    }

    /// Every value type the GPU compares, including 64-bit ones whose low words match.
    #[test]
    fn value_types() {
        let keys = [4_u16, 1, 2, 3, 5];
        check(&keys, &[1_u8, 1, 1, 2, 2], true);
        check(&keys, &[-1_i16, -1, -1, 0, 0], true);
        check(&keys, &[1_u64, 1, 1 << 32 | 1, 1 << 32 | 1, 1], true);
        check(&keys, &[i64::MIN, i64::MIN, i64::MAX, -1, -1], true);
        check(&keys, &[usize::MAX, 0, 0, 0, 1], true);
        check(&keys, &['a', 'a', 'b', 'b', 'a'], true);
        check(
            &keys,
            &[nnf64(-0.0), nnf64(0.0), nnf64(1.0), nnf64(1.0), nnf64(2.0)],
            true,
        );
        check(
            &keys,
            &[nnf32(1.5), nnf32(1.5), nnf32(1.5), nnf32(-1.5), nnf32(1.5)],
            true,
        );
    }

    /// Key and value types the GPU doesn't handle still work, on the CPU.
    #[test]
    fn cpu_only_types() {
        // Keys wider than 32 bits, or not castable.
        check(&[u64::MAX, 0, 1, u64::MAX - 1], &[1_u32, 1, 1, 1], false);
        check(&[1_u128 << 100, 0, 1], &[1_u32, 1, 1], false);
        check(
            &[
                Ipv4Addr::new(10, 0, 0, 2),
                Ipv4Addr::new(10, 0, 0, 1),
                Ipv4Addr::BROADCAST,
            ],
            &[1_u32, 1, 1],
            false,
        );
        // Values that aren't the crate's integers: owned, borrowed (not `'static`), and wide.
        check(
            &[3_u32, 1, 2, 2],
            &["c", "a", "x", "a"].map(String::from),
            false,
        );
        let owned = ["c", "a", "x", "a"].map(String::from);
        let borrowed: Vec<&str> = owned.iter().map(String::as_str).collect();
        check(&[3_u32, 1, 2, 2], &borrowed, false);
        check(&[3_u32, 1, 2, 2], &[u128::MAX, 1, 2, 1], false);
    }

    /// Non-`Sync` values work on every CPU engine.
    #[test]
    fn non_sync_values() {
        use std::rc::Rc;
        let keys = [3_u32, 1, 2, 2];
        let values = [Rc::new(3), Rc::new(1), Rc::new(9), Rc::new(1)];
        let map = RangeMapBlaze::from_shuffled_slices(&keys, &values, Engine::WgpuOrCpu);
        assert_eq!(map.to_string(), "(1..=2, 1), (3..=3, 3)");
    }

    #[test]
    #[should_panic(expected = "no GPU representation")]
    fn wgpu_panics_for_types_without_a_gpu_representation() {
        let _ = RangeMapBlaze::from_shuffled_slices(&[1_u32], &["a"], Engine::Wgpu);
    }

    #[test]
    #[should_panic(expected = "same length")]
    fn lengths_must_match() {
        let _ = RangeMapBlaze::from_shuffled_slices(&[1_u32, 2], &[1_u32], Engine::WgpuOrCpu);
    }

    /// Seeded random cases around the 256-wide workgroup and the CPU chunking, mixing
    /// duplicates, conflicting values, runs, and gaps.
    #[test]
    fn random_cases() {
        let mut next = random(0x0dd5_eed5_1234_5678);
        for len in [1_usize, 2, 255, 256, 257, 1_000, 65_537] {
            for spread in [4_u64, 64, 1 << 20] {
                let keys: Vec<u32> = (0..len)
                    .map(|_| u32::try_from(next() % spread).expect("small"))
                    .collect();
                let values: Vec<u32> = (0..len)
                    .map(|_| u32::try_from(next() % 3).expect("small"))
                    .collect();
                check(&keys, &values, true);
                let wide: Vec<u64> = keys.iter().map(|&key| u64::from(key)).collect();
                check(&wide, &values, false);
                let names: Vec<String> = values.iter().map(ToString::to_string).collect();
                check(&keys, &names, false);
            }
        }
    }
}
