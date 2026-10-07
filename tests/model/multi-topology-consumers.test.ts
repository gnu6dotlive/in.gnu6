import { describe, expect, it } from "vitest";
import { dockEdgeFrame, layoutDocklabFan } from "../../consumers/docklab/harness";
import { createSpjWorld, layoutSpjHand } from "../../consumers/spj-deck/harness";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { SceneEngine } from "../../src/scene/engine";
import { SpatialFrameGraph, asFrameId } from "../../src/spatial/frame";
import { OwnershipRegistry, asTopologyId } from "../../src/topology/instance";

describe("MQ-02 multi-consumer falsification", () => {
  it("hosts Docklab edge topology and SPJ viewport topology in one World", () => {
    const frames = new SpatialFrameGraph();
    frames.add(dockEdgeFrame("bottom", { width: 1440, height: 900 }));
    const spj = createSpjWorld(frames, { width: 1280, height: 800 });

    const dock = layoutDocklabFan(frames, ["dock-a", "dock-b", "dock-c"].map(asEntityId));
    const hand = layoutSpjHand(
      frames,
      spj.handFrameId,
      ["spj-a", "spj-b", "spj-c"].map(asEntityId),
    );

    expect(dock).toHaveLength(3);
    expect(hand).toHaveLength(3);
    expect(frames.snapshot().has(asFrameId("dock:bottom"))).toBe(true);
    expect(frames.snapshot().has(spj.handFrameId)).toBe(true);
  });

  it("allows SPJ hand and playzone to coexist, then transfers one card's pose authority", () => {
    const frames = new SpatialFrameGraph();
    const spj = createSpjWorld(frames, { width: 1280, height: 800 });
    const card = asEntityId("spj-card-1");
    const ownership = new OwnershipRegistry();

    ownership.claim(asTopologyId("spj:hand"), card, ["position.x", "position.y", "rotation"]);
    ownership.transfer(asTopologyId("spj:hand"), asTopologyId("spj:drag"), card, [
      "position.x",
      "position.y",
      "rotation",
    ]);
    ownership.transfer(asTopologyId("spj:drag"), asTopologyId("spj:playzone"), card, [
      "position.x",
      "position.y",
      "rotation",
    ]);

    expect(ownership.ownerOf(card, "position.x")).toBe(asTopologyId("spj:playzone"));
    expect(frames.snapshot().has(spj.playzoneFrameId)).toBe(true);
  });

  it("changes topology authority without recreating the subject or erasing velocity", () => {
    const subject = asEntityId("shared-card");
    const scene = new SceneEngine();
    scene.mutate({
      add: [
        {
          id: subject,
          role: "card",
          pose: {
            position: { x: 10, y: 20 },
            rotation: 0.1,
            scale: { x: 1, y: 1 },
          },
          velocity: {
            ...zeroVelocity(),
            linear: { x: 120, y: -30 },
            angular: 0.7,
          },
          shape: { kind: "rect", width: 72, height: 112, radius: 12 },
          z: 3,
          visible: true,
          parent: null,
        },
      ],
    });

    const ownership = new OwnershipRegistry();
    const hand = asTopologyId("spj:hand");
    const drag = asTopologyId("spj:drag");

    ownership.claim(hand, subject, ["position.x", "position.y", "rotation"]);

    const before = scene.snapshot().entities.get(subject);
    ownership.transfer(hand, drag, subject, ["position.x", "position.y", "rotation"]);
    const after = scene.snapshot().entities.get(subject);

    expect(after?.id).toBe(subject);
    expect(after?.velocity).toEqual(before?.velocity);
    expect(after?.pose).toEqual(before?.pose);
    expect(ownership.ownerOf(subject, "rotation")).toBe(drag);
  });
});
