import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import type { PolygonShape, RadialField, RectShape } from "../../src/algebra/types";
import {
  createDeformableContourTrack,
  currentDeformableShape,
  deformContourByFields,
  retargetDeformableContour,
  stepDeformableContour,
} from "../../src/topology/deformation";

const windowShape: RectShape = {
  kind: "rect",
  width: 420,
  height: 260,
  radius: 18,
};

const cardShape: RectShape = {
  kind: "rect",
  width: 84,
  height: 128,
  radius: 14,
};

const diamondShape: PolygonShape = {
  kind: "polygon",
  closed: true,
  vertices: [
    { x: 0, y: -90 },
    { x: 70, y: 0 },
    { x: 0, y: 90 },
    { x: -70, y: 0 },
  ],
};

describe("MQ-05 deformable contour laws", () => {
  it("retargets without recreating subject identity or resetting node velocity", () => {
    let track = createDeformableContourTrack(
      asEntityId("surface"),
      windowShape,
      cardShape,
      { stiffness: 180, damping: 24, mass: 1 },
      48,
    );

    for (let i = 0; i < 8; i += 1) {
      track = stepDeformableContour(track, 1 / 60);
    }

    const before = structuredClone(track.nodes);
    expect(before.some((node) => Math.hypot(node.velocity.x, node.velocity.y) > 0)).toBe(true);

    const retargeted = retargetDeformableContour(track, diamondShape);

    expect(retargeted.subjectId).toBe(track.subjectId);
    expect(retargeted.nodes).toEqual(before);
    expect(retargeted.target.vertices).toHaveLength(48);
  });

  it("can reverse toward a third topology while preserving live node state", () => {
    let track = createDeformableContourTrack(
      asEntityId("surface"),
      windowShape,
      diamondShape,
      { stiffness: 150, damping: 20, mass: 1 },
      48,
    );

    for (let i = 0; i < 10; i += 1) {
      track = stepDeformableContour(track, 1 / 60);
    }

    track = retargetDeformableContour(track, cardShape);
    for (let i = 0; i < 10; i += 1) {
      track = stepDeformableContour(track, 1 / 60);
    }

    const beforeReverse = structuredClone(track.nodes);
    const reversed = retargetDeformableContour(track, windowShape);

    expect(reversed.nodes).toEqual(beforeReverse);
    expect(reversed.subjectId).toBe(asEntityId("surface"));
    expect(currentDeformableShape(reversed).vertices).toHaveLength(48);
  });

  it("applies continuous fields to contour nodes without mutating the source contour", () => {
    const source: PolygonShape = {
      kind: "polygon",
      closed: true,
      vertices: [
        { x: -20, y: -20 },
        { x: 20, y: -20 },
        { x: 20, y: 20 },
        { x: -20, y: 20 },
      ],
    };
    const fields: RadialField[] = [
      {
        id: "focus",
        kind: "radial",
        center: { x: 80, y: 0 },
        radius: 160,
        strength: 18,
        falloff: 2,
      },
    ];

    const deformed = deformContourByFields(source, fields);

    expect(deformed.vertices).not.toEqual(source.vertices);
    expect(source.vertices).toEqual([
      { x: -20, y: -20 },
      { x: 20, y: -20 },
      { x: 20, y: 20 },
      { x: -20, y: 20 },
    ]);
  });

  it("remains finite under repeated integration and retargeting", () => {
    let track = createDeformableContourTrack(
      asEntityId("stable"),
      windowShape,
      cardShape,
      { stiffness: 220, damping: 28, mass: 0.8 },
      32,
    );

    for (let i = 0; i < 3000; i += 1) {
      if (i === 1000) track = retargetDeformableContour(track, diamondShape);
      if (i === 2000) track = retargetDeformableContour(track, windowShape);
      track = stepDeformableContour(track, i % 97 === 0 ? 10 : 1 / 60);
    }

    for (const node of track.nodes) {
      expect(Number.isFinite(node.position.x)).toBe(true);
      expect(Number.isFinite(node.position.y)).toBe(true);
      expect(Number.isFinite(node.velocity.x)).toBe(true);
      expect(Number.isFinite(node.velocity.y)).toBe(true);
    }
  });
});
