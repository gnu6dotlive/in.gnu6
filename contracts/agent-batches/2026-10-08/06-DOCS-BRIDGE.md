# A06 — Préparer des fiches publiques sans publier le privé

**Objectif:** créer un **registre de candidatures documentaires**, sûr pour revue humaine, entre les résultats internes et le portail public `docs.gnu6.live`.

**Sources RO:** `in.gnu6/README.md`, `CONTRACT.md`, `QUESTLOG.md`, preuves choisies et portail `engineering-corpus` public en lecture seule; états exacts des repos Gnosix/gnu6-live si utiles mais privés.

**Write-set exclusif:** `docs/agent-results/par-01/public-docs-candidates.md`.

**Gates:** pour chaque phrase candidate, `source+revision | acceptable-public-summary | owner | public_authorized? | evidence | confidence | blocked_reason`. Les noms de modules internes ne donnent pas droit à l'exposer. Une fiche conceptuelle publique sans source publique autorisée doit rester `PROPOSED_FOR_REVIEW`.

**Exclusions:** ne pas écrire dans `engineering-corpus`, ne pas pousser, ne pas publier, ne pas déclencher Coolify. Ne pas divulguer endpoints internes, chemins secrets ou matériaux d'autorité.

**Remise:** version rédactionnelle FR/EN et manifeste de redaction; une proposition d'intégration documentaire (sans pseudo-SDK).
