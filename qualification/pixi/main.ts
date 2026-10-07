import type { Graphics } from "pixi.js";
import { PixiBrowserRenderer } from "../../adapters/pixi";
import { asEntityId } from "../../src/algebra/laws";
import type { EntityId, Pose2, SceneEntity, Shape2 } from "../../src/algebra/types";
import {
  applyDeckPointerField,
  deckPoseTargets,
  layoutTarotFan,
  type DeckSpec,
} from "../../src/deck/deck";
import { FixedStepClock } from "../../src/dynamics/clock";
import { zeroVelocity } from "../../src/dynamics/math";
import {
  InteractionProjector,
  TopologyHandoffAuthority,
  createTopologyHandoffRequest,
} from "../../src/interaction/projection";
import {
  buildRendererFrame,
  rendererSemanticDigest,
  validateRendererResult,
  type RendererMotionMode,
} from "../../src/renderer/contract";
import { SceneEngine } from "../../src/scene/engine";
import { SpatialFrameGraph, asFrameId, localToWorld } from "../../src/spatial/frame";
import { TopologyRegistry, asTopologyId, defineTopology } from "../../src/topology/instance";
import {
  createDeformableContourTrack,
  currentDeformableShape,
  retargetDeformableContour,
  stepDeformableContour,
  type DeformableContourTrack,
} from "../../src/topology/deformation";

const WIDTH = 1200;
const HEIGHT = 720;
const spring = { stiffness: 180, damping: 24, mass: 1 };

const deckSpec: DeckSpec = {
  center: { x: 0, y: 0 },
  cardWidth: 132,
  cardHeight: 206,
  overlap: 78,
  fanAngle: Math.PI / 7,
  fanDepth: 26,
  hoverRadius: 205,
  hoverScale: 1.16,
  hoverLift: 42,
  liftDirection: { x: 0, y: -1 },
  neighborYield: 26,
  neighborFalloff: 2.5,
};

const dockIds = [
  asEntityId("dock-a"),
  asEntityId("dock-b"),
  asEntityId("dock-c"),
  asEntityId("dock-d"),
  asEntityId("dock-e"),
];
const handIds = [
  asEntityId("hand-a"),
  asEntityId("hand-b"),
  asEntityId("hand-c"),
  asEntityId("hand-d"),
];
const morphId = asEntityId("morph-surface");
const insertedId = asEntityId("inserted-card");

const scene = new SceneEngine();
const clock = new FixedStepClock({
  hz: 60,
  maxStepsPerAdvance: 8,
});
const frames = new SpatialFrameGraph();
const topologies = new TopologyRegistry();

const worldFrame = asFrameId("world");
const dockFrame = asFrameId("dock-frame");
const handFrame = asFrameId("hand-frame");
const playzoneFrame = asFrameId("playzone-frame");

frames.add({
  id: worldFrame,
  parent: null,
  origin: { x: 0, y: 0 },
  basisU: { x: 1, y: 0 },
  basisV: { x: 0, y: 1 },
});
frames.add({
  id: dockFrame,
  parent: worldFrame,
  origin: { x: 330, y: 555 },
  basisU: { x: 1, y: 0 },
  basisV: { x: 0, y: 1 },
});
frames.add({
  id: handFrame,
  parent: worldFrame,
  origin: { x: 890, y: 520 },
  basisU: { x: 1, y: 0 },
  basisV: { x: 0, y: 1 },
});
frames.add({
  id: playzoneFrame,
  parent: worldFrame,
  origin: { x: 900, y: 190 },
  basisU: { x: 1, y: 0 },
  basisV: { x: 0, y: 1 },
});

const dockLocal = layoutTarotFan(dockIds, deckSpec);
const handLocal = layoutTarotFan(handIds, {
  ...deckSpec,
  overlap: 66,
  fanAngle: Math.PI / 8,
  hoverLift: 34,
});

scene.mutate({
  add: [
    ...dockLocal.map((card) =>
      cardEntity(
        card.id,
        {
          ...card.pose,
          position: localToWorld(frames.snapshot(), dockFrame, card.pose.position),
        },
        card.z,
      ),
    ),
    ...handLocal.map((card) =>
      cardEntity(
        card.id,
        {
          ...card.pose,
          position: localToWorld(frames.snapshot(), handFrame, card.pose.position),
        },
        20 + card.z,
      ),
    ),
    {
      id: morphId,
      role: "surface",
      pose: {
        position: { x: 600, y: 225 },
        rotation: 0,
        scale: { x: 1, y: 1 },
      },
      velocity: zeroVelocity(),
      shape: {
        kind: "rect",
        width: 320,
        height: 170,
        radius: 28,
      },
      z: 40,
      visible: true,
      parent: null,
    },
  ],
});

topologies.add(
  defineTopology({
    id: asTopologyId("dock"),
    frameId: dockFrame,
    members: dockIds,
    solverKind: "tarot-fan",
    constraints: [],
    claims: dockIds.map((entityId) => ({
      entityId,
      degrees: ["position.x", "position.y", "rotation"],
    })),
  }),
  frames.snapshot(),
);

topologies.add(
  defineTopology({
    id: asTopologyId("hand"),
    frameId: handFrame,
    members: handIds,
    solverKind: "tarot-fan",
    constraints: [],
    claims: handIds.map((entityId) => ({
      entityId,
      degrees: ["position.x", "position.y", "rotation"],
    })),
  }),
  frames.snapshot(),
);

topologies.add(
  defineTopology({
    id: asTopologyId("playzone"),
    frameId: playzoneFrame,
    members: [handIds[1]!],
    solverKind: "playzone",
    constraints: [],
    claims: [],
  }),
  frames.snapshot(),
);

const mount = requiredElement("pixi-mount");
const semanticMount = requiredElement("semantic-mount");
const status = requiredElement("status");
const sceneState = requiredElement("scene-state");

const renderer = await PixiBrowserRenderer.create({
  mount,
  semanticMount,
  width: WIDTH,
  height: HEIGHT,
  preference: "webgl",
  resolution: 1,
  antialias: true,
});

let sequence = 0;
let motionMode: RendererMotionMode = "normal";
let morphTrack: DeformableContourTrack = createDeformableContourTrack(
  morphId,
  scene.snapshot().entities.get(morphId)!.shape,
  diamondShape(),
  spring,
  48,
);

const handleTokens = new WeakMap<object, string>();
let nextHandleToken = 1;

function accessibility() {
  return [
    ...dockIds,
    ...handIds,
    morphId,
    ...(scene.snapshot().entities.has(insertedId) ? [insertedId] : []),
  ].map((entityId) => ({
    entityId,
    role: "button",
    label: "Scene entity " + entityId,
  }));
}

function renderNow(): void {
  const frame = buildRendererFrame(scene.snapshot(), {
    sequence: sequence++,
    motionMode,
    accessibility: accessibility(),
  });
  const result = renderer.render(frame);
  validateRendererResult(frame, result);

  status.textContent =
    renderer.diagnostics().rendererName +
    " / " +
    motionMode +
    " / " +
    scene.snapshot().entities.size +
    " entities";

  sceneState.textContent = "t=" + scene.snapshot().time.toFixed(3) + " · seq=" + frame.sequence;
}

async function tick(count: number): Promise<void> {
  for (let index = 0; index < count; index += 1) {
    clock.advance(scene, 1 / 60);
    renderNow();
    await animationFrame();
  }
}

async function focusDock(index = 2): Promise<void> {
  const base = layoutTarotFan(dockIds, deckSpec);
  const focus = base[index];
  if (!focus) throw new Error("Invalid dock focus index");

  const influenced = applyDeckPointerField(base, focus.pose.position, deckSpec).map((card) => ({
    ...card,
    pose: {
      ...card.pose,
      position: localToWorld(frames.snapshot(), dockFrame, card.pose.position),
    },
  }));

  scene.mutate({
    targets: deckPoseTargets(influenced, spring),
  });

  if (motionMode === "reduced") {
    for (const card of influenced) {
      scene.settleTarget(card.id);
    }
    renderNow();
    return;
  }

  await tick(18);
}

async function morphToDiamond(): Promise<void> {
  morphTrack = retargetDeformableContour(morphTrack, diamondShape());
  await runMorph(30);
}

async function morphToCard(): Promise<void> {
  morphTrack = retargetDeformableContour(morphTrack, {
    kind: "rect",
    width: 116,
    height: 178,
    radius: 22,
  });
  await runMorph(30);
}

async function runMorph(steps: number): Promise<void> {
  if (motionMode === "reduced") {
    for (let index = 0; index < 240; index += 1) {
      morphTrack = stepDeformableContour(morphTrack, 1 / 60);
    }
    scene.setShape(morphId, currentDeformableShape(morphTrack));
    renderNow();
    return;
  }

  for (let index = 0; index < steps; index += 1) {
    morphTrack = stepDeformableContour(morphTrack, 1 / 60);
    scene.setShape(morphId, currentDeformableShape(morphTrack));
    renderNow();
    await animationFrame();
  }
}

async function insertCard(): Promise<void> {
  if (scene.snapshot().entities.has(insertedId)) return;

  scene.mutate({
    add: [
      cardEntity(
        insertedId,
        {
          position: { x: 585, y: 650 },
          rotation: 0,
          scale: { x: 0.2, y: 0.2 },
        },
        60,
      ),
    ],
  });

  scene.setTarget({
    entityId: insertedId,
    pose: {
      position: { x: 600, y: 540 },
      rotation: -0.08,
      scale: { x: 1, y: 1 },
    },
    spring,
  });

  if (motionMode === "reduced") {
    scene.settleTarget(insertedId);
    renderNow();
    return;
  }

  await tick(20);
}

async function removeCard(): Promise<void> {
  if (!scene.snapshot().entities.has(insertedId)) return;

  scene.setTarget({
    entityId: insertedId,
    pose: {
      position: { x: 630, y: 680 },
      rotation: 0.1,
      scale: { x: 0.02, y: 0.02 },
    },
    spring,
  });

  if (motionMode === "reduced") {
    scene.settleTarget(insertedId);
  } else {
    await tick(20);
  }

  scene.mutate({ remove: [insertedId] });
  renderNow();
}

async function handoffToPlayzone(): Promise<void> {
  const subject = handIds[1]!;
  const subjectEntity = scene.snapshot().entities.get(subject);
  if (!subjectEntity) throw new Error("Missing handoff subject");

  const projector = new InteractionProjector(frames.snapshot());
  const projection = projector.project(
    {
      providerId: "pixi-qualification-provider",
      pointerId: "primary",
      sequence: 1,
      phase: "down",
      coordinates: {
        space: "world",
        point: { ...subjectEntity.pose.position },
      },
    },
    [
      {
        id: subject,
        visible: true,
        bounds: {
          minX: subjectEntity.pose.position.x - 60,
          minY: subjectEntity.pose.position.y - 90,
          maxX: subjectEntity.pose.position.x + 60,
          maxY: subjectEntity.pose.position.y + 90,
        },
      },
    ],
  );

  const request = createTopologyHandoffRequest(projection, {
    requestId: "hand-to-playzone",
    from: asTopologyId("hand"),
    to: asTopologyId("playzone"),
    degrees: ["position.x", "position.y", "rotation"],
  });

  new TopologyHandoffAuthority([
    {
      from: asTopologyId("hand"),
      to: asTopologyId("playzone"),
      allowedDegrees: ["position.x", "position.y", "rotation"],
    },
  ]).execute(request, topologies.ownership);

  const playzoneCenter = localToWorld(frames.snapshot(), playzoneFrame, { x: 0, y: 0 });

  scene.setTarget({
    entityId: subject,
    pose: {
      position: playzoneCenter,
      rotation: 0,
      scale: { x: 1.08, y: 1.08 },
    },
    spring,
  });

  if (motionMode === "reduced") {
    scene.settleTarget(subject);
    renderNow();
    return;
  }

  await tick(26);
}

function setMotionMode(next: RendererMotionMode): void {
  motionMode = next;
  renderNow();
}

function handleToken(entityId: EntityId): string | null {
  const handle = renderer.getHandle(entityId);
  if (!handle) return null;
  return tokenForHandle(handle);
}

function tokenForHandle(handle: Graphics): string {
  const existing = handleTokens.get(handle);
  if (existing) return existing;

  const token = "handle-" + nextHandleToken++;
  handleTokens.set(handle, token);
  return token;
}

function snapshot() {
  const state = scene.snapshot();
  return {
    diagnostics: renderer.diagnostics(),
    sceneTime: state.time,
    motionMode,
    semanticDigest: rendererSemanticDigest(
      buildRendererFrame(state, {
        sequence: 0,
        motionMode,
        accessibility: accessibility(),
      }),
    ),
    entities: [...state.entities.values()]
      .map((entity) => ({
        id: entity.id,
        pose: entity.pose,
        shapeKind: entity.shape.kind,
        z: entity.z,
        handleToken: handleToken(entity.id),
      }))
      .sort((a, b) => a.id.localeCompare(b.id)),
    ownership: topologies.ownership.observation(),
    topologies: [...topologies.snapshot().keys()]
      .map((id) => topologies.observe(id))
      .map((topology) => ({
        id: topology.id,
        frameId: topology.frameId,
        members: [...topology.members],
        solverKind: topology.solverKind,
      }))
      .sort((a, b) => a.id.localeCompare(b.id)),
    frames: [...frames.snapshot().values()]
      .map((frame) => ({
        id: frame.id,
        parent: frame.parent,
        origin: { ...frame.origin },
        basisU: { ...frame.basisU },
        basisV: { ...frame.basisV },
      }))
      .sort((a, b) => a.id.localeCompare(b.id)),
    semanticNodes: [...semanticMount.querySelectorAll<HTMLElement>("[data-entity-id]")]
      .map((node) => ({
        id: node.dataset.entityId ?? "",
        role: node.getAttribute("role"),
        label: node.getAttribute("aria-label"),
        hidden: node.hidden !== false,
        tag: node.tagName,
      }))
      .sort((a, b) => a.id.localeCompare(b.id)),
  };
}

function compareMotionParity() {
  const id = asEntityId("parity");
  const base = parityEntity(id);
  const normal = new SceneEngine();
  const reduced = new SceneEngine();
  normal.mutate({ add: [base] });
  reduced.mutate({
    add: [structuredClone(base)],
  });

  const target = {
    entityId: id,
    pose: {
      position: { x: 220, y: 140 },
      rotation: 0.35,
      scale: { x: 1.25, y: 0.9 },
    },
    spring,
  };

  normal.setTarget(target);
  reduced.setTarget(target);

  for (let index = 0; index < 600 && !normal.isTargetSettled(id); index += 1) {
    normal.step(1 / 60);
  }
  reduced.settleTarget(id);

  const normalFrame = buildRendererFrame(normal.snapshot(), {
    sequence: 1,
    motionMode: "normal",
    accessibility: [],
  });
  const reducedFrame = buildRendererFrame(reduced.snapshot(), {
    sequence: 999,
    motionMode: "reduced",
    accessibility: [],
  });

  return {
    normalPose: normal.snapshot().entities.get(id)!.pose,
    reducedPose: reduced.snapshot().entities.get(id)!.pose,
    normalTarget: normal.snapshot().targets.get(id)!.pose,
    reducedTarget: reduced.snapshot().targets.get(id)!.pose,
    semanticDestinationEqual:
      JSON.stringify(normal.snapshot().targets.get(id)!.pose) ===
      JSON.stringify(reduced.snapshot().targets.get(id)!.pose),
    normalMotionMode: normalFrame.motionMode,
    reducedMotionMode: reducedFrame.motionMode,
  };
}

function parityEntity(id: EntityId): SceneEntity {
  return {
    id,
    role: "card",
    pose: {
      position: { x: 30, y: 40 },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    velocity: zeroVelocity(),
    shape: {
      kind: "rect",
      width: 90,
      height: 140,
      radius: 14,
    },
    z: 1,
    visible: true,
    parent: null,
  };
}

function cardEntity(id: EntityId, pose: Pose2, z: number): SceneEntity {
  return {
    id,
    role: "card",
    pose: clonePose(pose),
    velocity: zeroVelocity(),
    shape: {
      kind: "rect",
      width: 132,
      height: 206,
      radius: 18,
    },
    z,
    visible: true,
    parent: null,
  };
}

function clonePose(pose: Pose2): Pose2 {
  return {
    position: { ...pose.position },
    rotation: pose.rotation,
    scale: { ...pose.scale },
  };
}

function diamondShape(): Shape2 {
  return {
    kind: "polygon",
    closed: true,
    vertices: [
      { x: 0, y: -105 },
      { x: 150, y: 0 },
      { x: 0, y: 105 },
      { x: -150, y: 0 },
    ],
  };
}

function requiredElement(id: string): HTMLElement {
  const element = document.getElementById(id);
  if (!element) {
    throw new Error("Missing qualification element #" + id);
  }
  return element;
}

function animationFrame(): Promise<void> {
  return new Promise((resolve) => {
    requestAnimationFrame(() => resolve());
  });
}

renderNow();

const qualificationApi = {
  ready: true,
  focusDock,
  morphToDiamond,
  morphToCard,
  insertCard,
  removeCard,
  handoffToPlayzone,
  setMotionMode,
  snapshot,
  compareMotionParity,
  ids: {
    dock: dockIds,
    hand: handIds,
    morph: morphId,
    inserted: insertedId,
  },
};

declare global {
  interface Window {
    __IN_GNU6_PIXI_QUALIFICATION__: typeof qualificationApi;
  }
}

window.__IN_GNU6_PIXI_QUALIFICATION__ = qualificationApi;

status.textContent = renderer.diagnostics().rendererName + " / ready";
