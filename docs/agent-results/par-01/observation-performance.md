# A05 — Observation cost, reproducibility and the performance boundary

Worker: A05 (batch IN-GNU6-PAR-01). Base: `c9dcdab997f6`. Branch: `feat/par-01-A05`.
Write-set: this file only. No engine change, no cache implemented, no oracle changed.
Status of this document: **research note; nothing here is RATIFIED.** SQ-008 stays `DISCOVERED`, operator priority `UNSET`, NON_CAUSAL for current completeness.

## 0. Method and freshness

Sources read (read-only): `src/introspection/frame.ts`, `src/spatial/environment.ts` (`computeOcclusion`, `intersectsAabb`), `tests/performance/core-envelope.perf.ts`, `vitest.performance.config.ts`, `benchmarks/mq14-browser.mjs` (listed), `docs/evidence/mq-09-*.md`, `docs/evidence/mq-14-*.md`, `docs/evidence/assets/mq-14-core-envelope.json`, `QUESTLOG.md` SQ-008.

**No fresh timing was taken.** The worktree has no `node_modules`; running the perf harness would need a dependency install (network, outside this write-set). Claim status: fresh benchmark = `NOT_RUN`. The only new computation is a deterministic, non-timing geometry count (section 2), run with plain `node` outside the repo.

Historical instrument (from the raw receipt): git HEAD `a2f5adbde7c2`, tree clean, Node v24.20.0, linux x64, i7-12700F, 20 logical CPUs, ~33.4 GB. Current host Node is v26.10.0 (`node -v` now), so even a re-run would be a different instrument.

## 1. Workload table

All rows are `HISTORICAL_RECEIPT`; none is `VERIFIED_NOW`. Timestamp for core rows: `2026-10-07T04:45:39Z` (raw JSON `observedAt`). Browser rows: MQ-14 receipt dated 2026-10-07; its raw JSON was not re-read here for a per-row timestamp (UNKNOWN beyond the date). Unit is milliseconds per operation; "median" is the median of 9 batches, each batch the mean of N iterations, after a warm-up. So "p95" over 9 samples is effectively the max; treat it as a spread indicator, not a tail percentile.

| workload | baseline | instrument | unit | timestamp | overhead factor | freshness | possible error |
|---|---|---|---|---|---|---|---|
| semantic step, 256 entities | 0.055 median (0.062 mean, max 0.110) | `core-envelope.perf.ts` `measure`, 40 it x 9 batches, Node 24.20.0 | ms/op | 2026-10-07T04:45Z | 1x | HISTORICAL_RECEIPT | Sub-0.1 ms with `performance.now()`; JIT state and timer granularity matter. |
| step + `buildSceneReceipt`, 256 entities | 19.83 median (17.23 mean, min 10.93, max 20.27) | same | ms/op | same | ~360x vs step (receipt alone ~19.8 ms, i.e. ~99.7% of the sum) | HISTORICAL_RECEIPT | Min/max span ~2x, so run-to-run noise is large. See confounds C1-C4 below. |
| semantic step 24 / 128 / 512 / 2048 | 0.021 / 0.037 / 0.131 / 0.596 median | same | ms/op | same | 1x | HISTORICAL_RECEIPT | Different iteration counts (30 vs 12) across sizes. |
| contour retarget 1024 nodes | 2.18 median | same | ms/op | same | n/a | HISTORICAL_RECEIPT | Alternating rect/polygon target. |
| picking 5000 occupants | 0.035 median | same | ms/op | same | n/a | HISTORICAL_RECEIPT | Query is a miss (-100,-100), so a best-case scan. |
| RendererFrame build, 512 entities | 6.02 mean | headless Chrome 153, SwiftShader (not a GPU), Pixi 8.22.0 | ms/op | 2026-10-07 | n/a | HISTORICAL_RECEIPT | Software rasterizer; says nothing about physical GPU. |
| Pixi render, 512 entities | 1.47 mean, p95 1.80 | same | ms/op | 2026-10-07 | n/a | HISTORICAL_RECEIPT | Same. |

Frame-build vs semantic step, 512 entities: 6.02 ms (browser, mean) vs 0.131 ms (Node, median), about 46x, but these are different instruments (browser vs Node) so the ratio is `HYPOTHESIS`-grade only.

## 2. What the 360x actually compares (confounds, not corrections)

The ~360x ratio is a true historical observation of a specific harness. It is not a general law of the engine. Facts from the source:

- **C1 — Ratio denominator is tiny.** MQ-14 already says this. The absolute number (~20 ms) is the load-bearing fact.
- **C2 — Occlusion cost is O(n^2) pairs plus O(n x pairs) lookups.** `computeOcclusion` loops all back/front pairs, each calling `intersectsAabb`, which calls `assertAabb` twice. Then, in `buildSceneRece