import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import {
  buildSceneReceipt,
  diffSceneReceipts,
  formatSceneReceiptDiff,
} from "../../src/introspection/frame";
import { SceneEngine } from "../../src/scene/engine";

function sceneAt(targetX: number) {
  const id = asEntityId("card");
  const scene = new SceneEngine();

  scene.mutate({
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
        shape: { kind: "rect", width: 80, height: 120, radius: 10 },
        z: 0,
        visible: true,
        parent: null,
      },
    ],
  });

  scene.setTarget({
    entityId: id,
    pose: {
      position: { x: targetX, y: 0 },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    spring: { stiffness: 180, damping: 24, mass: 1 },
  });

  return { scene, id };
}

describe("MQ-09 causal receipt diff", () => {
  it("names the exact causal path when two scene receipts diverge", () => {
    const a = sceneAt(100);
    const b = sceneAt(140);

    const receiptA = buildSceneReceipt({
      scene: a.scene.snapshot(),
      topologies: [],
      ownership: [],
      constraints: new Map(),
      occupants: [],
    });
    const receiptB = buildSceneReceipt({
      scene: b.scene.snapshot(),
      topologies: [],
      ownership: [],
      constraints: new Map(),
      occupants: [],
    });

    const diff = diffSceneReceipts(receiptA, receiptB);

    expect(diff).toContainEqual({
      path: "entities[0].target.pose.position.x",
      before: 100,
      after: 140,
    });

    expect(formatSceneReceiptDiff(diff)).toContain(
      "entities[0].target.pose.position.x: 100 -> 140",
    );
  });

  it("returns no differences for equivalent receipts", () => {
    const a = sceneAt(100);
    const b = sceneAt(100);

    const receiptA = buildSceneReceipt({
      scene: a.scene.snapshot(),
      topologies: [],
      ownership: [],
      constraints: new Map(),
      occupants: [],
    });
    const receiptB = buildSceneReceipt({
      scene: b.scene.snapshot(),
      topologies: [],
      ownership: [],
      constraints: new Map(),
      occupants: [],
    });

    expect(diffSceneReceipts(receiptA, receiptB)).toEqual([]);
  });
});
