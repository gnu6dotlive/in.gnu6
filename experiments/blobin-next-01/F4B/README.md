# BLOBIN-NEXT-01 / F4B — GPU lifetime, ROI compositor and bounded recovery

**Private original experiment.** This is an isolated clean-room test harness
built from the already qualified F1–F4 path dependencies. It does not modify
the historical blob.in donor, Gnosix, in.gnu6, any public source or production runtime.

## Scope actually implemented

F4B creates a **persistent wgpu/Vulkan device, compute pipeline and output
buffer**. It accepts a fully-computed F4 reference image and dispatches only
the indicated rectangular region to copy changed presentation pixels into
its resident output. The GPU compute dispatch grid is sized to the ROI.
Its host-side oracle rejects underdeclared damage before any submission.

The **F4 effect shader itself is not evaluated regionally**. The source is a
full-frame F4 CPU result and F4B uploads the full image and reads back the full
target for comparison. This laboratory proves **region compositing and guarded
submission semantics**, NOT a production-quality dirty-region effects pipeline,
GPU partial source upload, hardware zero-idle power or GPU frame p95.

Empty damage causes no explicit `queue.submit`, command encoder or mapping
in this F4B update method; every no-work iteration checks monotonic counters.
These counters do not measure internal driver submissions or global GPU idle.
A conservative oracle scans all pixels on the CPU; dirty **discovery is not yet
incremental**.

An application-controlled `wgpu::Device::destroy()` test verifies that the
destroyed context cannot submit another F4B update, then continues with the
same semantic scene and image bytes on CPU. It creates a fresh Vulkan device
from the CPU snapshot and proves GPU parity on another region update.
**This is an intentional device destruction, NOT physical device loss,
GPU hang recovery, OS driver reset or a submission interrupted mid-flight.**
No GPU reset/driver changes are made to the workstation.

Reduced-motion input uses F4's existing reduced-motion path. The F4B
presentational overlay does not accumulate a temporal history of its own.
No telemetry, network access, hidden input or public API is introduced.

## Reproduce

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

The fail-closed absence test sets `VK_ICD_FILENAMES` to a missing JSON path
within the child process: `--mode auto` must fall back to CPU;
`--mode gpu` must report a non-zero GPU_REQUIRED error.

A single-run p95 is computed from host wallclock around each GPU patch:
source buffer creation/upload, one ROI dispatch, full-frame copy, mapping
and CPU readback are all included. It is **not a GPU timestamp, sustained
frame latency, power budget, p95 across system loads or F4 effect rendering**.

Evidence: `fixtures/qualification-20261008/` (four TSV measurement ledgers,
four summary files and one SHA256 manifest). The offline validator checks
every row, semantic invariants, boundaries, explicit submission increments,
file hashes and a negative falsifier.

The next gate is a qualified **incremental F4 effect evaluation** with
true bounded invalidation from F2/F3 semantics, GPU error callback or device-loss
monitoring where safe, warm steady-state presentation and multi-device proofs.
No release, merge, host integration or public publication is authorized.
