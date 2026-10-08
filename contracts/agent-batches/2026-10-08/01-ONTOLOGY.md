# A01 — Architecture/ontologie du moteur

**Objectif:** établir un atlas sémantique du moteur, sans refaire les Main Quests ratifiées.

**Sources RO:** `src/algebra/**`, `src/scene/**`, `src/spatial/**`, `src/topology/**`, `src/dynamics/**`, `src/interaction/**`, `src/deck/**`, `tests/laws/**`, MQ-01→MQ-10.

**Write-set unique:** `docs/agent-results/par-01/engine-ontology.md`.

**Questions:** où se trouvent identité, hiérarchie, relation, temps, contraintes, topologies simultanées, scènes et handoffs? Quels concepts seraient illégalement apportés par des providers ou par les produits? Quels invariants manquent de falsification reproductible?

**Gate:** tableau `assertion → symbol + version → oracle/test → contre-exemple → classe de vérité`. Tests ciblés facultatifs, avec état `NOT_RUN` si non exécutés. Aucun ajout de core TS, aucune qualification d'API publique.

**Remise:** graphe textuel des modules, frontières de responsabilité, hypothèses réfutables et liens vers les tests existants.
