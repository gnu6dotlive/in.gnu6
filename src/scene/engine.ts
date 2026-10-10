import { assertSceneLaws } from "../algebra/laws";
import { integrateScalar, zeroVelocity } from "../dynamics/math";
import type {
  EntityId,
  Pose2,
  PoseTarget,
  SceneEntity,
  SceneRelation,
  SceneState,
  Shape2,
} from "../algebra/types";

export interface SceneMutation {
  add?: readonly SceneEntity[];
  remove?: readonly EntityId[];
  relations?: readonly SceneRelation[];
  targets?: readonly PoseTarget[];
}

export interface SettleThresholds {
  position: number;
  rotation: number;
  scale: number;
  velocity: number;
}

export const DEFAULT_SETTLE_THRESHOLDS: SettleThresholds = {
  position: 0.001,
  rotation: 0.001,
  scale: 0.001,
  velocity: 0.001,
};

/**
 * SceneEngine owns time integration, not product intent.
 *
 * It does not know what a "Dock", "menu" or "window" is.
 * It only knows:
 *
 *   - finite identity set E
 *   - relation set R
 *   - deterministic order permutation π(E)
 *   - target map T : E ⇀ P
 *   - continuous state evolution
 *
 * Surface adapters may propose targets, but they may not bypass these laws.
 */
export class SceneEngine {
  private state: SceneState;

  constructor(initial?: Partial<SceneState>) {
    this.state = {
      time: initial?.time ?? 0,
      entities: initial?.entities ?? new Map(),
      relations: initial?.relations ?? [],
      order: initial?.order ?? [],
      targets: initial?.targets ?? new Map(),
      fields: initial?.fields ?? [],
    };
    assertSceneLaws(this.state);
  }

  snapshot(): SceneState {
    return this.state;
  }

  /**
   * Transactional scene mutation.
   *
   * If E is the current identity set:
   *
   *   E' = (E ∪ add) \ remove
   *
   * The mutation commits only after E', R', order' and T' satisfy all laws.
   * This makes invalid intermediate graph states unobservable.
   */
  mutate(change: SceneMutation): SceneState {
    const entities = new Map(this.state.entities);
    const order = [...this.state.order];
    const targets = new Map(this.state.targets);

    for (const id of change.remove ?? []) {
      entities.delete(id);
      targets.delete(id);
      const index = order.indexOf(id);
      if (index >= 0) order.splice(index, 1);
    }

    for (const entity of change.add ?? []) {
      if (entities.has(entity.id)) {
        throw new Error("Duplicate scene identity: " + entity.id);
      }
      entities.set(entity.id, entity);
      order.push(entity.id);
    }

    for (const target of change.targets ?? []) {
      if (!entities.has(target.entityId)) {
        throw new Error("Target owner is not in scene: " + target.entityId);
      }
      targets.set(target.entityId, target);
    }

    const relations =
      change.relations ??
      this.state.relations.filter(
        (relation) => entities.has(relation.from) && entities.has(relation.to),
      );

    const next: SceneState = {
      ...this.state,
      entities,
      order,
      targets,
      relations,
    };

    assertSceneLaws(next);
    this.state = next;
    return next;
  }

  setTarget(target: PoseTarget): SceneState {
    return this.mutate({ targets: [target] });
  }

  clearTarget(id: EntityId): SceneState {
    const targets = new Map(this.state.targets);
    targets.delete(id);
    const next = { ...this.state, targets };
    assertSceneLaws(next);
    this.state = next;
    return next;
  }

  /**
   * Replace only the geometric representation of an existing subject.
   *
   * Identity, traversal order, pose, velocity, parentage and active target are
   * preserved. This is the scene-level bridge required by continuous topology
   * morphing:
   *
   *   same subject id
   *   + evolving Shape2
   *   != remove(old) + add(new)
   */
  setShape(id: EntityId, shape: Shape2): SceneState {
    const entity = this.state.entities.get(id);
    if (!entity) throw new Error("Unknown scene entity: " + id);

    const entities = new Map(this.state.entities);
    entities.set(id, {
      ...entity,
      shape: cloneShape(shape),
    });

    const next: SceneState = {
      ...this.state,
      entities,
    };

    assertSceneLaws(next);
    this.state = next;
    return next;
  }

  /**
   * Settled is a relation between current dynamic state and its exact target.
   *
   * Elapsed time is not evidence of sleep.
   *
   * An entity is settled only when:
   * - positional error is small;
   * - rotational error is small;
   * - scale error is small;
   * - every velocity component is small.
   */
  isTargetSettled(id: EntityId, thresholds: SettleThresholds = DEFAULT_SETTLE_THRESHOLDS): boolean {
    const entity = this.state.entities.get(id);
    const target = this.state.targets.get(id);
    if (!entity || !target) return false;
    return isEntityTargetSettled(entity, target, thresholds);
  }

  /**
   * Direct settle is a traversal policy, not a different destination.
   *
   * Reduced-motion consumers may choose this path:
   *
   *   current pose -> exact target pose
   *
   * while preserving the same target declaration in scene state.
   */
  settleTarget(id: EntityId): SceneState {
    const entity = this.state.entities.get(id);
    const target = this.state.targets.get(id);
    if (!entity) throw new Error("Unknown scene entity: " + id);
    if (!target) throw new Error("Scene entity has no target: " + id);

    const entities = new Map(this.state.entities);
    entities.set(id, {
      ...entity,
      pose: {
        position: { ...target.pose.position },
        rotation: target.pose.rotation,
        scale: { ...target.pose.scale },
      },
      velocity: zeroVelocity(),
    });

    const next = {
      ...this.state,
      entities,
    };

    assertSceneLaws(next);
    this.state = next;
    return next;
  }

  /**
   * Advance every targeted entity by one bounded simulation step.
   *
   * We clamp dt because a suspended tab can resume with an arbitrarily large
   * wall-clock delta. Physics time is a model choice, not an obligation to
   * replay seconds of missed integration in one frame.
   */
  step(dtSeconds: number): SceneState {
    const dt = Math.min(1 / 15, Math.max(0, dtSeconds));
    if (dt === 0) return this.state;

    const entities = new Map(this.state.entities);

    for (const [id, target] of this.state.targets) {
      const entity = entities.get(id);
      if (!entity) continue;

      entities.set(id, integrateEntity(entity, target, dt));
    }

    const next: SceneState = {
      ...this.state,
      time: this.state.time + dt,
      entities,
    };

    assertSceneLaws(next);
    this.state = next;
    return next;
  }
}

export function isEntityTargetSettled(
  entity: SceneEntity,
  target: PoseTarget,
  thresholds: SettleThresholds = DEFAULT_SETTLE_THRESHOLDS,
): boolean {
  const positionError = Math.hypot(
    entity.pose.position.x - target.pose.position.x,
    entity.pose.position.y - target.pose.position.y,
  );
  const scaleError = Math.max(
    Math.abs(entity.pose.scale.x - target.pose.scale.x),
    Math.abs(entity.pose.scale.y - target.pose.scale.y),
  );
  const velocity = Math.max(
    Math.abs(entity.velocity.linear.x),
    Math.abs(entity.velocity.linear.y),
    Math.abs(entity.velocity.angular),
    Math.abs(entity.velocity.scale.x),
    Math.abs(entity.velocity.scale.y),
  );

  return (
    positionError <= thresholds.position &&
    Math.abs(entity.pose.rotation - target.pose.rotation) <= thresholds.rotation &&
    scaleError <= thresholds.scale &&
    velocity <= thresholds.velocity
  );
}

function cloneShape(shape: Shape2): Shape2 {
  if (shape.kind === "rect") return { ...shape };

  return {
    kind: "polygon",
    closed: true,
    vertices: shape.vertices.map((point) => ({ ...point })),
  };
}

function integrateEntity(entity: SceneEntity, target: PoseTarget, dt: number): SceneEntity {
  const law = target.spring;

  const x = integrateScalar(
    entity.pose.position.x,
    entity.velocity.linear.x,
    target.pose.position.x,
    dt,
    law.stiffness,
    law.damping,
    law.mass,
  );
  const y = integrateScalar(
    entity.pose.position.y,
    entity.velocity.linear.y,
    target.pose.position.y,
    dt,
    law.stiffness,
    law.damping,
    law.mass,
  );
  const rotation = integrateScalar(
    entity.pose.rotation,
    entity.velocity.angular,
    target.pose.rotation,
    dt,
    law.stiffness,
    law.damping,
    law.mass,
  );
  const scaleX = integrateScalar(
    entity.pose.scale.x,
    entity.velocity.scale.x,
    target.pose.scale.x,
    dt,
    law.stiffness,
    law.damping,
    law.mass,
  );
  const scaleY = integrateScalar(
    entity.pose.scale.y,
    entity.velocity.scale.y,
    target.pose.scale.y,
    dt,
    law.stiffness,
    law.damping,
    law.mass,
  );

  const pose: Pose2 = {
    position: { x: x.position, y: y.position },
    rotation: rotation.position,
    scale: { x: scaleX.position, y: scaleY.position },
  };

  return {
    ...entity,
    pose,
    velocity: {
      linear: { x: x.velocity, y: y.velocity },
      angular: rotation.velocity,
      scale: { x: scaleX.velocity, y: scaleY.velocity },
    },
  };
}
