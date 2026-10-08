# BLOBIN-NEXT-01 / F1 — Private clean-room renderer

**Status: isolated experiment, not blob.in production source.** This is newly authored
Rust and WGSL. No code was copied from the historical GPL/AGPL-ambiguous snapshot.
No public SDK, ABI, release, repository push or site publication is authorized.

## Goal and scope

One canonical, deterministic `FrameIr` owns identity, topology epoch, logical
dimensions, phase and focus. The CPU software implementation and GPU `wgpu`
compute shader render the same organic contour with a real hole and a beveled
focus rim. The GPU returns an offscreen RGBA buffer; Rust writes PNG images.

The field function is an *approximate* radial distance, not a mathematically exact
signed distance around all curvature. For F1 its two contour boundaries are
fixed and topological events (merge/split) **are not yet implemented**.
Output RGB is an opaque composite in nominal linear channels; PNG has no color
profile. Not yet a shipping color pipeline.

## Reproduction

```sh
cd experiments/blobin-next-01/F1
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target CARGO_BUILD_JOBS=4 cargo test --offline
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target cargo run --offline -- --backend compare --dpr 1 --out /mnt/workbench/build/blobin-f1-runs
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target cargo run --offline -- --backend cpu --dpr 2 --out /mnt/workbench/build/blobin-f1-runs
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target cargo run --offline -- --backend auto --simulate-gpu-loss --out /mnt/workbench/build/blobin-f1-runs
```

For CPU-only/no GPU, `--backend cpu` is independent of Vulkan.
`--backend auto` falls back if GPU init/render returns an error.
`--simulate-gpu-loss` is a **controlled pre-dispatch fault**, not an actual
physical device-loss test; it checks semantic preservation and CPU recovery.
`--backend gpu` and `compare` fail closed when GPU is unavailable.
No host drivers or OS packages are installed.

Expected output: `cpu-dpr1.png`, `gpu-dpr1.png`, `diff-dpr1.png` for
compare, and corresponding dpr2 files. Exact differences are measured, not
assumed. `--backend compare` fails when pixel differences exceed the
documented F1 thresholds; passing is bounded to the tested GPU and driver.

**Next gate:** test real device loss, timings and more adversarial contours;
then merge/split topology identity in E02. This F1 is not a production
GPU backend or proof of cross-hardware parity.
