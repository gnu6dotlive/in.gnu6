# A02 — Providers de rendu et enveloppe visuelle

**Objectif:** matrice des projections/renderer APIs réellement vérifiées et de leurs limites.

**Sources RO:** `src/renderer/**`, `adapters/**`, `qualification/**`, `tests/browser/**`, `tests/model/renderer-*.test.ts`, preuves MQ-11→14.

**Write-set unique:** `docs/agent-results/par-01/renderer-envelope.md`.

**Invariants:** la scène dicte les faits; le renderer les projette. PixiJS 8.22.0 / WebGL = provider de référence ratifié, WebGPU = SQ-007 LATER (pas une migration décidée). Distinguer non-visuel/SVG/Pixi des possibilités hypothétiques.

**Gate:** extraire `RendererFrame`, scénarios de parité, preuve d'identité et continuité, limites des tests. Un résultat headless n'établit pas la qualité réelle sur toutes les GPU. Ne pas modifier de source ni d'outils de benchmark.

**Remise:** `provider | source | capacités | preuve | inconnu`, et tests de qualification les plus discriminants pour toute future alternative.
