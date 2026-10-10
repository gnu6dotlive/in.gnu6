# BLOBIN-NEXT-01 / F4C — Real incremental GPU effect evaluation

**Status:** private, clean-room research, not a production integration or release.

F4C owns a persistent `wgpu 30.0.1` Vulkan effect graph with four persistent
intermediate storage buffers. It performs bounded compute dispatches for the
actual **shadow, box blur, bloom and temporal echo operations**. For a source
RGBA rectangle change, the spatial invalidations are:

- shadow: exact changed source region (F2 mask unchanged);
- blur: source region expanded by the box-filter radius;
- bloom: source region expanded by the box-filter radius;
- echo: same spatial region when reduced-motion disables history; **full frame
  when previous-frame echo is active**, even if the source itself is unchanged.

Dirty rectangles are verified against actual RGB(A) changes before dispatch.
Each GPU intermediate stage is compared to a separately written CPU **full-frame
reference**. Unlike F4B, this proves localized *effect shader evaluation*, not
only a rectangular display copy.

**Important scope boundary:** F3 source generation, GPU source upload, CPU
source-damage discovery and all four verification readbacks are still **full
frame**. The new graph also constructs a device per test case, not per
individual update. No end-to-end runtime speedup is claimed. F4C demonstrates
that the spatial effect *dispatch areas* are bounded, not that a complete
frame is cheap. F2 semantic identity and topology are unchanged; after a
topology epoch transition the graph is initialized fully with a new mask.

```sh
cd experiments/blobin-next-01/F4C
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo fmt --check
python3 scripts/validate_f4c.py
cargo run --offline --locked -- --dpr 1 --reduced-motion --out /mnt/workbench/build/blobin-f4c-evidence
cargo run --offline --locked -- --dpr 1 --out /mnt/workbench/build/blobin-f4c-evidence
cargo run --offline --locked -- --dpr 2 --reduced-motion --out /mnt/workbench/build/blobin-f4c-evidence
cargo run --offline --locked -- --dpr 2 --out /mnt/workbench/build/blobin-f4c-evidence
```

Evidence: `fixtures/qualification-20261008/` has four trace TSVs and SHA-256.
The validator checks 80 independent state/material/DPR/motion cases and
rejects a tampered invalidation extent.

**Open:** physically lost GPU mid-submit; progressive source generation and
partial upload/readback; GPU timestamp, p95 and cold idle power; exact
multi-blob geometry, user-facing UI integration, and production fallback
policy. F4B's CPU recovery evidence is not silently promoted to F4C.
