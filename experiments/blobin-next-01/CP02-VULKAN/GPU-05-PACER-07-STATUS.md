# GPU05-PACER-07 — Reversed startup A/B and first-present latency

**Verdict:** LOCAL REPLICATION POSITIVE / STARTUP PRESENTATION CLOCK QUALIFIED / CP03 HYBRID-60 OPEN.

**Branch:** `feat/blobin-gpu05-pacer07-20261010`
**Parent:** `6441890ed310f63c0f2706e2b4d2ef9155535cee`
**Evidence:** `/mnt/workbench/evidence/runs/blobin-next-01/gpu05-pacer07-reversed-20261010/`

## Question

PACER-06 found that delaying submission of frame #2 until the actual
`wp_presentation` outcome of frame #1 changed that first outcome
from **discarded** to **presented** in one short matched A/B pair.
Does this result survive **reversing execution order**? Can first
submit-to-compositor-presentation latency be measured without confusing
it with user input-to-photon latency?

## Implementation

- `src/first_present_latency.rs` records CLOCK_MONOTONIC anchors
  and accepts `wp_presentation` nanoseconds only when the advertised
  presentation clock is CLOCK_MONOTONIC. It refuses reversed timestamps,
  unrelated clock IDs, and a presentation before first submission.
- `src/bin/gpu05_native_vulkan.rs` emits an isolated startup-clock
  evidence file, bound to the owned Wayland surface and first actually
  presented frame sequence. The previous rendering shader is unchanged.
- `scripts/gpu05_pacer07_compare.py` validates reversed variant
  order, same monitor/scene/binary fingerprints, matching Wayland
  receipt identity, and correct CLOCK_MONOTONIC arithmetic. It
  explicitly refuses CP03 or input-to-photon promotion.
- `scripts/test_gpu05_pacer07_compare.py` contains focused negative
  replay/falsification tests.

## Focused qualification

**5 new Rust tests PASS**; **4 new Python tests PASS**;
`rustfmt --check`, `cargo clippy --offline --locked ... -D warnings`,
and `git diff --check` PASS. Historical closed suites not rerun.

Protected physical reversed-order A/B, NVIDIA RTX 3070, native
Wayland/Vulkan on pinned HDMI-A-1 at ~74.964Hz and 2560×1440:
`await-first-feedback` **before** `current`.

| Measure | Await first outcome | Current |
|---|---:|---:|
| Frame 1 | PRESENTED | DISCARDED |
| Discarded feedbacks | 0 | 1 |
| Actual compositor-presented FPS | 74.798 | 74.964 |
| First actually presented frame | 1 | 2 |
| First-submit-begin to first-present | 3.204113 ms | 18.837564 ms |

**Interpretation:** a second matched, opposite-order short pair
supports the first-outcome barrier, and rejects the assertion
that the improvement was merely caused by the original A/B order.
It does not establish a frequency of success across many cold
starts or independently qualify the input-to-photon response.

Clock proof: `wp_presentation.clock_id=1` and
`CLOCK_MONOTONIC=1` on the reference system. Latency measures
time from **first frame's CPU submission start** to the first
actual compositor presentation: the `current` run's first
presented frame is #2 after #1 was discarded.
This is **not** calibrated user input-to-display latency.
All first-frame outcomes, including discards, remain in the ledger.

### Exact evidence

- Release binary SHA-256:
  `1508a09b6bff7e50768d7f57dc43ea19c674b17666f39cd8982cd2910f7ce530`
- Scene fixture SHA-256:
  `8837d1e23dc59d4e01386efe7dda775f516262946c1559858d13052f268b6c82`
- `comparison.json`, `order.json`, `binary.sha256`,
  `fixture.sha256`, and variant-scoped
  `startup-clock.json`, `presentations.jsonl`,
  `callbacks.jsonl`, `stages.jsonl`, inventories, logs,
  analyzer JSON, and `traces.sha256` are kept outside Git
  under the evidence directory above.

## Boundaries / next

- No merge to `main`, public API, product deployment or CUDA-05F
  supervised-worker authorization.
- Full CP03 remains OPEN: separate 120s sustained compositor evidence
  is recorded on `feat/blobin-gpu05-endurance-20261010`,
  but independently calibrated **input-to-photon**, audit, and
  all ratified acceptance gates remain incomplete.
- Proceed with integration of the PACER-07 result and the
  endurance report in a **separate controlled documentation branch**.
  Do not rerun either closed physical trial merely to create a release.
- Public-repository authorization does not itself resolve license,
  privacy audit, source redaction, API stability or release gates.
