import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import {
  asConstraintId,
  solveSpatialConstraints,
  type SpatialConstraint,
} from "../../src/spatial/constraints";
import { SpatialEnvironment, asRegionId } from "../../src/spatial/environment";

function environment() {
  const world = new SpatialEnvironment();
  world.addRegion({
    id: asRegionId("safe"),
    kind: "admissible",
    bounds: { minX: 0, minY: 0, maxX: 800, maxY: 600 },
  });
  world.addRegion({
    id: asRegionId("reserved"),
    kind: "exclusion",
    bounds: { minX: 300, minY: 220, maxX: 500, maxY: 380 },
  });
  return world;
}

describe("MQ-07 explicit constraint solver laws", () => {
  it("evaluates every declared constraint exactly once and reports ownership", () => {
    const constraints: SpatialConstraint[] = [
      {
        id: asConstraintId("position"),
        ownerId: "layout",
        kind: "position-range",
        minX: 20,
        maxX: 700,
        minY: 10,
        maxY: 500,
      },
      {
        id: asConstraintId("extent"),
        ownerId: "surface",
        kind: "extent-range",
        minWidth: 60,
        maxWidth: 240,
        minHeight: 40,
        maxHeight: 180,
      },
      {
        id: asConstraintId("within"),
        ownerId: "environment",
        kind: "within-region",
        regionId: asRegionId("safe"),
      },
      {
        id: asConstraintId("avoid"),
        ownerId: "environment",
        kind: "avoid-region",
        regionId: asRegionId("reserved"),
      },
    ];

    const result = solveSpatialConstraints({
      desired: { x: 330, y: 250, width: 320, height: 220 },
      constraints,
      environment: environment(),
      peers: new Map(),
      topologyFacts: new Map(),
    });

    expect(result.status).toBe("adjusted");
    expect(result.evaluations.map((item) => item.id)).toEqual(
      constraints.map((constraint) => constraint.id),
    );
    expect(result.evaluations.map((item) => item.ownerId)).toEqual([
      "layout",
      "surface",
      "environment",
      "environment",
    ]);
    expect(result.evaluations).toHaveLength(constraints.length);
    expect(result.rect).not.toBeNull();
  });

  it("rejects deterministic incompatible position constraints instead of dropping one", () => {
    const result = solveSpatialConstraints({
      desired: { x: 0, y: 0, width: 100, height: 80 },
      constraints: [
        {
          id: asConstraintId("x-min"),
          ownerId: "one",
          kind: "position-range",
          minX: 500,
          maxX: 700,
        },
        {
          id: asConstraintId("x-max"),
          ownerId: "two",
          kind: "position-range",
          minX: 0,
          maxX: 300,
        },
      ],
      environment: environment(),
      peers: new Map(),
      topologyFacts: new Map(),
    });

    expect(result.status).toBe("rejected");
    expect(result.conflicts).toEqual([
      {
        constraintIds: [asConstraintId("x-min"), asConstraintId("x-max")],
        reason: "incompatible-position-x-range",
      },
    ]);
    expect(result.evaluations).toHaveLength(2);
    expect(result.evaluations.every((item) => item.status === "conflict")).toBe(true);
  });

  it("enforces peer overlap policy with explicit peer identity", () => {
    const peer = asEntityId("peer");
    const result = solveSpatialConstraints({
      desired: { x: 40, y: 20, width: 100, height: 100 },
      constraints: [
        {
          id: asConstraintId("peer-overlap"),
          ownerId: "deck",
          kind: "overlap-policy",
          peerId: peer,
          maxSubjectOverlapRatio: 0.2,
        },
      ],
      environment: environment(),
      peers: new Map([
        [
          peer,
          {
            x: 80,
            y: 20,
            width: 100,
            height: 100,
          },
        ],
      ]),
      topologyFacts: new Map(),
    });

    expect(result.status).toBe("rejected");
    expect(result.evaluations).toEqual([
      expect.objectContaining({
        id: asConstraintId("peer-overlap"),
        status: "violated",
        reason: "peer-overlap-ratio-exceeded",
      }),
    ]);
  });

  it("makes topology-specific admissibility explicit instead of hiding it in consumer code", () => {
    const result = solveSpatialConstraints({
      desired: { x: 40, y: 20, width: 100, height: 100 },
      constraints: [
        {
          id: asConstraintId("topology-policy"),
          ownerId: "topology",
          kind: "topology-admissibility",
          factId: "deck-capacity-available",
          expected: true,
        },
      ],
      environment: environment(),
      peers: new Map(),
      topologyFacts: new Map([["deck-capacity-available", false]]),
    });

    expect(result.status).toBe("rejected");
    expect(result.evaluations[0]).toEqual(
      expect.objectContaining({
        status: "violated",
        reason: "topology-admissibility-mismatch",
      }),
    );
  });

  it("is stable under small perturbations that do not cross a constraint boundary", () => {
    const constraints: SpatialConstraint[] = [
      {
        id: asConstraintId("safe"),
        ownerId: "environment",
        kind: "within-region",
        regionId: asRegionId("safe"),
      },
      {
        id: asConstraintId("reserved"),
        ownerId: "environment",
        kind: "avoid-region",
        regionId: asRegionId("reserved"),
      },
    ];

    const a = solveSpatialConstraints({
      desired: { x: 40, y: 40, width: 100, height: 80 },
      constraints,
      environment: environment(),
      peers: new Map(),
      topologyFacts: new Map(),
    });
    const b = solveSpatialConstraints({
      desired: { x: 40.0001, y: 40.0001, width: 100, height: 80 },
      constraints,
      environment: environment(),
      peers: new Map(),
      topologyFacts: new Map(),
    });

    expect(a.status).toBe("admissible");
    expect(b.status).toBe("admissible");
    expect(a.evaluations.map((item) => item.status)).toEqual(
      b.evaluations.map((item) => item.status),
    );
  });

  it("rejects duplicate constraint identities before solving", () => {
    const duplicate = asConstraintId("duplicate");

    expect(() =>
      solveSpatialConstraints({
        desired: { x: 0, y: 0, width: 100, height: 80 },
        constraints: [
          {
            id: duplicate,
            ownerId: "one",
            kind: "extent-range",
            minWidth: 20,
          },
          {
            id: duplicate,
            ownerId: "two",
            kind: "extent-range",
            maxWidth: 200,
          },
        ],
        environment: environment(),
        peers: new Map(),
        topologyFacts: new Map(),
      }),
    ).toThrow("constraint identities");
  });
});
