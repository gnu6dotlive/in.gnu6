# GPU05 ORACLE-04 — Host stall attribution and live Wayland optical cycles

**Status:** LOCAL_NATIVE_35S_PASS / 10_SOFTWARE_OPTICAL_CYCLES_OBSERVED / PREVIOUS_413MS_CAUSE_UNKNOWN / PHYSICAL_PHOTODIODE_NOT_RUN / CP03_OPEN.

Base: `aab2fd34888c1a106c3c9274a39fcbae75134285`. Separate branch `feat/gpu05-oracle04-stall-attribution-20261010`.

## Method and isolation

One guarded `gnu6-lab-run --exclusive` Release build and **one** 35-second Vulkan run on RTX 3070, full-screen HDMI-A-1 2560×1440 ~74.964Hz. Opt-in `--host-stall-jsonl` adds independent host-loop wall-time, per-thread CPU-time and Wayland-poll timings. No replay of 125-second endurance; no compositor or service changes.

## Results

- Actual compositor presentations: **2624**, discarded: **0**, callbacks reconciled.
- Presented FPS: **74.935**. P95 **13.451ms**, p99 **13.551ms**, maximum **26.81ms**.
- Gaps >33.4ms: **0**. Previously recorded ORACLE-03 **413.544ms** failure is retained and **not explained or closed** by the non-recurrence.
- Software `wl_pointer` presses: **14**; complete button→white ROI→compositor-presented→dark ROI cycles: **10**.
- Host loop max: **3.230632ms**, per-thread CPU gap max: **0.646958ms**, Wayland polling max: **0.319754ms**, host loop outliers: **0**.
- Runtime exit 0; compositor and native render daemon remained active.
- **Input provenance is native Wayland software**, not independently authenticated human intent. No live Rust-to-TypeScript semantic routing was exercised; no photodiode/input-to-photon latency was observed.

## Evidence

Private retained run folder: `/mnt/workbench/evidence/runs/gpu05-oracle04-host-stall-20261010`.

Release SHA-256: `217422b04b6e0f7787154ad3ba69b585ebe8b4694260910e7de51550479d75f9`.

Traces and manifest: `host-stalls.jsonl`, `optical-events.jsonl`, `presentations.jsonl`, `callbacks.jsonl`, `stages.jsonl`, `traces.sha256`; passive scheduler snapshots in `passive-process-schedstat.txt`. Corresponding local analyzers: `compositor-analysis.json`, `gap-audit.json`, `optical-analysis.json`.

## Gates

| Gate | State |
|---|---|
| Source-level host stall telemetry and targeted Rust tests | PASS (6 new Rust tests, strict Clippy) |
| Native renderer 35-second presentation and optical cycles | LOCAL PASS |
| Recurrence and underlying cause of ORACLE-03 413ms pause | **UNRESOLVED** |
| Independent operator origin of real input | **UNATTESTED** |
| Live semantic Rust→TypeScript scene bridge | **NOT IMPLEMENTED** |
| External physical contact→photodiode oracle | **NOT RUN** |
| CP03 HYBRID-60 full | **OPEN** |

Do not promote this to a cross-vendor result, general input-to-photon latency, release, or full CP03.
