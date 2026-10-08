# blob.in F3 — Materials, height, normals and synthetic heat

**Private clean-room research only.** This is not standalone blob.in product source,
not an upstream license resolution, and not a release. Original Rust/WGSL lives
here beside the qualified F2 and F1 experiments. Neither the donor snapshot nor
the in.gnu6 semantic engine is modified.

F3 preserves the F2 `Scene`, both identity values, topology epoch, five-state
transition trace and F2 raster oracle. The material backend projects the same
approximate signed contour with two organic lobes and a conditional punched hole.

This vertical slice provides four **parameter presets**, not a general MaterialGraph:
ceramic, gel, brushed metal and paper. Each combines an analytic heightfield,
finite-difference normal, directional rim light and small material-specific
grain. Synthetic heat uses two fixed Gaussian sources multiplied by an explicit
`heat_strength` parameter. There is no passive telemetry or network capture.
Images are composited in linear light using alpha once, then output as opaque
sRGB PNG over a fixed background. Translucent export, HDR and wide-gamut
color management are **not proven**.

The GPU backend uses wgpu 30 compute on Vulkan, and reads back RGBA as well as
independent analytical float buffers (height, heat, signed-field material mask,
normal Z). These fields are compared against the pure Rust CPU evaluator and
the F2 binary geometry oracle, not inferred from decorative pixels.

## Reproduction

```bash
cd experiments/blobin-next-01/F3
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target
cargo fmt --check
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo run --offline --locked -- --backend compare --dpr 1 --out /mnt/workbench/build/blobin-f3-runs
cargo run --offline --locked -- --backend compare --dpr 2 --out /mnt/workbench/build/blobin-f3-runs
cargo run --offline --locked -- --backend compare --heat 0 --dpr 1 --out /mnt/workbench/build/blobin-f3-cold
cargo run --offline --locked -- --backend auto --simulate-gpu-unavailable --dpr 2
```

The `auto` mode also recovers on adapter failure. To test absent Vulkan
without changing driver configuration, point `VK_ICD_FILENAMES` to a
nonexistent test ICD path: `auto` falls back to CPU, while `gpu` fails
explicitly.

## Parity thresholds (pre-qualification)

Max RGB channel delta ≤ 8/255; mean RGB absolute delta ≤ 0.35/255;
height, heat and normal Z max absolute delta ≤ 0.003 each. Masks must
match **exactly** with F2 independently measured geometry; every scene's
topology, IDs, epoch and digest are unchanged.

These bounds apply to the tested fixture set, **not** arbitrary topology
or GPU hardware. See `docs/agent-results/blobin-next-01/F3.md` for results.
Proof fixtures in `fixtures/qualification-20261008/` are synthetic only.

## Explicitly out of scope

Arbitrary Boolean path topology, nested holes, host/GPUI integration, real
in-flight device loss, multi-pass postprocessing, effect dirty regions,
temporal heat diffusion, anti-alias fidelity across devices, steady-state
GPU frame/p95 timings, standalone packaging, SDK, publication and license
resolution. F4 remains separate.
