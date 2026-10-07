# MQ-11 receipt — replaceable renderer contract

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-11**
Implementation commit: `93db69444c4c93e64679da22e2929f93b1cb778c`

## Claim

`[in.gnu6]` now exposes a renderer contract that keeps projection downstream from semantic scene truth.

The renderer contract does not own:
- scene identity;
- topology;
- scene time;
- ownership;
- product state;
- interaction authority.

## RendererFrame

A renderer receives a detached, deeply frozen frame containing:
- exact entity identity;
- world pose;
- full shape geometry;
- visibility;
- z;
- scene role;
- optional accessibility metadata;
- scene time;
- projection sequence;
- motion mode.

The frame contains no mutation-capable engine object.

## Identity partition law

Every semantic entity in a frame must be accounted for exactly once by a renderer result as either:
- emitted; or
- explicitly omitted with a reason.

A renderer may not invent a semantic identity.

## Accessibility strategy

Accessibility metadata is supplied explicitly and keyed by existing semantic entity identity.

The contract rejects accessibility metadata for identities absent from scene truth.

Renderers may project supplied role/label/hidden metadata, but do not invent product semantics.

## Cadence and reduced motion

Renderer sequence and `motionMode` are projection metadata.

`rendererSemanticDigest` excludes both.

For the same scene snapshot:
- normal and reduced projection modes preserve the same semantic digest;
- different renderer observation sequences preserve the same semantic digest.

Renderer cadence therefore does not redefine simulation truth.

## Qualification adapters

Two materially different adapters consume the same contract:

1. `CommandRenderer`
   - deterministic nonvisual draw-command oracle.

2. `SvgStringRenderer`
   - pure SVG string projection;
   - no DOM mutation;
   - accessibility metadata projection;
   - no scene authority.

Both:
- preserve exact semantic identities;
- explicitly omit invisible identities;
- produce deterministic output for the same frame.

The SVG adapter is a contract-qualification adapter only.
It is not yet the MQ-12 reference browser renderer.

## Mutation resistance

RendererFrame is deeply frozen.

A test attempting to mutate nested pose data throws.

The original scene snapshot remains unchanged.

## Causal mutation

Mutation:
the renderer-result validator was temporarily weakened so adapter-emitted identities not present in the semantic frame were accepted, while coverage of real frame identities remained enforced.

Expected RED:
the identity invention gate must fail.

Observed:
**1 renderer-contract test RED**.

The mutated validator accepted a synthetic `ghost` identity and the test correctly reported that the expected rejection did not occur.

Restoration:
- original `src/renderer/contract.ts` restored from byte copy;
- before/after SHA-256 identical:
  `6126cd889aad20250868d529e75a84f133f51ed5520bc834027ea01c21c190ba`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- full tests: **26 files / 102 tests PASS**;
- `git diff --check`: PASS.

## Known non-claims

MQ-11 does not prove:
- live browser rendering;
- browser mount/update continuity;
- PixiJS suitability;
- DOM/SVG performance envelope;
- no-cutoff behavior in a browser;
- reference renderer accessibility behavior in a live DOM.

Those remain MQ-12 / MQ-13 / MQ-14.

## Exit-gate decision

The MQ-11 exit gate is satisfied:

> the renderer contract supports materially different adapters without changing core ontology or moving semantic authority into the renderer.

Disposition: **RATIFIED**.
