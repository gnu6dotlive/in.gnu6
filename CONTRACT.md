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

**2026-10-10 amendment:** operator grants repository-public visibility permission
**after** a verified full-history secrets, rights/license and personal-data
publication gate. This is not a license grant or permission to publish npm
packages, expose consumer data or declare API stability.
See `docs/public/PUBLICATION-READINESS.md`.

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

## 11. Bootstrap provenance boundary

The initial repository imported the semantic seed from:

`gnu6-live @ b07be200ac3dbec8e6dd896c84bb043390ea4260`

The first gate was:

> The same 18 seed laws/tests must pass in this repository before new capability work starts.

At repository creation, no Dock wiring, renderer selection, PixiJS dependency or public API was authorized **by the act of repository creation itself**.

Later decisions are governed by the Questlog and their own evidence. In particular:
- PixiJS 8.22.0 / WebGL was later selected and ratified as the MQ-12 reference browser provider;
- that selection remains subordinate to the replaceable renderer contract;
- public API and package publication remain undecided / unauthorized.

## 12. Questlog authority

`QUESTLOG.md` defines the current Main Questline completeness contract and the operator-prioritized Side Quest registry.

The Main Questline may expand when reality reveals missing necessary invariants. It must not be shortened merely to make completion easier.

Side Quests remain optional unless explicitly promoted because evidence shows they are causally necessary for a Main Quest exit gate.
