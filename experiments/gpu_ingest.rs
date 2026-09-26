//! Representative type-family benchmark for the public GPU ingestion API.

use std::hint::black_box;
use std::time::{Duration, Instant};

use lampshade::Context;
use range_set_blaze::not_nan::{nnf32, nnf64};
use range_set_blaze::{NotNanF32, NotNanF64, RangeSetBlaze};

fn main() {
    let sizes: Vec<usize> = std::env::args()
        .skip(1)
        .map(|value| value.parse().expect("sizes must be positive integers"))
        .collect();
    let sizes = if sizes.is_empty() {
        vec![1_000_000]
    } else {
        sizes
    };
    if let Ok(context) = pollster::block_on(Context::init()) {
        eprintln!(
            "adapter={:?} backend={:?} device_type={:?}",
            context.adapter_info.name,
            context.adapter_info.backend,
            context.adapter_info.device_type,
        );
        if context.adapter_info.device_type == wgpu::DeviceType::Cpu {
            eprintln!("the public API will use CPU fallback for this adapter");
        }
    }
    println!("type,input_values,output_ranges,cpu_from_iter_ms,gpu_policy_ms");
    for size in sizes {
        let ranks = shuffled_clumps(size, 256);
        benchmark::<u32>(&ranks);
        benchmark::<i32>(&ranks);
        benchmark::<char>(&ranks);
        benchmark::<NotNanF32>(&ranks);
        benchmark::<u64>(&ranks);
        benchmark::<i64>(&ranks);
        benchmark::<NotNanF64>(&ranks);
    }
}

trait BenchElement: Copy + range_set_blaze::Integer {
    const NAME: &'static str;
    fn from_rank(rank: u32) -> Self;
    fn from_slice_gpu(values: &[Self]) -> RangeSetBlaze<Self>;
}

macro_rules! bench_element {
    ($type:ty, $name:literal, $convert:expr) => {
        impl BenchElement for $type {
            const NAME: &'static str = $name;
            fn from_rank(rank: u32) -> Self {
                $convert(rank)
            }
            fn from_slice_gpu(values: &[Self]) -> RangeSetBlaze<Self> {
                RangeSetBlaze::<Self>::from_slice_gpu(values)
            }
        }
    };
}

bench_element!(u32, "u32", |rank| rank);
bench_element!(i32, "i32", |rank: u32| (rank ^ 0x8000_0000).cast_signed());
bench_element!(char, "char", |rank: u32| {
    let scalar = rank % 0x10_f800;
    char::from_u32(if scalar < 0xd800 {
        scalar
    } else {
        scalar + 0x800
    })
    .expect("rank maps to a Unicode scalar")
});
bench_element!(NotNanF32, "NotNanF32", |rank: u32| nnf32(f32::from_bits(
    rank % 0x7f80_0001
)));
bench_element!(u64, "u64", |rank| 0xffff_ff00u64 + u64::from(rank));
bench_element!(i64, "i64", |rank| (0x7fff_ffff_ffff_ff00u64
    .wrapping_add(u64::from(rank))
    ^ 0x8000_0000_0000_0000)
    .cast_signed());
bench_element!(NotNanF64, "NotNanF64", |rank| nnf64(f64::from_bits(
    u64::from(rank)
)));

fn benchmark<T: BenchElement>(ranks: &[u32]) {
    let values: Vec<T> = ranks.iter().copied().map(T::from_rank).collect();
    drop(values.iter().copied().collect::<RangeSetBlaze<_>>());
    drop(T::from_slice_gpu(&values));

    let cpu = median_duration(3, || {
        black_box(values.iter().copied()).collect::<RangeSetBlaze<_>>()
    });
    let gpu = median_duration(3, || black_box(T::from_slice_gpu(black_box(&values))));

    let expected = values.iter().copied().collect::<RangeSetBlaze<_>>();
    let actual = T::from_slice_gpu(&values);
    assert_eq!(
        actual.ranges().collect::<Vec<_>>(),
        expected.ranges().collect::<Vec<_>>()
    );
    println!(
        "{},{},{},{:.6},{:.6}",
        T::NAME,
        values.len(),
        expected.ranges_len(),
        milliseconds(cpu),
        milliseconds(gpu),
    );
}

fn shuffled_clumps(len: usize, clump_len: usize) -> Vec<u32> {
    let mut values = Vec::with_capacity(len);
    let mut value = 0u32;
    for index in 0..len {
        value = value
            .checked_add(1 + u32::from(index != 0 && index % clump_len == 0))
            .expect("benchmark generator exceeded the u32 domain");
        values.push(value);
    }
    let mut state = 0xd1b5_4a32_d192_ed03u64;
    for index in (1..values.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let modulus = u64::try_from(index + 1).expect("index exceeded u64");
        let swap_index = usize::try_from(state % modulus).expect("swap index exceeded usize");
        values.swap(index, swap_index);
    }
    values
}

fn median_duration<T>(iterations: usize, mut operation: impl FnMut() -> T) -> Duration {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        drop(operation());
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
