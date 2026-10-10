import { describe, expect, it } from "vitest";
import {
  dockSurfaceFrame,
  layoutDocklabFanInFrame,
  layoutDocklabInteractiveFan,
  previewDocklabReorder,
  updateDockSurfaceFrame,
} from "../../consumers/docklab/harness";
import {
  createSpjWorld,
  layoutSpjHand,
  previewSpjReorder,
  spjDeckSpec,
} from "../../consumers/spj-deck/harness";
import { asEntityId } from "../../src/algebra/laws";
import type { EntityId, SceneEntity } from "../../src/algebra/types";
import { deckPoseTargets, type DeckCardPose } from "../../src/deck/deck";
import { FixedStepClock } from "../../src/dynamics/clock";
import { zeroVelocity } from "../../src/dynamics/math";
import {
  InteractionProjector,
  TopologyHandoffAuthority,
  createTopologyHandoffRequest,
} from "../../src/interaction/projection";
import { buildSceneReceipt } from "../../src/introspection/frame";
import { SceneEngine } from "../../src/scene/engine";
import {
  SpatialEnvironment,
  asRegionId,
  type SpatialOccupant,
} from "../../src/spatial/environment";
import { SpatialFrameGraph, asFrameId, localToWorld } from "../../src/spatial/frame";
import { TopologyRegistry, asTopologyId, defineTopology } from "../../src/topology/instance";

const viewport = { width: 1280, height: 800 };
const spring = { stiffness: 180, damping: 24, mass: 1 };

function entityFromPose(card: DeckCardPose): SceneEntity {
  return {
    id: card.id,
    role: "card",
    pose: {
      position: { ...card.pose.position },
      rotation: card.pose.rotation,
      scale: { ...card.pose.scale },
    },
    velocity: zeroVelocity(),
    shape: { kind: "rect", width: 80, height: 120, radius: 10 },
    z: card.z,
    visible: true,
    parent: null,
  };
}

function occupantFromEntity(entity: SceneEntity): SpatialOccupant {
  const width = 80 * entity.pose.scale.x;
  const height = 120 * entity.pose.scale.y;

  return {
    id: entity.id,
    visible: entity.visible,
    bounds: {
      minX: entity.pose.position.x - width / 2,
      minY: entity.pose.position.y - height / 2,
      maxX: entity.pose.position.x + width / 2,
      maxY: entity.pose.position.y + height / 2,
    },
  };
}

function poseClaims(ids: readonly EntityId[]) {
  return ids.map((entityId) => ({
    entityId,
    degrees: ["position.x", "position.y", "rotation"] as const,
  }));
}

describe("MQ-10 one-core multi-consumer falsification", () => {
  it("runs Docklab and SPJ together through one semantic engine", () => {
    const frames = new SpatialFrameGraph();
    const dockFrameId = asFrameId("dock:surface");
    frames.add(dockSurfaceFrame(dockFrameId, "bottom", viewport));
    const spj = createSpjWorld(frames, viewport);

    const dockIds = ["dock-a", "dock-b", "dock-c"].map(asEntityId);
    const spjIds = ["spj-a", "spj-b", "spj-c"].map(asEntityId);

    const dockInitial = layoutDocklabFanInFrame(frames, dockFrameId, dockIds);
    const spjInitial = layoutSpjHand(frames, spj.handFrameId, spjIds, viewport);

    const scene = new SceneEngine();
    scene.mutate({
      add: [...dockInitial.map(entityFromPose), ...spjInitial.map(entityFromPose)],
    });

    const topologies = new TopologyRegistry();
    topologies.add(
      defineTopology({
        id: asTopologyId("dock:deck"),
        frameId: dockFrameId,
        members: dockIds,
        solverKind: "tarot-fan",
        constraints: [],
        claims: poseClaims(dockIds),
      }),
      frames.snapshot(),
    );
    topologies.add(
      defineTopology({
        id: asTopologyId("spj:hand"),
        frameId: spj.handFrameId,
        members: spjIds,
        solverKind: "tarot-fan",
        constraints: [],
        claims: poseClaims(spjIds),
      }),
      frames.snapshot(),
    );
    topologies.add(
      defineTopology({
        id: asTopologyId("spj:playzone"),
        frameId: spj.playzoneFrameId,
        members: [spjIds[1]!],
        solverKind: "playzone",
        constraints: [],
        claims: [],
      }),
      frames.snapshot(),
    );

    const environment = new SpatialEnvironment();
    environment.addRegion({
      id: asRegionId("world"),
      kind: "admissible",
      bounds: { minX: 0, minY: 0, maxX: 1280, maxY: 800 },
    });

    // Docklab: same frame identity mutates from bottom to left.
    const frameIdentityBefore = frames.snapshot().get(dockFrameId)?.id;
    updateDockSurfaceFrame(frames, dockFrameId, "left", viewport);
    expect(frames.snapshot().get(dockFrameId)?.id).toBe(frameIdentityBefore);

    // Docklab: neighbor field is solved in local tangent/inward coordinates.
    const dockLeftBase = layoutDocklabFanInFrame(frames, dockFrameId, dockIds);
    const dockLeftInteractive = layoutDocklabInteractiveFan(frames, dockFrameId, dockIds, {
      x: 180,
      y: 44,
    });

    expect(dockLeftInteractive[1]!.pose.position.x).toBeGreaterThan(
      dockLeftBase[1]!.pose.position.x,
    );
    expect(dockLeftInteractive.map((card) => card.id)).toEqual(dockIds);

    // Docklab: reorder remains tangent-local and identity-preserving.
    const dockReorder = previewDocklabReorder(dockIds, dockIds[0]!, 260, null);
    expect(dockReorder.order).toEqual([dockIds[1], dockIds[2], dockIds[0]]);

    // SPJ: consumer-responsive metrics change without a new engine solver.
    const largeSpec = spjDeckSpec(viewport, spjIds.length);
    const smallSpec = spjDeckSpec({ width: 640, height: 400 }, spjIds.length);
    expect(smallSpec.cardWidth).toBeLessThan(largeSpec.cardWidth);
    expect(smallSpec.overlap).toBeLessThan(largeSpec.overlap);

    // SPJ: reorder uses the same generic permutation/hysteresis law.
    const spjReorder = previewSpjReorder(spjIds, spjIds[0]!, 200, viewport, null);
    expect(spjReorder.order).toEqual([spjIds[1], spjIds[2], spjIds[0]]);

    // One SceneEngine receives Docklab's new edge-relative targets.
    scene.mutate({
      targets: deckPoseTargets(dockLeftInteractive, spring),
    });

    // SPJ: renderer observation identifies one hand card, then bounded
    // topology authority transfers it to playzone.
    const spjCard = spjIds[1]!;
    const spjPose = spjInitial.find((card) => card.id === spjCard)!;
    const projector = new InteractionProjector(frames.snapshot());
    const projection = projector.project(
      {
        providerId: "synthetic-browser",
        pointerId: "primary",
        sequence: 1,
        phase: "down",
        coordinates: {
          space: "world",
          point: { ...spjPose.pose.position },
        },
      },
      [
        {
          id: spjCard,
          visible: true,
          bounds: {
            minX: spjPose.pose.position.x - 40,
            minY: spjPose.pose.position.y - 60,
            maxX: spjPose.pose.position.x + 40,
            maxY: spjPose.pose.position.y + 60,
          },
        },
      ],
    );

    const handoff = createTopologyHandoffRequest(projection, {
      requestId: "spj-hand-to-playzone",
      from: asTopologyId("spj:hand"),
      to: asTopologyId("spj:playzone"),
      degrees: ["position.x", "position.y", "rotation"],
    });

    new TopologyHandoffAuthority([
      {
        from: asTopologyId("spj:hand"),
        to: asTopologyId("spj:playzone"),
        allowedDegrees: ["position.x", "position.y", "rotation"],
      },
    ]).execute(handoff, topologies.ownership);

    const playzoneCenter = localToWorld(frames.snapshot(), spj.playzoneFrameId, { x: 0, y: 0 });
    const currentSpjCard = scene.snapshot().entities.get(spjCard)!;
    scene.setTarget({
      entityId: spjCard,
      pose: {
        position: playzoneCenter,
        rotation: currentSpjCard.pose.rotation,
        scale: { ...currentSpjCard.pose.scale },
      },
      spring,
    });

    // One fixed-step clock advances both consumer families.
    const clock = new FixedStepClock({
      hz: 60,
      maxStepsPerAdvance: 8,
    });
    for (let i = 0; i < 12; i += 1) {
      clock.advance(scene, 1 / 60);
    }

    const occupants = [...scene.snapshot().entities.values()].map(occupantFromEntity);
    const receipt = buildSceneReceipt({
      scene: scene.snapshot(),
      topologies: [...topologies.snapshot().keys()].map((id) => topologies.observe(id)),
      ownership: topologies.ownership.observation(),
      constraints: new Map(),
      environment: environment.observation(),
      occupants,
    });

    const dockReceipt = receipt.entities.find((entity) => entity.id === dockIds[1])!;
    const spjReceipt = receipt.entities.find((entity) => entity.id === spjCard)!;

    expect(dockReceipt.topologyIds).toContain(asTopologyId("dock:deck"));
    expect(spjReceipt.topologyIds).toEqual([
      asTopologyId("spj:hand"),
      asTopologyId("spj:playzone"),
    ]);
    expect(spjReceipt.ownership.find((entry) => entry.degree === "position.x")?.ownerId).toBe(
      asTopologyId("spj:playzone"),
    );

    expect(receipt.sceneTime).toBeCloseTo(12 / 60, 12);
    expect(receipt.entities).toHaveLength(6);
  });
});
