import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import type { SceneEntity } from "../../src/algebra/types";
import { zeroVelocity } from "../../src/dynamics/math";
import { buildRendererFrame } from "../../src/renderer/contract";
import { applyRendererDelta, diffRendererFrames } from "../../src/renderer/delta";
import { SceneEngine } from "../../src/scene/engine";

import fixture from "../../experiments/blobin-next-01/CP02-VULKAN/fixtures/scene-stream.v1.json";

function entity(id: "stage" | "cavity-control", x: number): SceneEntity {
  return {
    id: asEntityId(id),
    role: id === "stage" ? "surface" : "control",
    pose: { position: { x, y: 0 }, rotation: 0, scale: { x: 1, y: 1 } },
    velocity: zeroVelocity(),
    shape: {
      kind: "rect",
      width: id === "stage" ? 400 : 24,
      height: id === "stage" ? 300 : 24,
      radius: id === "stage" ? 10 : 4,
    },
    z: 0,
    visible: true,
    parent: null,
  };
}
function semanticFrame(sequence: number, x: number) {
  const engine = new SceneEngine({ time: sequence - 10 });
  engine.mutate({ add: [entity("stage", 0), entity("cavity-control", x)] });
  return buildRendererFrame(engine.snapshot(), {
    sequence,
    motionMode: "normal",
    accessibility: [],
  });
}

describe("GPU-03 TS→Rust shared immutable renderer fixture", () => {
  it("matches actual in.gnu6 scenes and three canonical deltas", () => {
    const frames = [0, 10, -11, 0].map((x, i) => semanticFrame(10 + i, x));
    expect(fixture.schema).toBe("blobin.gpu03.scene-stream.v1");
    expect(fixture.source).toBe("in.gnu6");
    expect(fixture.initial).toEqual(frames[0]);
    const deltas = frames.slice(1).map((frame, i) => diffRendererFrames(frames[i]!, frame));
    expect(fixture.deltas).toEqual(deltas);
    expect(fixture.binding).toEqual({
      kind: "cavity-control-x",
      entityId: "cavity-control",
      metersPerUnit: 0.01,
    });
    let replay = frames[0]!;
    for (const [i, delta] of deltas.entries()) {
      replay = applyRendererDelta(replay, delta);
      expect(replay).toEqual(frames[i + 1]);
    }
  });
  it("never grants renderer-to-scene mutation authority", () => {
    const engine = new SceneEngine();
    engine.mutate({ add: [entity("stage", 0), entity("cavity-control", 0)] });
    const prior = engine.snapshot();
    const projected = buildRendererFrame(prior, {
      sequence: 10,
      motionMode: "normal",
      accessibility: [],
    });
    const next = semanticFrame(11, 10);
    const delta = diffRendererFrames(projected, next);
    const replay = applyRendererDelta(projected, delta);
    expect(replay.entities[1]?.worldPose.position.x).toBe(10);
    expect(engine.snapshot()).toBe(prior);
    expect(engine.snapshot().entities.get(asEntityId("cavity-control"))?.pose.position.x).toBe(0);
  });
});
