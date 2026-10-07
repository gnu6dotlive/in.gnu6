import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { FixedStepClock } from "../../src/dynamics/clock";
import { SceneEngine } from "../../src/scene/engine";

function createScene() {
  const id = asEntityId("card");
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

describe("MQ-03 retarget, reversal and settle semantics", () => {
  it("retargets from current state without resetting pose or velocity", () => {
    const { engine, id } = createScene();
    const clock = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });

    engine.setTarget({
      entityId: id,
      pose: {
        position: { x: 200, y: 0 },
        rotation: 0.4,
        scale: { x: 1.2, y: 1.2 },
      },
      spring: { stiffness: 160, damping: 20, mass: 1 },
    });

    for (let i = 0; i < 8; i += 1) clock.advance(engine, 1 / 60);

    const beforeRetarget = structuredClone(engine.snapshot().entities.get(id)!);

    engine.setTarget({
      entityId: id,
      pose: {
        position: { x: -180, y: 40 },
        rotation: -0.3,
        scale: { x: 0.9, y: 1.1 },
      },
      spring: { stiffness: 160, damping: 20, mass: 1 },
    });

    const afterRetarget = engine.snapshot().entities.get(id)!;

    expect(afterRetarget.pose).toEqual(beforeRetarget.pose);
    expect(afterRetarget.velocity).toEqual(beforeRetarget.velocity);
    expect(Math.abs(afterRetarget.velocity.linear.x)).toBeGreaterThan(0);
  });

  it("reversal preserves current velocity before the new spring acceleration acts", () => {
    const { engine, id } = createScene();
    const clock = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });

    engine.setTarget({
      entityId: id,
      pose: {
        position: { x: 300, y: 0 },
        rotation: 0,
        scale: { x: 1, y: 1 },
      },
      spring: { stiffness: 120, damping: 14, mass: 1 },
    });

    for (let i = 0; i < 5; i += 1) clock.advance(engine, 1 / 60);

    const velocityBefore = engine.snapshot().entities.get(id)!.velocity.linear.x;
    expect(velocityBefore).toBeGreaterThan(0);

    engine.setTarget({
      entityId: id,
      pose: {
        position: { x: -300, y: 0 },
        rotation: 0,
        scale: { x: 1, y: 1 },
      },
      spring: { stiffness: 120, damping: 14, mass: 1 },
    });

    expect(engine.snapshot().entities.get(id)!.velocity.linear.x).toBe(velocityBefore);
  });

  it("has explicit settled criteria rather than inferring sleep from elapsed time", () => {
    const { engine, id } = createScene();
    const clock = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });

    engine.setTarget({
      entityId: id,
      pose: {
        position: { x: 80, y: 20 },
        rotation: 0.2,
        scale: { x: 1.1, y: 0.95 },
      },
      spring: { stiffness: 180, damping: 28, mass: 1 },
    });

    expect(engine.isTargetSettled(id)).toBe(false);

    for (let i = 0; i < 600 && !engine.isTargetSettled(id); i += 1) {
      clock.advance(engine, 1 / 60);
    }

    expect(engine.isTargetSettled(id)).toBe(true);
  });

  it("reduced motion changes traversal but preserves the exact semantic destination", () => {
    const normal = createScene();
    const reduced = createScene();
    const target = {
      entityId: normal.id,
      pose: {
        position: { x: 120, y: -30 },
        rotation: 0.6,
        scale: { x: 1.25, y: 0.85 },
      },
      spring: { stiffness: 170, damping: 26, mass: 1 },
    };

    normal.engine.setTarget(target);
    reduced.engine.setTarget({ ...target, entityId: reduced.id });

    const clock = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });
    for (let i = 0; i < 600 && !normal.engine.isTargetSettled(normal.id); i += 1) {
      clock.advance(normal.engine, 1 / 60);
    }

    reduced.engine.settleTarget(reduced.id);

    expect(normal.engine.snapshot().targets.get(normal.id)?.pose).toEqual(target.pose);
    expect(reduced.engine.snapshot().targets.get(reduced.id)?.pose).toEqual(target.pose);
    expect(reduced.engine.snapshot().entities.get(reduced.id)?.pose).toEqual(target.pose);
    expect(reduced.engine.snapshot().entities.get(reduced.id)?.velocity).toEqual(zeroVelocity());
    expect(normal.engine.isTargetSettled(normal.id)).toBe(true);
    expect(reduced.engine.isTargetSettled(reduced.id)).toBe(true);
  });
});
