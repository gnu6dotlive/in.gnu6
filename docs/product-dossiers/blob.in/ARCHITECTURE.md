# blob.in — Architecture (interne)

**Statut:** `DRAFT_INTERNAL / PUBLICATION_NOT_AUTHORIZED / HISTORICAL_SNAPSHOT`

Source de tout ce document: snapshot `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in` (lecture seule) et le receipt `docs/provenance/donor-blobin-appstack-b3-4603aed.md`. Les chemins entre backticks sont relatifs au snapshot. Ce n'est pas un relevé d'un HEAD amont. Retour: [README](README.md).

## Diagramme

```text
tokens.json ──blobin-gen──> blobin-core/src/tokens.rs        (constantes solveur)
                       ├──> gen/gnu_theme.rs                  (consommateurs Rust)
                       ├──> gen/gnu_theme.h                   (C++ constexpr)
                       └──> gen/gnu_theme.gpui.rs             (palette GPUI)

                 blobin-core  (Rust, staticlib + rlib)
   path / blob / group ─> tessellate (lyon) ─> engine / pool (scheduler)
   solve (easing, ressort)  state (mascotte)  material  raster/diff (oracle CPU)
                       │
                  ffi.rs  #[cxx::bridge]   FfiVertex / FfiMaterial (#[repr(C)])
                       │      buffers plats vertices/indices/material
        ┌──────────────┴───────────────┐
   blobin-qt (C++/CMake, Corrosion)   blobin-cxxqt (CXX-Qt 0.8)
   QQuickItem, QSGGeometry,           objets réactifs Mascot / Bloom
   QSGMaterial, shaders .qsb          (Q_PROPERTY + signaux)
                       │
              scène Qt Quick (consommateur)

   blobin-gpui ──> blobin-core + gnosis-gpui-theme (ABSENT) + gpui (Zed, rev épinglé)

   blobin-golden : raster CPU vs PNG de référence (pixel-diff + SSIM)
   blobin-bench  : Criterion (frame_budget, scheduler_throughput)
```

## Crates et composants

| Composant | Rôle (source) | Limite de l'assertion |
|---|---|---|
| `blobin-core` | Moteur: `path.rs` (Bézier cubique), `blob.rs` (`RectBlob`, `SuperBlob`, `Inverted`), `group.rs` (affine glam), `tessellate.rs` (lyon), `solve.rs`, `material.rs`, `raster.rs`, `diff.rs`, `state.rs`, `engine.rs`, `bloom.rs`, `pool.rs`, `backend.rs`, `ffi.rs`, `tokens.rs` (généré). Crate-type `staticlib` + `rlib`; feature `threaded-solver` (rayon) par défaut. `rust-version = 1.82`. | Présence du code OBSERVÉE; comportement = receipts historiques |
| `blobin-core::ffi` | Seul pont chaud: `#[cxx::bridge]` exposant `EngineHandle`, `SchedulerHandle`, `BloomHandle`, `FfiVertex`, `FfiMaterial`, `backend_from_env()` | Ne prouve ni FFI stable ni ABI publique |
| `blobin-qt` | Shim C++ (CMake + Corrosion, pas de `Cargo.toml`): `blobin_item.*`, `blobin_material.*`, `blobin_bloom_item.*`, `shaders/blob.{vert,frag}`, harnais `tests/render_goldens` | Garde LoC: 282/300 (VERIFIED_NOW, voir VERIFICATION) |
| `blobin-cxxqt` | Surface réactive CXX-Qt (`mascot.rs`, `bloom.rs`), crate `staticlib` | 1 test, historique |
| `blobin-gen` | Binaire: `tokens.json` → Rust, thèmes Rust/C++/GPUI (serde) | 7 tests, historique |
| `blobin-golden` | Goldens matériaux + binaire `diff-png`; preuves GPU archivées sous `r8-hardware/` | 4 tests, historique |
| `blobin-bench` | Benchmarks Criterion 0.5 | NOT_RUN |
| `blobin-gpui` | Adaptateur GPUI (`canvas.rs`, `orb.rs`); dépend de `gnosis-gpui-theme` (chemin `../../gnosis-gpui-theme`) et de `gpui` (git zed-industries/zed, rev `00bd72e7838f4b875a913cd112b47a0ebe1ca62b`) | Non buildable dans l'export: dépendance locale absente |

## Ownership (qui possède quoi)

| Sujet | Propriétaire dans la source | Ce qui n'est PAS transféré |
|---|---|---|
| Géométrie, tessellation, solveurs, état, ordonnancement, matériau (mathématique) | `blobin-core` (Rust) | Qt ne recalcule rien; il reçoit des buffers plats |
| Projection de scène (nœuds, géométrie GPU, shader) | `blobin-qt` (C++), budget ≤ 300 LoC | La sémantique des états reste côté Rust |
| Constantes de mouvement/couleur | `tokens.json` (source déclarative unique) | Les sorties générées sont des dérivés; leur synchro = garde R1 |
| Surface réacti