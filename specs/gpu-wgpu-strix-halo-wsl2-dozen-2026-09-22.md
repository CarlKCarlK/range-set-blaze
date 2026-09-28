<!-- todo​0 consider deleting this note once the WSL2 Dozen setup is no longer useful. -->

# wgpu/Dozen benchmark run: Strix Halo on WSL2, 2026-09-22

Provenance and reproduction notes for:

- [`gpu-wgpu-strix-halo-wsl2-dozen-2026-09-22.csv`](gpu-wgpu-strix-halo-wsl2-dozen-2026-09-22.csv)
- [`gpu-wgpu-strix-halo-windows-vulkan-2026-09-22.csv`](gpu-wgpu-strix-halo-windows-vulkan-2026-09-22.csv)

The WSL2 measurements use Mesa Dozen to translate Vulkan to D3D12 through
WSL's `/dev/dxg` bridge. They do not use RADV, a native Linux AMD kernel
driver, or llvmpipe. The native-Windows measurements are a matched control
using the AMD Vulkan driver selected by wgpu on Windows.

## Result

The WSL2 path is genuinely hardware accelerated. `vulkaninfo --summary`
reported:

```text
deviceType         = PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU
deviceName         = Microsoft Direct3D12 (AMD Radeon(TM) 8060S Graphics)
vendorID           = 0x1002
deviceID           = 0x1586
driverID           = DRIVER_ID_MESA_DOZEN
driverName         = Dozen
driverInfo         = Mesa 25.2.8
```

The RangeSetBlaze benchmark reported the same adapter through wgpu's Vulkan
backend, and both `request_adapter()` and `request_device()` succeeded without
weakening any feature or limit request:

```text
adapter="Microsoft Direct3D12 (AMD Radeon(TM) 8060S Graphics)" backend=Vulkan
```

Dozen declares Vulkan conformance version `0.0.0.0`, so wgpu 30 hides it by
default. `WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER=1` is required. Lampshade
0.13.0 constructs its instance descriptor with `.with_env()`, so the setting
is honored without a source change.

The initial staged benchmark completed all 100K, 1M, and 10M cases with its
normal correctness checks. It stopped before measuring 100M because Dozen
reports a 128 MiB maximum storage-buffer binding:

```text
100000000 items need a 400000000-byte storage buffer, but adapter
'Microsoft Direct3D12 (AMD Radeon(TM) 8060S Graphics)' allows a
134217728-byte binding and a 2147483647-byte buffer
```

This is a concrete Dozen-advertised binding limit. A single `u32` input buffer
can contain at most 33,554,432 values under that limit. No RangeSetBlaze GPU
algorithm or buffer layout was changed to work around it.

## Representative comparison

These are the 10M-value fused end-to-end medians. The fused measurement
includes upload, radix sort, normalization, range readback, and CPU-side set
construction. Ratios below 1.0 mean WSL2 was faster than native Windows in
that run.

| Distribution | WSL2 Dozen | Native Windows Vulkan | WSL2 / Windows | WSL2 GPU / CPU speedup |
| --- | ---: | ---: | ---: | ---: |
| random gaps | 202.93 ms | 247.52 ms | 0.82x | 2.43x |
| duplicate heavy | 38.14 ms | 43.10 ms | 0.88x | 9.68x |
| clumps 16 | 34.96 ms | 32.78 ms | 1.07x | 10.52x |
| clumps 256 | 30.56 ms | 24.11 ms | 1.27x | 11.93x |
| clumps 4096 | 29.19 ms | 22.16 ms | 1.32x | 12.43x |

The result is reasonable for a translation layer: WSL2 ranged from 18 percent
faster to 32 percent slower than the native-Windows control, depending on the
distribution. End-to-end numbers include CPU work, and the WSL2 CPU timings
were faster in this run, so the table should not be interpreted as a pure
Dozen-versus-native-driver comparison. Per-stage measurements are preserved in
the CSV files.

## Mappable-primary follow-up

Both native Windows Vulkan and WSL2/Dozen advertise wgpu 30's
`MAPPABLE_PRIMARY_BUFFERS` feature on the Radeon 8060S. The prototype writes
encoded input directly into `MAP_WRITE | STORAGE` buffers and maps
`MAP_READ | STORAGE` output/count buffers directly, avoiding the upload
staging copy and the explicit GPU-to-readback copies.

The production policy selects this path when the adapter is reported as an
integrated GPU and advertises the feature. Discrete GPUs and adapters without
the feature retain the staged path, while GPU-unavailable or unsuitable cases
retain the existing CPU fallback.

The experiment prints both detection and selection explicitly. `--staged`
forces the old path as a matched control:

```text
adapter="AMD Radeon(TM) 8060S Graphics" backend=Vulkan mappable_primary_available=true memory_path=mapped-primary
adapter="Microsoft Direct3D12 (AMD Radeon(TM) 8060S Graphics)" backend=Vulkan mappable_primary_available=true memory_path=mapped-primary
```

These are 1M-value fused end-to-end medians from one matched run, in
milliseconds. Each result passed the benchmark's CPU comparison.

| Distribution | WSL2 mapped | WSL2 staged | Windows mapped | Windows staged |
| --- | ---: | ---: | ---: | ---: |
| random gaps | 26.91 | 28.84 | 26.16 | 25.04 |
| duplicate heavy | 7.73 | 14.46 | 5.62 | 5.03 |
| clumps 16 | 5.89 | 13.01 | 4.32 | 3.74 |
| clumps 256 | 5.28 | 12.54 | 3.58 | 3.84 |
| clumps 4096 | 4.81 | 10.93 | 3.37 | 3.40 |

The mapped path materially improved the Dozen results in this run. Native
Windows was roughly comparable and workload-dependent: direct mapping reduced
the individual upload/readback timings, while the small fused end-to-end
differences went in both directions. This supports capability-driven selection
on shared-memory hardware but is not evidence that direct mapping is
universally faster.

The matched 10M run showed that the Dozen advantage persists but narrows as
the compute and CPU-side construction work grows:

| Distribution | WSL2 mapped | WSL2 staged | Windows mapped | Windows staged |
| --- | ---: | ---: | ---: | ---: |
| random gaps | 176.71 | 188.74 | 238.04 | 260.52 |
| duplicate heavy | 35.03 | 37.22 | 45.31 | 47.71 |
| clumps 16 | 30.54 | 35.49 | 35.47 | 34.30 |
| clumps 256 | 30.06 | 28.92 | 22.78 | 23.39 |
| clumps 4096 | 26.96 | 28.67 | 22.62 | 22.47 |

Mapped primary won four of five WSL2 fused cases and three of five Windows
cases. The Windows differences are roughly comparable and workload-dependent,
generally within about 10 percent. They do not justify a backend-specific
production heuristic; `--staged` keeps the decision easy to revisit.

Run either path under WSL2 after applying the new-shell environment above:

```bash
cargo run --release --bin gpu_normalize_wgpu --features gpu -- 1000000
cargo run --release --bin gpu_normalize_wgpu --features gpu -- 1000000 --staged
```

## Environment

- Hardware: AMD Strix Halo, 128 GB system memory
- GPU: AMD Radeon(TM) 8060S Graphics, Windows driver `32.0.22050.4002`
- WSL: 2.6.2.0
- WSL kernel: `6.6.87.2-1`
- WSLg: 1.0.71
- WSL Direct3D: `1.611.1-81528511`
- WSL DXCore: `10.0.26100.1-240331-1435.ge-release`
- Windows: version `10.0.26200.9457`; `Get-ComputerInfo` reported product name
  `Windows 10 Pro`, version `2009`, build `26200`
- Linux: Ubuntu 24.04.3 LTS, kernel
  `6.6.87.2-microsoft-standard-WSL2`
- Rust: `rustc 1.97.0 (2d8144b78 2026-07-07)` on WSL2
- Vulkan loader: 1.3.275
- System Mesa: 25.2.8 (`25.2.8-0ubuntu0.24.04.2`)
- Local Dozen build: Mesa 25.2.8
- RangeSetBlaze commit: `799d98ee8ae25321cf334e0b4ea4b733df828ace`
- wgpu 30.0.1, Lampshade 0.13.0

`/dev/dxg` was present. `/dev/dri` was absent. The WSL bridge libraries were:

```text
/usr/lib/wsl/lib/libd3d12.so      801840 bytes
/usr/lib/wsl/lib/libd3d12core.so 6880344 bytes
/usr/lib/wsl/lib/libdxcore.so     942048 bytes
```

Ubuntu's installed `mesa-vulkan-drivers` package did not include Dozen. Its
ICD directory contained Asahi, gfxstream, Intel, Intel HasVK, lavapipe,
Nouveau, RADV, and virtio manifests, but no `dzn_icd` manifest or
`libvulkan_dzn.so`. WSLg's system distro likewise contained no Dozen binary.
Before the local build, forcing Vulkan selected only llvmpipe.

## Local Dozen build

The build is isolated under `~/.local/opt/mesa-dzn-25.2.8`; it does not replace
the system Mesa installation. The Mesa source archive was downloaded from
`https://archive.mesa3d.org/mesa-25.2.8.tar.xz` and had SHA-256:

```text
097842f3e49d996868b38688db87b006f7d4541e93ce86d2f341d8b3e7be7c93
```

Install the Ubuntu build prerequisites:

```bash
sudo apt update
sudo apt install --no-install-recommends \
  vulkan-tools python3-venv ninja-build directx-headers-dev \
  bison flex pkg-config build-essential libdrm-dev \
  libexpat1-dev zlib1g-dev libzstd-dev
```

Create an isolated recent Meson environment. Ubuntu 24.04's Meson 1.3.2 is
too old for Mesa 25.2.8, which requires Meson 1.4 or newer.

```bash
python3 -m venv /tmp/mesa-dzn-venv
/tmp/mesa-dzn-venv/bin/pip install 'meson>=1.4' mako pyyaml packaging ply
```

Download, configure, build, and install only Dozen and its SPIR-V-to-DXIL
compiler:

```bash
curl -fL -o /tmp/mesa-25.2.8.tar.xz \
  https://archive.mesa3d.org/mesa-25.2.8.tar.xz
mkdir -p /tmp/mesa-25.2.8-src
tar -xf /tmp/mesa-25.2.8.tar.xz \
  -C /tmp/mesa-25.2.8-src --strip-components=1

PATH=/tmp/mesa-dzn-venv/bin:/usr/bin:/bin meson setup \
  /tmp/mesa-25.2.8-dzn-build /tmp/mesa-25.2.8-src \
  --prefix="$HOME/.local/opt/mesa-dzn-25.2.8" \
  --libdir=lib/x86_64-linux-gnu \
  -Dplatforms=[] \
  -Dgallium-drivers=[] \
  -Dvulkan-drivers=microsoft-experimental \
  -Dglx=disabled \
  -Degl=disabled \
  -Dgbm=disabled \
  -Dllvm=disabled \
  -Dbuild-tests=false \
  -Dvideo-codecs=[]

PATH=/tmp/mesa-dzn-venv/bin:/usr/bin:/bin \
  meson compile -C /tmp/mesa-25.2.8-dzn-build
PATH=/tmp/mesa-dzn-venv/bin:/usr/bin:/bin \
  meson install -C /tmp/mesa-25.2.8-dzn-build
```

The resulting local runtime consists of the Dozen ICD, Mesa's
SPIR-V-to-DXIL helper, and their manifest. Removing the prefix reverses the
local installation.

## New-shell setup

Set these variables in each shell that should use Dozen:

```bash
export VK_DRIVER_FILES="$HOME/.local/opt/mesa-dzn-25.2.8/share/vulkan/icd.d/dzn_icd.x86_64.json"
export LD_LIBRARY_PATH="$HOME/.local/opt/mesa-dzn-25.2.8/lib/x86_64-linux-gnu:/usr/lib/wsl/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export MESA_D3D12_DEFAULT_ADAPTER_NAME=AMD
export WGPU_BACKEND=vulkan
export WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER=1
```

Verify the Vulkan adapter without requiring a display server:

```bash
env -u DISPLAY -u WAYLAND_DISPLAY vulkaninfo --summary
```

The output must name `Microsoft Direct3D12 (AMD Radeon(TM) 8060S Graphics)`
and `DRIVER_ID_MESA_DOZEN`. Do not proceed if it names llvmpipe.

Run a tiny RangeSetBlaze probe, then the existing benchmark unchanged:

```bash
cargo run --release --bin gpu_normalize_wgpu --features gpu -- 1
cargo run --release --bin gpu_normalize_wgpu --features gpu
```

The full default sweep currently exits at 100M because of the Dozen binding
limit described above; the 100K through 10M rows complete normally.

## Initial staged measurement details

The initial WSL2 command used the benchmark source before the mappable-primary
follow-up:

```bash
cargo run --release --bin gpu_normalize_wgpu --features gpu
```

The matched native-Windows control used the same commit and requested 1M and
10M explicitly:

```powershell
cargo run --release --bin gpu_normalize_wgpu --features gpu -- 1000000 10000000
```

All timing columns are medians. The benchmark uses 9 iterations at 100K, 7 at
1M, and 5 at 10M. It validates both staged and fused GPU range output against
the CPU-derived `RangeSetBlaze` outside the timed regions before printing each
row.
