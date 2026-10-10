# Integrator — Réconciliation et handoff du lot IN-GNU6-PAR-01

Ce rôle n'est **pas** un neuvième agent qui implémente en parallèle. Il assemble les livrables lorsque les workers ont terminé.

**Entrées:** huit worktrees/branches individuels et leurs commits; baseline gelée; `README.md` de ce lot; `CONTRACT.md`, `QUESTLOG.md`.

**Étapes:**
1. Vérifier les IDs des workers, `base_sha`, write-sets et `git status`. Aucune branche obsolète n'est fusionnée à l'aveugle. Écarter les dossiers `node_modules`, outputs de tests non pertinents ou journaux bruts privés.
2. Vérifier qu'aucun worker n'a franchi sa frontière. Si oui, séparer le commit ou le rejeter; pas de conservation silencieuse de changements non mandatés.
3. Réconcilier B01 ↔ B02: les assertions blob.in issues de recherche deviennent `VERIFIED` dans la documentation seulement si preuves réelles concordantes. Les désaccords apparaissent dans la documentation comme `UNRESOLVED`, pas moyen de les masquer.
4. Aligner A01→A06 sur une matrice de concepts sémantiques, providers, consumers et candidats publics sans écraser les observations divergentes.
5. Établir un rapport d'intégration `docs/agent-results/par-01/INTEGRATION.md` (seul intégrateur propriétaire de ce chemin), chaque commit intégré et son test causal.
6. Effectuer les merges/cherry-picks seulement dans un **worktree d'intégration isolé**, puis valider un minimum de `npm run check`; le browser gate si des chemins concernés le demandent. **Pas de push** en absence d'autorisation explicite, surtout avec un `main` local en avance du remote.
7. Transmettre à l'humain une vue claire: GREEN/RED/NOT_RUN, conclusions, docs blob.in présentes, inconnues et autorisations restant nécessaires.

**Frontières:** pas de changement de propriétaire de scène, pas de nouvelle Main Quest, pas de licence publique, pas d'API publiée, pas de mutation d'infrastructure, pas de publication `docs.gnu6.live` implicite.

**Output state:** `CONTRACTS_READY` ≠ `AGENTS_DISPATCHED` ≠ `REPORTS_INTEGRATED` ≠ `PUBLIC_RELEASE`.
