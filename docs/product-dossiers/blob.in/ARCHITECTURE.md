# blob.in — Architecture des composants (interne)

**Statut:** `DRAFT_INTERNAL / HISTORICAL_SNAPSHOT`. Les descriptions proviennent du snapshot `appstack-b3-baseline-4603aed`, `engine/blob.in` et du receipt `SQ-003`; elles ne sont pas un relevé du HEAD actuel d'un dépôt amont.

## Chaîne des responsabilités

```text
tokens.json ── blobin-gen ──> tokens.rs / Rust + C++ + GPUI themes
                                   |
geometry/path + solver/state ──> blobin-core (Rust)
                                   |
                      mesh/flat buffer + cxx bridge
                                   |
                  Qt6 C++ / QSG / shaders (projection)
                                   |
                         scène native consommateur

blobin-golden: oracles CPU + images de référence + parité
blobin-bench: mesures spécialisées (pas qualification live par défaut)
blobin-gpui: adapter consommateur présent, mais dépendance non disponible
```

## Crates et modules

| Module | Responsabilité observée dans la source | Limite de l'assertion |
|---|---|---|
| `blobin-core` | `Path/Contour`, `RectBlob`, `SuperBlob`, `Inverted`, transforms glam, Lyon fill/tessellation | Core indépendant validé historiquement |
| `blobin-core` dynamics | Damped spring, easing cubic-Bézier, interpolation d'état, scheduler/dirty/cold | Causalité runtime à requalifier si changement |
| `blobin-core::ffi` | `#[cxx::bridge]` pour exporter les buffers vers la projection | Ne prouve pas un FFI universel ou stable |
| `blobin-qt` | QQuickItem, QSGGeometry/QSGMaterial, Qt RHI et shaders | Build de copie reconstruite historiquement PASS |
| `blobin-cxxqt` | Objets réactifs de surface | Test isolé historiquement PASS |
| `blobin-gen` | Production des tokens de thèmes et constantes depuis `tokens.json` | Vérifier la synchronisation sur toute nouvelle source |
| `blobin-golden` | Référence de rendu CPU, oracles pixel/SSIM | Parité historique ≠ nouveau run matériel |
| `blobin-bench` | Harnais Criterion pour métriques | Requiert machine, protocole et runs datés |
| `blobin-gpui` | Adaptateur/consommateur GPUI | Workspace parent incomplet dans l'export préservé |

## Distinctions nécessaires

**Le cœur géométrique Rust n'est pas la projection Qt.** L'interface peut exposer des buffers de maillage plats sans transférer toute la dynamique ni toute la gestion d'état à QSG.

**Les tokens sont une source déclarative.** Évaluer séparément les garanties réelles de synchronisation entre `tokens.json`, sorties générées et consommateurs. Les anciens choix QML/Dart sont retirés selon la documentation historique: ne pas les présenter comme des cibles actuelles garanties.

**Backend C++:** le nom de variable `BLOBIN_BACKEND=cpp` est documenté, mais le code et le receipt ne prouvent pas un fallback C++ complet. Pas de diagramme donnant ce chemin comme `AVAILABLE`.

**Lien avec `in.gnu6`:** seules des lois donor et des fixtures ont été récoltées (`tests/fixtures/blobin-donor.ts`, `tests/model/blobin-donor-laws.test.ts`). `in.gnu6` possède sa propre ontologie scène/topologie/temps. Il ne faut pas fusionner silencieusement ces deux moteurs.

## Prochaines preuves

1. Identifier un HEAD blob.in vivant et sa provenance si disponible, sans remplacer le snapshot historique par supposition.
2. Résoudre la frontière GPUI d'une manière non intrusive (copie de qualification, pas patch direct du donor).
3. Requalifier les chemins de rendu et les écarts GPU à la demande sur machine instrumentée.
4. Reconstituer le statut R10 et les décisions de release avant toute publication.
