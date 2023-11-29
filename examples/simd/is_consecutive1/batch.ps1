# Define the SIMD_LANES values
$SIMD_LANES_VALUES = 2, 4, 8, 16, 32, 64

# Define the RUSTFLAGS values
$RUSTFLAGS_VALUES = @("", "-C target-feature=+avx2", "-C target-feature=+avx512f")

# Iterate over SIMD_LANES values
foreach ($simdLanes in $SIMD_LANES_VALUES) {
    # Iterate over RUSTFLAGS values
    foreach ($rustFlags in $RUSTFLAGS_VALUES) {
        $env:SIMD_LANES = $simdLanes
        $env:RUSTFLAGS = $rustFlags
        Write-Host "Running with SIMD_LANES=$env:SIMD_LANES and RUSTFLAGS=$env:RUSTFLAGS"
        cargo bench
    }
}
