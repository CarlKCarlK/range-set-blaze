#!/bin/bash
SIMD_INTEGER_VALUES=("i64" "i32" "i16" "i8" "isize" "u64" "u32" "u16" "u8" "usize")
SIMD_LANES_VALUES=(64 32 16 8 4)
RUSTFLAGS_VALUES=("-C target-feature=+avx512f" "-C target-feature=+avx2" "")

for simdLanes in "${SIMD_LANES_VALUES[@]}"; do
    for simdInteger in "${SIMD_INTEGER_VALUES[@]}"; do
        for rustFlags in "${RUSTFLAGS_VALUES[@]}"; do
            echo "Running with SIMD_INTEGER=$simdInteger, SIMD_LANES=$simdLanes, RUSTFLAGS=$rustFlags"
            SIMD_LANES=$simdLanes SIMD_INTEGER=$simdInteger RUSTFLAGS="$rustFlags" cargo bench
        done
    done
done
