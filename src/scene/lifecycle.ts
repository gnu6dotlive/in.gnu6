import { assertSceneLaws } from "../algebra/laws";
import type { EntityId, SceneState } from "../algebra/types";

export interface SceneLifecycleDelta {
  born: readonly EntityId[];
  persisted: readonly EntityId[];
  terminated: readonly EntityId[];
}

/**
 * Scene lifecycle is set arithmetic over semantic identity.
 *
 * Given valid states S₀ and S₁ with identity sets E₀ and E₁:
 *
 *   born       = E₁ \ E₀
 *   persisted  = E₀ ∩ E₁
 *   terminated = E₀ \ E₁
 *
 * These three sets are pairwise disjoint and their union is E₀ ∪ E₁.
 *
 * Authority transfer, topology transfer, shape mutation and pose evolution do
 * not imply semantic birth/death when EntityId persists. This distinction is
 * what prevents an implementation handoff from masquerading as lifecycle.
 *
 * Ordering:
 * - born/persisted follow the destination scene traversal order;
 * - terminated follows the source scene traversal order.
 */
export function diffSceneLifecycle(before: SceneState, after: SceneState): SceneLifecycleDelta {
  assertSceneLaws(before);
  assertSceneLaws(after);

  const beforeIds = new Set(before.entities.keys());
  const afterIds = new Set(after.entities.keys());

  return {
    born: after.order.filter((id) => !beforeIds.has(id)),
    persisted: after.order.filter((id) => beforeIds.has(id)),
    terminated: before.order.filter((id) => !afterIds.has(id)),
  };
}
