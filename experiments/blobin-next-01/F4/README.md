# blob.in F4 — Explicit effects graph & bounded fallback (private laboratory)

**Scope:** newly authored F4 crate and shader only. No historic blob.in donor source
is copied, relicensed, or imported. No changes to F1–F3. No public SDK, mainline
integration, deployment, or publication authority.

## Proven implementation

F4 receives a qualified F3 scene/material image and F2 semantic mask. It
runs four ordered effect passes: directional **shadow**, bounded separable
**box blur**, threshold **bloom**, and temporal **echo**. The CPU reference
runs four ordered Rust functions. On the GPU, each pass is a **distinct Vulkan
compute dispatch** with explicit ping-pong storage buffers. A newly rendered
F3 image provides the input; F2 owns IDs, hit targets, epoch and topology.

The two-frame fixture tests actually exercise temporal history: the same scene
identity is rendered with two synthetic heat levels. On epoch changes, history
resets. With `--reduced-motion`, the echo pass does not store or consume
history, and output is deterministic. History is bounded to one RGBA frame.
The CPU path works with no Vulkan ICD; `auto` selects it if GPU access fails
while `gpu` and `compare` fail closed.

The typed plan validates pass order and effect radius (≤8 px), limits the
physical image to 1,048,576 pixels and reports a clipped conservative dirty
rectangle (2 + 2 × blur radius halo). **F4 computes full-frame passes**:
actual partial rasterization, scissoring and no-idle-dispatch behavior
remain **NOT_IMPLEMENTED / NOT_PROVEN**. CPU/GPU shaders implement a
separable **box filter**, NOT a bilateral filter or a final production bloom
quality profile. The renderer is offscreen and recreates Vulkan devices per
sample: no frame-p95 or energy claim is legitimate.

## Reproduce on qualified workstation

```sh
cd experiments/blobin-next-01/F4
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target
cargo test --offline --locked
cargo fmt --check
cargo clippy --offline --locked --all-targets -- -D warnings
python3 scripts/validate_f4.py

cargo run --offline --locked -- --backend compare --dpr 1 --out /mnt/workbench/build/blobin-f4-runs
cargo run --offline --locked -- --backend compare --dpr 2 --out /mnt/workbench/build/blobin-f4-runs
cargo run --offline --locked -- --backend compare --dpr 1 --reduced-motion --out /mnt/workbench/build/blobin-f4-reduced
cargo run --offline --locked -- --backend auto --simulate-gpu-unavailable --dpr 2 --out /mnt/workbench/build/blobin-f4-injected
```

For missing Vulkan, run `--backend auto` with `VK_ICD_FILENAMES`
set to a nonexistent test ICD JSON path. The full semantic CPU trace must
still qualify. With `--backend gpu`, the same condition must exit nonzero.
No host drivers are modified.

**Predeclared color budget:** max RGB channel difference ≤12/255 and mean
absolute channel difference ≤0.5/255; source geometry mask and semantic
digest must match F2 *exactly*; 4 dispatches on each GPU frame.

## Evidence

`fixtures/qualification-20261008/standard` (DPR1/DPR2) and
`fixtures/qualification-20261008/reduced` (DPR1) contain 360 synthetic PNG
images, three ledgers and `SHA256SUMS.txt`. These are fully reproducible,
independently validated (including a falsifying hole-count mutation), and
limited to the declared RTX 3070/Vulkan hardware.

Full qualification: `docs/agent-results/blobin-next-01/F4.md`.

## Open gates (do not silently qualify)

- A real **in-flight physical GPU device loss**, then CPU continuity, is
  NOT RUN. Adapter unavailability and pre-dispatch injection are not that.
- True **incremental dirty-region execution**, zero GPU submissions in idle
  state, reduced-motion UX integration, AA/premult fidelity beyond opaque
  sRGB PNG, bilateral blur, spatial/temporal profiler p95 and multi-device
  consistency are NOT PROVEN.
- Arbitrary nested shapes, 50 simultaneous blobs, real host/GPUI adapter and
  historical license reconciliation remain separate work.

**This laboratory qualifies F4's bounded effect graph, but does not close
the unproven production resilience gates.**
