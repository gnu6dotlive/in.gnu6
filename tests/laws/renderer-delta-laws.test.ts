import { describe, expect, it } from "vitest";
import type { SceneEntity } from "../../src/algebra/types";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { buildRendererFrame, type RendererMotionMode } from "../../src/renderer/contract";
import { diffRendererFrames, applyRendererDelta } from "../../src/renderer/delta";
import { SceneEngine } from "../../src/scene/engine";

function item(id: string, x: number): SceneEntity {
  return {
    id: asEntityId(id),
    role: id === "root" ? "surface" : "card",
    pose: {
      position: { x, y: 10 },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    velocity: zeroVelocity(),
    shape: { kind: "rect", width: 60, height: 40, radius: 6 },
    z: 0,
    visible: true,
    parent: null,
  };
}

function frame(
  sequence: number,
  entities: SceneEntity[],
  motionMode: RendererMotionMode = "normal",
) {
  const engine = new SceneEngine();
  engine.mutate({ add: entities });
  return buildRendererFrame(engine.snapshot(), {
    sequence,
    motionMode,
    accessibility: [],
  });
}

describe("BLOB-DONOR-PLM-01 EXP-B immutable scene deltas", () => {
  it("keeps unchanged entities by reference and does not invent identities", () => {
    const old = frame(5, [item("root", 1), item("card", 2)]);
    const next = frame(6, [item("root", 1), item("card", 12)]);
    const delta = diffRendererFrames(old, next);
    expect(delta.births).toHaveLength(0);
    expect(delta.deaths).toHaveLength(0);
    expect(delta.updates.map((e) => e.entityId)).toEqual([asEntityId("card")]);
    expect(Object.isFrozen(delta)).toBe(true);
    expect(Object.isFrozen(delta.updates)).toBe(true);
    const result = applyRendererDelta(old, delta);
    expect(result).toEqual(next);
    expect(result.entities[0]).toBe(old.entities[0]);
    expect(result.entities[1]).toBe(next.entities[1]);
    expect(old.entities[1]?.worldPose.position.x).toBe(2);
  });

  it("replays births, deaths and strict next-frame identity order", () => {
    const old = frame(1, [item("root", 1), item("removed", 2)]);
    const next = frame(2, [item("new", 4), item("root", 1)]);
    const delta = diffRendererFrames(old, next);
    expect(delta.births.map((e) => e.entityId)).toEqual([asEntityId("new")]);
    expect(delta.deaths).toEqual([asEntityId("removed")]);
    expect(delta.updates).toHaveLength(0);
    expect(applyRendererDelta(old, delta)).toEqual(next);
  });

  it("treats cadence and reduced motion as projection metadata", () => {
    const old = frame(1, [item("root", 5)], "normal");
    const next = frame(200, [item("root", 5)], "reduced");
    const delta = diffRendererFrames(old, next);
    expect(delta.updates).toHaveLength(0);
    expect(delta.births).toHaveLength(0);
    expect(delta.deaths).toHaveLength(0);
    expect(applyRendererDelta(old, delta)).toEqual(next);
  });

  it("fails closed on reversed, stale or duplicate delta operations", () => {
    const old = frame(3, [item("root", 1)]);
    const next = frame(4, [item("root", 1), item("new", 4)]);
    const delta = diffRendererFrames(old, next);
    expect(() => diffRendererFrames(next, old)).toThrow("strictly increasing");
    expect(() => applyRendererDelta(next, delta)).toThrow("Stale");
    expect(() =>
      applyRendererDelta(old, { ...delta, births: [...delta.births, ...delta.births] }),
    ).toThrow("Duplicate");
    expect(() => applyRendererDelta(old, { ...delta, order: [asEntityId("root")] })).toThrow(
      "identity partition",
    );
    expect(() => applyRendererDelta(old, { ...delta, deaths: [asEntityId("phantom")] })).toThrow(
      "Unknown",
    );
    expect(() =>
      applyRendererDelta(old, {
        ...delta,
        updates: [item("phantom", 8) as never],
      }),
    ).toThrow("Unknown");
  });

  it("rejects duplicate frame entity identities even if supplied by a forged caller", () => {
    const old = frame(1, [item("root", 1)]);
    const next = frame(2, [item("root", 1)]);
    const forged = { ...next, entities: [...next.entities, next.entities[0]!] };
    expect(() => diffRendererFrames(old, forged)).toThrow("Duplicate semantic identity");
  });
});
