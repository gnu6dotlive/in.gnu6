# blob.in — Vérification et limites de preuve (interne)

**Statut:** `DRAFT_INTERNAL / PUBLICATION_NOT_AUTHORIZED`.

Ce document classe les résultats du receipt de `in.gnu6/docs/provenance/donor-blobin-appstack-b3-4603aed.md` daté 2026-10-07. **Aucun gate Rust/Qt/GPU n'est revendiqué comme fraîchement exécuté par la création de cette documentation.**

| Sujet | Observation qualifiée par le receipt | État actuellement transmissible |
|---|---|---|
| Snapshot | 95 fichiers, manifest digest `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d` | HISTORICAL_VERIFIED |
| Git suffix `4603aed` | Le commit court n'a pas été résolu comme objet Git | UNRESOLVED |
| Rust core seul | 47/47 tests PASS sur default, sequential, threaded | HISTORICAL_VERIFIED |
| Workspace complet exporté | GPUI dépend de `../gnosis-gpui-theme/Cargo.toml` absent | FALSIFIED_SELF_CONTAINED |
| Workspace reconstruit sans GPUI | Core, CXX-Qt, générateur, golden et Qt6/Corrosion/FFI/GLSL ont passé | HISTORICAL_VERIFIED / MODIFIED_WORKSPACE |
| Loc Qt/C++ shim | 282 / 300 (guard) | HISTORICAL_VERIFIED |
| GPU R8 | Checksums de captures et comparaisons validés; GPU non réexécuté | HISTORICAL_ARTIFACT_ONLY |
| `BLOBIN_BACKEND=cpp` | Contrat/future fallback annoncé, moteur C++ complet non établi | NOT_PROVEN |
| R10 dogfood (14 jours) | Fenêtre complète non reproduite | NOT_PROVEN |

## Procédure pour l'agent B01

1. Considérer `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in` comme **lecture seule**. Inventorier et hacher le snapshot avant/après toute inspection.
2. Examiner `Cargo.toml`, modules, manifest de provenance et `readiness.md`; comparer les affirmations du README historique au code réel.
3. **Ne pas** lancer un `cargo test` en modifiant le workspace historique ou en inventant la dépendance GPUI. Faire des copies temporaires uniques et vérifier que les seules modifications de qualification sont structurelles, documentées et réversibles.
4. Exécuter le plus petit gate utile (core seul, génération de tokens, bridge ou oracles), en conservant précisément commande, entrées, sortie, résultat et durée. `NOT_RUN` signifie `NOT_RUN`, même si un rapport ancien dit PASS.
5. Ne pas dériver un verdict de production, de publication crates.io ou de licence juridique depuis un test Rust.
6. Transmettre au rédacteur B02 via l'intégrateur des assertions sourcées, afin de mettre à jour ce dossier sans mélanger états historiques et nouveaux.

## Sources minimales

- `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in/Cargo.toml`
- `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in/README.md`
- `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in/readiness.md`
- `/mnt/workbench/workspaces/gnu.in.labs/in.gnu6/docs/provenance/donor-blobin-appstack-b3-4603aed.md`
- `/mnt/workbench/workspaces/gnu.in.labs/in.gnu6/QUESTLOG.md` SQ-003

**Interdiction:** publication publique automatique sur `docs.gnu6.live`, dépôt public blob.in, npm/crates.io ou VPS. Les contacts/permissions de publication devront faire l'objet d'une décision distincte.
