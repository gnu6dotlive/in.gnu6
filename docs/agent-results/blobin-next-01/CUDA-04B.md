# CUDA-04B — Resident Cache Reconciliation (qualification partielle)

Parent : CUDA-04 `6ee8363`. Branche : `feat/blobin-cuda-04b-resident`.

## Implémentation
- Adaptateur `ResidentBridge` qui exploite directement les sessions résidentes CUDA-03 CUDA/Vulkan, sans modifier le donneur.
- Empreinte de contenu géométrique SHA-256 héritée de CUDA-04; provenance F2 réattachée à la réponse sans mutation.
- Le cache de la dernière sortie n'est éligible qu'avec une session vivante et un digest identique.
- Modification du contenu à forme identique : abandon de la sortie précédente avant `update → launch → readback`.
- Modification de largeur, hauteur ou nombre de primitives : destruction et reconstruction de la session. Les dimensions sont comparées individuellement (CUDA-03 comparait seulement le nombre de pixels).
- Erreur de mise à jour, d'exécution, de lecture ou sortie de mauvaise taille/non finie : invalidation de la session et du cache; pas de restitution de l'ancienne sortie.
- `resident_bytes` reflète uniquement le comptage CUDA-03, sans mémoire driver/staging.

## Périmètre de qualification
Les tests Rust contrôlent les chemins négatifs CPU interdit, bibliothèque CUDA absente, dimensions égales en pixels mais différentes en forme. Ils ne simulent pas de device loss ni la validité d'un cache après perte silencieuse de contexte.

## Limites ouvertes (bloquantes pour clôture complète)
- Mesures natives CUDA et Vulkan, mutation réelle et identité exacte des pixels face à l'oracle CPU.
- Latence p50/p95 de la chaîne complète, mises à jour et readbacks.
- Signaux de device loss : un cache-hit en mémoire ne peut détecter seul une perte asynchrone du GPU; ajouter un epoch/session-health externe avant usage production.
- Aucun runtime blob.in/Gnosix branché; aucune publication ni déploiement.
