# Publication readiness — in.gnu6

**2026-10-10 · Autorisation de principe accordée pour rendre le dépôt GitHub public.**

Cette autorisation concerne la **visibilité du code après revue**, pas une API publique, une mise sur npm, un transfert de propriété intellectuelle, une licence OSS implicite ou le déploiement de services.

## Gates avant le basculement `PRIVATE → PUBLIC`

| Gate | Statut initial | Sortie exigée |
|---|---|---|
| Historique Git complet | PENDING | Scan de secrets de toutes les références qui deviendront visibles, triage des faux positifs, aucun vrai secret exposé |
| Fichiers suivis et artefacts | PENDING | Inventaire des données, chemins privés, logs, tokens, captures, noms d'utilisateurs et ressources non distribuables |
| Provenance/droits | OPEN | Origine du seed et des donors attribuable ; licences des dépendances et extraits vérifiées |
| Licence du nouveau dépôt | **UNDECIDED** | Choix explicite de l'opérateur. Sans licence, la visibilité GitHub n'accorde pas le droit général de copier ou modifier |
| README et promesses | IN_PROGRESS | Claims adossés à un gate et un environnement ; CP03 complet déclaré OPEN |
| API et package | BLOCKED | `private: true`, `UNLICENSED`, pas de package public ni promesse ABI/API |
| Portail et docs externes | DEFERRED | Mise à jour par preuves publiables, FR/EN, après cohérence du dépôt |

Les résultats des scans doivent vivre dans un registre **non versionné** ; ne jamais commiter les valeurs de secrets détectés. Si un secret est retrouvé dans l'historique, **rotation et assainissement de l'historique** sont des opérations distinctes : ne pas simplement supprimer le fichier dans HEAD puis publier.

## Travail progressif, sans changer le chantier principal

1. À chaque checkpoint GPU-05, relier ce qui est réellement démontré aux documents existants.
2. Marquer les claims en trois classes : `mesuré`, `ratifié pour un mandat borné`, `recherche ouverte`.
3. Traiter l'audit d'historique et la licence avant la modification de visibilité ; le dépôt reste privé en attendant.
4. Ne pas fusionner les travaux d'autres agents ni pousser `main` par raccourci sous prétexte de « lancement public ».
5. Actualiser les surfaces externes seulement avec une traçabilité version/commit/preuve.

**STOP obligatoire :** si un résultat touche à l'identité, des accès, une licence inconnue, du code tiers non autorisé ou un secret, maintenir la visibilité privée jusqu'à résolution. Il est préférable de publier plus tard que de rendre une fuite irréversible.

## Politique éditoriale

Une introduction courte, une preuve concrète, puis des limites claires. Aucune emphase sur « bleeding edge » sans instrument de mesure, aucune vitesse annoncée comme générale à partir d'une RTX 3070 unique, aucun mélange entre `queue.present`, `wp_presentation` et latence input-to-photon. En français d'abord, anglais technique disponible, en évitant les doublons automatiques de prose.
