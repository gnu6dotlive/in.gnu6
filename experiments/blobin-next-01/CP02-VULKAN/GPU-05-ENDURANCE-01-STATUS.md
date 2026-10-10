# GPU05-ENDURANCE-01 — Protected 125-second Wayland presentation

**Date:** 2026-10-10 (EDT)
**State:** COMPOSITOR_PRESENTATION_125S_LOCAL_PASS / ZERO_DISCARDS / CP03_FULL_OPEN
**Branch:** feat/blobin-gpu05-endurance-20261010
**Source baseline:** 6441890ed310f63c0f2706e2b4d2ef9155535cee (PACER-06)

## Controlled scope

This is an isolated worktree from the already committed PACER-06 baseline.
It does not contain, edit, or inherit the concurrently active PACER-07
reversed-A/B and startup-latency changes. No source or shader changes were
needed for the endurance test.

GPU: NVIDIA RTX 3070; native Rust/wgpu 30 Vulkan Wayland host;
signed tetra MeshSolid/SDF shader; read-only semantic scene fixture;
pinned Wayland output HDMI-A-1, compositor 74.964Hz, 2560x1440 fullscreen.
Startup variant: await-first-feedback with immediate (0ms) settling.

## Safety

- gnu6-lab-run --check: GNU6_LAB_READY.
- Lab shared memory.high 18GiB, memory.max 22GiB, swap.max 2GiB.
- One independent offline Rust Release build and one 125-second physical
  run executed in gnu6-lab-run --exclusive; light commands under launcher.
- No automatic retry on 73/75; no service, systemd-oomd, Hyprland,
  CUDA-05F, repository main, external deployment or production change.
- Hyprland PID 2544527 and blobin_renderd PID 1974196 survived.
- No new oomd kill logged during this experiment.
- The separate memory.current sampling path was inaccessible from
  the sampling shell (all samples unknown). Do NOT claim a measured
  continuous RAM peak. The process RSS was ~98MiB at spot checks.

## Physical results — local compositor feedback

Evidence location:
/mnt/workbench/evidence/runs/blobin-next-01/gpu05-endurance-20261010/

Normal exit: runtime=125.019s, submitted=9370, wp_presented=9370,
discarded=0, feedback_pending=0, callbacks=9370, callback_pending=0,
max_inflight=2, WSI timeouts=0, WSI occluded=0,
watchdog trips=0, aborted=false, exit 0.

| Gate dimension | Actual result | Local finding |
|---|---:|---|
| Compositor-presented frames | 9,370 | minimum 7,200 met |
| First-to-last presentation duration | 124.993938 s | minimum 120 s met |
| Real compositor-presented FPS | 74.956 | minimum 60 met |
| Frame interarrival p95 | 13.478 ms | <=17ms met |
| Frame interarrival p99 | 13.580 ms | <=20ms met |
| Longest compositor interval | 26.650 ms | <=33.4ms met |
| Compositor discarded feedbacks | 0 | zero discard met |
| Same-surface callback/frame identities | 9,370 / 9,370 | exact partition |
| Independently calibrated input-to-photon samples | 0 | NOT QUALIFIED |
| Independent technical auditor | None | NOT QUALIFIED |
| Supervised production CUDA-05F ingress | Not tested | NOT QUALIFIED |

The independent JSONL analyzer returned full frame/callback/stage
identity consistency and CP03 false. The numeric gate explicitly
reported NOT_QUALIFIED, input_samples_count=0, cp03_pass=false.
Do not infer input-to-photon from queue.present, callback timing,
or unrelated CPU-vs-compositor clock domains.

## SHA-256 fingerprints

Release binary:
49b59f4b5691d71154dd45e3e9caebbdeb0121353c50ea173c256803f11dda0e

Native compositor presentations JSONL:
d23020138222ec901a4717bb9dfd43b7fa88b25e333e4f28c63bf0240d38f8f1

Same-surface callbacks JSONL:
b4b59c4e566dc215380f3298fea44fae49bb10ed70747a2088ee7b689a706083

CPU stage JSONL:
9318cad562579c6875527263199bf722d5c84d5b56fa0ad09fb7915f00e297e2

Other evidence: binary.sha256, source-assets.sha256, source-commit.txt,
traces.sha256, build.log, run.log, analysis.json,
cp03-numeric-gate.json, window.json, monitors-before.json,
hypr-before.txt, hypr-after.txt, renderd-after.txt, cgroup.txt,
gpu-before.txt, gpu-after.txt, memory-samples.log.

## Parallel PACER-07 correlation (read-only observation)

The other agent's independent reversed A/B is archived at:
/mnt/workbench/evidence/runs/blobin-next-01/gpu05-pacer07-reversed-20261010/

Reversed order: await-first-feedback first, current second.
- With the first-feedback barrier: frame 1 PRESENTED, zero discarded,
  first actual submit-to-presentation ~3.204 ms, ~74.798 presented FPS.
- With current behavior: frame 1 DISCARDED, one discarded,
  first actual presented frame is sequence 2 at ~18.838 ms,
  ~74.964 presented FPS.
- Both were run with the same monitor and captured source/binary hashes.
- The first-present clock uses CLOCK_MONOTONIC and an actual
  wp_presentation clock ID; this is NOT input-to-photon latency.

The other agent's PACER-07 worktree remained unchanged by this task;
it was still uncommitted when last observed. Do not merge it until
its owner completes and qualifies its own work.

## Verdict / next gates

**Local 125-second compositor-presented cadence and zero-discard evidence
is PASS for that narrow dimension. Full CP03 HYBRID-60 is still OPEN.**

Remaining: independent calibrated input-to-photon oracle, external
attribution/review, stress/independent E2E and the exact supervised
runtime gates prescribed by the ratified CP03 contract. The standalone
GPU demonstration does not grant CUDA-05F worker authority.
CP02 full, CP07 and CP08 stay separately OPEN.

Do not repeat the already completed 125-second baseline without
a new falsifiable purpose. Do not push, merge, deploy or advertise
a full CP03 PASS based on this local result.
