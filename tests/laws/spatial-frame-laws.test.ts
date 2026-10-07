import { describe, expect, it } from "vitest";
import { SpatialFrameGraph, asFrameId, localToWorld, worldToLocal } from "../../src/spatial/frame";

describe("MQ-02 spatial frame laws", () => {
  it("round-trips a point through an arbitrary invertible basis", () => {
    const graph = new SpatialFrameGraph();
    const frame = graph.add({
      id: asFrameId("oblique"),
      parent: null,
      origin: { x: 100, y: 50 },
      basisU: { x: 0, y: 2 },
      basisV: { x: -3, y: 0 },
    });

    const local = { x: 7, y: -4 };
    const world = localToWorld(graph.snapshot(), frame.id, local);
    expect(worldToLocal(graph.snapshot(), frame.id, world)).toEqual(local);
  });

  it("composes nested frames instead of forcing one global deck orientation", () => {
    const graph = new SpatialFrameGraph();
    graph.add({
      id: asFrameId("world-edge"),
      parent: null,
      origin: { x: 20, y: 700 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: -1 },
    });
    graph.add({
      id: asFrameId("deck"),
      parent: asFrameId("world-edge"),
      origin: { x: 40, y: 10 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });

    expect(localToWorld(graph.snapshot(), asFrameId("deck"), { x: 5, y: 8 })).toEqual({
      x: 65,
      y: 682,
    });
  });

  it("rejects singular bases because inverse spatial truth would be undefined", () => {
    const graph = new SpatialFrameGraph();
    expect(() =>
      graph.add({
        id: asFrameId("singular"),
        parent: null,
        origin: { x: 0, y: 0 },
        basisU: { x: 1, y: 0 },
        basisV: { x: 2, y: 0 },
      }),
    ).toThrow("invertible");
  });

  it("rejects frame cycles", () => {
    const graph = new SpatialFrameGraph();
    graph.add({
      id: asFrameId("a"),
      parent: null,
      origin: { x: 0, y: 0 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });
    graph.add({
      id: asFrameId("b"),
      parent: asFrameId("a"),
      origin: { x: 0, y: 0 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });

    expect(() => graph.reparent(asFrameId("a"), asFrameId("b"))).toThrow("cycle");
  });
});
