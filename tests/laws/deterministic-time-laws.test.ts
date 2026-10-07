import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { FixedStepClock } from "../../src/dynamics/clock";
import { SceneEngine } from "../../src/scene/engine";

function createMovingScene(): { engine: SceneEngine; id: ReturnType<typeof asEntityId> } {
  const id = asEntityId("subject");
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

  engine.setTarget({
    entityId: id,
    pose: {
      position: { x: 240, y: -60 },
      rotation: 0.5,
      scale: { x: 1.2, y: 0.9 },
    },
    spring: { stiffness: 180, damping: 24, mass: 1 },
  });

  return { engine, id };
}

describe("MQ-03 deterministic scene time", () => {
  it("produces the same scene state under different renderer cadences", () => {
    const a = createMovingScene();
    const b = createMovingScene();
    const clockA = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });
    const clockB = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });

    for (let frame = 0; frame < 120; frame += 1) {
      clockA.advance(a.engine, 1 / 60);
    }

    for (let frame = 0; frame < 60; frame += 1) {
      clockB.advance(b.engine, 1 / 30);
    }

    expect(a.engine.snapshot().time).toBe(b.engine.snapshot().time);
    expect(a.engine.snapshot().entities.get(a.id)).toEqual(b.engine.snapshot().entities.get(b.id));
    expect(clockA.observation()).toEqual(clockB.observation());
  });

  it("bounds work after a long wall-clock pause and records discarded time", () => {
    const { engine } = createMovingScene();
    const clock = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 6 });

    const result = clock.advance(engine, 10);

    expect(result.executedSteps).toBe(6);
    expect(result.droppedSteps).toBeGreaterThan(500);
    expect(result.droppedSeconds).toBeGreaterThan(9);
    expect(engine.snapshot().time).toBeCloseTo(6 / 60, 12);
  });

  it("preserves fractional fixed-step phase between renderer frames", () => {
    const { engine } = createMovingScene();
    const clock = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });

    const first = clock.advance(engine, 1 / 120);
    const second = clock.advance(engine, 1 / 120);

    expect(first.executedSteps).toBe(0);
    expect(second.executedSteps).toBe(1);
    expect(engine.snapshot().time).toBeCloseTo(1 / 60, 12);
  });

  it("rejects invalid elapsed-time inputs instead of corrupting scene time", () => {
    const { engine } = createMovingScene();
    const clock = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });

    expect(() => clock.advance(engine, -1)).toThrow("elapsed");
    expect(() => clock.advance(engine, Number.NaN)).toThrow("elapsed");
    expect(engine.snapshot().time).toBe(0);
  });
});
