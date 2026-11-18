<!-- TODO0 consider deleting this spec once the work below is implemented and released. -->

# GPU evidence review and branch integration handoff

## Reviewed state

November 14, 2026: local `gpu-on-simd` is at `55540f3f`; its GitHub workflow
completed with all 16 jobs successful. That CI result precedes these documentation
edits. The evidence review covers the benchmark harness, committed summary CSVs,
run notes, and available local runners, not a new hardware benchmark or a full
correctness review of the kernels.

## Findings and documentation corrections

- The 100M timing ranges agree with the committed CSVs. Each GPU has three run
  medians; WSL2 CPU ranges pool six across two builds. Each quick-run median uses
  three samples. Raw individual sample durations are not saved in those CSVs.
- Windows wgpu results now have their own same-OS CPU table. WSL2 rankings use
  WSL2 measurements. Earlier AMD Dozen hardware results remain valid historical
  evidence; selecting llvmpipe in the later runs does not establish a general
  WSL2 limitation.
- Billion-key results mix revisions and have only one corrected CPU-radix run
  median per machine. The desktop comparison-sort range is now 6.47–6.66 seconds,
  matching the two cited, committed CSVs. A local adaptive-radix CSV contains the
  former 6.71-second endpoint but is outside the report's committed manifest.
- The billion-key desktop memory requirement exceeds VRAM, but the saved note
  supplies no paging trace. Paging and shared-memory benefits are hypotheses,
  not isolated causes established by these timings.
- HIP and offload share kernel source but have different host paths. The guide
  now reflects the repeated HIP 222–238 ms versus offload 273–275 ms results.
- Historical profiling numbers lack an identified capture manifest. The report
  retains them as reported observations and flags the 838/880 ms discrepancy.
- Full reproduction still needs per-build source/lockfile identity, compiler,
  driver/runtime versions, backend, power state, and individual sample durations.

## Branch integration status

The documentation review is committed as `b97205e7` on `gpu-on-simd`. Inspection
of the actual laptop found `amd-gpu-portability` at `fb0bfa0b`, six commits ahead
of its published branch, with 10 modified tracked files and 3 untracked Rust
files. All 13 files were compared byte-for-byte with desktop Git history: each
matches either the current implementation or a historical version. No unique
source content was found among them.

The laptop's committed history was imported into `review/amd-laptop-committed`.
The local `review/amd-gpu-integration` branch merges it with `b97205e7` without
conflicts and updates the historical AMD planning documents. The laptop branch
was fast-forwarded to `a2f0daa9` after its historical working-file copies were
saved in stash `preserve-historical-gpu-copies-before-20261114-integration`.
Its worktree is clean. Do not pop that stash over the newer implementation.

## Validation

On the Radeon 8060S laptop, both `gpu::hip::tests::gpu_ranges_match_host_ranges`
and `gpu::offload::tests::gpu_ranges_match_host_ranges` passed on `a2f0daa9`
(2 passed, 0 failed). They compare GPU results with host ranges across multiple
sizes and key patterns. The command was:

```sh
cargo +nightly-2026-11-12 test --locked --features hip_nightly_experimental,offload_nightly_experimental --lib gpu_ranges_match_host_ranges -- --test-threads=1
```

The existing user-local ROCm runtime paths were used, with
`HSA_ENABLE_DXG_DETECTION=1`, `RANGE_SET_BLAZE_OFFLOAD_ARCH=gfx1151`,
`OMP_TARGET_OFFLOAD=MANDATORY`, `RANGE_SET_BLAZE_REQUIRE_OFFLOAD_GPU=1`, and
`RANGE_SET_BLAZE_REQUIRE_HIP_GPU=1`. Build output is under
`~/.cache/range-set-blaze-gpu-review` on the laptop.

The desktop `just check-all` run on `b97205e7` encountered the previously
recorded pinned-nightly/CUDA dependency failure: `cuda-host` calls
`core::intrinsics::type_id`, unavailable in that nightly (`E0425`). This review
does not change the affected library code, dependency revisions, or toolchains.
The integration branch has identical Rust source, manifests, and lockfile to
`b97205e7`; its additional changes are documentation. Relative file links and
`git diff --check` passed on the integration branch.

## Remaining integration work

1. Prepare PR #38's title and description around the current
   `from_shuffled_slice` / `from_shuffled_slices` API and `shuffled_slices` feature,
   including CPU comparison/radix, wgpu, CUDA-Oxide, cuTile, offload, and HIP.
   Its present description still names the earlier in-place API and engines.
   Keep existing public-API changes visible to reviewers; this review adds none.
2. Track hosted CI separately from the local gate limitation above. Full
   offload/HIP suite coverage and a deliberate resolution of the pinned-nightly
   CUDA dependency mismatch remain follow-up work; the two targeted hardware
   checks do not establish that every hardware test passed.
3. Prepare PR #39's description around the remaining branch delta, including the
   preserved historical laptop report and updated handoff documents. Its
   published delta previously consisted of two historical planning documents.
   Decide whether keeping that PR serves a purpose after synchronization.
4. Inspect `~/programs/range-set-blaze-hip` for unique commits, uncommitted changes,
   and results before proposing cleanup. Keep runner scripts and raw output under
   `/mnt/c/deldir/rsb-gpu-bench/`.

Neither PR has been merged or edited as part of this review. The historical
laptop work is preserved through merge ancestry, not replaced by the desktop's
stale local `amd-gpu-portability` ref.

## Follow-up measurements

For stronger workshop claims, repeat billion-key measurements on one recorded
revision with three full runs per supported engine and explicit failures for
unsupported configurations. Capture residency/paging evidence if explaining the
desktop slowdown, and archive the historical profiling captures before presenting
their stage timings as reproduced results. These follow-ups need not delay a
documentation review that clearly labels the existing evidence's limits.
