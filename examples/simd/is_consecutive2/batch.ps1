$SIMD_INTEGER_VALUES = "i64", "i32", "i16", "i8", "isize", "u64", "u32", "u16", "u8", "usize"
$SIMD_LANES_VALUES = 64, 32, 16, 8, 4
$RUSTFLAGS_VALUES = @("-C target-feature=+avx512f", "-C target-feature=+avx2", "")

foreach ($simdLanes in $SIMD_LANES_VALUES) {
    $env:SIMD_LANES = $simdLanes
    foreach ($simdInteger in $SIMD_INTEGER_VALUES) {
        $env:SIMD_INTEGER = $simdInteger
        foreach ($rustFlags in $RUSTFLAGS_VALUES) {
            $env:RUSTFLAGS = $rustFlags
            Write-Host "Running with SIMD_INTEGER=$env:SIMD_INTEGER, SIMD_LANES=$env:SIMD_LANES, RUSTFLAGS=$env:RUSTFLAGS"
            cargo bench vector
        }
    }
}
