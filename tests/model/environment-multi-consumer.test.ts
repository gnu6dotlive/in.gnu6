import { describe, expect, it } from "vitest";
import { SpatialEnvironment, asRegionId } from "../../src/spatial/environment";

describe("MQ-04 shared environment solver", () => {
  it("places context-like, deck-like and playzone-like subjects through the same world model", () => {
    const world = new SpatialEnvironment();
    const workArea = asRegionId("work-area");

    world.addRegion({
      id: workArea,
      kind: "admissible",
      bounds: { minX: 0, minY: 0, maxX: 1280, maxY: 800 },
    });
    world.addRegion({
      id: asRegionId("reserved-edge"),
      kind: "exclusion",
      bounds: { minX: 0, minY: 720, maxX: 1280, maxY: 800 },
    });

    const contextLike = world.place({
      desired: { x: 1190, y: 690, width: 180, height: 220 },
      within: workArea,
    });
    const deckLike = world.place({
      desired: { x: 420, y: 610, width: 440, height: 140 },
      within: workArea,
    });
    const playzoneLike = world.place({
      desired: { x: 440, y: 120, width: 400, height: 260 },
      within: workArea,
    });

    expect(contextLike.status).toBe("adjusted");
    expect(deckLike.status).toBe("adjusted");
    expect(playzoneLike.status).toBe("admissible");
    expect(contextLike.rect).not.toBeNull();
    expect(deckLike.rect).not.toBeNull();
    expect(playzoneLike.rect).not.toBeNull();
  });
});
