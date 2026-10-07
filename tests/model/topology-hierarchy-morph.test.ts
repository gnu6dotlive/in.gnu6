import { describe, expect, it } from "vitest";
import { assertAcyclicHierarchy, composePose, worldPose } from "../../src/scene/hierarchy";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { createShapeMorphTrack, currentShape, stepShapeMorph } from "../../src/topology/morph";
import type { SceneEntity, SceneState } from "../../src/algebra/types";

function entity(id: string, parent: string | null, x: number, y: number): SceneEntity {
  return {
    id: asEntityId(id),
    role: "surface",
    parent: parent ? asEntityId(parent) : null,
    pose: {
      position: { x, y },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    velocity: zeroVelocity(),
    shape: { kind: "rect", width: 10, height: 10, radius: 2 },
    z: 0,
    visible: true,
  };
}

function state(entities: SceneEntity[]): SceneState {
  return {
    time: 0,
    entities: new Map(entities.map((item) => [item.id, item])),
    relations: [],
    order: entities.map((item) => item.id),
    targets: new Map(),
    fields: [],
  };
}

describe("Experience scene hierarchy and topology morph", () => {
  it("composes parent affine pose into child local pose", () => {
    const parent = {
      position: { x: 100, y: 50 },
      rotation: Math.PI / 2,
      scale: { x: 2, y: 2 },
    };
    const child = {
      position: { x: 10, y: 0 },
      rotation: Math.PI / 4,
      scale: { x: 0.5, y: 1 },
    };

    const result = composePose(parent, child);
    expect(result.position.x).toBeCloseTo(100, 8);
    expect(result.position.y).toBeCloseTo(70, 8);
    expect(result.rotation).toBeCloseTo((3 * Math.PI) / 4, 8);
    expect(result.scale).toEqual({ x: 1, y: 2 });
  });

  it("resolves a unique world pose through a parent forest", () => {
    const scene = state([
      entity("root", null, 100, 0),
      entity("deck", "root", 20, 0),
      entity("card", "deck", 5, 0),
    ]);

    expect(worldPose(scene, asEntityId("card")).position).toEqual({
      x: 125,
      y: 0,
    });
  });

  it("rejects parent cycles because world pose would be undefined", () => {
    const scene = state([entity("a", "b", 0, 0), entity("b", "a", 0, 0)]);

    expect(() => assertAcyclicHierarchy(scene)).toThrow("Scene hierarchy contains cycle");
  });

  it("gives one identity a continuous window-to-card boundary path", () => {
    let track = createShapeMorphTrack(
      asEntityId("window:scratch-2"),
      { kind: "rect", width: 1120, height: 620, radius: 14 },
      { kind: "rect", width: 72, height: 82, radius: 16 },
      { stiffness: 180, damping: 24, mass: 1 },
      48,
    );

    const source = currentShape(track);
    track = stepShapeMorph(track, 1 / 60);
    const moved = currentShape(track);

    expect(track.subjectId).toBe(asEntityId("window:scratch-2"));
    expect(source.vertices).toHaveLength(48);
    expect(moved.vertices).toHaveLength(48);
    expect(track.progress).toBeGreaterThan(0);
    expect(track.progress).toBeLessThan(1);
    expect(moved.vertices).not.toEqual(source.vertices);
  });
});
