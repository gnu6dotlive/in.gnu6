# blob.in — Vérification et limites de preuve (interne)

**Statut:** `DRAFT_INTERNAL / PUBLICATION_NOT_AUTHORIZED`. Retour: [README](README.md).

Ce document sépare trois choses: ce qui est **VERIFIED_NOW** (exécuté lors de la rédaction, 2026-10-08), ce qui est un **HISTORICAL_RECEIPT** (receipt `docs/provenance/donor-blobin-appstack-b3-4603aed.md`, intake 2026-10-07, ou sign-offs du snapshot), et ce qui est **NOT_RUN / NOT_PROVEN**. Un PASS historique n'est pas un PASS frais; un `exit 0` n'est pas une fonctionnalité produit.

Sources du snapshot: voir [SOURCES.md](SOURCES.md). Racine citée ci-dessous: `$S=/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in` (lecture seule).

## Tableau de statut

| Sujet | Résultat | Statut | Pointeur |
|---|---|---|---|
| Snapshot: 95 fichiers | 95 trouvés | VERIFIED_NOW | `find $S -type f \| wc -l` |
| Digest du manifeste `5a7831ec…` | non reproduit (méthode non spécifiée) | NOT_PROVEN | SOURCES.md |
| Suffixe Git `4603aed` | non résolu | UNKNOWN | receipt §Revision-boundary |
| Garde LoC shim C++ | 282 / 300 OK | VERIFIED_NOW (et receipt: 282/300) | `bash $S/tools/loc-guard.sh 300` |
| Garde no-unsafe (R6) | « OK (R6): no unsafe outside ffi.rs » | VERIFIED_NOW (et receipt PASS) | `bash $S/tools/no-unsafe.sh` |
| `blobin-core` isolé, 3 configurations | 47/47 PASS chacune | HISTORICAL_RECEIPT | receipt §Isolated blobin-core |
| Workspace exporté tel quel | `cargo test` bloqué avant compilation | HISTORICAL_RECEIPT; cause revérifiée en source | voir piège ci-dessous |
| Copie reconstruite sans `blobin-gpui` | tests workspace (hors bench), Qt6/Corrosion, pont cxx, `.qsb`, `blobinqt` PASS | HISTORICAL_RECEIPT / MODIFIED_WORKSPACE | receipt §Reconstructed |
| Intégrité des artefacts GPU R8 | `sha256sum -c checksums.sha256` PASS | HISTORICAL_RECEIPT | `blobin-golden/r8-hardware/20260611-nvidia-rtx3070/` |
| Exécution GPU R8 | non répétée | NOT_RUN | |
| `BLOBIN_BACKEND=cpp` fonctionnel | non établi | NOT_PROVEN | ARCHITECTURE.md |
| R10 dogfood 14 jours | `pending`, dates nulles | NOT_PROVEN | `tools/r10-sign-off.json` |
| Release prête à distribuer | aucune qualification | BLOCKED_AUTHORITY | |

Aucun `cargo`, Qt, CMake ou GPU n'a été exécuté pour écrire ce dossier. Seuls `find`, `wc`, `sha256sum` (lecture), et les deux scripts de garde ci-dessus (lecture seule) ont tourné.

## Le piège `gnosis-gpui-theme`

`$S/Cargo.toml` liste `blobin-gpui` parmi les `members`. `$S/blobin-gpui/Cargo.toml` déclare:

```toml
gnosis-gpui-theme = { path = "../../gnosis-gpui-theme" }
```

Ce chemin se résout (depuis `blobin-gpui/`) vers `engine/gnosis-gpui-theme`, **absent** de l'export (vérifié 2026-10-08; le receipt le note sous la forme `../gnosis-gpui-theme/Cargo.toml`, même constat d'absence). Cargo résout tout le workspace avant de compiler quoi que ce soit, donc:

- ne pas lancer `cargo test` à la racine `$S` en le présentant comme garantie: le workspace historique ne se résout pas (HISTORICAL_RECEIPT);
- la configuration Qt/CMake échoue aussi, car Corrosion demande d'abord les métadonnées Cargo du workspace;
- `blobin-gpui` tire aussi `gpui` depuis git (Zed, rev épinglé): réseau requis, non évalué ici.

Le README du snapshot indique `cargo test --manifest-path blob.in/blobin-core/Cargo.toml`: cette forme **hérite quand même du workspace** (`[lints] workspace = true`) et n'échappe donc pas au piège dans l'export. Ne pas la citer comme alternative.

## Procédures reproductibles

Règle commune: ne jamais écrire dans `$S`. Travailler dans un répertoire unique, copie d'octets, et consigner commande, entrées, sortie, durée.

### P0 — Gardes statiques (VERIFIED_NOW possible sans toolchain)

```bash
S=/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in
find "$S" -type f | wc -l            # attendu: 95
bash "$S/tools/loc-guard.sh" 300     # attendu: TOTAL 282, OK
bash "$S/tools/no-unsafe.sh"         # attendu: OK (R6)
```

### P1 — Core isolé, `QUALIFICATION_COPY` (NOT_RUN ici)

```bash
Q=$(mktemp -d)                       # répertoire unique
cp -r "$S/blobin-core" "$Q/"
# Seule modification de qualification: supprimer le bloc "[lints] workspace = true"
# de $Q/blobin-core/Cargo.toml, sans toucher aux sources.
( cd "$Q/blobin-core" && cargo test )                              # défaut
( cd "$Q/blobin-core" && cargo test --no-default-features )        # séquentiel
( cd "$Q/blobin-core" && cargo test --features threaded-solver )   # explicite
```

Attendu historique: 47/47 dans chaque cas. Marquer le résultat `QUALIFICATION_COPY`, jamais « le snapshot passe ». Les dépendances crates.io (`lyon`, `glam`, `cxx`, `rayon`) exigent l'accès au registre ou un cache local; cela ne fait pas partie de ce dossier. Si le registre est inaccessible, le statut est `NOT_RUN`, pas un échec.

### P2 — Workspace reconstruit sans GPUI, `QUALIFICATION_COPY` (NOT_RUN ici)

Copie complète du snapshot, puis seule modification structurelle: retirer `"blobin-gpui"` de `members`. Sources inchangées. Ensuite `cargo test --workspace --exclude blobin-bench`, puis le build Qt6/Corrosion (`just qt`, nécessite Qt ≥ 6.6, CMake, Corrosion). Documenter `Cargo.lock` : il référence encore les dépendances du membre retiré.

### P3 — Parité matériaux (CPU)

`cargo test -p blobin-golden` dans la copie P2. Seuil déclaré: delta canal max ≤ 4, ≤ 1 % de pixels au-dessus du seuil, SSIM ≥ 0.99 (README du snapshot). Ne régénérer les goldens (`UPDATE_GOLDEN=1`) qu'en copie.

### P4 — Intégrité des preuves GPU archivées

```bash
cd "$S/blobin-golden/r8-hardware/20260611-nvidia-rtx3070" && sha256sum -c checksums.sha256
```

Prouve l'intégrité des fichiers, pas le run GPU du 2026-06-11. Statut attendu: HISTORICAL_RECEIPT (le receipt rapporte un PASS frais de cette commande; elle n'a pas été relancée pour ce dossier).

## Gates R1-R10 (readiness)

D'après `readiness.md`: R1-R6 automatiques, R7-R10 manuels. `tools/readiness.sh` les enchaîne via cargo; non exécuté.

| Gate | Contenu | Ce que dit le snapshot | Statut ici |
|---|---|---|---|
| R1 | Tokens synchrones | `check-tokens` | NOT_RUN |
| R2 | Tests moteur + générateur | `cargo test --workspace` (voir piège) | HISTORICAL_RECEIPT sur copie reconstruite |
| R3 | Threadé == séquentiel | `--features threaded-solver` | HISTORICAL_RECEIPT |
| R4 | Parité matériaux | `blobin-golden` | HISTORICAL_RECEIPT (4 tests) |
| R5 | Shim ≤ 300 LoC | `loc-guard.sh` | VERIFIED_NOW (282) |
| R6 | Pas d'`unsafe` hors FFI | `no-unsafe.sh` | VERIFIED_NOW |
| R7 | Budget de frame vs baseline v3.1 | signé 2026-06-10, commit `d07d161`, `scheduler_tick_all/16 ≈ 81.6 µs` (0,8 % de 10 ms), i7-12700F, Qt 6.11, rustc 1.96.0 | HISTORICAL_RECEIPT; benchmark non refait |
| R8 | Shader GPU == goldens raster | logiciel llvmpipe 4/4 pixel-perfect (2026-06-11, `664c277`); RTX 3070 numeric-pass: idle max_delta 4, SSIM 0,99535, autres états quasi exacts (2026-06-11, `339a0ca`); revue qualitative humaine OP 2026-06-12 (`02835e9`) | HISTORICAL_RECEIPT; GPU non réexécuté |
| R9 | Fallback « zéro régression » | le Qt-5 initial est déclaré inexécutable; scope re-signé 2026-06-12 vers `BLOBIN_BACKEND=cpp` sur Qt 6; fonctionnel et visuel signés `pass` | HISTORICAL_RECEIPT (voir réserve) |
| R10 | Dogfood 14 jours consécutifs, pas de P0/P1 | `tools/r10-sign-off.json`: `pending`, dates `null`; différé pour les phases précoces | NOT_PROVEN |

**Réserve R9 (HYPOTHESIS).** R9 est signé sur le backend `cpp`, mais dans ce snapshot `blobin-qt`/`blobin-cxxqt` ne contiennent aucune logique de sélection de backend et `backend.rs` ne fait que parser/rapporter. Si le chemin C++ hérité existe, il vit hors de ce snapshot (shell QML/natif qui l'a exécuté). Cela n'est ni établi ni réfuté ici; le sign-off R9 ne prouve donc pas qu'un « moteur C++ de fallback » existe dans cette source. Rien n'est à déduire non plus sur la possibilité de basculer le défaut: `readiness.md` exige R1-R10 pour un flip « rust par défaut », et R10 n'est pas rempli.

## Delta de B01 — `PENDING_RECONCILIATION`

Le travail de recherche B01 s'exécute en parallèle. À la rédaction de ce dossier aucun de ses résultats n'était disponible; **aucun fait B01 n'est intégré ni supposé**. Quand l'intégrateur les transmet, les ajouter ici sous la forme: assertion, commande, entrées, résultat, durée, statut `PENDING_RECONCILIATION`, puis les promouvoir dans le tableau principal seulement après qualification. Tout résultat frais contredisant un HISTORICAL_RECEIPT ci-dessus doit être traité comme une contradiction à départager, non comme une moyenne.

## Non-claims

- Pas de verdict de production, de publication crates.io ou de licence juridique.
- Pas de preuve d'un fallback C++ complet.
- Pas de preuve que l'export est autonome (c'est faux).
- Pas de reproduction GPU fraîche, pas de benchmark frais.
- Pas d'identification d'un commit Git amont.
- Publication publique (docs.gnu6.live, dépôt public, crates.io, npm, VPS): non autorisée.
