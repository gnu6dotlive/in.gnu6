# UPSTREAM-R&D-01 — Questlog du lot

Statut: **SOURCE_INTAKE_COMPLETE / CONTRACTS_DRAFTED / EMPIRICAL_NOT_RUN**
Date: 2026-10-10 · Autorité: GO explicite de l'opérateur, publication de la branche de recherche privée uniquement.

## Acquis vérifiables
- Poste réel `gnosisos-x8664`, `gnu6-lab-run --check` → `GNU6_LAB_READY`.
- Base isolée `in.gnu6@4042faa` depuis `origin/main`; `in.gnu6/main@d160639` (53 commits locaux) exclu du push.
- Ancien worktree `in-gnu6-agent-contracts` possède un lien Git cassé; documentation conservée, aucune réparation destructive.
- Six dépôts upstream, six SHA et six README blob SHA : `sources.json` (acquisition en six tâches concurrentes).
- Douze fichiers à SHA exact vérifiés par l'API GitHub : `evidence/upstream-rd-01/intake/source-probe.json`.
- Rapports d'intake W01–W06 : `docs/agent-results/upstream-rd-01/`. **Ce sont des probes source**, pas six agents indépendants.
- Douze contrats W00–W11 rédigés, dispatch structuré; aucun produit modifié.

## Gates
- G00 baseline/isolation: PASS; G01 pins/source access: PASS pour six commits et douze fichiers, licences distributives à approfondir.
- G02 lecture source: PASS pour l'intake; revue approfondie des API et consommateurs: OPEN.
- G03 convergence interprojets: DRAFT; G04 bwrap startup host smoke: PROBE_PASS after fixture repair; MXC SDK, GPU, runtime and performance tests: NOT_RUN.
- G05 transfert projet: NOT_RUN; G06 publication: DRAFT_ONLY; G07 premier commit/push: VERIFIED_REMOTE at d9bcb0488c88a4447d5731d35e00b9bf230a5462, remote SHA equals local SHA.

## Interfaces et conflits à préserver
- `in.gnu6` n'acquiert pas l'autorité runtime/security/UI du fait de cette recherche.
- `gnostral.rs` a un worktree local divergent/dirty; aucune édition.
- `engineering-corpus` et `gnosix` ont des branches actives; aucune édition.
- L'autorisation de rendre public `in.gnu6` ne doit pas être inférée de l'autorisation de pousser une branche sur son dépôt **privé**.

## Suite
D'abord vérifier le push privé isolé; ensuite former les expériences légères à oracle indépendant, puis la revue des licences et des projets bénéficiaires. Ni release publique ni preuve runtime acquise.

## 2026-10-10 — Tranche W01–W08, reprise après le push 947bd79

State: **RESEARCH_SOURCE_QUALIFIED / HOST_BWRAP_BOUNDARY_PASS / PRODUCT_NOT_RUN**.

- Préflight repris : branche centrale `947bd79` propre et SHA remote concordant; `in.gnu6/main` toujours 53 commits locaux en avance et non modifié.
- Six worktrees `upstream-rd-01-w01-20261010`…`w06` créés sur la base `db794a6`, chacun avec contrat distinct et deux chemins d'écriture autorisés.
- Six workers de collecte déterministes lancés simultanément avec `gnu6-lab-run`; ils ne sont **pas des agents LLM autonomes**. Commits locaux source: W01 `f646b5a`, W02 `05edcf2`, W03 `13be7bc`, W04 `b100fb2`, W05 `a8088df`, W06 `54f0b00`.
- Six cherry-picks réconciliés dans la branche centrale après vérification parent/écriture (2 chemins distincts chacun), aucun conflit. 18 fichiers supplémentaires à révision exacte vérifiés, trois par upstream; ajoutés aux 12 probes initiales (30 reçus de fichiers, chemins parfois redondants).
- W07 [synthèse](../../../docs/agent-results/upstream-rd-01/cross-analysis.md) : tous les projets bénéficiaires cartographiés, limites et hypothèses distinguées.
- W08 [expériences](../../../docs/agent-results/upstream-rd-01/experiments.md) : oracle Bubblewrap hôte **PASS_BOUNDARY_FIXTURE** (EROFS sur fichier témoin, /tmp writable, ENETUNREACH sur l'adresse test, namespace réseau distinct, témoin intact). MXC SDK, Deno, GPUI Wayland, Uzu CUDA/CPU et Hyperfine réel : NOT_RUN/BLOCKED selon le cas.
- W09 [réconciliation](../../../docs/agent-results/upstream-rd-01/integration.md) : décision de **pousser la recherche privée seulement**, aucune promotion produit. W10 [surfaces](../../../docs/agent-results/upstream-rd-01/docs-bridge.md) : statut documentaire privé, aucune propagation/déploiement de site.

Gates: G00/G01 PASS; G02 SOURCE_DEEP_REVIEW_PASS; G03 SOURCE_CROSS_ANALYSIS_PASS; G04 HOST_FIXTURE_PASS / PRODUCT_SDK_NOT_RUN; G05 TRANSFER_NOT_RUN; G06 PUBLIC_DRAFT_ONLY; G07 REMOTE_SHA_RECHECK_REQUIRED_AFTER_NEXT_PUSH.

No public API, release, donor vendoring, merge main, source deletion, control plane mutation or production service changed.

## 2026-10-10 — Tranche MXC + Deno / W01A, W03A

**SOURCE_GUARDS_PASS / HOST_NETWORK_NAMESPACE_PASS / SDK_AND_JS_RUNTIME_NOT_RUN.**

- Préflight `gnu6-lab-run --check` PASS; branche `7006adb9` propre et SHA remote égal. Branche `main` de in.gnu6 inchangée, toujours 53 commits locaux hors du lot.
- MXC: clonage Git exact en lecture de code uniquement à `c6f301d53a1430c4c921a05c57af838f7392348f` dans `.probes/upstream-rd-01/mxc-c6f301`. Le code est vérifié; aucune modification dans la copie upstream.
- MXC: ordre des validations `BubblewrapScriptRunner` vérifié par un gate à **7 mutations RED / 7**. Deux contrôles TCP loopback sous bwrap : contrôle non isolé atteint le port local et conteneur `--unshare-net` ne l'atteint pas; namespaces différents. C'est un **test Linux hôte**, pas une exécution MXC SDK.
- MXC: `cargo metadata --offline --locked --no-deps` parse les métadonnées. Résolution `cargo tree -p mxc-sdk --offline --locked --depth 1` bloquée sur la dépendance `cidr` non présente; toolchain amont 1.93 non installée, 1.97/1.97.1 disponibles. Aucun téléchargement de dépendance ni compilation SDK.
- Deno: `broker.rs` + `lib.rs` téléchargés à SHA exact `a18ce33715e30cd2b0d99c7e322ef11a65490e4d`; 10 contrôles et **10 mutations ciblées RED**, oracle amélioré suite à une première faiblesse des correspondances.
- Deno: broker opère avant le `PermissionState` normal lorsque présent; allow/deny explicite, réponse ID corrélée, variantes invalides refusées, erreurs broker conduisant à la terminaison du processus dans le code lu. **Pas d'exécutable Deno sur PATH, aucun test runtime.**
- Contrats spécialisés `12-MXC-SECURITY-01.md`, `13-DENO-PERMISSION-01.md` ; analyse commune future `14-MXC-DENO-BOUNDARY.md` conçue, **non exécutée**.
- Deux worktrees temporaires isolés réconciliés par cherry-pick: MXC `4dd46289001f1e128a0b599807df0a8c5be59bff`, Deno `0102ce58873f72f8f18397ababc6f085df8c2f0f` ; chaque write-set unique, aucun autre projet modifié.

Gates: MXC_SOURCE=PASS, MXC_HOST_LOOPBACK=PASS, MXC_SDK_RUNTIME=NOT_RUN/BLOCKED_DEPENDENCY; DENO_BROKER_SOURCE=PASS, DENO_RUNTIME=NOT_RUN/BINARY_ABSENT; COMPOSITE_MXC_DENO=DESIGNED_ONLY; PUBLICATION=PRIVATE_GIT_BRANCH_ONLY.

## 2026-10-10 — Tranche MXC/Deno exécutables · runtime-02

- Préflight: branche `f6f1e23` propre et synchronisée. Deux branches/worktrees neufs à write-sets disjoints.
- **MXC 1.0.0** : `microsoft/mxc@c6f301d53a1430c4c921a05c57af838f7392348f` checkout exact; Cargo fetch dans cache isolé; `cargo check --offline --locked -p mxc-sdk --lib` PASS (Rust 1.97.1). Harnais Rust du SDK compilé dans répertoire de laboratoire; scénario readonly : lecture réussie et écriture refusée avec fichier marqueur absent; scénario readwrite : écriture du marqueur réussie. **MXC_SDK_FILESYSTEM=PASS_2_OF_2**. Aucun code amont modifié. Référence `docs/agent-results/upstream-rd-01/mxc-runtime-02.md`.
- **Deno v2.9.7** : release officielle Linux téléchargée et archive SHA-256 vérifiée; exécutable isolé dans `.probes`. Six permissions réellement exercées : read absent, read exact allow, autre chemin refusé, deny explicite prioritaire, réseau local refusé par défaut, réseau local exact allow. **DENO_RUNTIME=PASS_6_OF_6**; refus `NotCapable`. Deno v2.9.7 est antérieur au SHA source 2026-10-06 : **non équivalent à la qualification du commit épinglé ni de son broker**.
- Les preuves, harnais et rapports ont été acquis sur branches MXC `a70a7c5` et Deno `2d38a5a`, puis réconciliés par cherry-pick sur le lot central. Aucun projet bénéficiaire ni politique de production modifié.
- **W14 MXC×Deno composite** : non exécuté. MXC et Deno séparément qualifiés sur des fixtures ne prouvent ni l'intégration des deux, ni la non-amplification de `ExecutionGrant`.
- Visibilité GitHub, contrôle ponctuel du 2026-10-10 : `tension-atoi/gnosix` PUBLIC confirmé; `tension-atoi/in.gnu6` PRIVATE selon l'API au moment de la vérification, malgré une annonce de publication. Aucune mutation de visibilité demandée ou réalisée dans cette tranche.

Gates : MXC_BUILD=PASS; MXC_RUNTIME_FILESYSTEM=PASS_2_OF_2; DENO_RELEASE_SHA256=PASS; DENO_RUNTIME_PERMISSIONS=PASS_6_OF_6; DENO_PINNED_RUNTIME=NOT_RUN; DENO_BROKER_LIVE=NOT_RUN; MXC_DENO_COMPOSITE=NOT_RUN; PRODUCT_TRANSFER=NOT_RUN; GIT_REMOTE=TO_VERIFY_AFTER_PUSH.
