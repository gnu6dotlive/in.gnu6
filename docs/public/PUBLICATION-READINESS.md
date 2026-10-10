# Publication readiness — in.gnu6

**2026-10-10 · Autorisation de principe accordée pour rendre le dépôt GitHub public.**

Cette autorisation concerne la **visibilité du code après revue**, pas une API publique, une mise sur npm, un transfert de propriété intellectuelle, une licence OSS implicite ou le déploiement de services.

## État observé : dépôt PUBLIC, distribution non ratifiée

Le 2026-10-10, la visibilité PUBLIC de tension-atoi/in.gnu6 a été vérifiée sur GitHub après annonce de l'opérateur. Les gates ci-dessous représentent le dossier de préparation historique et restent à examiner ; ils ne sont pas tous clos. La visibilité Git ne constitue ni une licence OSS, ni un contrat npm, ni une API stable.

## Gates avant le basculement `PRIVATE → PUBLIC`

| Gate | Statut initial | Sortie exigée |
|---|---|---|
| Historique Git complet | **Automated gitleaks PASS, manual review OPEN** | Le scan `gitleaks git --log-opts=--all` a détecté **0 finding** sur les références locales ; vérifier séparément les secrets non reconnus et le droit de publier toutes les branches distantes |
| Fichiers suivis et artefacts | **TRIAGE REQUIRED** | 974 fichiers suivis ; 80 fichiers avec chemins absolus d'environnement ; 28 avec références aux consommateurs ; 27 avec URL/email ; 18 avec termes liés à l'authentification. Correspondances heuristiques, pas 80 secrets prouvés |
| Provenance/droits | OPEN | Origine du seed et des donors attribuable ; licences des dépendances et extraits vérifiées |
| Licence du nouveau dépôt | **UNDECIDED** | Choix explicite de l'opérateur. Sans licence, la visibilité GitHub n'accorde pas le droit général de copier ou modifier |
| README et promesses | IN_PROGRESS | Claims adossés à un gate et un environnement ; CP03 complet déclaré OPEN |
| API et package | BLOCKED | `private: true`, `UNLICENSED`, pas de package public ni promesse ABI/API |
| Portail et docs externes | DEFERRED | Mise à jour par preuves publiables, FR/EN, après cohérence du dépôt |

Les résultats des scans doivent vivre dans un registre **non versionné** ; ne jamais commiter les valeurs de secrets détectés. Si un secret est retrouvé dans l'historique, **rotation et assainissement de l'historique** sont des opérations distinctes : ne pas simplement supprimer le fichier dans HEAD puis publier.

## Contrôles initiaux consignés

Le 2026-10-10, sur le worktree de consolidation : analyse automatisée
`gitleaks git --log-opts=--all`, sortie 0 et **zéro finding**. Une
seconde recherche superficielle sur les fichiers suivis a classé les
occurrences de chemins absolus, marques/consommateurs, adresses et mots
d'authentification. **Ces classes peuvent se chevaucher** et ne sont
pas des preuves de secrets. Les listes détaillées et le rapport scanner
sont gardés **hors Git**, dans
`/mnt/workbench/evidence/runs/in-gnu6-public-readiness-20261010/`.

Au moment du contrôle : **4 branches distantes**, dont trois nouvelles
branches GPU-05 de qualification et `main`. Avant de passer public, il
faut examiner aussi l'historique reachable depuis chacune de ces
branches, les licences des donors et les noms de consommateurs.
Un scan de secrets automatisé n'est pas un contrôle de droits d'auteur,
de données personnelles ou de confidentialité commerciale.

**Décision historique (avant changement de visibilité) : conserver PRIVATE.** Le dépôt est depuis PUBLIC ; aucune licence, publication npm ou API publique n'est pour autant ratifiée.

## Travail progressif, sans changer le chantier principal

1. À chaque checkpoint GPU-05, relier ce qui est réellement démontré aux documents existants.
2. Marquer les claims en trois classes : `mesuré`, `ratifié pour un mandat borné`, `recherche ouverte`.
3. Poursuivre activement l'audit d'historique, de confidentialité et de licence du dépôt déjà public ; documenter et corriger toute exposition indue.
4. Ne pas fusionner les travaux d'autres agents ni pousser `main` par raccourci sous prétexte de « lancement public ».
5. Actualiser les surfaces externes seulement avec une traçabilité version/commit/preuve.

**STOP de diffusion :** si une exposition de secrets, données privées, droits inconnus ou code tiers non autorisé est détectée, suspendre la diffusion, évaluer l'exposition existante et appliquer la remédiation adaptée. Il est préférable de publier plus tard que de rendre une fuite irréversible.

## Politique éditoriale

Une introduction courte, une preuve concrète, puis des limites claires. Aucune emphase sur « bleeding edge » sans instrument de mesure, aucune vitesse annoncée comme générale à partir d'une RTX 3070 unique, aucun mélange entre `queue.present`, `wp_presentation` et latence input-to-photon. En français d'abord, anglais technique disponible, en évitant les doublons automatiques de prose.
