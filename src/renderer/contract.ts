import type { EntityId, Pose2, SceneRole, SceneState, Shape2 } from "../algebra/types";
import { worldPose } from "../scene/hierarchy";

export type RendererMotionMode = "normal" | "reduced";

export interface RendererAccessibilityInput {
  entityId: EntityId;
  role: string;
  label?: string;
  hidden?: boolean;
}

export interface RendererAccessibility {
  role: string;
  label?: string;
  hidden: boolean;
}

export interface RendererEntityFrame {
  entityId: EntityId;
  role: SceneRole;
  visible: boolean;
  z: number;
  worldPose: Pose2;
  shape: Shape2;
  accessibility: RendererAccessibility | null;
}

export interface RendererFrame {
  schemaVersion: 1;
  sequence: number;
  sceneTime: number;
  motionMode: RendererMotionMode;
  entities: readonly RendererEntityFrame[];
}

export interface RendererFrameOptions {
  sequence: number;
  motionMode: RendererMotionMode;
  accessibility: readonly RendererAccessibilityInput[];
}

export type RendererOmissionReason = "not-visible" | "culled" | "unsupported";

export interface RendererOmission {
  entityId: EntityId;
  reason: RendererOmissionReason;
}

export interface RendererResult<T> {
  adapterId: string;
  sequence: number;
  emittedEntityIds: readonly EntityId[];
  omitted: readonly RendererOmission[];
  output: T;
}

export interface RendererAdapter<T> {
  readonly id: string;
  render(frame: RendererFrame): RendererResult<T>;
}

/**
 * Renderer projection boundary.
 *
 * Scene truth enters as an immutable state snapshot.
 * The renderer receives a detached, deeply frozen frame containing only facts
 * needed for projection.
 *
 * A renderer does not receive:
 * - SceneEngine mutation authority;
 * - topology ownership authority;
 * - clock stepping authority;
 * - product/business state.
 *
 * Therefore:
 *
 *   scene truth -> RendererFrame -> renderer artifact
 *
 * is downstream-only.
 */
export function buildRendererFrame(
  scene: SceneState,
  options: RendererFrameOptions,
): RendererFrame {
  if (!Number.isInteger(options.sequence) || options.sequence < 0) {
    throw new Error("Renderer frame sequence must be a non-negative integer");
  }

  const accessibility = normalizeAccessibility(scene, options.accessibility);

  const entities = scene.order.map((entityId): RendererEntityFrame => {
    const entity = scene.entities.get(entityId);
    if (!entity) {
      throw new Error("Renderer frame order references missing entity: " + entityId);
    }

    return {
      entityId,
      role: entity.role,
      visible: entity.visible,
      z: entity.z,
      worldPose: clonePose(worldPose(scene, entityId)),
      shape: cloneShape(entity.shape),
      accessibility: accessibility.get(entityId) ?? null,
    };
  });

  return deepFreeze({
    schemaVersion: 1 as const,
    sequence: options.sequence,
    sceneTime: scene.time,
    motionMode: options.motionMode,
    entities,
  });
}

/**
 * Validate that an adapter accounts for every frame identity exactly once.
 *
 * Every frame entity must be either:
 * - emitted; or
 * - explicitly omitted with a reason.
 *
 * No adapter may invent a semantic entity identity.
 */
export function validateRendererResult<T>(frame: RendererFrame, result: RendererResult<T>): void {
  if (!result.adapterId) {
    throw new Error("Renderer adapter identity must be non-empty");
  }
  if (result.sequence !== frame.sequence) {
    throw new Error("Renderer result sequence does not match frame");
  }

  const frameIds = new Set(frame.entities.map((entity) => entity.entityId));
  const accounted = new Set<EntityId>();

  for (const id of result.emittedEntityIds) {
    if (!frameIds.has(id)) {
      throw new Error("Renderer invented semantic identity: " + id);
    }
    if (accounted.has(id)) {
      throw new Error("Renderer accounted for identity more than once: " + id);
    }
    accounted.add(id);
  }

  for (const omission of result.omitted) {
    if (!frameIds.has(omission.entityId)) {
      throw new Error("Renderer omitted invented semantic identity: " + omission.entityId);
    }
    if (accounted.has(omission.entityId)) {
      throw new Error("Renderer accounted for identity more than once: " + omission.entityId);
    }
    accounted.add(omission.entityId);
  }

  if (accounted.size !== frameIds.size) {
    throw new Error("Renderer identity partition does not cover the entire frame");
  }
}

/**
 * Digest the semantic render payload while intentionally excluding:
 * - renderer observation sequence;
 * - normal/reduced traversal mode.
 *
 * Those values can affect projection/traversal behavior but cannot redefine the
 * semantic scene state being projected.
 */
export function rendererSemanticDigest(frame: RendererFrame): string {
  return JSON.stringify({
    sceneTime: frame.sceneTime,
    entities: frame.entities,
  });
}

function normalizeAccessibility(
  scene: SceneState,
  inputs: readonly RendererAccessibilityInput[],
): Map<EntityId, RendererAccessibility> {
  const result = new Map<EntityId, RendererAccessibility>();

  for (const input of inputs) {
    if (!scene.entities.has(input.entityId)) {
      throw new Error("Renderer accessibility references unknown entity: " + input.entityId);
    }
    if (result.has(input.entityId)) {
      throw new Error("Renderer accessibility identity must be injective: " + input.entityId);
    }
    if (!input.role) {
      throw new Error("Renderer accessibility role must be non-empty");
    }

    const normalized: RendererAccessibility = {
      role: input.role,
      hidden: input.hidden ?? false,
    };

    if (input.label !== undefined) {
      normalized.label = input.label;
    }

    result.set(input.entityId, normalized);
  }

  return result;
}

function clonePose(pose: Pose2): Pose2 {
  return {
    position: { ...pose.position },
    rotation: pose.rotation,
    scale: { ...pose.scale },
  };
}

function cloneShape(shape: Shape2): Shape2 {
  if (shape.kind === "rect") {
    return { ...shape };
  }

  return {
    kind: "polygon",
    closed: true,
    vertices: shape.vertices.map((point) => ({ ...point })),
  };
}

function deepFreeze<T>(value: T): T {
  if (typeof value !== "object" || value === null || Object.isFrozen(value)) {
    return value;
  }

  Object.freeze(value);

  for (const child of Object.values(value)) {
    deepFreeze(child);
  }

  return value;
}
