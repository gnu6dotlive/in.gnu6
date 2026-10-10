import { validPose } from "../dynamics/math";
import type { EntityId, SceneState } from "./types";

export interface LawViolation {
  law: string;
  detail: string;
}

export function asEntityId(value: string): EntityId {
  if (!value) throw new Error("Entity identity must be non-empty");
  return value as EntityId;
}

/**
 * Valid scene states form a subset Ω of all structurally representable states:
 *
 *   Ω = { s | L1(s) and L2(s) and ... and Ln(s) }
 *
 * Mutation is allowed only if the resulting state remains in Ω.
 * There is intentionally no ALLOW_PARTIAL branch: an invalid state is not a
 * weaker success; it is outside the scene algebra.
 */
export function inspectSceneLaws(state: SceneState): LawViolation[] {
  const violations: LawViolation[] = [];
  const entityIds = new Set(state.entities.keys());
  const orderedIds = new Set(state.order);

  if (orderedIds.size !== state.order.length) {
    violations.push({
      law: "ORDER_IS_INJECTIVE",
      detail: "scene order contains duplicate identity",
    });
  }

  const sameCardinality = orderedIds.size === entityIds.size;
  const sameMembers = [...entityIds].every((id) => orderedIds.has(id));
  if (!sameCardinality || !sameMembers) {
    violations.push({
      law: "ORDER_IS_PERMUTATION_OF_ENTITIES",
      detail: "scene order and entity set differ",
    });
  }

  for (const entity of state.entities.values()) {
    if (!validPose(entity.pose)) {
      violations.push({
        law: "POSE_IS_FINITE_WITH_POSITIVE_SCALE",
        detail: "invalid pose for " + entity.id,
      });
    }
    if (entity.parent && !state.entities.has(entity.parent)) {
      violations.push({
        law: "PARENT_BELONGS_TO_ENTITY_SET",
        detail: entity.id + " refers to missing parent " + entity.parent,
      });
    }
  }

  /**
   * Parent is a partial function parent : E ⇀ E.
   *
   * Admissibility requires every repeated parent walk to terminate at null:
   *
   *   ∀e∈E, ∃n≥0 : parentⁿ(e) = null
   *
   * A cycle would make composed world pose undefined, so it is rejected at
   * scene admission rather than discovered later by a renderer/query.
   */
  for (const id of entityIds) {
    const seen = new Set<EntityId>();
    let cursor: EntityId | null = id;

    while (cursor) {
      if (seen.has(cursor)) {
        violations.push({
          law: "HIERARCHY_IS_ACYCLIC",
          detail: "parent cycle reaches " + cursor + " from " + id,
        });
        break;
      }

      seen.add(cursor);
      cursor = state.entities.get(cursor)?.parent ?? null;
    }
  }

  /**
   * R is a set of semantic triples:
   *
   *   R ⊆ K × E × E
   *
   * Therefore multiplicity of the same triple is invalid.
   */
  const relationKeys = new Set<string>();
  for (const relation of state.relations) {
    if (!validRelationKind(relation.kind)) {
      violations.push({
        law: "RELATION_KIND_IS_EXPLICIT",
        detail:
          "relation kind must be attached or ext:<name>, received " + JSON.stringify(relation.kind),
      });
    }

    const relationKey = relation.kind + "\u0000" + relation.from + "\u0000" + relation.to;

    if (relationKeys.has(relationKey)) {
      violations.push({
        law: "RELATIONS_ARE_SET",
        detail: "duplicate relation " + relation.kind + ": " + relation.from + " -> " + relation.to,
      });
    }
    relationKeys.add(relationKey);

    if (!state.entities.has(relation.from) || !state.entities.has(relation.to)) {
      violations.push({
        law: "RELATION_ENDPOINTS_EXIST",
        detail: relation.kind + ": " + relation.from + " -> " + relation.to,
      });
    }
  }

  for (const [id, target] of state.targets) {
    if (id !== target.entityId || !state.entities.has(id)) {
      violations.push({
        law: "TARGET_HAS_EXACT_ENTITY_OWNER",
        detail: "inconsistent target owner " + id,
      });
    }
  }

  return violations;
}

function validRelationKind(kind: string): boolean {
  if (kind === "attached") return true;
  if (!kind.startsWith("ext:")) return false;
  return kind.length > "ext:".length;
}

export function assertSceneLaws(state: SceneState): void {
  const violations = inspectSceneLaws(state);
  if (violations.length === 0) return;

  throw new Error(
    violations.map((violation) => violation.law + ": " + violation.detail).join("\n"),
  );
}
