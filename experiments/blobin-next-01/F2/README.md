# BLOBIN-NEXT-01 / F2 — typed topology events, renderer parity

**Status:** private, original Rust/WGSL research. F2 inherits F1 as a path dependency
without rewriting or ratifying historical blob.in GPL/AGPL snapshot source.

Canonical sequence: `Split(2 components,0 holes)` → `Merge(1,0)` →
`PunchHole(1,1)` → `SealHole(1,0)` → `Split(2,0)`.
Semantic identities of two lobes are constant; each valid transition advances a checked
epoch and emits a deterministic before/after event. Invalid transitions do not mutate.
The engine owns hit-test and topology state. The GPU only receives render layout.

An independent flood-fill oracle measures 4-connected material components and
bounded background holes in the canonical mask, actual CPU PNG and actual GPU PNG.
This qualifies raster topology **at the tested resolution**, not all continuous
geometries. One deliberately altered foreground mask must fail the oracle.
A hollow region must never grant a hit target.

```bash
cd experiments/blobin-next-01/F2
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target cargo test --offline --locked
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target cargo run --offline --locked -- --backend compare --dpr 1 --out /mnt/workbench/build/blobin-f2-runs
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target cargo run --offline --locked -- --backend compare --dpr 2 --out /mnt/workbench/build/blobin-f2-runs
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target cargo run --offline --locked -- --backend auto --simulate-gpu-unavailable
```

Output: 5 CPU and 5 GPU frames and difference images per DPR, plus two
machine-readable ledgers per DPR. No network or OS changes necessary. `gpu` /
`compare` must fail closed with no GPU; `auto` preserves semantic ledger with
CPU-only rendering. The injected failure is **pre-dispatch**, not a physical
mid-frame device loss.

*Limitations*: transition topology is explicitly defined by the scene engine;
it is not inferred from changes to a shader field. The signed field is an
approximation; these two organic lobes do not yet cover self-intersection,
tangency, 3+ components, variable winding, nested holes or dynamic arbitrary
Boolean path graphs. The GPU/CPU pipeline is still offscreen only; host/GPUI
integration, real device-loss handoff, image color space and frame p95 are not proven.
