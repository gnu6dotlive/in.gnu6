# UPSTREAM-R&D-01 — Questlog du lot

Statut: **SOURCE_INTAKE_COMPLETE / CONTRACTS_DRAFTED / EMPIRICAL_NOT_RUN**
Date: 2026-10-10 · Autorité: GO explicite de l'opérateur, publication de la branche de recherche privée uniquement.

## Acquis vérifiables
- Poste réel `gnosisos-x8664`, `gnu6-lab-run --check` → `GNU6_LAB_READY`.
- Base isolée `in.gnu6@4042faa` depuis `origin/main`; `in.gnu6/main@d160639` (53 commits locaux) exclu du push.
- Ancien worktree `in-gnu6-agent-contracts` possède un lien Git cassé; documentation conservée, aucune réparation destructive.
- Six dépôts upstream, six SHA et six README blob SHA : `sources.json` (acquisition en six tâches concurrentes).
- Douze fichiers à SHA exact vérifiés par l'API GitHub : `evidence/upstream-rd-01/intake/source-probe.json`.
- Rapports d'intake W01–W06 : `docs/agent-results/upstream-rd-01/`. **Ce sont des probes source**, pas six agents indépendants.
- Douze contrats W00–W11 rédigés, dispatch structuré; aucun produit modifié.

## Gates
- G00 baseline/isolation: PASS; G01 pins/source access: PASS pour six commits et douze fichiers, licences distributives à approfondir.
- G02 lecture source: PASS pour l'intake; revue approfondie des API et consommateurs: OPEN.
- G03 convergence interprojets: DRAFT; G04 bwrap startup host smoke: PROBE_PASS after fixture repair; MXC SDK, GPU, runtime and performance tests: NOT_RUN.
- G05 transfert projet: NOT_RUN; G06 publication: DRAFT_ONLY; G07 premier commit/push: VERIFIED_REMOTE at d9bcb0488c88a4447d5731d35e00b9bf230a5462, remote SHA equals local SHA.

## Interfaces et conflits à préserver
- `in.gnu6` n'acquiert pas l'autorité runtime/security/UI du fait de cette recherche.
- `gnostral.rs` a un worktree local divergent/dirty; aucune édition.
- `engineering-corpus` et `gnosix` ont des branches actives; aucune édition.
- L'autorisation de rendre public `in.gnu6` ne doit pas être inférée de l'autorisation de pousser une branche sur son dépôt **privé**.

## Suite
D'abord vérifier le push privé isolé; ensuite former les expériences légères à oracle indépendant, puis la revue des licences et des projets bénéficiaires. Ni release publique ni preuve runtime acquise.

## 2026-10-10 — Tranche W01–W08, reprise après le push 947bd79

State: **RESEARCH_SOURCE_QUALIFIED / HOST_BWRAP_BOUNDARY_PASS / PRODUCT_NOT_RUN**.

- Préflight repris : branche centrale `947bd79` propre et SHA remote concordant; `in.gnu6/main` toujours 53 commits locaux en avance et non modifié.
- Six worktrees `upstream-rd-01-w01-20261010`…`w06` créés sur la base `db794a6`, chacun avec contrat distinct et deux chemins d'écriture autorisés.
- Six workers de collecte déterministes lancés simultanément avec `gnu6-lab-run`; ils ne sont **pas des agents LLM autonomes**. Commits locaux source: W01 `f646b5a`, W02 `05edcf2`, W03 `13be7bc`, W04 `b100fb2`, W05 `a8088df`, W06 `54f0b00`.
- Six cherry-picks réconciliés dans la branche centrale après vérification parent/écriture (2 chemins distincts chacun), aucun conflit. 18 fichiers supplémentaires à révision exacte vérifiés, trois par upstream; ajoutés aux 12 probes initiales (30 reçus de fichiers, chemins parfois redondants).
- W07 [synthèse](../../../docs/agent-results/upstream-rd-01/cross-analysis.md) : tous les projets bénéficiaires cartographiés, limites et hypothèses distinguées.
- W08 [expériences](../../../docs/agent-results/upstream-rd-01/experiments.md) : oracle Bubblewrap hôte **PASS_BOUNDARY_FIXTURE** (EROFS sur fichier témoin, /tmp writable, ENETUNREACH sur l'adresse test, namespace réseau distinct, témoin intact). MXC SDK, Deno, GPUI Wayland, Uzu CUDA/CPU et Hyperfine réel : NOT_RUN/BLOCKED selon le cas.
- W09 [réconciliation](../../../docs/agent-results/upstream-rd-01/integration.md) : décision de **pousser la recherche privée seulement**, aucune promotion produit. W10 [surfaces](../../../docs/agent-results/upstream-rd-01/docs-bridge.md) : statut documentaire privé, aucune propagation/déploiement de site.

Gates: G00/G01 PASS; G02 SOURCE_DEEP_REVIEW_PASS; G03 SOURCE_CROSS_ANALYSIS_PASS; G04 HOST_FIXTURE_PASS / PRODUCT_SDK_NOT_RUN; G05 TRANSFER_NOT_RUN; G06 PUBLIC_DRAFT_ONLY; G07 REMOTE_SHA_RECHECK_REQUIRED_AFTER_NEXT_PUSH.

No public API, release, donor vendoring, merge main, source deletion, control plane mutation or production service changed.
