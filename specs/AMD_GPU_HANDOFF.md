<!-- todo​0 consider deleting this handoff once the portability work is complete. -->

# AMD portability session handoff

Work on branch `amd-gpu-portability`. Its parent is `gpu-on-simd` at
`ce4580c65430b1b7ed199ce21479d7678f1a7d52`.

The desktop session owns the NVIDIA offload implementation and shared wgpu
resource-cache changes. The laptop session owns AMD hardware validation,
CPU/SIMD measurements, Windows/WSL wgpu measurements, and an AMD offload probe.
Read [the experiment procedure](AMD_NVIDIA_PORTABILITY_SPEC.md) and root
`AGENTS.md` before starting.

The desktop's new offload and resource-cache changes were still uncommitted
when this sister branch was created. This branch does not yet contain them.
Before a matched comparison or an AMD offload port, ask the desktop session for
the published implementation commit and merge that commit into this branch.
Do not claim the initial branch already contains the new offload backend.

Use the draft portability PR's conversation for durable coordination. Each
session posts the machine and source commit, work claimed, exact commands,
results/artifact paths, limitations, and requests for the other session. Read
new comments before changing shared code. Agree on shared-file ownership
before editing; use explicit commits to exchange changes. Do not force-push
the other session's branch or merge either PR automatically.

PR comments do not wake an idle local Codex session. A human must prompt the
other session to check the PR unless a separate watcher has been configured.
No watcher or autonomous laptop session has been started by this setup.

Start on the laptop by recording Windows/WSL versions, driver, adapter,
toolchains, and existing GPU configuration. Preserve the existing Dozen setup;
the repository already contains historical Radeon controls. Validate the
current revision before tuning. CPU comparisons always use the same machine
and OS as the GPU result.

Suggested initial instruction for the laptop session:

> Read AGENTS.md, specs/AMD_GPU_HANDOFF.md, and
> specs/AMD_NVIDIA_PORTABILITY_SPEC.md on branch amd-gpu-portability. Read the
> portability draft PR conversation, post an AMD-session check-in, and begin
> environment inspection and correctness checks. Coordinate shared changes
> through that PR. Run native Windows commands from WSL2. Check with the
> desktop session for its published implementation commit before benchmarking
> the new cached wgpu path or attempting to port the offload backend. Record
> each machine's own CPU baseline; never label software rendering as GPU work.
