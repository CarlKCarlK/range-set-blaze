$SIMD_INTEGER_VALUES = "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize"
$SIMD_LANES_VALUES = 1, 2, 4, 8, 16, 32, 64
$RUSTFLAGS_VALUES = @("", "-C target-feature=+avx2", "-C target-feature=+avx512f")

foreach ($simdInteger in $SIMD_INTEGER_VALUES) {
    $env:SIMD_INTEGER = $simdInteger
    foreach ($simdLanes in $SIMD_LANES_VALUES) {
        $env:SIMD_LANES = $simdLanes
        foreach ($rustFlags in $RUSTFLAGS_VALUES) {
            $env:RUSTFLAGS = $rustFlags
            Write-Host "Running with SIMD_INTEGER=$env:SIMD_INTEGER, SIMD_LANES=$env:SIMD_LANES, RUSTFLAGS=$env:RUSTFLAGS"
            cargo bench
        }
    }
}
