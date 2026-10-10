# blob.in — Sources et inventaire (interne)

**Statut:** `DRAFT_INTERNAL / PUBLICATION_NOT_AUTHORIZED`. Retour: [README](README.md).

## Sources primaires

| Source | Chemin | Nature |
|---|---|---|
| Snapshot blob.in | `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in` | lecture seule, aucune écriture par ce lot |
| Receipt de qualification | `docs/provenance/donor-blobin-appstack-b3-4603aed.md` (ce dépôt) | HISTORICAL_RECEIPT, intake 2026-10-07 |
| Contrat de ce lot | `contracts/agent-batches/2026-10-08/08-BLOBIN-DOCS.md` | périmètre d'écriture |

## Identité

| Champ | Valeur | Statut |
|---|---|---|
| Libellé du snapshot | `appstack-b3-baseline-4603aed` | nom de répertoire préservé |
| Suffixe `4603aed` | non résolu en objet Git; le snapshot n'est pas un dépôt Git (`git rev-parse` échoue) | UNKNOWN |
| Nombre de fichiers | 95 (`find -type f`, 2026-10-08) | VERIFIED_NOW, égal au receipt |
| Digest du receipt | `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d` | HISTORICAL_RECEIPT |
| Recalcul du digest | **reproduit** par B01 et l'intégrateur avec `find . -type f -print0 \| sort -z \| xargs -0 sha256sum \| sha256sum` depuis le snapshot: 95 fichiers, digest exact | VERIFIED_NOW (méthode empirique compatible avec le reçu, recette historique non spécifiée) |

B01 et l'intégrateur ont recomputé le digest de façon indépendante, avant et après lecture: `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d`. La recette est **empiriquement reproduite**, sans prouver que `4603aed` désigne un commit Git vivant.

## Version, licence, metadata (déclaratives)

- Version workspace Rust: `0.14.0` (Cargo/readiness); version de `tokens.json`: `0.20.0` (TOKEN_AUTHORITY_REPORT). Les deux déclarations existent; leur rôle respectif dans la version produit n'est pas ratifié.
- Licence déclarative **contradictoire**: `GPL-3.0-or-later` dans Cargo, `AGPL-3.0` dans `components.toml` du baseline parent et texte AGPL-3.0 dans `LICENSE` du parent. Aucun fichier `LICENSE`/`COPYING` dans le sous-répertoire blob.in; aucune conclusion juridique ou droit de redistribution ne découle de ce constat.
- Repository: `https://github.com/gnu-in-labs/blob.in` (metadata `repository`), état non qualifié.
- `rust-version = 1.82` (`blobin-core`); `edition = 2021`.
- Dépendances notables: `lyon 1.0`, `glam 0.27`, `cxx 1.0`, `rayon 1.10` (optionnel), `cxx-qt 0.8`, `png 0.17`, `criterion 0.5`, `serde 1.0`, `gpui` (git Zed, rev épinglé).
- La GPL est traitée ici comme metadata source. Ce dossier ne recopie aucune source GPL et ne donne aucun avis juridique.

## Inventaire (95 fichiers, par zone)

| Zone | Contenu |
|---|---|
| Racine | `Cargo.toml`, `Cargo.lock`, `README.md`, `readiness.md`, `ENGINEERING-NOTES.md`, `TOKEN_AUTHORITY_REPORT.md`, `tokens.json`, `justfile` |
| `blobin-core/` | `build.rs`, 16 fichiers `src/` (dont `lib.rs` et `tokens.rs` généré), `tests/lifecycle.rs`, `tests/scene.rs` |
| `blobin-qt/` | 6 fichiers C++/h, 2 shaders, `CMakeLists.txt`, `tests/render_goldens/` |
| `blobin-cxxqt/`, `blobin-gen/`, `blobin-gpui/`, `blobin-bench/` | sources et `Cargo.toml` |
| `blobin-golden/` | 5 PNG de référence, `tests/material_parity.rs`, `src/bin/diff_png.rs`, `r8-hardware/20260611-nvidia-rtx3070/` (README, checksums, 6 images) |
| `gen/` | `gnu_theme.rs`, `.h`, `.gpui.rs` |
| `tools/` | 22 fichiers: scripts/protocoles/sign-offs (R5-R10, readiness, flip) |

## Pointeurs de preuve internes

`tools/r7-sign-off.json` (R7, 2026-06-10, commit `d07d161`), `tools/r8-sign-off.json` (R8 software/hardware/qualitative), `tools/r9-sign-off.json` (R9), `tools/r10-sign-off.json` (R10 `pending`), `blobin-golden/r8-hardware/20260611-nvidia-rtx3070/checksums.sha256`.
