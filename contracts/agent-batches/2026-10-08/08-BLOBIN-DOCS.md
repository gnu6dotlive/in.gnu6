# B02 — Construction de la documentation blob.in

**Objectif:** **écrire une vraie documentation privée, consultable et reproductible pour blob.in** en parallèle avec B01. La documentation est un dossier produit indépendant; la relation donor avec `in.gnu6` est une annexe, pas sa définition.

**Sources RO:** snapshot `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in`, et reçu `docs/provenance/donor-blobin-appstack-b3-4603aed.md`. B01 travaille simultanément sur son rapport; les résultats B01 non disponibles ne deviennent pas des faits par anticipation.

**Write-set exclusif:** `docs/product-dossiers/blob.in/**` (les trois documents initiaux de ce lot peuvent être améliorés). B01 n'écrit pas ici.

**Livrables minimaux:** (1) `README.md` vue produit, statut, audience, navigation, frontières et version; (2) `ARCHITECTURE.md` crates Rust + FFI + Qt/GPUI, diagramme et ownership; (3) `VERIFICATION.md` procédures de preuve reproductibles, piège `gnosis-gpui-theme`, résultat historique vs exécution fraîche, R7-R10; (4) au besoin `SOURCES.md` pour digest, version, licence, metadata et inventaire.

**Documentation vraie:** pas de `cargo test` au root donné comme garantie lorsque le workspace historique ne se résout pas; proposer l'isolation du crate ou la reconstruction explicitement marquée `QUALIFICATION_COPY`. Signaler `cpp fallback` non prouvé, R10 14-day non prouvé, GPU R8 historique seulement; ne pas appeler ce snapshot une release prête à distribuer. Décrire la GPL comme metadata source sans recopier les sources GPL.

**Gate:** toutes les assertions ont un pointeur file/receipt, les liens internes se résolvent, le statut `DRAFT_INTERNAL / PUBLICATION_NOT_AUTHORIZED` figure partout; les docs restent parfaitement utilisables même si B01 n'a pas encore livré.

**Remise:** docs privées sourcées + delta de faits reçus de B01, marqués `PENDING_RECONCILIATION` tant qu'ils ne sont pas qualifiés. Ne pas toucher au portail public.
