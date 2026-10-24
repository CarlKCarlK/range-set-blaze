#!/usr/bin/env bash
# Regenerates ../src/gpu/cuda_kernels.ptx from src/main.rs. Needs cargo-oxide and the CUDA toolkit.
# With --check, compares instead, and fails if the committed PTX is out of date.
set -euo pipefail
cd "$(dirname "$0")"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
touch src/main.rs # so the codegen backend runs again
CUDA_OXIDE_PTX_DIR="$out" cargo oxide build --arch sm_80 # PTX for sm_80 runs on every newer GPU
if [[ "${1:-}" == "--check" ]]; then
    if cmp -s "$out/cuda_kernels.ptx" ../src/gpu/cuda_kernels.ptx; then
        echo "../src/gpu/cuda_kernels.ptx is up to date"
    else
        echo "../src/gpu/cuda_kernels.ptx is out of date: run ./build-ptx.sh" >&2
        exit 1
    fi
else
    cp "$out/cuda_kernels.ptx" ../src/gpu/cuda_kernels.ptx
    echo "wrote ../src/gpu/cuda_kernels.ptx"
fi
