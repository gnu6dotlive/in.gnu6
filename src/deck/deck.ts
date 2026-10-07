import { clamp } from "../dynamics/math";
import type { EntityId, Pose2, PoseTarget, SpringLaw, Vec2 } from "../algebra/types";
import { computeOcclusion } from "../spatial/environment";
import { type FrameId, localToWorld, type SpatialFrameSnapshot } from "../spatial/frame";

export interface DeckInfluenceSpec {
  hoverRadius: number;
  hoverScale: number;
  hoverLift: number;
  neighborYield: number;
  neighborFalloff: number;
}

export interface DeckSpec extends DeckInfluenceSpec {
  center: Vec2;
  cardWidth: number;
  cardHeight: number;
  overlap: number;
  fanAngle: number;
  fanDepth: number;
}

export interface StackSpec {
  origin: Vec2;
  offset: Vec2;
  rotationStep: number;
  scaleStep: number;
}

export interface SpreadSpec {
  center: Vec2;
  gap: number;
}

export interface CardExtent {
  width: number;
  height: number;
}

export interface VelocityTiltSpec {
  gain: number;
  maxTilt: number;
}

export interface DeckOrderMove {
  id: EntityId;
  fromIndex: number;
  toIndex: number;
}

export interface DeckOrderTransition {
  entering: readonly EntityId[];
  leaving: readonly EntityId[];
  moved: readonly DeckOrderMove[];
}

export interface DeckTransferResult {
  source: EntityId[];
  destination: EntityId[];
  subjectId: EntityId;
  sourceIndex: number;
  destinationIndex: number;
}

export interface DeckCardPose {
  id: EntityId;
  index: number;
  pose: Pose2;
  z: number;
  influence: number;
}

/**
 * A deck order is a permutation, never a multiset.
 *
 * For card set C:
 *
 *   order ∈ Sym(C)
 *
 * Reordering changes π, not C.
 */
export function assertPermutation(ids: readonly EntityId[]): void {
  if (new Set(ids).size !== ids.length) {
    throw new Error("Deck order must be injective");
  }
}

/**
 * Tarot fan baseline.
 *
 * Let n = |C| and i be card index.
 * Normalize index around zero:
 *
 *   u(i) = i - (n - 1)/2
 *
 * Then:
 *
 *   x(i) = cx + u(i) * overlap
 *   y(i) = cy + fanDepth * |u(i)| / max(1,(n-1)/2)
 *   θ(i) = fanAngle * u(i) / max(1,(n-1)/2)
 *
 * Consequences:
 *   x(center-k) and x(center+k) are symmetric around cx
 *   rotations have opposite sign
 *   edge cards sit deeper than center cards
 *
 * This gives the minimum useful "hand of cards" topology without encoding
 * any consumer-specific styling.
 */
export function layoutTarotFan(ids: readonly EntityId[], spec: DeckSpec): DeckCardPose[] {
  assertPermutation(ids);

  const middle = (ids.length - 1) / 2;
  const denominator = Math.max(1, middle);

  return ids.map((id, index) => {
    const offset = index - middle;
    const normalized = offset / denominator;

    return {
      id,
      index,
      z: index,
      influence: 0,
      pose: {
        position: {
          x: spec.center.x + offset * spec.overlap,
          y: spec.center.y + spec.fanDepth * Math.abs(normalized),
        },
        rotation: spec.fanAngle * normalized,
        scale: { x: 1, y: 1 },
      },
    };
  });
}

/**
 * Ordered stack topology.
 *
 * Index order remains semantic order:
 *
 *   z(i) = i
 *
 * The last card is visually highest. Lower cards may be slightly reduced in
 * scale so depth can be expressed without changing identity or order.
 */
export function layoutCardStack(ids: readonly EntityId[], spec: StackSpec): DeckCardPose[] {
  assertPermutation(ids);

  return ids.map((id, index) => {
    const depthFromTop = ids.length - 1 - index;
    const scale = Math.max(0.05, 1 - spec.scaleStep * depthFromTop);

    return {
      id,
      index,
      z: index,
      influence: 0,
      pose: {
        position: {
          x: spec.origin.x + spec.offset.x * index,
          y: spec.origin.y + spec.offset.y * index,
        },
        rotation: spec.rotationStep * index,
        scale: { x: scale, y: scale },
      },
    };
  });
}

/**
 * Linear spread topology.
 *
 * Unlike a fan, spread has zero intrinsic rotation and no depth curvature.
 * The topology is therefore a distinct solver, not fan(angle=0) by accident.
 */
export function layoutCardSpread(ids: readonly EntityId[], spec: SpreadSpec): DeckCardPose[] {
  assertPermutation(ids);

  const middle = (ids.length - 1) / 2;

  return ids.map((id, index) => ({
    id,
    index,
    z: index,
    influence: 0,
    pose: {
      position: {
        x: spec.center.x + (index - middle) * spec.gap,
        y: spec.center.y,
      },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
  }));
}

/**
 * Continuous pointer influence over a card hand.
 *
 * For each card center xi and pointer xp:
 *
 *   d = |xi - xp| / radius
 *   w = exp(-falloff * d²)
 *
 * Then:
 *
 *   scale = 1 + (hoverScale - 1) w
 *   lift  = hoverLift w
 *
 * Neighbor yield uses the signed distance to the pointer:
 *
 *   yield = sign(xi - xp) * neighborYield * w
 *
 * There is no hovered/not-hovered branch. The field is continuous, so a
 * continuous pointer path yields continuous target poses.
 */
export function applyDeckPointerField(
  poses: readonly DeckCardPose[],
  pointer: Vec2 | null,
  spec: DeckInfluenceSpec,
): DeckCardPose[] {
  if (!pointer) return poses.map((pose) => ({ ...pose, influence: 0 }));
  return applyDeckInfluenceField(poses, pointer, spec);
}

/**
 * Focus is a semantic identity relation, not a pointer state.
 *
 * The same continuous influence field can therefore be centered on a focused
 * card even when no pointer observation exists.
 */
export function applyDeckFocusField(
  poses: readonly DeckCardPose[],
  focusId: EntityId | null,
  spec: DeckInfluenceSpec,
): DeckCardPose[] {
  if (!focusId) return poses.map((pose) => ({ ...pose, influence: 0 }));

  const focused = poses.find((pose) => pose.id === focusId);
  if (!focused) throw new Error("Focused deck identity is not present: " + focusId);

  return applyDeckInfluenceField(poses, focused.pose.position, spec);
}

function applyDeckInfluenceField(
  poses: readonly DeckCardPose[],
  center: Vec2,
  spec: DeckInfluenceSpec,
): DeckCardPose[] {
  const radius = Math.max(1, spec.hoverRadius);

  return poses.map((card) => {
    const dx = card.pose.position.x - center.x;
    const dy = card.pose.position.y - center.y;
    const normalizedDistance = Math.hypot(dx, dy) / radius;
    const influence = Math.exp(
      -Math.max(0.01, spec.neighborFalloff) * normalizedDistance * normalizedDistance,
    );
    const direction = Math.sign(dx);
    const yieldX = direction * spec.neighborYield * influence;
    const lift = spec.hoverLift * influence;
    const scale = 1 + (spec.hoverScale - 1) * influence;

    return {
      ...card,
      influence,
      pose: {
        ...card.pose,
        position: {
          x: card.pose.position.x + yieldX,
          y: card.pose.position.y - lift,
        },
        scale: { x: scale, y: scale },
      },
    };
  });
}

/**
 * Reorder preview with hysteresis.
 *
 * Static slot centers define an ordered partition of the tangent axis.
 * The dragged card crosses a slot boundary only after passing hysteresis h.
 *
 * This prevents:
 *   x ≈ boundary
 * from creating order A,B,A,B... under pointer noise.
 */
export function previewDeckOrder(
  order: readonly EntityId[],
  dragId: EntityId,
  dragX: number,
  slotCenters: readonly number[],
  previousIndex: number | null,
  hysteresis = 12,
): { order: EntityId[]; index: number } {
  assertPermutation(order);

  const staticIds = order.filter((id) => id !== dragId);
  if (slotCenters.length !== staticIds.length) {
    throw new Error("Slot center cardinality must equal static card cardinality");
  }

  let index = previousIndex ?? 0;

  if (previousIndex === null) {
    index = 0;
    for (let i = 0; i < slotCenters.length; i += 1) {
      const center = slotCenters[i]!;
      if (dragX >= center) index = i + 1;
    }
  } else {
    index = clamp(index, 0, staticIds.length);

    while (index > 0 && dragX < (slotCenters[index - 1] ?? Number.NEGATIVE_INFINITY) - hysteresis) {
      index -= 1;
    }

    while (
      index < staticIds.length &&
      dragX > (slotCenters[index] ?? Number.POSITIVE_INFINITY) + hysteresis
    ) {
      index += 1;
    }
  }

  const next = [...staticIds];
  next.splice(index, 0, dragId);

  return { order: next, index };
}

export function deckOcclusionPairs(
  poses: readonly DeckCardPose[],
  extent: CardExtent,
): Array<{ occluder: EntityId; occluded: EntityId }> {
  if (
    !Number.isFinite(extent.width) ||
    !Number.isFinite(extent.height) ||
    extent.width <= 0 ||
    extent.height <= 0
  ) {
    throw new Error("Card extent must be finite and positive");
  }

  const ordered = [...poses].sort((a, b) => {
    if (a.z !== b.z) return a.z - b.z;
    return a.index - b.index;
  });

  return computeOcclusion(
    ordered.map((card) => ({
      id: card.id,
      bounds: cardPoseAabb(card.pose, extent),
      visible: true,
    })),
  );
}

/**
 * Insert/remove/transfer operate on deck order only.
 *
 * They preserve semantic card identities and leave motion to the dynamics
 * layer, which can animate the before/after pose targets without recreating
 * surviving subjects.
 */
export function insertDeckCard(
  order: readonly EntityId[],
  id: EntityId,
  index: number,
): EntityId[] {
  assertPermutation(order);
  if (order.includes(id)) {
    throw new Error("Deck already contains identity: " + id);
  }
  assertDeckIndex(index, order.length, true);

  const next = [...order];
  next.splice(index, 0, id);
  return next;
}

export function removeDeckCard(order: readonly EntityId[], id: EntityId): EntityId[] {
  assertPermutation(order);
  const index = order.indexOf(id);
  if (index < 0) throw new Error("Deck does not contain identity: " + id);

  const next = [...order];
  next.splice(index, 1);
  return next;
}

export function planDeckOrderTransition(
  before: readonly EntityId[],
  after: readonly EntityId[],
): DeckOrderTransition {
  assertPermutation(before);
  assertPermutation(after);

  const beforeIndex = new Map(before.map((id, index) => [id, index] as const));
  const afterIndex = new Map(after.map((id, index) => [id, index] as const));

  const entering = after.filter((id) => !beforeIndex.has(id));
  const leaving = before.filter((id) => !afterIndex.has(id));
  const moved = after
    .filter((id) => beforeIndex.has(id))
    .map((id) => ({
      id,
      fromIndex: beforeIndex.get(id)!,
      toIndex: afterIndex.get(id)!,
    }))
    .filter((move) => move.fromIndex !== move.toIndex);

  return { entering, leaving, moved };
}

export function transferDeckCard(
  source: readonly EntityId[],
  destination: readonly EntityId[],
  id: EntityId,
  destinationIndex: number,
): DeckTransferResult {
  assertPermutation(source);
  assertPermutation(destination);

  const sourceIndex = source.indexOf(id);
  if (sourceIndex < 0) {
    throw new Error("Source deck does not contain identity: " + id);
  }
  if (destination.includes(id)) {
    throw new Error("Destination deck already contains identity: " + id);
  }

  const nextSource = removeDeckCard(source, id);
  const nextDestination = insertDeckCard(destination, id, destinationIndex);

  return {
    source: nextSource,
    destination: nextDestination,
    subjectId: id,
    sourceIndex,
    destinationIndex,
  };
}

export function tiltFromVelocity(velocity: Vec2, spec: VelocityTiltSpec): number {
  if (
    !Number.isFinite(spec.gain) ||
    !Number.isFinite(spec.maxTilt) ||
    spec.gain < 0 ||
    spec.maxTilt < 0
  ) {
    throw new Error("Velocity tilt parameters must be finite and non-negative");
  }

  return clamp(velocity.x * spec.gain, -spec.maxTilt, spec.maxTilt);
}

/**
 * Project deck centers through a SpatialFrame without pretending Pose2 can
 * encode arbitrary affine reflection/shear.
 *
 * Position becomes World-space. Rotation and scale remain local to the deck
 * frame and must be composed by a renderer/affine adapter that understands the
 * full frame transform.
 */
export function projectDeckCentersToWorld(
  poses: readonly DeckCardPose[],
  frames: SpatialFrameSnapshot,
  frameId: FrameId,
): DeckCardPose[] {
  return poses.map((card) => ({
    ...card,
    pose: {
      ...card.pose,
      position: localToWorld(frames, frameId, card.pose.position),
    },
  }));
}

function cardPoseAabb(pose: Pose2, extent: CardExtent) {
  const width = extent.width * pose.scale.x;
  const height = extent.height * pose.scale.y;
  const cos = Math.cos(pose.rotation);
  const sin = Math.sin(pose.rotation);
  const halfX = Math.abs(cos) * (width / 2) + Math.abs(sin) * (height / 2);
  const halfY = Math.abs(sin) * (width / 2) + Math.abs(cos) * (height / 2);

  return {
    minX: pose.position.x - halfX,
    minY: pose.position.y - halfY,
    maxX: pose.position.x + halfX,
    maxY: pose.position.y + halfY,
  };
}

function assertDeckIndex(index: number, length: number, allowEnd: boolean): void {
  const max = allowEnd ? length : Math.max(0, length - 1);
  if (!Number.isInteger(index) || index < 0 || index > max) {
    throw new Error("Deck index out of bounds");
  }
}

/**
 * Convert static deck topology output into SceneEngine dynamic targets.
 *
 * Deck topology answers:
 *   "where should this identity be?"
 *
 * Scene dynamics answers:
 *   "how does the same identity continuously approach that pose?"
 *
 * This boundary is what lets insertion/removal reflow move surviving cards
 * without teleporting them or giving the deck solver its own animation clock.
 */
export function deckPoseTargets(poses: readonly DeckCardPose[], spring: SpringLaw): PoseTarget[] {
  return poses.map((card) => ({
    entityId: card.id,
    pose: {
      position: { ...card.pose.position },
      rotation: card.pose.rotation,
      scale: { ...card.pose.scale },
    },
    spring: { ...spring },
  }));
}
