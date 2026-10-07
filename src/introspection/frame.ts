import type {
  EntityId,
  Pose2,
  PoseTarget,
  RadialField,
  SceneRelation,
  SceneRole,
  SceneState,
  Velocity2,
} from "../algebra/types";
import { isEntityTargetSettled } from "../scene/engine";
import { worldPose } from "../scene/hierarchy";
import type { SpatialConstraintSolveResult } from "../spatial/constraints";
import {
  type EnvironmentObservation,
  type RegionId,
  type SpatialOccupant,
  computeOcclusion,
  containsAabb,
  intersectsAabb,
} from "../spatial/environment";
import type {
  OwnershipObservation,
  TopologyId,
  TopologyObservation,
  DegreeOfFreedom,
} from "../topology/instance";

export interface SceneReceiptInput {
  scene: SceneState;
  topologies: readonly TopologyObservation[];
  ownership: readonly OwnershipObservation[];
  constraints: ReadonlyMap<EntityId, SpatialConstraintSolveResult>;
  environment?: EnvironmentObservation;
  occupants: readonly SpatialOccupant[];
}

export type MotionState = "untargeted" | "active" | "settled";

export interface SceneShapeObservation {
  kind: "rect" | "polygon";
  width?: number;
  height?: number;
  radius?: number;
  vertexCount?: number;
}

export interface EntityOwnershipObservation {
  degree: DegreeOfFreedom;
  ownerId: TopologyId;
}

export interface EntityTopologyObservation {
  topologyId: TopologyId;
  frameId: TopologyObservation["frameId"];
  solverKind: string;
}

export interface SceneEntityReceipt {
  id: EntityId;
  role: SceneRole;
  visible: boolean;
  z: number;
  parentId: EntityId | null;
  localPose: Pose2;
  worldPose: Pose2;
  velocity: Velocity2;
  shape: SceneShapeObservation;
  topologyIds: readonly TopologyId[];
  topologies: readonly EntityTopologyObservation[];
  ownership: readonly EntityOwnershipObservation[];
  target: PoseTarget | null;
  constraint: SpatialConstraintSolveResult | null;
  containedBy: readonly RegionId[];
  intersectsExclusions: readonly RegionId[];
  occludes: readonly EntityId[];
  occludedBy: readonly EntityId[];
  settled: boolean | null;
  motionState: MotionState;
}

export interface SceneReceipt {
  schemaVersion: 1;
  sceneTime: number;
  order: readonly EntityId[];
  entities: readonly SceneEntityReceipt[];
  relations: readonly SceneRelation[];
  fields: readonly RadialField[];
  topologies: readonly TopologyObservation[];
  environment: EnvironmentObservation | null;
}

export interface SceneReceiptDifference {
  path: string;
  before: unknown;
  after: unknown;
}

/**
 * Deterministic, renderer-neutral scene observation.
 *
 * The builder consumes values/snapshots only. It has no reference to:
 * - SceneEngine mutation methods;
 * - TopologyRegistry mutation methods;
 * - OwnershipRegistry mutation methods;
 * - renderer/provider authority.
 *
 * Therefore:
 *
 *   observe(scene) -> receipt
 *
 * is a one-way epistemic projection. Reading a receipt cannot mutate scene
 * truth.
 */
export function buildSceneReceipt(input: SceneReceiptInput): SceneReceipt {
  const topologyObservations = [...input.topologies]
    .map(cloneTopologyObservation)
    .sort((a, b) => a.id.localeCompare(b.id));

  const ownership = [...input.ownership]
    .map((entry) => ({ ...entry }))
    .sort((a, b) => {
      const entityOrder = a.entityId.localeCompare(b.entityId);
      return entityOrder !== 0 ? entityOrder : a.degree.localeCompare(b.degree);
    });

  const environment = input.environment ? cloneEnvironmentObservation(input.environment) : null;

  const occlusion = computeOcclusion(input.occupants);
  const occupantById = new Map(
    input.occupants.map((occupant) => [
      occupant.id,
      {
        ...occupant,
        bounds: { ...occupant.bounds },
      },
    ]),
  );

  const entities = [...input.scene.entities.values()]
    .sort((a, b) => a.id.localeCompare(b.id))
    .map((entity): SceneEntityReceipt => {
      const target = input.scene.targets.get(entity.id) ?? null;
      const entityOwnership = ownership
        .filter((entry) => entry.entityId === entity.id)
        .map((entry) => ({
          degree: entry.degree,
          ownerId: entry.ownerId,
        }));

      const memberships = topologyObservations
        .filter((topology) => topology.members.includes(entity.id))
        .map((topology) => ({
          topologyId: topology.id,
          frameId: topology.frameId,
          solverKind: topology.solverKind,
        }))
        .sort((a, b) => a.topologyId.localeCompare(b.topologyId));

      const occupant = occupantById.get(entity.id);
      const containedBy =
        occupant && environment
          ? environment.regions
              .filter((region) => region.kind !== "exclusion")
              .filter((region) => containsAabb(region.bounds, occupant.bounds))
              .map((region) => region.id)
              .sort((a, b) => a.localeCompare(b))
          : [];

      const intersectsExclusions =
        occupant && environment
          ? environment.regions
              .filter((region) => region.kind === "exclusion")
              .filter((region) => intersectsAabb(region.bounds, occupant.bounds))
              .map((region) => region.id)
              .sort((a, b) => a.localeCompare(b))
          : [];

      const occludes = occlusion
        .filter((pair) => pair.occluder === entity.id)
        .map((pair) => pair.occluded)
        .sort((a, b) => a.localeCompare(b));

      const occludedBy = occlusion
        .filter((pair) => pair.occluded === entity.id)
        .map((pair) => pair.occluder)
        .sort((a, b) => a.localeCompare(b));

      const settled = target ? isEntityTargetSettled(entity, target) : null;

      return {
        id: entity.id,
        role: entity.role,
        visible: entity.visible,
        z: entity.z,
        parentId: entity.parent,
        localPose: clonePose(entity.pose),
        worldPose: clonePose(worldPose(input.scene, entity.id)),
        velocity: cloneVelocity(entity.velocity),
        shape:
          entity.shape.kind === "rect"
            ? {
                kind: "rect",
                width: entity.shape.width,
                height: entity.shape.height,
                radius: entity.shape.radius,
              }
            : {
                kind: "polygon",
                vertexCount: entity.shape.vertices.length,
              },
        topologyIds: memberships.map((membership) => membership.topologyId),
        topologies: memberships,
        ownership: entityOwnership,
        target: target ? cloneTarget(target) : null,
        constraint: cloneConstraintResult(input.constraints.get(entity.id) ?? null),
        containedBy,
        intersectsExclusions,
        occludes,
        occludedBy,
        settled,
        motionState: target === null ? "untargeted" : settled ? "settled" : "active",
      };
    });

  return {
    schemaVersion: 1,
    sceneTime: input.scene.time,
    order: [...input.scene.order],
    entities,
    relations: [...input.scene.relations]
      .map((relation) => ({ ...relation }))
      .sort(compareRelation),
    fields: [...input.scene.fields].map(cloneField).sort((a, b) => a.id.localeCompare(b.id)),
    topologies: topologyObservations,
    environment,
  };
}

export function serializeSceneReceipt(receipt: SceneReceipt): string {
  return JSON.stringify(receipt);
}

/**
 * Structural causal diff.
 *
 * Receipts are deterministic arrays/objects, so a path such as:
 *
 *   entities[0].target.pose.position.x
 *
 * identifies the exact observed fact that diverged.
 */
export function diffSceneReceipts(
  before: SceneReceipt,
  after: SceneReceipt,
): SceneReceiptDifference[] {
  const differences: SceneReceiptDifference[] = [];
  diffValue(before, after, "", differences);
  return differences;
}

export function formatSceneReceiptDiff(differences: readonly SceneReceiptDifference[]): string {
  return differences
    .map(
      (difference) =>
        difference.path +
        ": " +
        formatValue(difference.before) +
        " -> " +
        formatValue(difference.after),
    )
    .join("\n");
}

function diffValue(
  before: unknown,
  after: unknown,
  path: string,
  output: SceneReceiptDifference[],
): void {
  if (Object.is(before, after)) return;

  if (Array.isArray(before) && Array.isArray(after)) {
    if (before.length !== after.length) {
      output.push({
        path: path + ".length",
        before: before.length,
        after: after.length,
      });
    }

    const length = Math.min(before.length, after.length);
    for (let index = 0; index < length; index += 1) {
      diffValue(before[index], after[index], path + "[" + index + "]", output);
    }
    return;
  }

  if (isRecord(before) && isRecord(after)) {
    const keys = new Set([...Object.keys(before), ...Object.keys(after)]);

    for (const key of [...keys].sort()) {
      const childPath = path ? path + "." + key : key;
      diffValue(before[key], after[key], childPath, output);
    }
    return;
  }

  output.push({
    path: path || "$",
    before: cloneUnknown(before),
    after: cloneUnknown(after),
  });
}

function cloneTopologyObservation(observation: TopologyObservation): TopologyObservation {
  return {
    id: observation.id,
    frameId: observation.frameId,
    solverKind: observation.solverKind,
    members: [...observation.members],
    constraints: observation.constraints
      .map((constraint) => ({ ...constraint }))
      .sort((a, b) => a.id.localeCompare(b.id)),
    authority: observation.authority
      .map((entry) => ({ ...entry }))
      .sort((a, b) => {
        const entityOrder = a.entityId.localeCompare(b.entityId);
        return entityOrder !== 0 ? entityOrder : a.degree.localeCompare(b.degree);
      }),
  };
}

function cloneEnvironmentObservation(observation: EnvironmentObservation): EnvironmentObservation {
  return {
    revision: observation.revision,
    regions: observation.regions
      .map((region) => ({
        ...region,
        bounds: { ...region.bounds },
      }))
      .sort((a, b) => a.id.localeCompare(b.id)),
  };
}

function cloneConstraintResult(
  result: SpatialConstraintSolveResult | null,
): SpatialConstraintSolveResult | null {
  if (!result) return null;

  return {
    status: result.status,
    desired: { ...result.desired },
    rect: result.rect
      ? {
          ...result.rect,
          bounds: { ...result.rect.bounds },
        }
      : null,
    evaluations: result.evaluations
      .map((evaluation) => ({ ...evaluation }))
      .sort((a, b) => a.id.localeCompare(b.id)),
    conflicts: result.conflicts
      .map((conflict) => ({
        constraintIds: [...conflict.constraintIds].sort((a, b) => a.localeCompare(b)),
        reason: conflict.reason,
      }))
      .sort((a, b) => {
        const reasonOrder = a.reason.localeCompare(b.reason);
        return reasonOrder !== 0
          ? reasonOrder
          : a.constraintIds.join("\u0000").localeCompare(b.constraintIds.join("\u0000"));
      }),
  };
}

function clonePose(pose: Pose2): Pose2 {
  return {
    position: { ...pose.position },
    rotation: pose.rotation,
    scale: { ...pose.scale },
  };
}

function cloneVelocity(velocity: Velocity2): Velocity2 {
  return {
    linear: { ...velocity.linear },
    angular: velocity.angular,
    scale: { ...velocity.scale },
  };
}

function cloneTarget(target: PoseTarget): PoseTarget {
  return {
    entityId: target.entityId,
    pose: clonePose(target.pose),
    spring: { ...target.spring },
  };
}

function cloneField(field: RadialField): RadialField {
  return {
    ...field,
    center: { ...field.center },
  };
}

function compareRelation(a: SceneRelation, b: SceneRelation): number {
  const kindOrder = a.kind.localeCompare(b.kind);
  if (kindOrder !== 0) return kindOrder;

  const fromOrder = a.from.localeCompare(b.from);
  return fromOrder !== 0 ? fromOrder : a.to.localeCompare(b.to);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function cloneUnknown(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(cloneUnknown);
  }
  if (isRecord(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([key, child]) => [key, cloneUnknown(child)]),
    );
  }
  return value;
}

function formatValue(value: unknown): string {
  const serialized = JSON.stringify(value);
  return serialized === undefined ? String(value) : serialized;
}
