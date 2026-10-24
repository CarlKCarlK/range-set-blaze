//! Benchmarks `from_slice` against `from_shuffled_slice` with each `Engine`, on the same data.
//!
//! Usage:
//! `cargo run --release --features shuffled_slices --bin gpu_ingest -- [--cpu N]... [--type T]... [SIZES...]`
//! (default size: 1,000,000). Each `--cpu N` adds an `Engine::Cpu(N)` column; the default is
//! every available thread. `Engine::Cpu(1)` is always included. Each `--type T` (such as `u32`)
//! limits the run to those types; the default is every type. Prints CSV. The GPU column is
//! empty when there is no hardware GPU. With the `cuda` feature, a `cuda_ms` column times
//! `Engine::CudaOnly`.
//!
//! Each type gets data built within its own range of values, in two workloads:
//!
//! - `distinct`: `n` distinct values in clumps of 256 (a gap of one value between clumps). A
//!   type too small to hold `n` distinct values in that layout (for example `u16` at 10 million)
//!   is reported as `n/a` rather than silently wrapped.
//! - `duplicates`: the same layout with fewer distinct values, each repeated
//!   `DUPLICATE_REPEATS` times (or more, for types too small to hold that many distinct values).
//!
//! Both run in order (sorted, duplicates adjacent) and shuffled.

use std::fmt::Write;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use lampshade::Context;
use range_set_blaze::not_nan::{nnf32, nnf64};
use range_set_blaze::{Engine, NotNanF32, NotNanF64, RangeSetBlaze};

/// Seed for the shuffle; fixed so every machine benchmarks the same data.
const SEED: u64 = 0xd1b5_4a32_d192_ed03;
const CLUMP_LEN: u64 = 256;
const DUPLICATE_REPEATS: u64 = 16;
/// The least time each timed operation runs before its samples (see `median`).
const WARM_UP: Duration = Duration::from_millis(1000);

fn main() {
    let mut sizes: Vec<u64> = Vec::new();
    let mut thread_counts: Vec<NonZeroUsize> = Vec::new();
    let mut types: Vec<String> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--cpu" {
            let count = args.next().expect("--cpu needs a thread count");
            thread_counts.push(
                count
                    .parse()
                    .expect("thread counts must be positive integers"),
            );
        } else if arg == "--type" {
            types.push(args.next().expect("--type needs a type name"));
        } else {
            sizes.push(arg.parse().expect("sizes must be positive integers"));
        }
    }
    let sizes = if sizes.is_empty() {
        vec![1_000_000]
    } else {
        sizes
    };
    let has_gpu = pollster::block_on(Context::init()).is_ok_and(|context| {
        eprintln!(
            "adapter={:?} backend={:?} device_type={:?}",
            context.adapter_info.name,
            context.adapter_info.backend,
            context.adapter_info.device_type,
        );
        context.adapter_info.device_type != wgpu::DeviceType::Cpu
    });
    let available = std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);
    eprintln!("available threads={available}");
    if thread_counts.is_empty() {
        thread_counts.push(available);
    }
    let mut cpu_columns = String::new();
    for count in &thread_counts {
        write!(cpu_columns, ",cpu_{count}_ms").expect("writing to a String can't fail");
    }
    println!(
        "workload,order,type,input_values,distinct_values,output_ranges,from_slice_ms,cpu_1_ms{cpu_columns},gpu_only_ms,cuda_ms"
    );
    for n in sizes {
        for workload in [Workload::Distinct, Workload::Duplicates] {
            for shuffle in [true, false] {
                let run = Run {
                    n,
                    workload,
                    shuffle,
                    has_gpu,
                    thread_counts: &thread_counts,
                    types: &types,
                };
                run.benchmark::<u8>();
                run.benchmark::<u16>();
                run.benchmark::<u32>();
                run.benchmark::<i32>();
                run.benchmark::<char>();
                run.benchmark::<NotNanF32>();
                run.benchmark::<u64>();
                run.benchmark::<i64>();
                run.benchmark::<NotNanF64>();
            }
        }
    }
}

/// A key type, with how many values it can hold and an order-preserving way to make the
/// `k`-th smallest one, so that consecutive `k` give values the set treats as adjacent.
trait BenchElement: Copy + range_set_blaze::Integer {
    const NAME: &'static str;
    /// How many distinct values `from_index` accepts (`0..DOMAIN`).
    const DOMAIN: u64;
    fn from_index(k: u64) -> Self;
    fn shuffled(values: &mut [Self], engine: Engine) -> RangeSetBlaze<Self>;
}

macro_rules! bench_element {
    ($type:ty, $name:literal, $domain:expr, $from_index:expr) => {
        impl BenchElement for $type {
            const NAME: &'static str = $name;
            const DOMAIN: u64 = $domain;
            fn from_index(k: u64) -> Self {
                debug_assert!(k < Self::DOMAIN);
                $from_index(k)
            }
            fn shuffled(values: &mut [Self], engine: Engine) -> RangeSetBlaze<Self> {
                RangeSetBlaze::<Self>::from_shuffled_slice(values, engine)
            }
        }
    };
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
mod elements {
    use super::{BenchElement, Engine, NotNanF32, NotNanF64, RangeSetBlaze, nnf32, nnf64};

    bench_element!(u8, "u8", 1 << 8, |k| k as u8);
    bench_element!(u16, "u16", 1 << 16, |k| k as u16);
    bench_element!(u32, "u32", 1 << 32, |k| k as u32);
    bench_element!(i32, "i32", 1 << 32, |k: u64| (k as u32 ^ 0x8000_0000)
        as i32);
    // Unicode scalar values skip the surrogates `0xD800..=0xDFFF`; the set treats `0xD7FF` and
    // `0xE000` as adjacent.
    bench_element!(char, "char", 0x10_f800, |k: u64| {
        let k = k as u32;
        char::from_u32(if k < 0xd800 { k } else { k + 0x800 }).expect("valid scalar")
    });
    // Non-negative floats in bit order, `+0.0` through `+inf`.
    bench_element!(NotNanF32, "NotNanF32", 0x7f80_0001, |k: u64| nnf32(
        f32::from_bits(k as u32)
    ));
    // Start just below 2^32, so runs cross from the low 32-bit word into the high one.
    bench_element!(u64, "u64", u64::MAX - 0xffff_ff00, |k| 0xffff_ff00 + k);
    // Start just below zero, so runs cross from negative to positive.
    bench_element!(i64, "i64", u64::MAX / 2, |k: u64| (k as i64)
        .wrapping_sub(256));
    bench_element!(NotNanF64, "NotNanF64", 0x7ff0_0000_0000_0001, |k| nnf64(
        f64::from_bits(k)
    ));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Workload {
    Distinct,
    Duplicates,
}

struct Run<'a> {
    n: u64,
    workload: Workload,
    shuffle: bool,
    has_gpu: bool,
    thread_counts: &'a [NonZeroUsize],
    types: &'a [String],
}

impl Run<'_> {
    fn benchmark<T: BenchElement>(&self) {
        if !self.types.is_empty() && !self.types.iter().any(|name| name == T::NAME) {
            return;
        }
        let workload = match self.workload {
            Workload::Distinct => "distinct",
            Workload::Duplicates => "duplicates",
        };
        let order = if self.shuffle { "shuffled" } else { "in_order" };
        let Some((values, distinct)) = self.values::<T>() else {
            let empty = ",".repeat(self.thread_counts.len() + 5);
            println!("{workload},{order},{},{},n/a{empty}", T::NAME, self.n);
            return;
        };
        let expected = RangeSetBlaze::from_slice(&values);
        let from_slice = median(|| (), |()| RangeSetBlaze::from_slice(black_box(&values)));
        let shuffled = |engine: Engine| {
            let mut check = values.clone();
            assert_eq!(T::shuffled(&mut check, engine), expected, "{engine:?}");
            median(
                || values.clone(),
                |mut copy| T::shuffled(black_box(&mut copy), engine),
            )
        };
        let cpu_1 = shuffled(Engine::Cpu(NonZeroUsize::MIN));
        let mut cpu_many = String::new();
        for &count in self.thread_counts {
            let elapsed = milliseconds(shuffled(Engine::Cpu(count)));
            write!(cpu_many, ",{elapsed:.3}").expect("writing to a String can't fail");
        }
        let gpu = self.has_gpu.then(|| shuffled(Engine::GpuOnly));
        #[cfg(feature = "cuda")]
        let cuda = Some(shuffled(Engine::CudaOnly));
        #[cfg(not(feature = "cuda"))]
        let cuda: Option<Duration> = None;
        println!(
            "{workload},{order},{},{},{},{},{:.3},{:.3}{cpu_many},{},{}",
            T::NAME,
            values.len(),
            distinct,
            expected.ranges_len(),
            milliseconds(from_slice),
            milliseconds(cpu_1),
            gpu.map_or_else(String::new, |gpu| format!("{:.3}", milliseconds(gpu))),
            cuda.map_or_else(String::new, |cuda| format!("{:.3}", milliseconds(cuda))),
        );
    }

    /// The `n` values for this run and how many are distinct, or `None` if `T` can't hold this
    /// workload.
    fn values<T: BenchElement>(&self) -> Option<(Vec<T>, u64)> {
        let distinct = match self.workload {
            Workload::Distinct => self.n,
            Workload::Duplicates => self
                .n
                .div_ceil(DUPLICATE_REPEATS)
                .min(max_distinct(T::DOMAIN)),
        };
        let max_index = clump_index(distinct.checked_sub(1)?);
        if max_index >= T::DOMAIN {
            return None;
        }
        // In order: each distinct value repeated `repeats` times, next to each other.
        let repeats = self.n.div_ceil(distinct);
        let mut values: Vec<T> = (0..self.n)
            .map(|i| T::from_index(clump_index(i / repeats)))
            .collect();
        if self.shuffle {
            shuffle(&mut values, SEED);
        }
        Some((values, distinct))
    }
}

/// The index of the `j`-th distinct value: clumps of `CLUMP_LEN`, a gap of one between them.
const fn clump_index(j: u64) -> u64 {
    j + j / CLUMP_LEN
}

/// The most distinct values whose clump layout fits in `domain`.
const fn max_distinct(domain: u64) -> u64 {
    // `clump_index(d - 1) < domain` holds for every `d` up to about
    // `domain * CLUMP_LEN / (CLUMP_LEN + 1)`; start just above that and step down.
    let mut d = domain - domain / (CLUMP_LEN + 1);
    while d > 0 && clump_index(d - 1) >= domain {
        d -= 1;
    }
    d
}

/// Fisher–Yates shuffle driven by xorshift from `seed`.
fn shuffle<T>(values: &mut [T], seed: u64) {
    let mut state = seed;
    for index in (1..values.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let modulus = u64::try_from(index + 1).expect("index exceeded u64");
        let swap_index = usize::try_from(state % modulus).expect("swap index exceeded usize");
        values.swap(index, swap_index);
    }
}

/// Median of five timed runs after one warm-up; `setup` runs outside the timer.
fn median<S, R>(mut setup: impl FnMut() -> S, mut operation: impl FnMut(S) -> R) -> Duration {
    // Warm up for at least `WARM_UP`, not just one call: after a few idle seconds a GPU drops to
    // its idle clocks (P8 on an RTX 3060: 210 MHz, a tenth of its peak) and needs sustained work
    // to come back up. The same rule applies to every engine.
    let warm_up = Instant::now();
    drop(operation(setup()));
    while warm_up.elapsed() < WARM_UP {
        drop(operation(setup()));
    }
    let mut samples: Vec<Duration> = (0..5)
        .map(|_| {
            let input = setup();
            let start = Instant::now();
            drop(operation(input));
            start.elapsed()
        })
        .collect();
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
