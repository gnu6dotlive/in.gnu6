# B01 — Recherche indépendante sur blob.in

**Objectif:** qualifier **blob.in comme produit** (moteur de formes organiques), distinct de son rôle historique de donor pour `in.gnu6`. Exécuter **en parallèle** de B02.

**Sources RO:** snapshot conservé `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in` et son parent uniquement en lecture; `docs/provenance/donor-blobin-appstack-b3-4603aed.md`, `QUESTLOG.md` SQ-003, `tests/fixtures/blobin-donor.ts`. Lire le metadata du repo déclaré `https://github.com/gnu-in-labs/blob.in` seulement si l'accès est autorisé, sans supposer que le dépôt est public.

**Write-set exclusif:** `docs/agent-results/par-01/blobin-research.md`. Aucune écriture sur le snapshot, ni dans `docs/product-dossiers/blob.in/` (réservé à B02).

**Point de départ vérifié historiquement (pas résultats du jour):**
- Snapshot 95 fichiers, digest manifest `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d`; suffixe `4603aed` **non résolu comme objet Git vivant**.
- `blobin-core` isolé: 47/47 tests avec default, sequential et threaded; copie reconstruite excluant `blobin-gpui`: Qt6/Corrosion/CXX/GLSL/tests compilables.
- Snapshot complet non autonome: dépendance absente `../gnosis-gpui-theme/Cargo.toml`.
- `BLOBIN_BACKEND=cpp` ne prouve pas un moteur C++ fallback implémenté; R10 dogfood 14 jours non prouvé.
- `version=0.14.0` et `GPL-3.0-or-later` sont **déclarés dans metadata**, pas une validation juridique.

**Investigation:** reconstruire la topologie réelle des crates, imports, geometry/tessellation/solver/FFI, Qt/GPUI/token authority, état de sign-off R7→R10 et exactitude du README historique; séparer `REPRODUCED_NOW`, `HISTORICAL_ONLY`, `BLOCKED` et `FALSE`. Si reproduction nécessaire, **copie temporaire isolée**, manifest pré/post inchangé; pas de modification de source.

**Gate:** vérifier la provenance par inventaire/digest, inspecter au moins une contradiction (pristine workspace / fallback / Qt), et enregistrer les commandes et contre-preuves. Pas de nouvelle revendication de GPU benchmark sans GPU test réel.

**Remise:** matrice composants/proofs/gaps, statut produit non marketing, recommendations de docs pour B02 et intégrateur. Ne pas rouvrir SQ-003 de `in.gnu6` automatiquement.
