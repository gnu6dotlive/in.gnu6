import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import {
  buildRendererFrame,
  rendererSemanticDigest,
  validateRendererResult,
} from "../../src/renderer/contract";
import { SceneEngine } from "../../src/scene/engine";

function scene() {
  const root = asEntityId("root");
  const card = asEntityId("card");
  const engine = new SceneEngine();

  engine.mutate({
    add: [
      {
        id: root,
        role: "surface",
        pose: {
          position: { x: 100, y: 50 },
          rotation: 0,
          scale: { x: 1, y: 1 },
        },
        velocity: zeroVelocity(),
        shape: { kind: "rect", width: 400, height: 300, radius: 12 },
        z: 0,
        visible: true,
        parent: null,
      },
      {
        id: card,
        role: "card",
        pose: {
          position: { x: 20, y: 30 },
          rotation: 0.2,
          scale: { x: 1.1, y: 1.1 },
        },
        velocity: zeroVelocity(),
        shape: {
          kind: "polygon",
          closed: true,
          vertices: [
            { x: -10, y: -20 },
            { x: 10, y: -20 },
            { x: 10, y: 20 },
            { x: -10, y: 20 },
          ],
        },
        z: 1,
        visible: true,
        parent: root,
      },
    ],
  });

  return { engine, root, card };
}

describe("MQ-11 renderer contract laws", () => {
  it("builds a detached deeply frozen render frame from scene truth", () => {
    const f = scene();
    const frame = buildRendererFrame(f.engine.snapshot(), {
      sequence: 7,
      motionMode: "normal",
      accessibility: [
        {
          entityId: f.card,
          role: "button",
          label: "Open card",
        },
      ],
    });

    expect(frame.sequence).toBe(7);
    expect(frame.sceneTime).toBe(0);
    expect(frame.entities.map((entity) => entity.entityId)).toEqual([f.root, f.card]);
    expect(frame.entities[1]!.worldPose.position).toEqual({
      x: 120,
      y: 80,
    });
    expect(frame.entities[1]!.accessibility).toEqual({
      role: "button",
      label: "Open card",
      hidden: false,
    });

    expect(Object.isFrozen(frame)).toBe(true);
    expect(Object.isFrozen(frame.entities)).toBe(true);
    expect(Object.isFrozen(frame.entities[1]!.worldPose.position)).toBe(true);

    expect(() => {
      (frame.entities[1]!.worldPose.position as { x: number }).x = 999;
    }).toThrow();

    expect(f.engine.snapshot().entities.get(f.card)?.pose.position.x).toBe(20);
  });

  it("rejects accessibility metadata for identities absent from scene truth", () => {
    const f = scene();

    expect(() =>
      buildRendererFrame(f.engine.snapshot(), {
        sequence: 1,
        motionMode: "normal",
        accessibility: [
          {
            entityId: asEntityId("ghost"),
            role: "button",
            label: "Ghost",
          },
        ],
      }),
    ).toThrow("accessibility");
  });

  it("treats cadence sequence and motion mode as projection metadata, not semantic state", () => {
    const f = scene();

    const normal = buildRendererFrame(f.engine.snapshot(), {
      sequence: 1,
      motionMode: "normal",
      accessibility: [],
    });
    const reduced = buildRendererFrame(f.engine.snapshot(), {
      sequence: 999,
      motionMode: "reduced",
      accessibility: [],
    });

    expect(normal.sceneTime).toBe(reduced.sceneTime);
    expect(rendererSemanticDigest(normal)).toBe(rendererSemanticDigest(reduced));
  });

  it("rejects renderer results that invent, duplicate or silently drop identities", () => {
    const f = scene();
    const frame = buildRendererFrame(f.engine.snapshot(), {
      sequence: 3,
      motionMode: "normal",
      accessibility: [],
    });

    expect(() =>
      validateRendererResult(frame, {
        adapterId: "bad",
        sequence: 3,
        emittedEntityIds: [f.root, f.card, asEntityId("ghost")],
        omitted: [],
        output: null,
      }),
    ).toThrow("invented");

    expect(() =>
      validateRendererResult(frame, {
        adapterId: "bad",
        sequence: 3,
        emittedEntityIds: [f.root],
        omitted: [],
        output: null,
      }),
    ).toThrow("partition");
  });
});
