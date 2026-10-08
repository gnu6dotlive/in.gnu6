# A05 — Coût d'observation, reproductibilité et performance

**Objectif:** identifier la frontière performance entre pas sémantique, fabrication de receipts et rendu; ne pas changer un oracle pour accélérer un score.

**Sources RO:** `src/introspection/**`, `tests/performance/**`, `benchmarks/**`, `docs/evidence/mq-09*.md`, `docs/evidence/mq-14*.md`, `QUESTLOG.md` SQ-008. Environnement et machine notés pour tout test neuf.

**Write-set exclusif:** `docs/agent-results/par-01/observation-performance.md`.

**Question:** quand un receipt exhaustif est-il nécessaire et quand un niveau de détail explicite suffit-il? Les mesures historiques de ~19,8 ms pour receipt/256 entités et ~0,055 ms pour pas sémantique ne sont PAS une mesure fraîche. `SQ-008` reste optionnel, priorité UNSET.

**Gate:** un tableau `workload, baseline, instrument, unité, timestamp, facteur d'overhead, fraîcheur, erreur possible`; au besoin des tests déterministes ciblés en isolation, sans modifier le moteur ni implémenter de caching.

**Remise:** hypothèses de réduction de coût avec conditions de non-falsification de la vérité, risques de stale cache, plan de test minimal pour chacune.
