@rem @echo off
setlocal EnableDelayedExpansion

for %%i in (2 4 8 16 32 64) do (
    set SIMD_LANES=%%i

    REM No RUSTFLAGS
    set RUSTFLAGS=
    echo Running with SIMD_LANES=!SIMD_LANES! and no RUSTFLAGS
    cargo bench

    REM With AVX2
    set RUSTFLAGS=-C target-feature=+avx2
    echo Running with SIMD_LANES=!SIMD_LANES! and RUSTFLAGS=!RUSTFLAGS!
    cargo bench

    REM With AVX512F
    set RUSTFLAGS=-C target-feature=+avx512f
    echo Running with SIMD_LANES=!SIMD_LANES! and RUSTFLAGS=!RUSTFLAGS!
    cargo bench
)

endlocal
