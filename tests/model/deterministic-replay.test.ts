import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { SceneEngine } from "../../src/scene/engine";
import { replayScene, type ReplayEvent } from "../fixtures/scene-replay";

function createReplayScene() {
  const id = asEntityId("replay-card");
  const engine = new SceneEngine();

  engine.mutate({
    add: [
      {
        id,
        role: "card",
        pose: {
          position: { x: 0, y: 0 },
          rotation: 0,
          scale: { x: 1, y: 1 },
        },
        velocity: zeroVelocity(),
        shape: { kind: "rect", width: 72, height: 112, radius: 12 },
        z: 0,
        visible: true,
        parent: null,
      },
    ],
  });

  return { engine, id };
}

describe("MQ-03 deterministic replay fixture", () => {
  it("reconstructs the same final state from the same ordered semantic timeline", () => {
    const first = createReplayScene();
    const second = createReplayScene();

    const events: ReplayEvent[] = [
      {
        tick: 0,
        kind: "set-target",
        target: {
          entityId: first.id,
          pose: {
            position: { x: 160, y: 40 },
            rotation: 0.35,
            scale: { x: 1.1, y: 0.95 },
          },
          spring: { stiffness: 150, damping: 22, mass: 1 },
        },
      },
      {
        tick: 24,
        kind: "set-target",
        target: {
          entityId: first.id,
          pose: {
            position: { x: -90, y: 70 },
            rotation: -0.2,
            scale: { x: 0.92, y: 1.08 },
          },
          spring: { stiffness: 190, damping: 26, mass: 1 },
        },
      },
    ];

    const secondEvents = events.map((event) =>
      event.kind === "set-target"
        ? {
            ...event,
            target: {
              ...event.target,
              entityId: second.id,
            },
          }
        : event,
    ) as ReplayEvent[];

    const a = replayScene(first.engine, events, 120, 60);
    const b = replayScene(second.engine, secondEvents, 120, 60);

    expect(a.state.time).toBe(b.state.time);
    expect(a.state.entities.get(first.id)).toEqual(b.state.entities.get(second.id));
    expect(a.clock).toEqual(b.clock);
  });

  it("makes semantic event order observable rather than averaging contradictions", () => {
    const a = createReplayScene();
    const b = createReplayScene();

    const targetA = {
      entityId: a.id,
      pose: {
        position: { x: 100, y: 0 },
        rotation: 0,
        scale: { x: 1, y: 1 },
      },
      spring: { stiffness: 180, damping: 24, mass: 1 },
    };
    const targetB = {
      entityId: a.id,
      pose: {
        position: { x: -100, y: 0 },
        rotation: 0,
        scale: { x: 1, y: 1 },
      },
      spring: { stiffness: 180, damping: 24, mass: 1 },
    };

    const forward = replayScene(
      a.engine,
      [
        { tick: 0, kind: "set-target", target: targetA },
        { tick: 20, kind: "set-target", target: targetB },
      ],
      80,
    );
    const reversed = replayScene(
      b.engine,
      [
        {
          tick: 0,
          kind: "set-target",
          target: { ...targetB, entityId: b.id },
        },
        {
          tick: 20,
          kind: "set-target",
          target: { ...targetA, entityId: b.id },
        },
      ],
      80,
    );

    expect(forward.state.entities.get(a.id)?.pose.position.x).not.toBe(
      reversed.state.entities.get(b.id)?.pose.position.x,
    );
  });
});
