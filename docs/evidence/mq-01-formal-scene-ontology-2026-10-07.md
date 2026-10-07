# MQ-01 receipt — formal scene ontology

Date: 2026-10-07
Status: **RATIFIED**
Quest: **MQ-01**
Implementation commit: `cefa6fb651585f98cac0ce7fadac238f95934093`

## Claim

`[in.gnu6]` now has a deterministic formal scene ontology sufficient to generate, mutate, query, observe and reject finite semantic scene graphs without consumer or renderer concepts.

## Core ontology

Scene state remains a finite attributed graph:

`G = (E, R)`

where:
- `E` is the finite semantic entity set;
- `R` is a finite set of typed directed relations;
- scene traversal order is an explicit permutation of `E`;
- parentage is a partial acyclic function over `E`.

The scene core contains:
- stable renderer-independent EntityId;
- role;
- pose;
- velocity;
- shape;
- z;
- visibility;
- parent;
- target;
- relation set;
- traversal order;
- scene time;
- fields.

## Query algebra

MQ-01 adds an explicit serializable query AST over the finite entity set.

Supported entity query terms include:
- all;
- exact identity set;
- role;
- visibility;
- parent;
- targeted / untargeted;
- relation incidence;
- set intersection;
- set union;
- set complement.

Semantics:

`[[all]] = E`

`[[ids(S)]] = E ∩ S`

`[[and(q1...qn)]] = intersection([[qi]])`

`[[or(q1...qn)]] = union([[qi]])`

`[[not(q)]] = E \ [[q]]`

Query changes membership, not traversal semantics.

Entity query results preserve the explicit scene traversal permutation rather than Map insertion order or lexical sort.

Relations can also be queried deterministically and are returned in canonical lexical order over:

`(kind, from, to)`

## Relation vocabulary

Repeated evidence currently justifies one core relation family:

`attached`

Extensible meanings use an explicit namespaced form:

`ext:<name>`

Examples:
- `ext:depends-on`;
- `ext:follows`.

Arbitrary empty/opaque relation kinds are rejected.

Derived spatial facts such as occlusion remain derived facts rather than being forced into stored scene relations.

## Relation-set law

Relations are a mathematical set:

`R ⊆ K × E × E`

The same `(kind, from, to)` triple may occur at most once.

Duplicate relations are inadmissible scene state.

## Hierarchy admissibility

Parentage is validated at scene admission.

For every entity `e`, repeated parent traversal must terminate at null:

`forall e in E, exists n >= 0 : parent^n(e) = null`

A parent cycle is rejected before world-pose query or renderer projection.

## Lifecycle semantics

Lifecycle is explicit set arithmetic over semantic identity.

For before/after entity sets `E0` and `E1`:

`born = E1 \ E0`

`persisted = E0 ∩ E1`

`terminated = E0 \ E1`

Authority transfer, topology transfer, shape mutation and pose evolution do not imply semantic birth or termination when EntityId persists.

Ordering:
- born/persisted follow destination scene order;
- terminated follows source scene order.

Failed lifecycle mutations remain transactional.

## Ownership

The MQ-01 completeness requirement for independent degree-of-freedom ownership is satisfied by ratified MQ-02.

The ownership model is a partial function:

`owner : E × Q -> T`

with:

`|Owner(e,q)| <= 1`

Disjoint ownership is legal; silent double ownership is rejected.

## Exact observation frame

The MQ-01 completeness requirement for exact observation is satisfied by ratified MQ-09.

`SceneReceipt` provides deterministic observation of:
- identity;
- local/world pose;
- velocity;
- parentage;
- topology membership;
- ownership;
- target;
- constraints;
- containment;
- occlusion;
- fields;
- motion state.

Observation does not gain mutation authority.

## Invalid-state policy

There is no `ALLOW_PARTIAL` scene state.

Current admission laws include:
- entity identity/map-key agreement;
- order is a permutation of entity set;
- parent belongs to entity set;
- parent hierarchy is acyclic;
- relation endpoints exist;
- relation kind is explicit;
- relations are a mathematical set;
- targets reference existing entities.

Scene mutations are transactional: a law failure prevents the candidate state from becoming current scene truth.

## Causal mutation

Mutation:
the scene-law checker was temporarily weakened so parent cycles were no longer reported as `HIERARCHY_IS_ACYCLIC`.

Expected RED:
scene admission must stop rejecting cyclic parentage.

Observed:
**1 scene-admissibility test RED**.

The mutated SceneEngine admitted a two-node parent cycle that the test expected to reject.

Restoration:
- original `src/algebra/laws.ts` restored from byte copy;
- before/after SHA-256 identical:
  `4586e98136a2ccea9b04b3ace252032ed974c6f6dca245bab99e7fe6f623284c`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- full Vitest: **29 files / 119 tests PASS**;
- Pixi qualification build: PASS;
- Playwright browser regression: **2/2 PASS**;
- `git diff --check`: PASS.

## Consumer/provider neutrality

The new ontology/query/lifecycle modules contain no Docklab, SP Joaillerie, PixiJS, DOM, React or browser concepts.

Renderer identity remains separate from semantic EntityId under MQ-11/MQ-12.

## Exit-gate decision

The MQ-01 exit gate is satisfied:

> a deterministic law suite can generate, mutate, query and reject scene graphs without consumer concepts or renderer dependencies.

Disposition: **RATIFIED**.
