import { describe, expect, it } from "vitest";
import { asConstraintId, solveSpatialConstraints } from "../../src/spatial/constraints";
import { SpatialEnvironment, asRegionId } from "../../src/spatial/environment";

describe("MQ-07 deterministic constraint explanation", () => {
  it("explains adjustment with before/after and per-constraint results", () => {
    const environment = new SpatialEnvironment();
    environment.addRegion({
      id: asRegionId("safe"),
      kind: "admissible",
      bounds: { minX: 0, minY: 0, maxX: 500, maxY: 400 },
    });

    const result = solveSpatialConstraints({
      desired: { x: 480, y: 390, width: 120, height: 90 },
      constraints: [
        {
          id: asConstraintId("extent"),
          ownerId: "surface",
          kind: "extent-range",
          maxWidth: 100,
          maxHeight: 80,
        },
        {
          id: asConstraintId("safe"),
          ownerId: "environment",
          kind: "within-region",
          regionId: asRegionId("safe"),
        },
      ],
      environment,
      peers: new Map(),
      topologyFacts: new Map(),
    });

    expect(result.status).toBe("adjusted");
    expect(result.desired).toEqual({ x: 480, y: 390, width: 120, height: 90 });
    expect(result.rect).toEqual(
      expect.objectContaining({ x: 400, y: 320, width: 100, height: 80 }),
    );
    expect(result.evaluations).toEqual([
      expect.objectContaining({
        id: asConstraintId("extent"),
        status: "adjusted",
        reason: "extent-clamped",
      }),
      expect.objectContaining({
        id: asConstraintId("safe"),
        status: "adjusted",
        reason: "moved-inside-region",
      }),
    ]);
  });

  it("never returns success with an unevaluated declared constraint", () => {
    const environment = new SpatialEnvironment();

    const result = solveSpatialConstraints({
      desired: { x: 10, y: 10, width: 40, height: 30 },
      constraints: [
        {
          id: asConstraintId("position"),
          ownerId: "layout",
          kind: "position-range",
          minX: 0,
          maxX: 100,
          minY: 0,
          maxY: 100,
        },
        {
          id: asConstraintId("topology"),
          ownerId: "topology",
          kind: "topology-admissibility",
          factId: "allowed",
          expected: true,
        },
      ],
      environment,
      peers: new Map(),
      topologyFacts: new Map([["allowed", true]]),
    });

    expect(result.status).toBe("admissible");
    expect(result.evaluations).toHaveLength(2);
    expect(result.evaluations.every((item) => item.status === "satisfied")).toBe(true);
  });
});
