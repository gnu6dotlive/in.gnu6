import type { EntityId, Pose2, Shape2 } from "../algebra/types";
import type { RendererEntityFrame, RendererFrame, RendererMotionMode } from "./contract";

/**
 * EXP-B: a projection-only delta derived from two authorized semantic frames.
 * This is not an untrusted wire format and grants no mutation authority.
 */
export interface RendererFrameDelta {
  readonly schemaVersion: 1;
  readonly fromSequence: number;
  readonly toSequence: number;
  readonly sceneTime: number;
  readonly motionMode: RendererMotionMode;
  readonly order: readonly EntityId[];
  readonly births: readonly RendererEntityFrame[];
  readonly updates: readonly RendererEntityFrame[];
  readonly deaths: readonly EntityId[];
}

function samePose(a: Pose2, b: Pose2): boolean {
  return (
    a.position.x === b.position.x &&
    a.position.y === b.position.y &&
    a.rotation === b.rotation &&
    a.scale.x === b.scale.x &&
    a.scale.y === b.scale.y
  );
}

function sameShape(a: Shape2, b: Shape2): boolean {
  if (a.kind !== b.kind) return false;
  if (a.kind === "rect" && b.kind === "rect") {
    return a.width === b.width && a.height === b.height && a.radius === b.radius;
  }
  if (a.kind === "polygon" && b.kind === "polygon") {
    return (
      a.closed === b.closed &&
      a.vertices.length === b.vertices.length &&
      a.vertices.every((p, i) => p.x === b.vertices[i]?.x && p.y === b.vertices[i]?.y)
    );
  }
  return false;
}

function sameEntity(a: RendererEntityFrame, b: RendererEntityFrame): boolean {
  const x = a.accessibility;
  const y = b.accessibility;
  return (
    a.entityId === b.entityId &&
    a.role === b.role &&
    a.visible === b.visible &&
    a.z === b.z &&
    samePose(a.worldPose, b.worldPose) &&
    sameShape(a.shape, b.shape) &&
    (x === null
      ? y === null
      : y !== null && x.role === y.role && x.label === y.label && x.hidden === y.hidden)
  );
}

function index(frame: RendererFrame): Map<EntityId, RendererEntityFrame> {
  if (frame.schemaVersion !== 1) throw new Error("Unsupported renderer frame schema");
  const map = new Map<EntityId, RendererEntityFrame>();
  for (const entity of frame.entities) {
    if (map.has(entity.entityId)) throw new Error("Duplicate semantic identity");
    map.set(entity.entityId, entity);
  }
  return map;
}

export function diffRendererFrames(
  before: RendererFrame,
  after: RendererFrame,
): RendererFrameDelta {
  if (
    !Number.isSafeInteger(before.sequence) ||
    !Number.isSafeInteger(after.sequence) ||
    after.sequence <= before.sequence
  ) {
    throw new Error("Renderer delta needs a strictly increasing sequence");
  }
  const old = index(before);
  const next = index(after);
  const births: RendererEntityFrame[] = [];
  const updates: RendererEntityFrame[] = [];
  const deaths: EntityId[] = [];

  for (const previousId of old.keys()) {
    if (!next.has(previousId)) deaths.push(previousId);
  }
  for (const entity of after.entities) {
    const oldEntity = old.get(entity.entityId);
    if (!oldEntity) births.push(entity);
    else if (!sameEntity(oldEntity, entity)) updates.push(entity);
  }
  // The source RendererFrames were deeply frozen by MQ-11. Reuse their
  // entity references instead of re-cloning the entire scene every tick.
  return Object.freeze({
    schemaVersion: 1 as const,
    fromSequence: before.sequence,
    toSequence: after.sequence,
    sceneTime: after.sceneTime,
    motionMode: after.motionMode,
    order: Object.freeze(after.entities.map((x) => x.entityId)),
    births: Object.freeze(births),
    updates: Object.freeze(updates),
    deaths: Object.freeze(deaths),
  });
}

/**
 * Qualified locally only as pure replay. This MUST NOT be used as a
 * renderer-to-semantic mutation ingress.
 */
export function applyRendererDelta(
  previous: RendererFrame,
  delta: RendererFrameDelta,
): RendererFrame {
  if (
    delta.schemaVersion !== 1 ||
    previous.sequence !== delta.fromSequence ||
    !Number.isSafeInteger(delta.toSequence) ||
    delta.toSequence <= delta.fromSequence
  ) {
    throw new Error("Stale, reversed or invalid renderer delta");
  }
  const entities = index(previous);
  const touched = new Set<EntityId>();

  for (const id of delta.deaths) {
    if (!entities.has(id) || touched.has(id)) throw new Error("Unknown or duplicate delta death");
    touched.add(id);
    entities.delete(id);
  }
  for (const entity of delta.updates) {
    if (!entities.has(entity.entityId) || touched.has(entity.entityId)) {
      throw new Error("Unknown, removed or duplicate delta update");
    }
    touched.add(entity.entityId);
    entities.set(entity.entityId, entity);
  }
  for (const entity of delta.births) {
    if (entities.has(entity.entityId) || touched.has(entity.entityId)) {
      throw new Error("Duplicate or existing delta birth");
    }
    touched.add(entity.entityId);
    entities.set(entity.entityId, entity);
  }
  if (
    delta.order.length !== entities.size ||
    new Set(delta.order).size !== entities.size ||
    delta.order.some((id) => !entities.has(id))
  ) {
    throw new Error("Renderer delta identity partition does not match next frame");
  }

  // Existing deeply frozen entity references retain identity and are never
  // mutated. Order and frame metadata are new presentation-only values.
  return Object.freeze({
    schemaVersion: 1 as const,
    sequence: delta.toSequence,
    sceneTime: delta.sceneTime,
    motionMode: delta.motionMode,
    entities: Object.freeze(delta.order.map((id) => entities.get(id)!)),
  });
}
