# MQ-12 receipt — PixiJS reference browser renderer

Date: 2026-10-07
Status: **RATIFIED**
Quest: **MQ-12**
Implementation commit: `0e40a06b8f30e0031cfb863e800386ad4d5aea34`

## Provider decision

The operator selected **PixiJS latest** as the first reference browser renderer provider.

At qualification time:

- npm `latest`: **pixi.js 8.22.0**
- pinned dependency: **pixi.js 8.22.0**
- Playwright: **1.63.0**
- Vite: **8.3.3**
- browser instrument: **Chrome for Testing 153.0.8010.12**
- reference backend: **WebGL**
- Pixi ticker: **stopped**

The provider selection remains subordinate to the ratified MQ-11 renderer contract.

## Renderer authority boundary

The browser adapter consumes only `RendererFrame` projection data.

PixiJS does not own:
- semantic identity;
- scene time;
- topology;
- topology ownership;
- interaction authority;
- product/business state.

Pixi events are disabled on projected `Graphics` objects with `eventMode = "none"`.

Scene evolution is driven by `SceneEngine` + `FixedStepClock`; Pixi performs manual render only.

## Stable semantic identity → stable provider handle

The adapter maintains:

`EntityId -> Pixi Graphics`

Existing identities reuse the same `Graphics` handle across:
- deck magnification;
- neighbor yield;
- topology shape morph;
- insertion/removal of other entities;
- topology handoff.

Removing a semantic identity destroys only its downstream provider handle.

## Shape continuity blocker discovered and repaired

MQ-12 exposed a required scene-level gap:

`SceneEngine` could evolve pose targets but could not replace an entity's shape while preserving identity.

A new bounded `setShape(entityId, shape)` operation was added.

It preserves:
- entity identity;
- scene order;
- pose;
- velocity;
- parentage;
- active target.

Absent identities fail closed.

This enables the headless contour dynamics from MQ-05 to be projected frame-by-frame without remove/add recreation.

## Browser qualification surface

A dedicated qualification surface lives under:

`qualification/pixi/`

It is deliberately not a product UI.

Its purpose is to falsify renderer/provider behavior.

The surface contains:
- one Pixi canvas;
- one sibling semantic DOM projection;
- concurrent deck/hand topologies;
- nested spatial frames;
- one topology-morphing surface;
- explicit normal/reduced traversal modes.

## Real browser observations

Playwright drove a real Chrome for Testing browser against the Vite-served qualification surface.

Observed and asserted:

### Deck overlap / occlusion

Cards have center spacing smaller than their rendered width and explicit z ordering.

The browser screenshot shows overlapping card hands.

### Continuous magnification / neighbor yield

A focused deck card changes scale continuously through scene targets while neighboring cards move.

The exact same Pixi `Graphics` handle remains bound to the focused `EntityId`.

### Shape topology morph

One persistent entity evolves:

`rounded rect -> polygon/diamond -> card-like topology`

The same Pixi `Graphics` handle survives the morph.

### Insertion / removal

A new card is inserted through scene identity creation and animated target state.

Its removal deletes only that identity's provider handle.

Unrelated card handles remain stable.

### Topology handoff

A hand card transfers pose authority:

`hand -> playzone`

The same semantic identity and Pixi handle survive the handoff.

The final ownership observation reports `playzone` as owner of the transferred pose degrees.

### Nested frames / concurrent topologies

The live qualification World contains:
- dock topology;
- hand topology;
- playzone topology;
- nested spatial frames.

All render through the same Pixi adapter and semantic engine.

### Reduced-motion semantic parity

Normal and reduced traversal use the same target destination.

Reduced mode settles directly; normal mode integrates through scene time.

The semantic destination is identical.

### Accessibility strategy

The Pixi canvas is paired with a sibling semantic DOM projection keyed by the same `EntityId`.

The browser observation confirms semantic buttons with explicit:
- role;
- label;
- identity.

The DOM layer does not become the visual renderer or scene authority.

## Browser gate

Restored-state Playwright result:

- **1/1 PASS**
- duration: approximately 4 seconds

Static restored-state gate:

- format: PASS
- lint: PASS
- typecheck: PASS
- Vitest: **26 files / 104 tests PASS**
- Vite production build: PASS
- npm audit: **0 vulnerabilities**
- `git diff --check`: PASS

## Idempotent qualification tooling

A tooling quick win was taken during MQ-12:

`check -> build -> browser -> check`

must remain green.

Biome was explicitly bounded away from generated artifacts:
- `.tmp`;
- `test-results`;
- `playwright-report`;
- `dist`.

The full cycle passed after the correction.

## Causal mutation

Mutation:
the Pixi adapter was temporarily changed so an existing semantic identity destroyed and recreated its `Graphics` handle on every render.

Expected RED:
handle continuity across magnification/morph must fail.

Observed:
the focused `dock-c` handle changed:

- expected: `handle-3`
- mutated: `handle-13`

Playwright became RED exactly on the continuity assertion.

Restoration:
- original `adapters/pixi.ts` restored from byte copy;
- before/after SHA-256 identical:
  `cb2ce183587520f6341263211277b6651100769a690ca610aedb25dc6b81b6ad`;
- byte comparison: PASS.

## Visual evidence

Screenshot:

`docs/evidence/assets/mq-12-pixi-reference-browser.png`

The screenshot is evidence of the qualification surface only.

It is intentionally not presented as a final Gnosix/gnu6 product design.

## Instrument limitation

Playwright reported the host OS as not officially supported and downloaded its Ubuntu 24.04 fallback Chrome build.

Therefore this gate proves browser behavior in the observed Chrome for Testing environment only.

It does not establish a production browser-support matrix.

## Future WebGPU path

The operator requested a separate Future Quest for WebGPU.

That work is recorded as **SQ-007** and does not block MQ-12.

It may later compare:
- PixiJS WebGPU backend;
- native/custom WebGPU renderer;
- shader/deformation pipelines;
- batching/render graph strategies;
- performance against the Pixi/WebGL reference baseline.

## Known non-claims

MQ-12 does not yet prove:
- every transition is free from cut-off;
- full product visual design;
- production browser compatibility matrix;
- performance/scale envelope;
- native/custom WebGPU superiority.

Those remain MQ-13, MQ-14 and SQ-007.

## Exit-gate decision

The MQ-12 exit gate is satisfied:

> a real browser observation demonstrates the required reference behaviors from scene truth without moving semantic authority into the renderer.

Disposition: **RATIFIED**.
