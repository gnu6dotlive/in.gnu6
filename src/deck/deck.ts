import { clamp } from "../dynamics/math";
import type { EntityId, Pose2, Vec2 } from "../algebra/types";

export interface DeckSpec {
  center: Vec2;
  cardWidth: number;
  cardHeight: number;
  overlap: number;
  fanAngle: number;
  fanDepth: number;
  hoverRadius: number;
  hoverScale: number;
  hoverLift: number;
  neighborYield: number;
  neighborFalloff: number;
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
 * any Gnosix-specific styling.
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
  spec: DeckSpec,
): DeckCardPose[] {
  if (!pointer) return poses.map((pose) => ({ ...pose, influence: 0 }));

  const radius = Math.max(1, spec.hoverRadius);

  return poses.map((card) => {
    const dx = card.pose.position.x - pointer.x;
    const dy = card.pose.position.y - pointer.y;
    const normalizedDistance = Math.hypot(dx, dy) / radius;
    const influence = Math.exp(
      -Math.max(0.01, spec.neighborFalloff) * normalizedDistance * normalizedDistance,
    );
    const direction = Math.sign(dx || 1);
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

  if (previousIndex == null) {
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
