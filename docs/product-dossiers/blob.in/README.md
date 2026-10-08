# blob.in — Dossier produit (interne)

**État:** `DRAFT_INTERNAL / SOURCE_SNAPSHOT_OBSERVED / PUBLICATION_NOT_AUTHORIZED`

blob.in est un moteur de formes organiques et de mouvements issus des surfaces historiques Gnu.In-Shell. Le snapshot documenté montre un socle Rust géométrique, du rendu Qt/C++ ainsi que des adaptateurs/consommateurs et outils de génération. Il ne faut pas réduire blob.in à un « donor » de `in.gnu6`: ce sont **deux produits avec des responsabilités distinctes**.

## Identité et source

- Snapshot conservé, en lecture seule: `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in`
- Référence de qualification existante: `in.gnu6/docs/provenance/donor-blobin-appstack-b3-4603aed.md` (reproduction du 2026-10-07).
- Version **déclarée** par `Cargo.toml`: `0.14.0`; licence **déclarée** dans metadata Cargo: `GPL-3.0-or-later`.
- Repository **déclaré** dans metadata: `https://github.com/gnu-in-labs/blob.in`. Sa visibilité et son statut de publication courant **ne sont pas qualifiés ici**.
- Snapshot: 95 fichiers dans le manifeste de qualification existant. L'étiquette `4603aed` **n'est pas requalifiée comme un commit Git résoluble**.
- Digest SHA-256 du manifeste de fichiers cité par le receipt: `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d`.

## Ce qui est étayé

- Noyau Rust: chemins Bézier, `RectBlob`, `SuperBlob`, winding/inversion, transformations, tessellation lyon, matériaux, easing/ressorts, état mascotte et scheduler.
- Interface FFI `cxx::bridge`, connecteur Qt6/Corrosion, composantes CXX-Qt, shaders et oracles de parité de matériaux.
- Les tests du **core isolé** ont passé 47/47 sous trois configurations dans le receipt SQ-003. La copie reconstruite sans membre GPUI manquant a qualifié une compilation Rust/CXX/Qt. Ces résultats sont **historiques**, non réexécutés aujourd'hui.

## Ce qui n'est pas qualifié

- Le **workspace pristine** ne se construit pas seul: dépendance du membre `blobin-gpui` sur `../gnosis-gpui-theme/Cargo.toml` manquante dans l'export.
- Le mode `BLOBIN_BACKEND=cpp` n'établit pas un moteur C++ de fallback fonctionnel.
- Les signatures R7/R8/R9 et captures GPU appartiennent aux preuves archivées; pas de reproduction GPU fraîche.
- Le dogfood **R10 sur 14 jours** demeure non prouvé.
- Aucune release ou distribution publique n'est autorisée par la rédaction du présent dossier.

## Navigation

- [Architecture et responsabilités](ARCHITECTURE.md)
- [État de vérification et reproduction](VERIFICATION.md)

**Frontière avec `in.gnu6`:** certaines lois de courbes, winding, ressorts et continuité ont été récoltées comme capital technique dans `SQ-003`; cela ne transmet ni l'ontologie ni les choix Qt/CXX/GPU à `in.gnu6`.
