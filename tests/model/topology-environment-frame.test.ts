import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { SceneEngine } from "../../src/scene/engine";
import { SpatialEnvironment, asRegionId } from "../../src/spatial/environment";
import { SpatialFrameGraph, asFrameId } from "../../src/spatial/frame";
import {
  contourBounds,
  inspectContourEnvironment,
  transformContourToWorld,
} from "../../src/topology/deformation";
import { OwnershipRegistry, asTopologyId } from "../../src/topology/instance";

const localContour = {
  kind: "polygon" as const,
  closed: true as const,
  vertices: [
    { x: -10, y: -5 },
    { x: 10, y: -5 },
    { x: 10, y: 5 },
    { x: -10, y: 5 },
  ],
};

describe("MQ-05 topology composition with frames and environment", () => {
  it("projects contour nodes through nested local spatial frames", () => {
    const frames = new SpatialFrameGraph();
    frames.add({
      id: asFrameId("edge"),
      parent: null,
      origin: { x: 100, y: 200 },
      basisU: { x: 0, y: 1 },
      basisV: { x: -1, y: 0 },
    });
    frames.add({
      id: asFrameId("surface"),
      parent: asFrameId("edge"),
      origin: { x: 20, y: 30 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });

    const world = transformContourToWorld(localContour, frames.snapshot(), asFrameId("surface"));

    expect(world.vertices[0]).toEqual({ x: 75, y: 210 });
    expect(contourBounds(world)).toEqual({
      minX: 65,
      minY: 210,
      maxX: 75,
      maxY: 230,
    });
  });

  it("checks contour containment and exclusion against the shared environment", () => {
    const environment = new SpatialEnvironment();
    const safe = asRegionId("safe");
    const obstacle = asRegionId("obstacle");

    environment.addRegion({
      id: safe,
      kind: "admissible",
      bounds: { minX: -100, minY: -100, maxX: 100, maxY: 100 },
    });
    environment.addRegion({
      id: obstacle,
      kind: "exclusion",
      bounds: { minX: 5, minY: -20, maxX: 30, maxY: 20 },
    });

    const observation = inspectContourEnvironment(localContour, environment, safe, [obstacle]);

    expect(observation.contained).toBe(true);
    expect(observation.collisions).toEqual([obstacle]);
    expect(observation.admissible).toBe(false);
  });

  it("allows affine pose and shape deformation owners to coexist when claims are disjoint", () => {
    const subject = asEntityId("card");
    const ownership = new OwnershipRegistry();
    ownership.claim(asTopologyId("layout"), subject, ["position.x", "position.y", "rotation"]);
    ownership.claim(asTopologyId("shape-field"), subject, ["shape"]);

    const scene = new SceneEngine();
    scene.mutate({
      add: [
        {
          id: subject,
          role: "card",
          pose: {
            position: { x: 0, y: 0 },
            rotation: 0,
            scale: { x: 1, y: 1 },
          },
          velocity: zeroVelocity(),
          shape: localContour,
          z: 0,
          visible: true,
          parent: null,
        },
      ],
    });

    expect(ownership.ownerOf(subject, "position.x")).toBe(asTopologyId("layout"));
    expect(ownership.ownerOf(subject, "shape")).toBe(asTopologyId("shape-field"));
    expect(scene.snapshot().entities.get(subject)?.shape).toEqual(localContour);
  });
});
