import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import {
  InteractionProjector,
  type RendererPointerObservation,
} from "../../src/interaction/projection";
import { SpatialFrameGraph, asFrameId } from "../../src/spatial/frame";

const back = asEntityId("back");
const front = asEntityId("front");

const occupants = [
  {
    id: back,
    bounds: { minX: 0, minY: 0, maxX: 120, maxY: 120 },
    visible: true,
  },
  {
    id: front,
    bounds: { minX: 30, minY: 30, maxX: 90, maxY: 90 },
    visible: true,
  },
];

describe("MQ-08 interaction projection laws", () => {
  it("projects equivalent world and local-frame observations to the same semantic hit", () => {
    const frames = new SpatialFrameGraph();
    frames.add({
      id: asFrameId("local"),
      parent: null,
      origin: { x: 20, y: 30 },
      basisU: { x: 2, y: 0 },
      basisV: { x: 0, y: 3 },
    });

    const worldProjector = new InteractionProjector(frames.snapshot());
    const localProjector = new InteractionProjector(frames.snapshot());

    const world = worldProjector.project(
      {
        providerId: "world-harness",
        pointerId: "primary",
        sequence: 1,
        phase: "down",
        coordinates: {
          space: "world",
          point: { x: 60, y: 60 },
        },
      },
      occupants,
    );

    const local = localProjector.project(
      {
        providerId: "local-harness",
        pointerId: "primary",
        sequence: 1,
        phase: "down",
        coordinates: {
          space: "frame",
          frameId: asFrameId("local"),
          point: { x: 20, y: 10 },
        },
      },
      occupants,
    );

    expect(world.semantic).toEqual(local.semantic);
    expect(world.semantic.subjectId).toBe(front);
    expect(world.provenance.providerId).toBe("world-harness");
    expect(local.provenance.providerId).toBe("local-harness");
  });

  it("keeps topmost exact identity separate from provider provenance", () => {
    const projector = new InteractionProjector(new Map());

    const result = projector.project(
      {
        providerId: "renderer-a",
        pointerId: "p",
        sequence: 1,
        phase: "down",
        coordinates: { space: "world", point: { x: 40, y: 40 } },
      },
      occupants,
    );

    expect(result.semantic.subjectId).toBe(front);
    expect(result.provenance).toEqual({
      providerId: "renderer-a",
      coordinateSpace: "world",
    });
  });

  it("records pointer trajectory deterministically from ordered observations", () => {
    const projector = new InteractionProjector(new Map());

    const first = projector.project(
      {
        providerId: "renderer",
        pointerId: "p",
        sequence: 1,
        phase: "down",
        coordinates: { space: "world", point: { x: 10, y: 20 } },
      },
      occupants,
    );
    const second = projector.project(
      {
        providerId: "renderer",
        pointerId: "p",
        sequence: 2,
        phase: "move",
        coordinates: { space: "world", point: { x: 13, y: 24 } },
      },
      occupants,
    );

    expect(first.semantic.delta).toEqual({ x: 0, y: 0 });
    expect(first.semantic.travelDistance).toBe(0);
    expect(second.semantic.delta).toEqual({ x: 3, y: 4 });
    expect(second.semantic.travelDistance).toBe(5);
  });

  it("rejects non-monotonic provider sequences instead of reordering history", () => {
    const projector = new InteractionProjector(new Map());
    const observation: RendererPointerObservation = {
      providerId: "renderer",
      pointerId: "p",
      sequence: 4,
      phase: "down",
      coordinates: { space: "world", point: { x: 10, y: 20 } },
    };

    projector.project(observation, occupants);

    expect(() =>
      projector.project(
        {
          ...observation,
          sequence: 3,
          phase: "move",
        },
        occupants,
      ),
    ).toThrow("sequence");
  });
});
