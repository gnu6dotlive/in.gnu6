# GPU05-ORACLE-01 — Hardware input-to-photon measurement contract

**Checkpoint:** GPU05.ORACLE01 / date 2026-10-10
**State:** SOURCE_AND_NEGATIVE_LAWS_PASS / HISTORICAL_EVIDENCE_RECONCILED / PHYSICAL_CAPTURE_NOT_RUN / CP03_OPEN

## What has been proven locally

From the separate, already closed GPU05 evidence, with no repeat of either GPU experiment:

- GPU05 PACER-07 reversed-order startup A/B:
  first frame PRESENTED with the first-feedback barrier vs DISCARDED
  without the barrier, in the reference NVIDIA/Wayland HDMI-A-1 setup.
  First-submit-to-first-present **3.204113ms versus 18.837564ms**,
  CLOCK_MONOTONIC, not a physical user input latency.
- GPU05 ENDURANCE-01: 9,370 real same-surface compositor PRESENTED
  feedbacks, no discarded feedback, 124.993938 seconds, average
  74.956 FPS, p95 gap 13.478ms, p99 13.580ms, maximum 26.650ms.
- **Independent artifact-integrity reconciliation:** the historical
  SHA-256 manifests for the three endurance traces and four
  startup traces per A/B variant were rechecked against preserved
  original files. Reconciliation status:
  GPU05_LOCAL_EVIDENCE_RECONCILED_CP03_OPEN.
  JSON report kept OUTSIDE the repo at:
  /mnt/workbench/evidence/runs/gpu05-oracle01-20261010/reconciliation.json
  Result SHA-256:
  a9018c81ae80e404051b2a83126952500719971aaef5dbd16e77404c72144751.
- These historical local observations do NOT qualify external provenance,
  physical input-to-photon latency, GPU drivers generally or CP03 FULL.

## New ORACLE-01 source and tests

- scripts/gpu05_physical_oracle.py: bounded, deterministic JSONL
  parser for independent same-hardware-timer **physical-switch**
  and **screen-photodiode** rising edges, joined by explicit trial ID.
- scripts/test_gpu05_physical_oracle.py:
  **14 new tests PASS** — only synthetic law and negative fixtures,
  including duplicate/missing trials, mixed devices or clocks,
  unbounded calibration, wrong physical channel, reversed edge time,
  nonexistent input-to-scene binding, p95 over threshold.
- scripts/gpu05_cp03_reconcile.py: read-only verification against
  previously archived SHA-256 manifests (3 endurance trace files
  vs 4 files per PACER-07 startup variant). No GPU replay.
- scripts/test_gpu05_cp03_reconcile.py:
  **9 new tests PASS**, including tampered hash manifest and
  unauthorized full-CP03 truth promotion.

**Total new targeted tests: 23/23 PASS.** This does **not** demonstrate
a physical acquisition session, a user input trace or a capture device.

## Why software timing is insufficient

A wl_surface.frame callback is only scheduling credit. The actual
wp_presentation presented timestamp is a compositor observation.
Even the PACER-07 same-clock startup measurement begins when a
frame is submitted; it does **not** begin at a user's switch closure,
and it does **not** observe the display's photons.

A webcam (C922 was detected as /dev/video0 and /dev/video1) is
helpful for visual setup inspection. Without calibrated acquisition,
shutter timing and timestamp/error bounds, it is not the accepted
independent <=50ms-p95 hardware oracle. No dual-channel physical
capture device was detected on /dev/ttyACM* or /dev/ttyUSB*.

## Required physical topology (design, NOT YET IMPLEMENTED)

Use one independent acquisition clock for two observed channels:

1. **Input:** a real physical switch closure / contact edge measured
   on a hardware timer. This same action must also cause the actual
   OS/Wayland input event consumed by the renderer, not a substitute
   software trigger. The input capture must not create a separate
   side path that bypasses the Gnosix/in.gnu6 scene pipeline.
2. **Output:** an externally positioned photodiode / comparator,
   physically facing a precisely defined display pixel patch,
   measuring the transition from DARK to BRIGHT.
3. **Clock:** one calibrated hardware timebase, common to input and
   photon edges. Record counter tick frequency, calibration ID,
   worst-case aggregate uncertainty, sensor thresholds, trigger
   debounce, and hardware identification. Keep capture raw,
   calibration data and independent observer's wiring record outside
   the public source tree until redaction.
4. **Renderer:** implement a single real input-to-scene transition
   controlling the optical target. The current GPU05 showcase has
   static or freely animated geometry; it has **NOT** been proved
   to change a measured optical region in response to user input.
   Disable unrelated animation of the target during a trial.
5. **Trials:** >=30 real button-to-light pairs, full dark reset
   between trials, all trials retained including slow and failed
   responses. The operator/external reviewer checks actual wiring,
   correct input/scene path, capture origin, and calibration.

Strict parser schema, one JSON object per line:

```json
{"type":"header","schema":"blobin.gpu05.optical-input.v1","timebase":"shared-hardware-counter","tick_hz":1000000,"total_calibrated_uncertainty_us":2000,"capture_unit_id":"reference-dual-channel-capture-01","calibration_record_id":"calibrated-session-01","input_sensor":"physical-switch-contact","light_sensor":"external-photodiode","display_transition":"dark-to-bright","scene_response":"physical-switch-causes-target-pixel-brightening"}
{"type":"edge","trial":1,"channel":"switch","edge":"rising","tick":100000,"capture_unit_id":"reference-dual-channel-capture-01"}
{"type":"edge","trial":1,"channel":"light","edge":"rising","tick":113500,"capture_unit_id":"reference-dual-channel-capture-01"}
```

**The above is a SCHEMA EXAMPLE, NOT a real observation.**
The parser reports each latency in milliseconds and a conservative
bound `p95 + calibrated aggregate uncertainty`.
The numeric threshold is p95 upper bound <=50ms with at least 30
qualified pairs. The parser sets
`independent_capture_provenance_qualified=false` and
`cp03_pass=false` **even if the synthetic metrics are perfect**.

### CLI for future hardware observations

```bash
python3 scripts/gpu05_physical_oracle.py /path/to/hardware-capture.jsonl \
  --output /path/to/optical-metrics.json
```

The optional optical JSON report can subsequently be passed to
`gpu05_cp03_reconcile.py --optical-report`. No acceptance is
granted by copying self-reported latency into the compositor's
`input_latency` field.

## Gates and next causal work

| Gate | Actual status |
|---|---|
| Program-level local 125s compositor cadence | LOCAL PASS |
| PACER-07 reversed startup comparison | LOCAL PASS |
| Original trace SHA-256 integrity and reconciliation | PASS |
| Dual-channel measurement law tests | 14/14 PASS (synthetic) |
| Reconciliation negative law tests | 9/9 PASS |
| External dual-channel optical hardware | UNAVAILABLE / NOT CONNECTED |
| Actual hardware-button → Wayland → scene → optical ROI | NOT IMPLEMENTED |
| Hardware calibration and physical input-to-photon capture | NOT RUN |
| Independent technical witness / corroborated provenance | NOT QUALIFIED |
| CP03 HYBRID-60 full | OPEN |

**Next:** implement the bounded input event-to-scene optical test fixture,
then connect and calibrate the real dual-channel external sensor. Run
only new targeted short real tests via gnu6-lab-run --exclusive.
Do **not** repeat 125s endurance without a different falsifiable gate.

No public package distribution, scene authority amplification, main merge,
compositor service mutation, or license promotion is authorized by this work.
