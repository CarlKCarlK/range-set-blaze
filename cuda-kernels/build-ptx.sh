#!/usr/bin/env bash
# Regenerates ../src/gpu/cuda_kernels.ptx from src/main.rs. Needs cargo-oxide and the CUDA toolkit.
set -euo pipefail
cd "$(dirname "$0")"
out="$(mktemp -d)"
touch src/main.rs # so the codegen backend runs again
CUDA_OXIDE_PTX_DIR="$out" cargo oxide build --arch sm_80 # PTX for sm_80 runs on every newer GPU
cp "$out/cuda_kernels.ptx" ../src/gpu/cuda_kernels.ptx
rm -rf "$out"
echo "wrote ../src/gpu/cuda_kernels.ptx"
