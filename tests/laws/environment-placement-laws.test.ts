import { describe, expect, it } from "vitest";
import {
  SpatialEnvironment,
  asRegionId,
  containsAabb,
  intersectsAabb,
} from "../../src/spatial/environment";

describe("MQ-04 environment placement laws", () => {
  it("moves a desired rectangle to the nearest admissible position inside its region", () => {
    const world = new SpatialEnvironment();
    const safe = asRegionId("safe");

    world.addRegion({
      id: safe,
      kind: "admissible",
      bounds: { minX: 0, minY: 0, maxX: 800, maxY: 600 },
    });

    const result = world.place({
      desired: { x: 760, y: 570, width: 120, height: 80 },
      within: safe,
    });

    expect(result.status).toBe("adjusted");
    expect(result.rect).not.toBeNull();
    expect(containsAabb(world.getRegion(safe)!.bounds, result.rect!.bounds)).toBe(true);
    expect(result.rect).toEqual({
      x: 680,
      y: 520,
      width: 120,
      height: 80,
      bounds: {
        minX: 680,
        minY: 520,
        maxX: 800,
        maxY: 600,
      },
    });
  });

  it("avoids exclusion regions rather than clipping through them", () => {
    const world = new SpatialEnvironment();
    const safe = asRegionId("safe");
    const obstacle = asRegionId("obstacle");

    world.addRegion({
      id: safe,
      kind: "admissible",
      bounds: { minX: 0, minY: 0, maxX: 1000, maxY: 700 },
    });
    world.addRegion({
      id: obstacle,
      kind: "exclusion",
      bounds: { minX: 400, minY: 250, maxX: 600, maxY: 450 },
    });

    const result = world.place({
      desired: { x: 430, y: 300, width: 180, height: 120 },
      within: safe,
    });

    expect(result.status).toBe("adjusted");
    expect(result.rect).not.toBeNull();
    expect(intersectsAabb(result.rect!.bounds, world.getRegion(obstacle)!.bounds)).toBe(false);
  });

  it("returns rejected when no admissible rectangle can exist", () => {
    const world = new SpatialEnvironment();
    const safe = asRegionId("tiny");

    world.addRegion({
      id: safe,
      kind: "admissible",
      bounds: { minX: 0, minY: 0, maxX: 50, maxY: 50 },
    });

    const result = world.place({
      desired: { x: 0, y: 0, width: 100, height: 80 },
      within: safe,
    });

    expect(result.status).toBe("rejected");
    expect(result.rect).toBeNull();
    expect(result.reasons).toContain("subject-does-not-fit-within-region");
  });

  it("is deterministic for identical environment and placement input", () => {
    const world = new SpatialEnvironment();
    const safe = asRegionId("safe");

    world.addRegion({
      id: safe,
      kind: "admissible",
      bounds: { minX: 0, minY: 0, maxX: 900, maxY: 700 },
    });
    world.addRegion({
      id: asRegionId("one"),
      kind: "exclusion",
      bounds: { minX: 300, minY: 200, maxX: 500, maxY: 420 },
    });
    world.addRegion({
      id: asRegionId("two"),
      kind: "exclusion",
      bounds: { minX: 520, minY: 200, maxX: 700, maxY: 420 },
    });

    const request = {
      desired: { x: 440, y: 260, width: 160, height: 100 },
      within: safe,
    } as const;

    expect(world.place(request)).toEqual(world.place(request));
  });

  it("environment mutation changes placement through the same solver", () => {
    const world = new SpatialEnvironment();
    const safe = asRegionId("safe");

    world.addRegion({
      id: safe,
      kind: "admissible",
      bounds: { minX: 0, minY: 0, maxX: 800, maxY: 600 },
    });

    const request = {
      desired: { x: 300, y: 240, width: 180, height: 100 },
      within: safe,
    } as const;

    const before = world.place(request);
    expect(before.status).toBe("admissible");

    world.addRegion({
      id: asRegionId("new-obstacle"),
      kind: "exclusion",
      bounds: { minX: 280, minY: 220, maxX: 520, maxY: 380 },
    });

    const after = world.place(request);
    expect(after.status).toBe("adjusted");
    expect(after.rect).not.toEqual(before.rect);
  });
});
