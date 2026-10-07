import type {
  EntityId,
  SceneRelation,
  SceneRelationKind,
  SceneRole,
  SceneState,
} from "../algebra/types";

export type SceneEntityQuery =
  | { kind: "all" }
  | {
      kind: "ids";
      ids: readonly EntityId[];
    }
  | {
      kind: "role";
      roles: readonly SceneRole[];
    }
  | {
      kind: "visible";
      value: boolean;
    }
  | {
      kind: "parent";
      parent: EntityId | null;
    }
  | {
      kind: "targeted";
      value: boolean;
    }
  | {
      kind: "related";
      relationKinds?: readonly SceneRelationKind[];
      direction: "outgoing" | "incoming" | "either";
      counterpart?: EntityId;
    }
  | {
      kind: "and";
      terms: readonly SceneEntityQuery[];
    }
  | {
      kind: "or";
      terms: readonly SceneEntityQuery[];
    }
  | {
      kind: "not";
      term: SceneEntityQuery;
    };

export interface SceneRelationQuery {
  kinds?: readonly SceneRelationKind[];
  from?: EntityId;
  to?: EntityId;
}

/**
 * Explicit finite-set query algebra.
 *
 * A query denotes a subset of the finite entity set E:
 *
 *   [[all]]          = E
 *   [[ids(S)]]       = E ∩ S
 *   [[and(q₁…qₙ)]]   = ⋂ᵢ [[qᵢ]]
 *   [[or(q₁…qₙ)]]    = ⋃ᵢ [[qᵢ]]
 *   [[not(q)]]       = E \ [[q]]
 *
 * Attribute and relation terms define additional subsets of E.
 *
 * The AST is data, not an arbitrary callback. It is therefore:
 * - serializable;
 * - inspectable;
 * - replayable;
 * - independent from renderer/provider code.
 *
 * Result order is always the scene traversal permutation π(E), never Map
 * insertion order and never lexical sort. Query changes membership, not
 * traversal semantics.
 */
export function queryEntityIds(state: SceneState, query: SceneEntityQuery): EntityId[] {
  const selected = evaluateEntityQuery(state, query);
  return state.order.filter((id) => selected.has(id));
}

/**
 * Query the relation set R deterministically.
 *
 * Relations are returned in a canonical lexical order over:
 *
 *   (kind, from, to)
 *
 * Relation-array insertion order is not treated as semantic scene traversal.
 */
export function queryRelations(state: SceneState, query: SceneRelationQuery = {}): SceneRelation[] {
  const kinds = query.kinds ? new Set(query.kinds) : null;

  return state.relations
    .filter(
      (relation) =>
        (!kinds || kinds.has(relation.kind)) &&
        (query.from === undefined || relation.from === query.from) &&
        (query.to === undefined || relation.to === query.to),
    )
    .map((relation) => ({ ...relation }))
    .sort(compareRelation);
}

function evaluateEntityQuery(state: SceneState, query: SceneEntityQuery): Set<EntityId> {
  const universe = new Set(state.entities.keys());

  switch (query.kind) {
    case "all":
      return universe;

    case "ids":
      return new Set(query.ids.filter((id) => state.entities.has(id)));

    case "role": {
      const roles = new Set(query.roles);
      return new Set(
        [...state.entities.values()]
          .filter((entity) => roles.has(entity.role))
          .map((entity) => entity.id),
      );
    }

    case "visible":
      return new Set(
        [...state.entities.values()]
          .filter((entity) => entity.visible === query.value)
          .map((entity) => entity.id),
      );

    case "parent":
      return new Set(
        [...state.entities.values()]
          .filter((entity) => entity.parent === query.parent)
          .map((entity) => entity.id),
      );

    case "targeted":
      return new Set(
        [...state.entities.keys()].filter((id) => state.targets.has(id) === query.value),
      );

    case "related":
      return relationIncidenceSet(state, query);

    case "and": {
      if (query.terms.length === 0) {
        return universe;
      }

      let result = evaluateEntityQuery(state, query.terms[0]!);

      for (const term of query.terms.slice(1)) {
        result = intersection(result, evaluateEntityQuery(state, term));
      }

      return result;
    }

    case "or": {
      let result = new Set<EntityId>();

      for (const term of query.terms) {
        result = union(result, evaluateEntityQuery(state, term));
      }

      return result;
    }

    case "not":
      return difference(universe, evaluateEntityQuery(state, query.term));
  }
}

function relationIncidenceSet(
  state: SceneState,
  query: Extract<SceneEntityQuery, { kind: "related" }>,
): Set<EntityId> {
  const relationKinds = query.relationKinds ? new Set(query.relationKinds) : null;
  const result = new Set<EntityId>();

  for (const relation of state.relations) {
    if (relationKinds && !relationKinds.has(relation.kind)) {
      continue;
    }

    switch (query.direction) {
      case "outgoing":
        if (query.counterpart === undefined || relation.to === query.counterpart) {
          result.add(relation.from);
        }
        break;

      case "incoming":
        if (query.counterpart === undefined || relation.from === query.counterpart) {
          result.add(relation.to);
        }
        break;

      case "either":
        if (query.counterpart === undefined) {
          result.add(relation.from);
          result.add(relation.to);
        } else if (relation.from === query.counterpart) {
          result.add(relation.to);
        } else if (relation.to === query.counterpart) {
          result.add(relation.from);
        }
        break;
    }
  }

  return result;
}

function intersection(a: ReadonlySet<EntityId>, b: ReadonlySet<EntityId>): Set<EntityId> {
  return new Set([...a].filter((id) => b.has(id)));
}

function union(a: ReadonlySet<EntityId>, b: ReadonlySet<EntityId>): Set<EntityId> {
  return new Set([...a, ...b]);
}

function difference(a: ReadonlySet<EntityId>, b: ReadonlySet<EntityId>): Set<EntityId> {
  return new Set([...a].filter((id) => !b.has(id)));
}

function compareRelation(a: SceneRelation, b: SceneRelation): number {
  const kindOrder = a.kind.localeCompare(b.kind);
  if (kindOrder !== 0) return kindOrder;

  const fromOrder = a.from.localeCompare(b.from);
  return fromOrder !== 0 ? fromOrder : a.to.localeCompare(b.to);
}
