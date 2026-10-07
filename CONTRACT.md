# [in.gnu6] — repository contract

Status: **ACTIVE — INITIAL PROJECT AUTHORITY**

This contract specializes the general agent contract in `contracts/TRONC_COMMUN.md`.

## 1. One engine

There is one semantic engine: `[in.gnu6]`.

Docklab, gnu6.live, SP Joaillerie and future products are consumers.
They may own product state and product intent. They do not own parallel scene engines.

## 2. Scene, not product

`[in.gnu6]` owns:
- scene identity;
- scene relations;
- hierarchy;
- spatial constraints;
- scene time;
- dynamics;
- topology;
- deformation;
- scene queries;
- deterministic observations.

It does not know what a Workspace, Gnosix Dock, jewelry product, checkout, application or business action means.

## 3. Renderer, not ontology

DOM, SVG, Canvas, PixiJS, WebGL, WebGPU or later renderers are replaceable providers.

A renderer can observe and project scene truth.
A renderer cannot redefine identity, topology, physics ownership or product semantics.

## 4. Environment / space awareness

The core should be able to reason over a world W containing, as needed:

- entities E;
- relations R ⊆ E × E;
- obstacles O ⊆ W;
- fields F;
- safe regions S ⊆ W;
- occlusion relation Ω ⊆ E × E;
- containment relation C ⊆ E × E;
- proximity metric d : E × E → R+.

No surface-specific collection of `if near edge` rules should become the engine ontology.

## 5. No silent authority amplification

A consumer submitting a target does not gain authority to mutate unrelated scene identities.
A renderer receiving a scene frame does not gain product authority.
A donor library does not become the conceptual model merely because it is useful.

## 6. Donors

SP Joaillerie, blob.in, Docklab, Balatro-inspired mechanics, PixiJS and other projects may donate:
- algorithms;
- laws;
- fixtures;
- rendering techniques;
- interaction ideas.

Every donor remains provenance, not authority.

## 7. Truth classes

Use explicit states such as:
- HYPOTHESIS
- DESIGNED
- IMPLEMENTED
- OBSERVED
- FALSIFIED
- INVARIANT_CANDIDATE
- RATIFIED

No state implies the next one.

## 8. Public surface

Public API: **UNDECIDED**.

Until an operator decision changes this:
- package remains `private: true`;
- no npm/public registry publication;
- no CDN/public distribution contract;
- no compatibility promise to external consumers;
- internal consumers are allowed only through explicit repository/workspace integration.

## 9. Language

TypeScript is the first reference implementation because it allows rapid deterministic falsification across web consumers.

TypeScript types are not the ontology.
The conceptual model must remain portable to another implementation language if needed.

## 10. Challenge rule

An invariant may be challenged deliberately.

The challenge must name:
- the invariant;
- the hypothesis;
- the expected observation;
- the falsification instrument;
- the disposition after evidence.

An accidental regression is not a challenge.

## 11. Current bootstrap boundary

The initial repository imports the semantic seed from:

`gnu6-live @ b07be200ac3dbec8e6dd896c84bb043390ea4260`

The first gate is simple:

> The same 18 seed laws/tests must pass in this repository before new capability work starts.

No Dock wiring, renderer selection, PixiJS dependency or public API is authorized by repository creation.
