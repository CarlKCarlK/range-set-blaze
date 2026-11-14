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

## Branch integration plan

1. Review and commit the documentation corrections on `gpu-on-simd`.
2. Prepare PR #38's title and description around the current
   `from_shuffled_slice` / `from_shuffled_slices` API and `shuffled_slices` feature,
   including CPU comparison/radix, wgpu, CUDA-Oxide, cuTile, offload, and HIP.
   Its present description still names the earlier in-place API and engines.
   Keep existing public-API changes visible to reviewers; this review adds none.
3. Inspect the actual laptop worktree before integration. PR #39's discussion
   records a laptop-local merge at `8cb9dd21`, while its published head and the
   desktop branch remain `0852f9db`. The desktop's stale branch is not proof of
   the laptop's current state. Fetch and inspect local changes and commits before
   merging the reviewed `gpu-on-simd` commit there. Avoid overwriting or resetting
   laptop work.
4. Validate the resulting branch. Run `just check-all` before pushing; record the
   exact merged commit and distinguish hardware checks from hosted CI. Confirm
   GPU-required offload/HIP correctness on the laptop with its existing runtime
   environment before claiming hardware coverage for that merged revision.
5. Update the AMD handoff and portability spec to reflect completed measurements,
   then prepare PR #39's description around the remaining branch delta. Its
   published delta currently consists of two historical planning documents.
   Decide whether keeping that PR serves a purpose after synchronization.
6. Inspect `~/programs/range-set-blaze-hip` for unique commits, uncommitted changes,
   and results before proposing cleanup. Keep runner scripts and raw output under
   `/mnt/c/deldir/rsb-gpu-bench/`.

The local read-only merge preview of `55540f3f` and `0852f9db` found no conflicts:
the AMD-only commit adds two spec files. It does not validate the laptop-local
merge or a future merge involving the documentation corrections. Neither PR has
been merged or edited as part of this review.

## Follow-up measurements

For stronger workshop claims, repeat billion-key measurements on one recorded
revision with three full runs per supported engine and explicit failures for
unsupported configurations. Capture residency/paging evidence if explaining the
desktop slowdown, and archive the historical profiling captures before presenting
their stage timings as reproduced results. These follow-ups need not delay a
documentation review that clearly labels the existing evidence's limits.
