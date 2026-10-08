# Worker contract — tronc d'exécution paramétrable

**Input:** `WORKER_ID`, `BASE_SHA`, `WORKTREE`, `SPECIALIZED_CONTRACT`, `EXCLUSIVE_WRITESET`. Ce contrat complète, ne remplace pas, `contracts/TRONC_COMMUN.md` et `AGENTS.md`.

1. Inventorier le contexte réel: `git status -sb`, HEAD, source pins, branches/worktrees voisins, prochaine frontière d'autorité.
2. Lire uniquement les sources autorisées. Source ≠ comportement live; historique ≠ mesure fraîche; présence d'un projet ≠ autorisation de publication.
3. Construire l'oracle minimal qui pourrait **falsifier** une assertion. Ne pas lancer des campagnes entières de benchmarks sans question causale précise.
4. Écrire **seulement** le write-set du contrat spécialisé. Tests générateurs et fixtures temporaires restent confinés au worktree ou à un répertoire temporaire unique, sans nettoyer des fichiers d'autrui.
5. Pas de `git reset --hard`, nettoyage de worktrees étrangers, `push`, suppression, gestion de secrets, publication, déploiement, modification `in.gnu6` core, ni mandat sur un autre repo.
6. Rapporter chaque claim comme `VERIFIED_NOW`, `HISTORICAL_RECEIPT`, `HYPOTHESIS`, `FALSIFIED`, `NOT_RUN`, `BLOCKED_AUTHORITY` ou `UNKNOWN`.
7. Remise: HEAD, périmètre exact, sources & SHA, commandes réellement lancées, résultats/contre-preuves, fichiers modifiés, statut de chaque assertion, non-claims et prochaine décision.
8. Laisser un commit local (un ou plusieurs commits lisibles) et un résumé pour l'intégrateur. Un `exit 0` ne prouve pas une fonctionnalité de produit.

Launch envelope réutilisable:
```yaml
worker_id: B01
base_sha: "<validated-contract-batch-sha>"
worktree: "<unique-absolute-path>"
contract: "contracts/agent-batches/2026-10-08/07-BLOBIN-RESEARCH.md"
allowed_write_paths: ["docs/agent-results/par-01/blobin-research.md"]
network_publication: false
merge: integrator_only
```
