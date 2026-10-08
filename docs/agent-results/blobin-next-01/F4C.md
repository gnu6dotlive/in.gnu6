# BLOBIN-NEXT-01 / F4C — Incremental Effect Evaluation

**Status:** `GPU_EFFECT_ROI_IMPLEMENTED / CPU_GPU_FOUR_STAGE_PARITY_VERIFIED / PRODUCTION_NOT_CLAIMED`
**Branch:** `feat/blobin-next-01-f4c-incremental`
**Baseline:** F4B `c5f792d1f5e9fb4e25baa9e2be38ed1b8d69a91b`
**Private worktree:** `/mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-f4c-incremental`

## Demonstrated

- Original Rust and WGSL in `experiments/blobin-next-01/F4C/`.
- GPU graph retains four intermediate RGBA buffers. Every compute pass
  dispatches its own region of interest; old pixels outside its invalidated
  region remain resident rather than being recomputed by shader invocations.
- Regions are discovered by a strict CPU pixel-delta oracle. Underdeclared
  damage is rejected. For fixed F2 geometry, blur/bloom each expand source
  damage by radius 3; echo expands to full frame only with active temporal
  feedback. In reduced-motion mode, an unchanged frame causes **zero
  explicit GPU submissions**.
- After any topology change, a new graph is fully initialized. IDs and
  semantic digests remain owned by F2, never minted by F4C.
- A CPU full-frame reference implements the *same numeric effect contract*
  independently. All four GPU stages are read back and compared pixelwise,
  rather than comparing only the final composition.

## Qualification observed on RTX 3070, NVIDIA Vulkan

| Gate | Observed |
|---|---|
| Rust unit tests | **5/5 PASS** |
| WGSL parser and semantic validation (Naga) | **PASS** |
| Clippy strict, formatter | **PASS** |
| DPR1, reduced motion | 20/20 cases PASS, max RGB stage error **0**, 20 idle probes |
| DPR1, temporal echo | 20/20 cases PASS, max stage error **0**, echo recomputed full frame when history active |
| DPR2, reduced motion | 20/20 cases PASS, max stage error **0**, 20 idle probes |
| DPR2, temporal echo | 20/20 cases PASS, max stage error **0** |
| Independent fixture validator | 80 cases, 4 manifests, 4 pass oracles, negative ROI corruption rejected |

Four 20-row TSV ledgers with hashes live in
`experiments/blobin-next-01/F4C/fixtures/qualification-20261008/`.
The maximum summed *effect shader pixel areas* relative to four full-frame
passes, for the specific 12×9 logical region patch:

- DPR1 reduced-motion: **0.004669** (0.47%).
- DPR1 echo: **0.253296** (25.33%, because echo requires a full frame).
- DPR2 reduced-motion: **0.003296** (0.33%).
- DPR2 echo: **0.252380** (25.24%).

These ratios are **not speedups**, measured GPU durations, energy savings or
total system work. Input generation, damage detection, upload and full
verification readback remain whole-frame. Some empty/unchanged scenes with
active echo **must** perform a full echo dispatch to preserve causality.

The code uses `wgpu=30.0.1`, pinned Cargo.lock, offscreen compute. It does
not establish F4C's absent-adapter CPU fallback: that is previously qualified
for F4B but has not been integrated into this GPU-only F4C test harness.

## Falsification and provenance

```sh
cd experiments/blobin-next-01/F4C
python3 scripts/validate_f4c.py
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target cargo test --offline --locked
```

F4C's source was written independently. No code was copied from the historic
ambiguous GPL/AGPL blob.in snapshot and none of the F1–F4B source files were
edited. **No push, merge, deployment, public documentation update or release.**

## Gate still open

The full pipeline needs source-aware dirty propagation back into the **F3
material/geometry evaluation**, GPU partial uploads/readbacks, host event
integration, fault-handling for a genuinely lost in-flight device, repeatable
frame/power benchmarks and strict cross-adapter parity. CUDA geometry research
is isolated on another branch and is **not** a product backend yet.
