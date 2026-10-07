# MQ-08 receipt — interaction projection and bounded authority handoff

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-08**
Implementation commit: `1406c150152bda2a1491fe17687fab3da6aea9c7`

## Claim

`[in.gnu6]` now separates renderer observations from semantic interaction projection and from topology authority mutation.

A renderer/provider may report what it observed.
It does not gain authority to mutate scene/topology ownership.

## Projection pipeline

The interaction pipeline is:

`provider observation -> World coordinates -> exact geometric subject -> semantic pointer trajectory`

Provider provenance is retained separately from semantic output.

Implemented:
- provider identity;
- pointer identity;
- monotonic observation sequence;
- pointer phase;
- World coordinates;
- local SpatialFrame coordinates;
- exact topmost subject identity;
- deterministic pointer delta;
- accumulated trajectory distance.

## Multi-provider equivalence

Two different harnesses observed the same semantic point:

1. provider A supplied World coordinates;
2. provider B supplied coordinates in a nontrivial local SpatialFrame.

After projection:
- same World point;
- same exact subject identity;
- same semantic pointer phase/delta/travel state.

Provider provenance remained distinct.

This satisfies the MQ-08 requirement that different renderer/harness environments can produce equivalent semantic interaction truth.

## Sequence law

Pointer observation sequences are strictly monotonic per provider/pointer identity.

Out-of-order observations are rejected rather than silently reordered into history.

A new `down` phase begins a new trajectory while retaining provider sequence monotonicity.

## Geometric pick

MQ-08 consumes MQ-04 `pickTopmost`.

Pick determines exact visible spatial identity from explicit back-to-front ordering.

Projection does not mutate scene state.

## Handoff request

A projected exact subject may be used to construct a topology handoff request.

Request creation records:
- request identity;
- subject identity;
- from topology;
- to topology;
- requested degrees;
- provider/pointer provenance.

Request creation itself does not modify ownership.

A miss cannot produce a subject handoff request.

## Explicit handoff authority

`TopologyHandoffAuthority` owns an explicit route policy:

`(from, to) -> allowed degree set A`

For request degree set `D`:

`D subseteq A`

must hold.

The current owner must also be exactly `from` for every requested degree.

Only after those checks does authority call the already-ratified atomic OwnershipRegistry transfer.

## No authority amplification

A renderer cannot amplify authority by adding additional degrees to its request.

A route absent from authority policy is rejected.

Projection/request construction have no OwnershipRegistry mutation path.

## Causal mutation

Mutation:
the `D subseteq A` degree check was temporarily removed.

Expected RED:
a request including unauthorized `shape` authority should execute incorrectly and the anti-amplification test must fail.

Observed:
**1 of 5 handoff tests became RED**.

The expected rejection disappeared.

Restoration:
- original `projection.ts` restored byte-exactly;
- before/after SHA-256 identical:
  `7cce544e48d713a44b9a4094c8713a52440aa55f9557e8aa42269ee695f8f378`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- tests: **21 files / 87 tests PASS**;
- `git diff --check`: PASS.

## Known non-claims

MQ-08 does not prove:
- browser event adapter correctness;
- gesture recognition beyond pointer trajectory facts;
- keyboard/touch policy;
- renderer accessibility;
- scene introspection completeness.

## Exit-gate decision

At least two harness environments submit equivalent pointer observations and resolve the same semantic subject/interaction result, while topology ownership changes only through separately authorized handoff policy.

Disposition: **RATIFIED**.
