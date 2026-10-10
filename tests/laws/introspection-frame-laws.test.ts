import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import type { RadialField } from "../../src/algebra/types";
import { zeroVelocity } from "../../src/dynamics/math";
import { buildSceneReceipt, serializeSceneReceipt } from "../../src/introspection/frame";
import { SceneEngine } from "../../src/scene/engine";
import {
  SpatialEnvironment,
  asRegionId,
  type SpatialOccupant,
} from "../../src/spatial/environment";
import { SpatialFrameGraph, asFrameId } from "../../src/spatial/frame";
import { asConstraintId, solveSpatialConstraints } from "../../src/spatial/constraints";
import { TopologyRegistry, asTopologyId, defineTopology } from "../../src/topology/instance";

function fixture() {
  const root = asEntityId("root");
  const card = asEntityId("card");
  const scene = new SceneEngine({
    fields: [
      {
        id: "focus",
        kind: "radial",
        center: { x: 40, y: 20 },
        radius: 120,
        strength: 8,
        falloff: 2,
      } satisfies RadialField,
    ],
  });

  scene.mutate({
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
          position: { x: 25, y: 15 },
          rotation: 0.2,
          scale: { x: 1.1, y: 1.1 },
        },
        velocity: {
          linear: { x: 12, y: -4 },
          angular: 0.5,
          scale: { x: 0.1, y: 0.1 },
        },
        shape: { kind: "rect", width: 80, height: 120, radius: 10 },
        z: 1,
        visible: true,
        parent: root,
      },
    ],
    relations: [{ kind: "attached", from: card, to: root }],
  });

  scene.setTarget({
    entityId: card,
    pose: {
      position: { x: 60, y: 30 },
      rotation: 0.4,
      scale: { x: 1.2, y: 1.2 },
    },
    spring: { stiffness: 180, damping: 24, mass: 1 },
  });

  const frames = new SpatialFrameGraph();
  frames.add({
    id: asFrameId("world"),
    parent: null,
    origin: { x: 0, y: 0 },
    basisU: { x: 1, y: 0 },
    basisV: { x: 0, y: 1 },
  });

  const topologies = new TopologyRegistry();
  topologies.add(
    defineTopology({
      id: asTopologyId("hand"),
      frameId: asFrameId("world"),
      members: [card],
      solverKind: "tarot-hand",
      constraints: [{ id: "safe", kind: "within-region" }],
      claims: [
        {
          entityId: card,
          degrees: ["position.x", "position.y", "rotation"],
        },
      ],
    }),
    frames.snapshot(),
  );

  const environment = new SpatialEnvironment();
  const workArea = asRegionId("work-area");
  environment.addRegion({
    id: workArea,
    kind: "admissible",
    bounds: { minX: 0, minY: 0, maxX: 800, maxY: 600 },
  });

  const constraint = solveSpatialConstraints({
    desired: { x: 125, y: 65, width: 80, height: 120 },
    constraints: [
      {
        id: asConstraintId("within"),
        ownerId: "hand",
        kind: "within-region",
        regionId: workArea,
      },
    ],
    environment,
    peers: new Map(),
    topologyFacts: new Map(),
  });

  const occupants: SpatialOccupant[] = [
    {
      id: root,
      bounds: { minX: 100, minY: 50, maxX: 500, maxY: 350 },
      visible: true,
    },
    {
      id: card,
      bounds: { minX: 125, minY: 65, maxX: 205, maxY: 185 },
      visible: true,
    },
  ];

  return {
    root,
    card,
    scene,
    topologies,
    environment,
    constraint,
    occupants,
  };
}

describe("MQ-09 deterministic introspection", () => {
  it("observes minimum scene truth without renderer concepts", () => {
    const f = fixture();

    const receipt = buildSceneReceipt({
      scene: f.scene.snapshot(),
      topologies: [...f.topologies.snapshot().keys()].map((id) => f.topologies.observe(id)),
      ownership: f.topologies.ownership.observation(),
      constraints: new Map([[f.card, f.constraint]]),
      environment: f.environment.observation(),
      occupants: f.occupants,
    });

    const card = receipt.entities.find((entity) => entity.id === f.card)!;

    expect(receipt.sceneTime).toBe(0);
    expect(card.parentId).toBe(f.root);
    expect(card.localPose.position).toEqual({ x: 25, y: 15 });
    expect(card.worldPose.position).toEqual({ x: 125, y: 65 });
    expect(card.velocity.linear).toEqual({ x: 12, y: -4 });
    expect(card.topologyIds).toEqual([asTopologyId("hand")]);
    expect(card.ownership).toEqual([
      { degree: "position.x", ownerId: asTopologyId("hand") },
      { degree: "position.y", ownerId: asTopologyId("hand") },
      { degree: "rotation", ownerId: asTopologyId("hand") },
    ]);
    expect(card.target?.pose.position).toEqual({ x: 60, y: 30 });
    expect(card.constraint?.status).toBe("admissible");
    expect(card.containedBy).toEqual([asRegionId("work-area")]);
    expect(card.occludedBy).toEqual([]);
    expect(card.occludes).toEqual([f.root]);
    expect(card.settled).toBe(false);
    expect(receipt.fields.map((field) => field.id)).toEqual(["focus"]);
  });

  it("serializes identically regardless of observation-array insertion order", () => {
    const f = fixture();
    const topologyObservations = [...f.topologies.snapshot().keys()].map((id) =>
      f.topologies.observe(id),
    );
    const ownership = f.topologies.ownership.observation();

    const a = buildSceneReceipt({
      scene: f.scene.snapshot(),
      topologies: topologyObservations,
      ownership,
      constraints: new Map([[f.card, f.constraint]]),
      environment: f.environment.observation(),
      occupants: f.occupants,
    });
    const b = buildSceneReceipt({
      scene: f.scene.snapshot(),
      topologies: [...topologyObservations].reverse(),
      ownership: [...ownership].reverse(),
      constraints: new Map([[f.card, f.constraint]]),
      environment: {
        ...f.environment.observation(),
        regions: [...f.environment.observation().regions].reverse(),
      },
      occupants: [...f.occupants].reverse().reverse(),
    });

    expect(serializeSceneReceipt(a)).toBe(serializeSceneReceipt(b));
  });

  it("returns detached observations that cannot mutate scene truth", () => {
    const f = fixture();
    const receipt = buildSceneReceipt({
      scene: f.scene.snapshot(),
      topologies: [...f.topologies.snapshot().keys()].map((id) => f.topologies.observe(id)),
      ownership: f.topologies.ownership.observation(),
      constraints: new Map([[f.card, f.constraint]]),
      environment: f.environment.observation(),
      occupants: f.occupants,
    });

    const cardReceipt = receipt.entities.find((entity) => entity.id === f.card)!;
    (cardReceipt.localPose.position as { x: number }).x = 9999;
    (receipt.fields[0]!.center as { x: number }).x = 9999;

    expect(f.scene.snapshot().entities.get(f.card)?.pose.position.x).toBe(25);
    expect(f.scene.snapshot().fields[0]?.center.x).toBe(40);
  });
});
