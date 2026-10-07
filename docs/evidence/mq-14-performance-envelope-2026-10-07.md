# MQ-14 receipt — performance and scale envelope

Date: 2026-10-07
Status: **RATIFIED**
Quest: **MQ-14**
Instrumentation source HEAD: `a2f5adbde7c220cf822554f4cf24a3586328e21c`

## Claim

A reproducible performance envelope now exists for the current [in.gnu6] mandate.

No hard product thresholds were invented.

The measurements characterize current behavior on the observed development host and browser instrument.

## Core instrument

Runtime:
- Node: v24.20.0
- platform: linux x64
- CPU: 12th Gen Intel Core i7-12700F
- logical CPUs: 20
- system memory: ~33.4 GB

Raw receipt:
`docs/evidence/assets/mq-14-core-envelope.json`

### Fixed-step scaling

Median / p95 milliseconds per fixed semantic step:

- 24 entities: ~0.021 / 0.035 ms
- 128 entities: ~0.037 / 0.157 ms
- 512 entities: ~0.131 / 0.199 ms
- 2048 entities: ~0.596 / 0.961 ms

### Field evaluation

Radial field aggregation remained sub-0.02 ms across the measured 8/32/128-field points.

### Contour dynamics

Contour integration median:

- 16 nodes: ~0.0009 ms
- 64 nodes: ~0.0019 ms
- 256 nodes: ~0.0074 ms
- 1024 nodes: ~0.0294 ms

Contour retarget median:

- 16 nodes: ~0.0036 ms
- 64 nodes: ~0.0133 ms
- 256 nodes: ~0.145 ms
- 1024 nodes: ~2.18 ms

Retargeting is materially more expensive than steady contour integration at high node counts.

### Picking

Topmost picking median:

- 100 occupants: ~0.0007 ms
- 500 occupants: ~0.0035 ms
- 2000 occupants: ~0.0141 ms
- 5000 occupants: ~0.0349 ms

### Representative consumer workloads

Measured layout/influence operation:

- Docklab 24 cards: ~0.0042 ms median
- SP Joaillerie 12 cards: ~0.0015 ms median

These are semantic layout operations, not full browser frame costs.

## Observer overhead

At 256 entities:

- semantic step alone: ~0.055 ms median
- semantic step + full SceneReceipt: ~19.83 ms median
- observed ratio: ~360×

The ratio is large because the baseline semantic step is very small.

The absolute result is the important fact:

> full SceneReceipt construction is diagnostic/on-demand work at this scale, not a free per-frame telemetry primitive.

This does not invalidate MQ-09 because MQ-09 requires explainable failure receipts, not mandatory per-frame observation.

## Core memory observation

Approximate heap deltas during scene construction:

- 24 entities: ~47 KB
- 128 entities: ~212 KB
- 512 entities: ~840 KB
- 2048 entities: ~3.38 MB

RSS remained flat in this process observation due to allocator/process reservation behavior.

These values are observational and not GC-normalized memory guarantees.

## Browser / Pixi reference envelope

Browser instrument:
- Chrome for Testing 153.0.8010.12
- PixiJS 8.22.0
- backend: WebGL
- ticker: stopped
- resolution: 1
- antialias: false

Raw receipt:
`docs/evidence/assets/mq-14-browser-envelope.json`

Important limitation:
the headless Chrome reference used ANGLE + Vulkan + **SwiftShader**.

Therefore the observed renderer numbers characterize the qualification environment, not physical RTX 3070 GPU throughput.

### RendererFrame build + Pixi projection

Mean milliseconds:

- 16 entities:
  - frame build ~0.033 ms
  - Pixi render ~0.130 ms
- 64 entities:
  - frame build ~0.142 ms
  - Pixi render ~0.197 ms
- 256 entities:
  - frame build ~1.60 ms
  - Pixi render ~0.775 ms
- 512 entities:
  - frame build ~6.02 ms
  - Pixi render ~1.47 ms
  - Pixi p95 ~1.80 ms

At the highest measured browser cardinality, RendererFrame construction costs more than the Pixi projection itself.

This is useful optimization guidance but not yet a product threshold.

### Browser JS heap

Observed used JS heap:

- 16 entities: ~20.6 MB
- 64 entities: ~21.0 MB
- 256 entities: ~26.4 MB
- 512 entities: ~51.9 MB

This is browser JS heap only.

It does not measure GPU memory.

## Invariant preservation after instrumentation

After performance characterization:

- format: PASS
- lint: PASS
- typecheck: PASS
- unit/law suite: **27 files / 110 tests PASS**
- Pixi qualification build: PASS
- browser qualification: **2/2 PASS**
- no-cutoff gate remains GREEN

Performance instrumentation did not weaken semantic or renderer invariants.

## Discovered performance debt

The full SceneReceipt cost is significant enough to preserve as a Side Quest.

It is not promoted to Main Quest because current completeness does not require full receipts on every frame.

## Known non-claims

MQ-14 does not establish:
- production SLOs;
- physical GPU performance;
- mobile/browser matrix;
- WebGPU performance;
- long-duration leak freedom;
- GPU memory envelope;
- optimized incremental SceneReceipt behavior.

## Exit-gate decision

The MQ-14 exit gate is satisfied:

> a documented performance envelope exists for representative Docklab and SP Joaillerie workloads, and instrumentation overhead is known.

Disposition: **RATIFIED**.
