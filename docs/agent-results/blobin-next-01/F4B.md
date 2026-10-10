# BLOBIN-NEXT-01 / F4B — Hard resilience, scoped evidence

**Status:** `F4B_PERSISTENT_ROI_COMPOSITOR_VERIFIED / CPU_RECOVERY_CONTROLLED_VERIFIED / PHYSICAL_DEVICE_LOSS_NOT_RUN / EFFECT_DIRTY_EVALUATION_OPEN`

**Date:** 2026-10-08. **Baseline:** private qualified F4 commit `1082d5bc62438a9a2bac3c5fd095e75a5176ff14`.
**Branch:** `feat/blobin-next-01-f4b-resilience`.
**Worktree:** `/mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-f4b-resilience`.
**Scope:** exclusively `experiments/blobin-next-01/F4B/**` and this report. F1–F4 are path dependencies left unchanged. The historical GPL/AGPL-ambiguous donor source is not copied or modified.

## Implemented and qualified

- A *persistent* `wgpu 30.0.1` Vulkan device, compiled compute pipeline and full-frame storage target for region compositing. The shader dispatch grid covers only the validated ROI, rather than the full viewport. Region bounds, byte lengths and maximum pixel allocation are checked.
- CPU oracle verifies **every changed pixel** is inside the declared damage rectangle. An underspecified rectangle turns the gate RED *before* any GPU submission. A no-change event yields an empty rectangle, and the harness does not call its `queue.submit`, encoder or map-read path. This does not measure internal driver activity or true cold GPU idle power.
- A fully rendered **F4 CPU reference image** is the presentation source. F4B GPU copies the changed display pixels into a resident target. Importantly, this is **not incremental F4 shadow/blur/bloom evaluation**: CPU source calculation, change detection, GPU input upload and verification readback currently operate on full frames.
- A safe, application-initiated `wgpu::Device::destroy()` on the dedicated lab device yields a deterministic fail-closed update attempt. The authoritative scene digest and image continue on CPU, followed by a successful new GPU-device acquisition and further region update. This demonstrates an owned-context teardown/recovery policy, **not physical GPU loss mid-dispatch**.
- The F2 scene digest and topology are preserved. Reduced-motion initial F4 scene uses the qualified F4 reduced-motion implementation; F4B itself adds no temporal echo buffer.

## Actual gate outcomes

| Gate | Observation | Interpretation |
|---|---|---|
| F4B unit tests | **6/6 PASS** | Empty damage, exact box, adversarial underdeclared bounds, integer overflow/outside viewport, F2 digest inheritance, Naga WGSL |
| `cargo fmt --check` | **PASS** | Rust formatting |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | **PASS** | Strict lints |
| DPR1 Vulkan normal | **20/20 GPU patches + 20 idle checks** | 256×192 canvas, actual ROI 12×10; app-controlled destroy → CPU continuity → new GPU |
| DPR2 Vulkan normal | **24/24 GPU patches + 24 idle checks** | 512×384 canvas, actual ROI 24×20; app-controlled destroy → CPU continuity → new GPU |
| DPR1 Vulkan reduced-motion | **24/24 GPU patches + 24 idle checks** | F4 reduced-motion source; no extra F4B history |
| Adapter unavailable `VK_ICD_FILENAMES` override + `auto` | **8/8 CPU patches, 8 idle checks** | No available Vulkan adapter in process, semantic digest retained |
| Same adapter override + `gpu` | **EXPECTED FAILURE**, exit 1 | `F4B_GPU_REQUIRED`, no fake GPU result |
| Fixture validator | **PASS** | 76 trace rows, eight files/8 SHA-256, non-constant falsifier rejected |
| Single-run host-observed p95 DPR1 normal | **1.362463 ms** (20 samples) | ROI submit, source upload, *full-frame* readback and CPU map included |
| Single-run host-observed p95 DPR2 normal | **4.006895 ms** (24 samples) | Same path at DPR2 |
| Single-run host-observed p95 DPR1 reduced | **1.315456 ms** (24 samples) | Same path with reduced-motion source |

The timestamps are **CPU wall-clock elapsed durations**, not GPU timestamps, hardware frame times or rigorous p95 statistics across devices and workload distributions. The sample sizes are small and the device is only the local NVIDIA RTX 3070 under Vulkan. No GPU power/temperature/energy, no cold-idle wattage and no pixel shader p95 claim is made.

The adapter/device teardown did **not** interrupt any pending compute submission: tests await readback completion first, then explicitly destroy the owned device. Reacquisition establishes another context from the CPU-authoritative snapshot. A real external mid-flight device failure remains **NOT_RUN**.

## Reproduction and immutable evidence

Lab: `experiments/blobin-next-01/F4B/`.
Fixtures: `experiments/blobin-next-01/F4B/fixtures/qualification-20261008/` containing four TSV ledgers, four text summaries and `SHA256SUMS.txt`.
Offline verifier: `experiments/blobin-next-01/F4B/scripts/validate_f4b.py`.

```sh
cd experiments/blobin-next-01/F4B
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target
cargo fmt --check
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
python3 scripts/validate_f4b.py
cargo run --offline --locked -- --mode compare --dpr 1 --samples 20 --destroy-owned-device --out /mnt/workbench/build/blobin-f4b-evidence
cargo run --offline --locked -- --mode compare --dpr 2 --samples 24 --destroy-owned-device --out /mnt/workbench/build/blobin-f4b-evidence
cargo run --offline --locked -- --mode compare --dpr 1 --samples 24 --reduced-motion --out /mnt/workbench/build/blobin-f4b-evidence
```

To check absent Vulkan **without changing system drivers**, set the `VK_ICD_FILENAMES` environment variable to a nonexistent temporary file for the child process. `--mode auto` must use CPU, `--mode gpu` must fail nonzero. No kernel/driver, desktop, GPU processes or host services were reset.

## Unclosed resilience and production gates

**Still NOT PROVEN:** external GPU device-loss notification and recovery from an in-flight fault; dirty-driven F4 *effect computations* instead of only presentation compositing; truly partial GPU buffer upload/readback; dirty-area invalidation from scene/motion rather than an exhaustive CPU oracle; GPU timestamp/warm steady-state p95; physically measured idle and power; integration with GPUI or in.gnu6; arbitrary multi-blob topology, HDR/transparency and full a11y validation.

There is **no permission to merge, push, publish, release, modify F1–F4 or deploy any runtime**. This tranche closes the F4B *owned-context and ROI-compositing subgate*, not the entire hard-resilience program.
