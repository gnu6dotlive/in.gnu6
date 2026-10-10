# IN-GNU6-PAR-01 — Rapport d'intégration locale

**Statut :** `INTEGRATED_LOCAL / BLOBIN_RECONCILED / NO_PUSH / PUBLICATION_NOT_AUTHORIZED`
**Date :** 2026-10-08
**Intégrateur :** session opérateur ChatGPT, worktree privé `feat/par-01-integration-20261008`
**Dépôt :** `tension-atoi/in.gnu6` (privé)
**Baseline commune :** `c9dcdab997f63f67caaa182cb265d6cb9b970863`
**Dernier HEAD du moteur avant contrats :** `d160639e804c244a84197d8dc4906ffa2df313e8`.

## 1. État et autorité

Les huit workers ont terminé sur des worktrees exclusifs. Leur travail est strictement documentaire/recherche, ne modifie ni `src/**` ni `tests/**`, ni le moteur Gnosix, ni les consommateurs. Tous les worktrees agents étaient clean lors de l'intégration et tous leurs commits avaient `c9dcdab` pour parent direct.

La fusion a eu lieu dans un **nouveau worktree d'intégration**, et non dans `main`. Les branches workers ont été préservées; aucun `push`, merge dans `main`, publication ou déploiement n'a été effectué. Le `main` local était en avance de 53 commits sur `origin/main` au départ, et le worktree SQ-011 contenait `qualification/mesh-avatar/` non suivi : tous deux laissés inchangés.

Le `dispatch.json` de création contient encore `CONTRACTS_READY_WORKERS_NOT_STARTED` : c'est **son état historique au moment du dispatch**, pas un état opérationnel actuel. Le présent reçu fait foi pour la fin du batch. Seul un formatage mécanique des arrays de ce JSON a été ajouté à cette branche afin de respecter Biome.

## 2. Provenance des huit remises

| Worker | Commit worker | Cherry-pick intégration | Livraison |
|---|---|---|---|
| A01 | `43b246b` | `8a89e62` | `engine-ontology.md` |
| A02 | `30aa514` | `cda48db` | `renderer-envelope.md` |
| A03 | `73d4c69` | `2fd26f3` | `consumer-bridges.md` |
| A04 | `2d40004` | `bc32178` | `gnosux-compatibility.md` |
| A05 | `7345993` | `6d86967` | `observation-performance.md` |
| A06 | `750c1f6` | `c3e3fb8` | `public-docs-candidates.md` |
| B01 | `348b52f` | `e81b347` | `blobin-research.md` |
| B02 | `5d73fd8` | `21e4525` | `docs/product-dossiers/blob.in/{README,ARCHITECTURE,VERIFICATION,SOURCES}.md` |

**Chaque rapport original est préservé indépendamment.** La réconciliation B01/B02 est écrite dans le dossier produit et n'altère pas les receipts originaux.

## 3. Réconciliation blob.in

Source: snapshot immuable sous `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in` (aucune écriture); `SQ-003` receipt de `docs/provenance`; rapports B01/B02.

**Digest résolu — VERIFIED_NOW :**
- 95 fichiers.
- Depuis la racine du snapshot, `find . -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum` produit `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d`.
- Reproduit par B01, puis **indépendamment par l'intégrateur**, identique avant et après lecture.
- La recette du reçu historique n'était pas écrite; notre recette reconstitue son résultat. La tentative de remplacement de cette recette GNU par un calcul Python utilisant son propre ordre de tri a donné un hash différent : **ne pas la substituer comme un oracle équivalent** sans qualifier le tri/octets produits.

**B01 frais (sur QUALIFICATION_COPY, pas workspace original) :** blobin-core 47/47 sous default, no-default-features et threaded; workspace Rust sans membre GPUI, générateur 7 tests, golden 4, cxxqt 1, et `tokens.rs` régénéré identique. Vérification statique du shim 282/300 et no-unsafe; checksum des archives R8 valable. **Aucun build Qt/CMake, benchmark GPU ou test GPU frais**. Les affirmations correspondantes des anciens sign-offs restent `HISTORICAL_RECEIPT`. R10 14 jours reste `pending`.

**Contradictions conservées :**

| ID | Preuve | Conclusion bornée |
|---|---|---|
| C1 | `backend_from_env()` déclaré mais aucun appelant dans l'export; sign-off R9 de fallback C++ sur Qt6 | Fallback réellement opérationnel dans le snapshot **NOT_PROVEN**; chemin externe éventuellement existant **UNKNOWN** |
| C2 | `backend.rs`/README: défaut `rust`; `readiness.md`/`components.toml`: défaut `cpp` | `CONFLICTING_DECLARATIONS`, aucune bascule autorisée |
| C3 | README: 205 LoC vs garde: 282/300; modules GPUI/bench absents de son ancien diagramme | `STALE_DOCUMENTATION`, valeur actuelle 282/300 |
| C4 | Cargo 0.14.0/GPL-3.0-or-later; `tokens.json` 0.20.0; `components.toml` et LICENSE parent AGPL-3.0 | Versions et licence produit **NON RATIFIÉES**, aucune conclusion juridique |

L'éventuel SHA complet `4603aedde4d189d2e4bdd3e65872faff237aaf7d` vient d'un **nom de log** et n'est pas résolu en Git object ni lié aux fichiers du snapshot : `HYPOTHESIS`, jamais source Git qualifiée.

Les documents privés `README`, `SOURCES`, `VERIFICATION` ont été amendés pour distinguer les faits fraîchement reproduits, les rapports historiques et les décisions produit encore ouvertes.

## 4. Résultats des autres missions — portée réelle

- **A01 ontologie :** atlas avec 17 assertions et huit hypothèses. `SpatialFrameGraph.reparent(id,null)` (H2) paraît un candidat concret à falsification (opérateur `??` dans le chemin de patch); **pas de test H2 exécuté** ni de changement de code. H1 ownership et deck/SceneRole restent hypothèses/sélection de frontière.
- **A02 renderers :** tests source 12/12 et Playwright 2/2 par le worker; identité et continuité qualifiées dans l'instrument. Parité pixels multi-providers, GPU physique et WebGPU **NOT_RUN**; `SQ-007` reste LATER.
- **A03 consumers :** 22/22 fichiers source vendored égalés à `in.gnu6@2602d30` au tag `experience-v6.8.0`, tests moteur 14/14; tests consommateurs live **NOT_RUN**; SPJ-IN-01 proposition optionnelle non promue.
- **A04 gnos.ux :** 6 tests Rust surface PASS; suite TS 129 PASS avec dépendances copiées d'un checkout frère (lock non vérifié par le worker). `EntityId(string)` TS et `SceneNodeId(u64)` Rust ne sont **pas** une identité commune; aucun FFI/IPC ratifié. Mapping et ownership live **UNKNOWN**.
- **A05 performances :** chiffres de receipts historiques; benchmark nouveau **NOT_RUN**. Ratio ~360x observation/step historiquement mesuré sur un layout d'occlusion particulier; aucune généralisation. `SQ-008` priorité UNSET.
- **A06 docs publiques :** huit phrases conceptuelles proposées pour revue; toutes `public_authorized?=NO`. `RATIFIED COMPLETE` n'est pas une promesse de disponibilité publique. Aucune page, API ou SDK public produit par ce lot.

## 5. Gates d'intégration

- Reconstruction des huit commits sur base commune avec write-sets exclusifs : **PASS**. Un commit unique par worker; aucun changement des sources moteur.
- QA documentaire de réconciliation : **PASS** — 7 rapports individuels + dossier blob.in 4 fichiers, 14 liens locaux résolus, registre 8 workers avec write-sets indépendants, C1–C4 tous explicitement présents.
- Snapshot digest 95 fichiers, pipeline GNU original : **PASS**, hash identique au reçu et à B01.
- Formatage `dispatch.json` (seul écart découvert par le format gate) : corrigé par Biome en branche d'intégration, contenu sémantique JSON inchangé.
- `npm run check` dans le worktree d'intégration : **PASS**, format, lint, `tsc --noEmit` et Vitest **32 fichiers / 129 tests PASS**. Instrument: Node.js v26.10.0, dépendances résolues via un symlink *temporaire* vers le `node_modules` existant du dépôt frère; **pas de npm ci frais**. Le lien est supprimé en sortie du gate.
- `git diff --check` : **PASS**.
- Browser de produit, GPUI/Wayland, GPU/Qt, benchmarks, nouveaux tests de falsification A01 H2 : **NOT_RUN** par l'intégrateur.

Ces gates vérifient la cohérence de l'intégration documentaire et les tests internes courants; ils ne qualifient pas la readiness de blob.in ni une compatibilité de consommation live.

## 6. Décisions non prises, prochains mouvements recommandés

1. **Prochain petit slice de vérification :** H2 `reparent(id,null)` de A01 dans un worktree d'expérience isolé, test RED→GREEN uniquement si l'hypothèse est confirmée. Ne pas réouvrir MQ-00→MQ-15 par automatisme.
2. **Propriétaire blob.in :** préciser le default backend, version workspace vs tokens et autorité de la licence (vérification juridique séparée), puis déterminer si un moteur C++ legacy extérieur au snapshot rend R9 intelligible. Pas de copie de code GPL/AGPL ni publication automatique.
3. **Publication :** C01–C08 d'A06 demeurent bloquées. Une description publique de `in.gnu6`/blob.in exige une décision opérateur explicite et une revue de confidentialité des sources; aucune publication demandée ou réalisée.
4. **Plus tard :** choisir un consumer concret avant toute intégration supplémentaire gnos.ux/gnu6.live; conserver SQ-007 LATER, SQ-008 UNSET.

**Clôture :** `WORKER_REPORTS_INTEGRATED_LOCAL / BLOBIN_DOCS_RECONCILED / MAIN_UNTOUCHED / NO_PUSH / PUBLICATION_NOT_AUTHORIZED`.
