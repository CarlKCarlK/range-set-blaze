<!-- todo​0 consider deleting this handoff once the portability work is complete. -->

# AMD portability session handoff

Work on branch `amd-gpu-portability`. It originally branched from `gpu-on-simd`
at `ce4580c65430b1b7ed199ce21479d7678f1a7d52`. The November 14 integration
combines laptop history through `fb0bfa0b` with reviewed desktop work through
`b97205e7`, preserving both histories.

The desktop work now includes NVIDIA and AMD offload, direct HIP launches,
CPU radix sorting, and repeated 100M measurements. The laptop supplies AMD
hardware validation and native Windows/WSL measurements.
Read [the experiment procedure](AMD_NVIDIA_PORTABILITY_SPEC.md) and root
`AGENTS.md` before starting.

The implementation supports AMD RDNA wave32 as well as NVIDIA. The wgpu
cross-call scratch cache from the earlier experiment was removed for benchmark
fairness. See [CPU and GPU engine results](../experiments/GPU_ENGINES_RESULTS.md)
for the latest timings and evidence limitations, and the [offload guide](../docs/offload.md)
for toolchains, architecture selection, and runtime configuration. The earlier
[laptop measurements](../experiments/AMD_LAPTOP_BENCHMARK.md) remain historical
results from before AMD offload integration.

Use the draft portability PR's conversation for durable coordination. Each
session posts the machine and source commit, work claimed, exact commands,
results/artifact paths, limitations, and requests for the other session. Read
new comments before changing shared code. Agree on shared-file ownership
before editing; use explicit commits to exchange changes. Do not force-push
the other session's branch or merge either PR automatically.

PR comments do not wake an idle local Codex session. A human must prompt the
other session to check the PR unless a separate watcher has been configured.
The worktree can be managed over its existing SSH connection from the desktop.
No independent laptop process or PR watcher is established by this handoff.

Start on the laptop by recording Windows/WSL versions, driver, adapter,
toolchains, and existing GPU configuration. Preserve the existing Dozen setup;
the repository already contains historical Radeon controls. Validate the
current revision before tuning. CPU comparisons always use the same machine
and OS as the GPU result.

Suggested next instruction for the laptop session:

> Read AGENTS.md, specs/AMD_GPU_HANDOFF.md, and
> specs/AMD_NVIDIA_PORTABILITY_SPEC.md on branch amd-gpu-portability. Read the
> portability draft PR conversation, and inspect branch status and saved stashes
> before editing. Run GPU-required correctness checks on the integrated revision
> using the existing user-local ROCm runtime; record the exact commit and test
> outcomes. Keep native Windows and WSL2 CPU baselines separate. The earlier
> local GPU-file copies are historical snapshots; do not reapply them over the
> integrated implementation. See specs/GPU_REVIEW_HANDOFF.md for remaining work.
