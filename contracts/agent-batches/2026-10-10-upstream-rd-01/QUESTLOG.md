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
- G03 convergence interprojets: DRAFT; G04 expériences empiriques: NOT_RUN.
- G05 transfert projet: NOT_RUN; G06 publication: DRAFT_ONLY; G07 push: PENDING avant receipt du remote.

## Interfaces et conflits à préserver
- `in.gnu6` n'acquiert pas l'autorité runtime/security/UI du fait de cette recherche.
- `gnostral.rs` a un worktree local divergent/dirty; aucune édition.
- `engineering-corpus` et `gnosix` ont des branches actives; aucune édition.
- L'autorisation de rendre public `in.gnu6` ne doit pas être inférée de l'autorisation de pousser une branche sur son dépôt **privé**.

## Suite
D'abord vérifier le push privé isolé; ensuite former les expériences légères à oracle indépendant, puis la revue des licences et des projets bénéficiaires. Ni release publique ni preuve runtime acquise.
