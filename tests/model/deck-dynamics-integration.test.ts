import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { deckPoseTargets, insertDeckCard, layoutCardSpread } from "../../src/deck/deck";
import { FixedStepClock } from "../../src/dynamics/clock";
import { zeroVelocity } from "../../src/dynamics/math";
import { SceneEngine } from "../../src/scene/engine";

describe("MQ-06 deck mutation through scene dynamics", () => {
  it("moves surviving cards continuously after insertion instead of teleporting", () => {
    const beforeOrder = ["a", "b", "c"].map(asEntityId);
    const afterOrder = insertDeckCard(beforeOrder, asEntityId("x"), 1);
    const before = layoutCardSpread(beforeOrder, {
      center: { x: 0, y: 0 },
      gap: 80,
    });
    const after = layoutCardSpread(afterOrder, {
      center: { x: 0, y: 0 },
      gap: 80,
    });

    const scene = new SceneEngine();
    scene.mutate({
      add: before.map((card) => ({
        id: card.id,
        role: "card" as const,
        pose: card.pose,
        velocity: zeroVelocity(),
        shape: { kind: "rect" as const, width: 72, height: 112, radius: 12 },
        z: card.z,
        visible: true,
        parent: null,
      })),
    });

    const survivingTargets = deckPoseTargets(
      after.filter((card) => beforeOrder.includes(card.id)),
      { stiffness: 180, damping: 24, mass: 1 },
    );
    scene.mutate({ targets: survivingTargets });

    const bBefore = scene.snapshot().entities.get(asEntityId("b"))!.pose.position.x;
    const bTarget = scene.snapshot().targets.get(asEntityId("b"))!.pose.position.x;

    expect(bBefore).toBe(0);
    expect(bTarget).toBe(40);

    const clock = new FixedStepClock({ hz: 60, maxStepsPerAdvance: 8 });
    clock.advance(scene, 1 / 60);

    const bAfter = scene.snapshot().entities.get(asEntityId("b"))!;
    expect(bAfter.id).toBe(asEntityId("b"));
    expect(bAfter.pose.position.x).toBeGreaterThan(0);
    expect(bAfter.pose.position.x).toBeLessThan(40);
    expect(bAfter.velocity.linear.x).toBeGreaterThan(0);
  });
});
