# blob.in — Dossier produit (interne)

**Statut:** `DRAFT_INTERNAL / PUBLICATION_NOT_AUTHORIZED / SOURCE_SNAPSHOT_OBSERVED`

Ce dossier est privé. Il ne constitue ni une documentation publique, ni une release note, ni une autorisation de distribution. Rien ici ne doit être copié vers un portail public (docs.gnu6.live, dépôt public, crates.io, npm, VPS) sans décision distincte de l'opérateur.

## Vue produit

blob.in est un moteur de formes organiques et de mouvements, issu des surfaces Gnu.In-Shell (anciennement `alaelestia`). Il sert à dessiner la mascotte, des menus radiaux et des bulles de notification (source: `README.md` du snapshot, lignes d'ouverture).

Architecture déclarée par le snapshot (« 4.0 »): le moteur est en **Rust** (`blobin-core`); Qt/C++ (`blobin-qt`) est une couche fine de rendu. Géométrie, tessellation, solveurs ressort/easing, matériaux et état sont calculés en Rust puis transmis à Qt par un pont `cxx` sous forme de buffers plats.

blob.in est un produit à part entière. Sa relation avec `in.gnu6` (donor de lois et de fixtures) est une **annexe** de ce dossier, voir [ARCHITECTURE.md §Annexe](ARCHITECTURE.md#annexe--relation-avec-ingnu6).

## Audience

| Lecteur | Où aller |
|---|---|
| Qui veut comprendre ce qu'est le produit | ce fichier, puis [ARCHITECTURE.md](ARCHITECTURE.md) |
| Qui veut reproduire une preuve | [VERIFICATION.md](VERIFICATION.md) |
| Qui doit citer une version, un digest, une licence | [SOURCES.md](SOURCES.md) |

## Identité, version, licence (déclaratives)

| Élément | Valeur | Source | Statut |
|---|---|---|---|
| Version | `0.14.0` | `Cargo.toml` (`[workspace.package]`) et `readiness.md` §Engine version | DECLARED_IN_SOURCE |
| Licence | `GPL-3.0-or-later` | metadata Cargo (workspace, `blobin-core`, `blobin-gen`, `blobin-golden`, `blobin-gpui`) | DECLARED_IN_SOURCE, voir note GPL |
| Repository | `https://github.com/gnu-in-labs/blob.in` | metadata Cargo | DECLARED_IN_SOURCE; visibilité et état courant NOT_QUALIFIED |
| Snapshot | `appstack-b3-baseline-4603aed`, 95 fichiers | receipt `docs/provenance/donor-blobin-appstack-b3-4603aed.md` | HISTORICAL_RECEIPT; 95 fichiers revérifié |
| Commit `4603aed` | non résolu comme objet Git | même receipt | UNKNOWN |

**Note GPL.** La licence GPL-3.0-or-later est citée ici uniquement comme *metadata déclarée par la source*. Ce dossier ne recopie aucune source GPL, ne donne pas d'avis juridique, et ne qualifie pas les obligations d'une éventuelle distribution. Le snapshot ne contient pas de fichier `LICENSE`/`COPYING` (revérifié, voir SOURCES.md).

## État en un coup d'œil

| Capacité | État | Détail |
|---|---|---|
| Noyau Rust `blobin-core` | exécutable en isolation (historique) | 47/47 tests, voir VERIFICATION.md |
| Pont `cxx` + shim Qt6 | buildable dans une copie reconstruite (historique) | pas dans le workspace exporté tel quel |
| Workspace exporté autonome | **FAUX** | dépendance `blobin-gpui` → `gnosis-gpui-theme` absente |
| Backend `cpp` (fallback) | **NON PROUVÉ** | `backend.rs` ne fait que sélectionner/rapporter |
| Parité GPU (R8) | artefacts historiques seulement | GPU non réexécuté |
| Dogfood R10 (14 jours) | **NON PROUVÉ** | `tools/r10-sign-off.json` = `pending` |
| Release / distribution | **NON AUTORISÉE** | ce snapshot n'est pas une release prête à distribuer |

## Navigation

- [ARCHITECTURE.md](ARCHITECTURE.md): crates, FFI, Qt/GPUI, diagramme, ownership.
- [VERIFICATION.md](VERIFICATION.md): procédures reproductibles, piège `gnosis-gpui-theme`, historique vs frais, R7-R10.
- [SOURCES.md](SOURCES.md): digest, version, licence, metadata, inventaire.

## Frontières de ce dossier

- Il décrit un **snapshot historique**, pas le HEAD d'un dépôt amont vivant (non identifié).
- Les résultats du travail de recherche parallèle B01 ne sont **pas** intégrés. Toute information venant de B01 sera marquée `PENDING_RECONCILIATION` dans la section dédiée de [VERIFICATION.md](VERIFICATION.md) tant qu'elle n'est pas qualifiée.
- Il ne modifie ni le snapshot, n