import type { EntityId, Pose2, SceneState } from "../algebra/types";

/**
 * Parent relation must form a forest, never an arbitrary directed graph.
 *
 * If parent : E ⇀ E, then:
 *
 *   for every e in E, repeated parent(e) must terminate at null.
 *
 * Equivalently, the directed parent graph is acyclic.
 *
 * This matters because world transforms are function composition along the
 * unique ancestor path. A cycle would make world pose undefined.
 */
export function assertAcyclicHierarchy(state: SceneState): void {
  for (const id of state.entities.keys()) {
    const seen = new Set<EntityId>();
    let cursor: EntityId | null = id;

    while (cursor) {
      if (seen.has(cursor)) {
        throw new Error("Scene hierarchy contains cycle at " + cursor);
      }
      seen.add(cursor);
      cursor = state.entities.get(cursor)?.parent ?? null;
    }
  }
}

/**
 * Compose child pose into parent pose.
 *
 * Scale is component-wise.
 * Rotation is additive.
 * Position is first scaled in parent space, then rotated, then translated.
 *
 * Given parent transform T_p and child local point x:
 *
 *   world(x) = p + R(theta_p) S_p x
 *
 * This is enough for card groups / deck containers without importing renderer
 * matrix types into the semantic engine.
 */
export function composePose(parent: Pose2, child: Pose2): Pose2 {
  const scaledX = child.position.x * parent.scale.x;
  const scaledY = child.position.y * parent.scale.y;
  const cos = Math.cos(parent.rotation);
  const sin = Math.sin(parent.rotation);

  return {
    position: {
      x: parent.position.x + scaledX * cos - scaledY * sin,
      y: parent.position.y + scaledX * sin + scaledY * cos,
    },
    rotation: parent.rotation + child.rotation,
    scale: {
      x: parent.scale.x * child.scale.x,
      y: parent.scale.y * child.scale.y,
    },
  };
}

export function worldPose(state: SceneState, id: EntityId): Pose2 {
  assertAcyclicHierarchy(state);

  const chain = [];
  let cursor = state.entities.get(id);
  if (!cursor) throw new Error("Unknown scene entity " + id);

  while (cursor) {
    chain.push(cursor.pose);
    cursor = cursor.parent ? state.entities.get(cursor.parent) : undefined;
  }

  return chain.reverse().reduce<Pose2>((parent, child) => composePose(parent, child), {
    position: { x: 0, y: 0 },
    rotation: 0,
    scale: { x: 1, y: 1 },
  });
}
