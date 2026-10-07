# MQ-09 receipt — introspection and explainable scene frames

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-09**
Implementation commit: `c38c661eafa4944eda09a7644ff4b3cd90049293`

## Claim

`[in.gnu6]` can emit deterministic, renderer-neutral scene receipts that explain scene state and causal divergence without granting observation any mutation authority.

## SceneReceipt

The receipt captures, per entity:
- exact identity;
- role / visibility / z;
- parentage;
- local pose;
- composed world pose;
- velocity;
- shape summary;
- topology membership;
- topology frame / solver identity;
- ownership by degree of freedom;
- active target;
- constraint result;
- containment;
- exclusion intersection;
- occlusion / occluded-by;
- settled state;
- motion state.

Global receipt facts include:
- scene time;
- explicit scene order;
- relations;
- active fields;
- topology observations;
- environment observation.

## One-way epistemic boundary

The introspector consumes:
- immutable scene state;
- topology observations;
- detached ownership observations;
- constraint results;
- environment observation;
- spatial occupants.

It does not receive mutation-capable engine or registry objects.

OwnershipRegistry now exposes a detached deterministic observation surface rather than requiring consumers to inspect internal state.

The settled predicate was factored into a pure function so introspection does not require SceneEngine authority.

## Determinism

Equivalent semantic observations produce identical serialized receipts even when non-semantic input arrays arrive in different insertion orders.

Semantic order that matters remains preserved, such as scene traversal order and back-to-front occupant order.

## Causal diff

`diffSceneReceipts` produces addressable differences.

Example:

`entities[0].target.pose.position.x: 100 -> 140`

This allows a failing consumer test to identify the exact diverging fact rather than only reporting an opaque visual mismatch.

## Detached observation proof

A test mutates the returned receipt's copied local pose and field center.

Observed:
- original scene entity pose unchanged;
- original scene field unchanged.

The receipt therefore does not alias mutable scene truth.

## Causal mutation

Mutation:
the introspector was temporarily changed so `worldPose` was copied from the entity's local pose rather than composed through scene parentage.

Expected RED:
parent/child introspection must expose the incorrect world transform.

Observed:
**1 introspection test RED**.

Expected world child position:
`{ x: 125, y: 65 }`

Mutated observation:
`{ x: 25, y: 15 }`

Restoration:
- original `src/introspection/frame.ts` restored from byte copy;
- before/after SHA-256 identical:
  `c458f1eadb334314e2dfd1a85e2aed3980d55414e92a821316825fd62c2a39b8`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- full tests: **23 files / 92 tests PASS**;
- `git diff --check`: PASS.

## Known non-claims

MQ-09 does not prove:
- renderer correctness;
- observer overhead envelope;
- browser no-cutoff behavior;
- remote telemetry;
- public debug API stability.

Observer overhead remains MQ-14.

## Exit-gate decision

The MQ-09 exit gate is satisfied:

> a failing consumer test can produce a compact deterministic scene receipt sufficient to explain the causal difference.

Disposition: **RATIFIED**.
