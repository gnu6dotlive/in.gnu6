import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import {
  SpatialEnvironment,
  aabbDistance,
  asRegionId,
  containsAabb,
  computeOcclusion,
  intersectsAabb,
  pickTopmost,
} from "../../src/spatial/environment";

describe("MQ-04 environment relations", () => {
  it("computes containment and intersection as separate relations", () => {
    const outer = { minX: 0, minY: 0, maxX: 100, maxY: 100 };
    const inner = { minX: 20, minY: 30, maxX: 80, maxY: 90 };
    const overlap = { minX: 70, minY: 70, maxX: 130, maxY: 130 };

    expect(containsAabb(outer, inner)).toBe(true);
    expect(containsAabb(outer, overlap)).toBe(false);
    expect(intersectsAabb(outer, overlap)).toBe(true);
  });

  it("defines proximity as zero for touching/intersecting bounds and positive otherwise", () => {
    const a = { minX: 0, minY: 0, maxX: 10, maxY: 10 };
    const touching = { minX: 10, minY: 2, maxX: 20, maxY: 8 };
    const far = { minX: 13, minY: 14, maxX: 20, maxY: 20 };

    expect(aabbDistance(a, touching)).toBe(0);
    expect(aabbDistance(a, far)).toBe(5);
  });

  it("derives occlusion from explicit back-to-front scene order", () => {
    const back = asEntityId("back");
    const middle = asEntityId("middle");
    const front = asEntityId("front");

    const pairs = computeOcclusion([
      {
        id: back,
        bounds: { minX: 0, minY: 0, maxX: 100, maxY: 100 },
        visible: true,
      },
      {
        id: middle,
        bounds: { minX: 150, minY: 0, maxX: 250, maxY: 100 },
        visible: true,
      },
      {
        id: front,
        bounds: { minX: 50, minY: 20, maxX: 120, maxY: 90 },
        visible: true,
      },
    ]);

    expect(pairs).toEqual([{ occluder: front, occluded: back }]);
  });

  it("finds the nearest matching region deterministically", () => {
    const world = new SpatialEnvironment();
    world.addRegion({
      id: asRegionId("left"),
      kind: "container",
      bounds: { minX: 0, minY: 0, maxX: 100, maxY: 100 },
    });
    world.addRegion({
      id: asRegionId("right"),
      kind: "container",
      bounds: { minX: 300, minY: 0, maxX: 400, maxY: 100 },
    });

    expect(world.nearestRegion({ x: 220, y: 50 }, ["container"])).toEqual({
      region: {
        id: asRegionId("right"),
        kind: "container",
        bounds: { minX: 300, minY: 0, maxX: 400, maxY: 100 },
      },
      distance: 80,
    });

    expect(world.regionsAt({ x: 50, y: 50 }, ["container"]).map((r) => r.id)).toEqual([
      asRegionId("left"),
    ]);
  });

  it("picks topmost visible geometric identity without granting interaction authority", () => {
    const back = asEntityId("back");
    const hiddenFront = asEntityId("hidden-front");
    const front = asEntityId("front");

    const hit = pickTopmost(
      [
        {
          id: back,
          bounds: { minX: 0, minY: 0, maxX: 100, maxY: 100 },
          visible: true,
        },
        {
          id: hiddenFront,
          bounds: { minX: 0, minY: 0, maxX: 100, maxY: 100 },
          visible: false,
        },
        {
          id: front,
          bounds: { minX: 20, minY: 20, maxX: 80, maxY: 80 },
          visible: true,
        },
      ],
      { x: 40, y: 40 },
    );

    expect(hit).toBe(front);
    expect(pickTopmost([], { x: 40, y: 40 })).toBeNull();
  });
});
