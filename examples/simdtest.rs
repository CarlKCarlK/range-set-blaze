use lazy_static::lazy_static;
use packed_simd::u32x16;
use std::mem::align_of;

lazy_static! {
    static ref DECREASE_U32: packed_simd::Simd<[u32; 16]> =
        packed_simd::u32x16::from([15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
}

macro_rules! is_consecutive {
    ($chunk:expr, $scalar:ty, $simd:ty, $decrease:expr) => {{
        debug_assert!($chunk.len() == <$simd>::lanes(), "Chunk is wrong length");
        debug_assert!(
            $chunk.as_ptr() as usize % align_of::<$simd>() == 0,
            "Chunk is not aligned"
        );

        const LAST_INDEX: usize = <$simd>::lanes() - 1;
        let (expected, overflowed) = $chunk[0].overflowing_add(LAST_INDEX as $scalar);
        if overflowed || expected != $chunk[LAST_INDEX] {
            return false;
        }

        let a = unsafe { <$simd>::from_slice_aligned_unchecked($chunk) } + $decrease;
        let compare_mask = a.eq(<$simd>::splat(a.extract(0)));
        compare_mask.all()
    }};
}

fn is_consecutive(chunk: &[u32]) -> bool {
    is_consecutive!(chunk, u32, u32x16, *DECREASE_U32)
}

#[repr(align(64))]
struct AlignedArray([u32; 16]);

static CHUNK1: AlignedArray = AlignedArray([
    100u32, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
]);

static CHUNK2: AlignedArray = AlignedArray([
    100u32, 99, 3, 4, 5, 6, 7, 8, 9, 10, 110, 111, 112, 113, 114, 115,
]);

pub fn main() {
    let result = is_consecutive(&CHUNK1.0);
    println!("result: {}", result);

    let result = is_consecutive(&CHUNK2.0);
    println!("result: {}", result);
}
