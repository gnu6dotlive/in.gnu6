import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { layoutCardStack, layoutCardSpread, projectDeckCentersToWorld } from "../../src/deck/deck";
import { SpatialFrameGraph, asFrameId } from "../../src/spatial/frame";

describe("MQ-06 deck projection through arbitrary local frames", () => {
  it("projects the same stack topology into horizontal and vertical worlds", () => {
    const frames = new SpatialFrameGraph();
    frames.add({
      id: asFrameId("horizontal"),
      parent: null,
      origin: { x: 100, y: 500 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });
    frames.add({
      id: asFrameId("vertical"),
      parent: null,
      origin: { x: 1200, y: 100 },
      basisU: { x: 0, y: 1 },
      basisV: { x: -1, y: 0 },
    });

    const local = layoutCardStack(["a", "b", "c"].map(asEntityId), {
      origin: { x: 0, y: 0 },
      offset: { x: 20, y: 4 },
      rotationStep: 0,
      scaleStep: 0,
    });

    const horizontal = projectDeckCentersToWorld(local, frames.snapshot(), asFrameId("horizontal"));
    const vertical = projectDeckCentersToWorld(local, frames.snapshot(), asFrameId("vertical"));

    expect(horizontal[1]!.pose.position).toEqual({ x: 120, y: 504 });
    expect(vertical[1]!.pose.position).toEqual({ x: 1196, y: 120 });
  });

  it("keeps topology choice independent from frame choice", () => {
    const frames = new SpatialFrameGraph();
    frames.add({
      id: asFrameId("world"),
      parent: null,
      origin: { x: 50, y: 70 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });

    const spread = layoutCardSpread(["a", "b", "c"].map(asEntityId), {
      center: { x: 0, y: 0 },
      gap: 60,
    });
    const world = projectDeckCentersToWorld(spread, frames.snapshot(), asFrameId("world"));

    expect(world.map((pose) => pose.pose.position.x)).toEqual([-10, 50, 110]);
  });
});
