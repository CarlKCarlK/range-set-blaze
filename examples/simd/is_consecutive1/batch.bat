@echo off
setlocal EnableDelayedExpansion

:loop
for %%i in (2 4 8 16 32 64) do (
    set SIMD_LANES=%%i

    REM No RUSTFLAGS
    set RUSTFLAGS=
    echo Running with SIMD_LANES=!SIMD_LANES! and no RUSTFLAGS
    cargo bench
    call :check_interrupt

    REM With AVX2
    set RUSTFLAGS=-C target-feature=+avx2
    echo Running with SIMD_LANES=!SIMD_LANES! and RUSTFLAGS=!RUSTFLAGS!
    cargo bench
    call :check_interrupt

    REM With AVX512F
    set RUSTFLAGS=-C target-feature=+avx512f
    echo Running with SIMD_LANES=!SIMD_LANES! and RUSTFLAGS=!RUSTFLAGS!
    cargo bench
    call :check_interrupt
)

endlocal
goto :eof

:check_interrupt
echo Press Ctrl-C to stop or any other key to continue
pause > nul
goto :eof
