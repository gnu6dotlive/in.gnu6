import type { EntityId } from "../algebra/types";
import {
  type Aabb,
  type RectPlacement,
  type RectPlacementInput,
  type RegionId,
  type SpatialEnvironment,
  containsAabb,
  intersectsAabb,
} from "./environment";

export type ConstraintId = string & {
  readonly __constraintId: unique symbol;
};

interface ConstraintBase {
  id: ConstraintId;
  ownerId: string;
}

export interface PositionRangeConstraint extends ConstraintBase {
  kind: "position-range";
  minX?: number;
  maxX?: number;
  minY?: number;
  maxY?: number;
}

export interface ExtentRangeConstraint extends ConstraintBase {
  kind: "extent-range";
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
}

export interface WithinRegionConstraint extends ConstraintBase {
  kind: "within-region";
  regionId: RegionId;
}

export interface AvoidRegionConstraint extends ConstraintBase {
  kind: "avoid-region";
  regionId: RegionId;
}

export interface OverlapPolicyConstraint extends ConstraintBase {
  kind: "overlap-policy";
  peerId: EntityId;
  maxSubjectOverlapRatio: number;
}

export interface TopologyAdmissibilityConstraint extends ConstraintBase {
  kind: "topology-admissibility";
  factId: string;
  expected: boolean;
}

export type SpatialConstraint =
  | PositionRangeConstraint
  | ExtentRangeConstraint
  | WithinRegionConstraint
  | AvoidRegionConstraint
  | OverlapPolicyConstraint
  | TopologyAdmissibilityConstraint;

export type ConstraintEvaluationStatus =
  | "satisfied"
  | "adjusted"
  | "violated"
  | "conflict"
  | "blocked";

export interface ConstraintEvaluation {
  id: ConstraintId;
  ownerId: string;
  kind: SpatialConstraint["kind"];
  status: ConstraintEvaluationStatus;
  reason: string;
}

export interface ConstraintConflict {
  constraintIds: readonly ConstraintId[];
  reason: string;
}

export interface SpatialConstraintSolveInput {
  desired: RectPlacementInput;
  constraints: readonly SpatialConstraint[];
  environment: SpatialEnvironment;
  peers: ReadonlyMap<EntityId, RectPlacementInput>;
  topologyFacts: ReadonlyMap<string, boolean>;
}

export interface SpatialConstraintSolveResult {
  status: "admissible" | "adjusted" | "rejected";
  desired: RectPlacementInput;
  rect: RectPlacement | null;
  evaluations: readonly ConstraintEvaluation[];
  conflicts: readonly ConstraintConflict[];
}

/**
 * Explicit constraint solver.
 *
 * Let C be the declared finite set of constraints.
 *
 * Solver completeness law:
 *
 *   |evaluations| = |C|
 *
 * and every constraint identity appears exactly once in evaluations.
 *
 * This prevents "best effort" solving from silently discarding a constraint.
 *
 * Positional and containment constraints reduce to intersections of admissible
 * top-left intervals:
 *
 *   X = intersection_i Xi
 *   Y = intersection_i Yi
 *
 * If X or Y is empty, the contradiction is explicit and the responsible
 * constraint identities are reported.
 *
 * Exclusion regions generate candidate boundaries. The nearest candidate is
 * selected by squared displacement from the desired position with deterministic
 * y/x tie-breaking.
 *
 * Peer-overlap and topology-admissibility constraints are policy gates. They
 * evaluate the resolved geometric candidate but do not invent hidden movement.
 */
export function solveSpatialConstraints(
  input: SpatialConstraintSolveInput,
): SpatialConstraintSolveResult {
  assertRectInput(input.desired);
  assertConstraintSet(input.constraints);

  const conflicts = detectStaticConflicts(input.constraints, input.environment, input.desired);

  if (conflicts.length > 0) {
    return rejectedByConflicts(input, conflicts);
  }

  const extent = resolveExtent(input.desired, input.constraints);
  const desiredAfterExtent: RectPlacementInput = {
    x: input.desired.x,
    y: input.desired.y,
    width: extent.width,
    height: extent.height,
  };

  const admissible = resolvePositionDomain(
    desiredAfterExtent,
    input.constraints,
    input.environment,
  );

  if (admissible.conflicts.length > 0) {
    return rejectedByConflicts(input, admissible.conflicts);
  }

  const obstacles = resolveAvoidRegions(input.constraints, input.environment);

  const candidate = findNearestCandidate(
    desiredAfterExtent,
    admissible,
    obstacles.map((item) => item.bounds),
  );

  if (!candidate) {
    const evaluations = input.constraints.map((constraint) => ({
      id: constraint.id,
      ownerId: constraint.ownerId,
      kind: constraint.kind,
      status: "violated" as const,
      reason: "no-admissible-candidate",
    }));

    return {
      status: "rejected",
      desired: { ...input.desired },
      rect: null,
      evaluations,
      conflicts: [],
    };
  }

  const evaluations = input.constraints.map((constraint) =>
    evaluateConstraint(constraint, input, desiredAfterExtent, candidate),
  );

  const violated = evaluations.some((evaluation) => evaluation.status === "violated");

  if (violated) {
    return {
      status: "rejected",
      desired: { ...input.desired },
      rect: null,
      evaluations,
      conflicts: [],
    };
  }

  const adjusted = evaluations.some((evaluation) => evaluation.status === "adjusted");

  return {
    status: adjusted ? "adjusted" : "admissible",
    desired: { ...input.desired },
    rect: candidate,
    evaluations,
    conflicts: [],
  };
}

export function asConstraintId(value: string): ConstraintId {
  if (!value) throw new Error("Constraint identity must be non-empty");
  return value as ConstraintId;
}

interface PositionDomain {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
  conflicts: ConstraintConflict[];
}

function detectStaticConflicts(
  constraints: readonly SpatialConstraint[],
  environment: SpatialEnvironment,
  desired: RectPlacementInput,
): ConstraintConflict[] {
  const conflicts: ConstraintConflict[] = [];

  const extentConstraints = constraints.filter(
    (constraint): constraint is ExtentRangeConstraint => constraint.kind === "extent-range",
  );

  const minWidth = maxDefined(
    extentConstraints.map((constraint) => constraint.minWidth),
    0,
  );
  const maxWidth = minDefined(
    extentConstraints.map((constraint) => constraint.maxWidth),
    Number.POSITIVE_INFINITY,
  );
  const minHeight = maxDefined(
    extentConstraints.map((constraint) => constraint.minHeight),
    0,
  );
  const maxHeight = minDefined(
    extentConstraints.map((constraint) => constraint.maxHeight),
    Number.POSITIVE_INFINITY,
  );

  if (minWidth > maxWidth) {
    conflicts.push({
      constraintIds: extentConstraints
        .filter(
          (constraint) => constraint.minWidth !== undefined || constraint.maxWidth !== undefined,
        )
        .map((constraint) => constraint.id),
      reason: "incompatible-width-range",
    });
  }

  if (minHeight > maxHeight) {
    conflicts.push({
      constraintIds: extentConstraints
        .filter(
          (constraint) => constraint.minHeight !== undefined || constraint.maxHeight !== undefined,
        )
        .map((constraint) => constraint.id),
      reason: "incompatible-height-range",
    });
  }

  // Validate referenced regions and policy values before any solving.
  for (const constraint of constraints) {
    switch (constraint.kind) {
      case "within-region": {
        const region = inputRegion(environment, constraint.regionId);
        if (region.kind === "exclusion") {
          throw new Error("within-region cannot reference exclusion: " + constraint.regionId);
        }
        break;
      }
      case "avoid-region": {
        const region = inputRegion(environment, constraint.regionId);
        if (region.kind !== "exclusion") {
          throw new Error("avoid-region must reference exclusion: " + constraint.regionId);
        }
        break;
      }
      case "overlap-policy":
        if (
          !Number.isFinite(constraint.maxSubjectOverlapRatio) ||
          constraint.maxSubjectOverlapRatio < 0 ||
          constraint.maxSubjectOverlapRatio > 1
        ) {
          throw new Error("Overlap ratio must be finite in [0,1]");
        }
        break;
      case "topology-admissibility":
        if (!constraint.factId) {
          throw new Error("Topology admissibility fact identity must be non-empty");
        }
        break;
      case "position-range":
      case "extent-range":
        break;
    }
  }

  // Position contradictions must consider the final clamped extent because
  // within-region constraints restrict top-left position by subject size.
  if (conflicts.length === 0) {
    const extent = resolveExtent(desired, constraints);
    conflicts.push(
      ...resolvePositionDomain(
        { ...desired, width: extent.width, height: extent.height },
        constraints,
        environment,
      ).conflicts,
    );
  }

  return uniqueConflicts(conflicts);
}

function resolveExtent(
  desired: RectPlacementInput,
  constraints: readonly SpatialConstraint[],
): { width: number; height: number } {
  const extents = constraints.filter(
    (constraint): constraint is ExtentRangeConstraint => constraint.kind === "extent-range",
  );

  const minWidth = maxDefined(
    extents.map((constraint) => constraint.minWidth),
    1e-9,
  );
  const maxWidth = minDefined(
    extents.map((constraint) => constraint.maxWidth),
    Number.POSITIVE_INFINITY,
  );
  const minHeight = maxDefined(
    extents.map((constraint) => constraint.minHeight),
    1e-9,
  );
  const maxHeight = minDefined(
    extents.map((constraint) => constraint.maxHeight),
    Number.POSITIVE_INFINITY,
  );

  return {
    width: clamp(desired.width, minWidth, maxWidth),
    height: clamp(desired.height, minHeight, maxHeight),
  };
}

function resolvePositionDomain(
  subject: RectPlacementInput,
  constraints: readonly SpatialConstraint[],
  environment: SpatialEnvironment,
): PositionDomain {
  let minX = Number.NEGATIVE_INFINITY;
  let maxX = Number.POSITIVE_INFINITY;
  let minY = Number.NEGATIVE_INFINITY;
  let maxY = Number.POSITIVE_INFINITY;

  const xConstraints: ConstraintId[] = [];
  const yConstraints: ConstraintId[] = [];

  for (const constraint of constraints) {
    if (constraint.kind === "position-range") {
      if (constraint.minX !== undefined || constraint.maxX !== undefined) {
        xConstraints.push(constraint.id);
      }
      if (constraint.minY !== undefined || constraint.maxY !== undefined) {
        yConstraints.push(constraint.id);
      }

      if (constraint.minX !== undefined) minX = Math.max(minX, constraint.minX);
      if (constraint.maxX !== undefined) maxX = Math.min(maxX, constraint.maxX);
      if (constraint.minY !== undefined) minY = Math.max(minY, constraint.minY);
      if (constraint.maxY !== undefined) maxY = Math.min(maxY, constraint.maxY);
    }

    if (constraint.kind === "within-region") {
      const region = inputRegion(environment, constraint.regionId);
      xConstraints.push(constraint.id);
      yConstraints.push(constraint.id);
      minX = Math.max(minX, region.bounds.minX);
      maxX = Math.min(maxX, region.bounds.maxX - subject.width);
      minY = Math.max(minY, region.bounds.minY);
      maxY = Math.min(maxY, region.bounds.maxY - subject.height);
    }
  }

  const conflicts: ConstraintConflict[] = [];

  if (minX > maxX) {
    conflicts.push({
      constraintIds: uniqueIds(xConstraints),
      reason: "incompatible-position-x-range",
    });
  }

  if (minY > maxY) {
    conflicts.push({
      constraintIds: uniqueIds(yConstraints),
      reason: "incompatible-position-y-range",
    });
  }

  return { minX, maxX, minY, maxY, conflicts };
}

function resolveAvoidRegions(
  constraints: readonly SpatialConstraint[],
  environment: SpatialEnvironment,
) {
  return constraints
    .filter((constraint): constraint is AvoidRegionConstraint => constraint.kind === "avoid-region")
    .map((constraint) => inputRegion(environment, constraint.regionId))
    .sort((a, b) => a.id.localeCompare(b.id));
}

function findNearestCandidate(
  desired: RectPlacementInput,
  domain: PositionDomain,
  obstacles: readonly Aabb[],
): RectPlacement | null {
  const preferredX = clampFinite(desired.x, domain.minX, domain.maxX);
  const preferredY = clampFinite(desired.y, domain.minY, domain.maxY);

  const xCandidates = new Set<number>([preferredX]);
  const yCandidates = new Set<number>([preferredY]);

  addFinite(xCandidates, domain.minX);
  addFinite(xCandidates, domain.maxX);
  addFinite(yCandidates, domain.minY);
  addFinite(yCandidates, domain.maxY);

  for (const obstacle of obstacles) {
    addFinite(xCandidates, obstacle.minX - desired.width);
    addFinite(xCandidates, obstacle.maxX);
    addFinite(yCandidates, obstacle.minY - desired.height);
    addFinite(yCandidates, obstacle.maxY);
  }

  const candidates: RectPlacement[] = [];

  for (const x of xCandidates) {
    for (const y of yCandidates) {
      if (x < domain.minX || x > domain.maxX) continue;
      if (y < domain.minY || y > domain.maxY) continue;

      const rect = placement(x, y, desired.width, desired.height);

      if (obstacles.some((obstacle) => intersectsAabb(rect.bounds, obstacle))) {
        continue;
      }

      candidates.push(rect);
    }
  }

  candidates.sort((a, b) => {
    const da = squaredDistance(a.x, a.y, desired.x, desired.y);
    const db = squaredDistance(b.x, b.y, desired.x, desired.y);
    if (da !== db) return da - db;
    if (a.y !== b.y) return a.y - b.y;
    return a.x - b.x;
  });

  return candidates[0] ?? null;
}

function evaluateConstraint(
  constraint: SpatialConstraint,
  input: SpatialConstraintSolveInput,
  preEnvironment: RectPlacementInput,
  candidate: RectPlacement,
): ConstraintEvaluation {
  switch (constraint.kind) {
    case "position-range": {
      const desiredSatisfied = positionRangeSatisfied(input.desired, constraint);
      const finalSatisfied = positionRangeSatisfied(candidate, constraint);

      return makeEvaluation(
        constraint,
        finalSatisfied ? (desiredSatisfied ? "satisfied" : "adjusted") : "violated",
        finalSatisfied
          ? desiredSatisfied
            ? "position-within-range"
            : "position-clamped"
          : "position-range-violated",
      );
    }

    case "extent-range": {
      const desiredSatisfied = extentRangeSatisfied(input.desired, constraint);
      const finalSatisfied = extentRangeSatisfied(candidate, constraint);

      return makeEvaluation(
        constraint,
        finalSatisfied ? (desiredSatisfied ? "satisfied" : "adjusted") : "violated",
        finalSatisfied
          ? desiredSatisfied
            ? "extent-within-range"
            : "extent-clamped"
          : "extent-range-violated",
      );
    }

    case "within-region": {
      const region = inputRegion(input.environment, constraint.regionId);
      const pre = placement(
        preEnvironment.x,
        preEnvironment.y,
        preEnvironment.width,
        preEnvironment.height,
      );
      const before = containsAabb(region.bounds, pre.bounds);
      const after = containsAabb(region.bounds, candidate.bounds);

      return makeEvaluation(
        constraint,
        after ? (before ? "satisfied" : "adjusted") : "violated",
        after ? (before ? "inside-region" : "moved-inside-region") : "outside-region",
      );
    }

    case "avoid-region": {
      const region = inputRegion(input.environment, constraint.regionId);
      const pre = placement(
        preEnvironment.x,
        preEnvironment.y,
        preEnvironment.width,
        preEnvironment.height,
      );
      const before = intersectsAabb(pre.bounds, region.bounds);
      const after = intersectsAabb(candidate.bounds, region.bounds);

      return makeEvaluation(
        constraint,
        after ? "violated" : before ? "adjusted" : "satisfied",
        after
          ? "intersects-exclusion-region"
          : before
            ? "moved-outside-exclusion"
            : "outside-exclusion-region",
      );
    }

    case "overlap-policy": {
      const peer = input.peers.get(constraint.peerId);
      if (!peer) {
        return makeEvaluation(constraint, "violated", "peer-unavailable");
      }

      const ratio = subjectOverlapRatio(candidate, peer);
      const satisfied = ratio <= constraint.maxSubjectOverlapRatio + 1e-12;

      return makeEvaluation(
        constraint,
        satisfied ? "satisfied" : "violated",
        satisfied ? "peer-overlap-within-policy" : "peer-overlap-ratio-exceeded",
      );
    }

    case "topology-admissibility": {
      const observed = input.topologyFacts.get(constraint.factId);
      if (observed === undefined) {
        return makeEvaluation(constraint, "violated", "topology-fact-unavailable");
      }

      const satisfied = observed === constraint.expected;
      return makeEvaluation(
        constraint,
        satisfied ? "satisfied" : "violated",
        satisfied ? "topology-admissibility-matched" : "topology-admissibility-mismatch",
      );
    }
  }
}

function rejectedByConflicts(
  input: SpatialConstraintSolveInput,
  conflicts: readonly ConstraintConflict[],
): SpatialConstraintSolveResult {
  const conflictIds = new Set(conflicts.flatMap((conflict) => conflict.constraintIds));

  const evaluations = input.constraints.map((constraint) => ({
    id: constraint.id,
    ownerId: constraint.ownerId,
    kind: constraint.kind,
    status: conflictIds.has(constraint.id) ? ("conflict" as const) : ("blocked" as const),
    reason: conflictIds.has(constraint.id)
      ? conflicts.find((conflict) => conflict.constraintIds.includes(constraint.id))!.reason
      : "blocked-by-constraint-conflict",
  }));

  return {
    status: "rejected",
    desired: { ...input.desired },
    rect: null,
    evaluations,
    conflicts,
  };
}

function makeEvaluation(
  constraint: SpatialConstraint,
  status: ConstraintEvaluationStatus,
  reason: string,
): ConstraintEvaluation {
  return {
    id: constraint.id,
    ownerId: constraint.ownerId,
    kind: constraint.kind,
    status,
    reason,
  };
}

function assertConstraintSet(constraints: readonly SpatialConstraint[]): void {
  const ids = new Set<ConstraintId>();

  for (const constraint of constraints) {
    if (ids.has(constraint.id)) {
      throw new Error("Spatial constraint identities must be injective");
    }
    ids.add(constraint.id);

    if (!constraint.ownerId) {
      throw new Error("Spatial constraint owner must be non-empty");
    }

    switch (constraint.kind) {
      case "position-range":
        assertOptionalFinite(constraint.minX, constraint.maxX, constraint.minY, constraint.maxY);
        break;
      case "extent-range":
        assertOptionalFinite(
          constraint.minWidth,
          constraint.maxWidth,
          constraint.minHeight,
          constraint.maxHeight,
        );
        for (const value of [
          constraint.minWidth,
          constraint.maxWidth,
          constraint.minHeight,
          constraint.maxHeight,
        ]) {
          if (value !== undefined && value <= 0) {
            throw new Error("Extent constraints must be positive");
          }
        }
        break;
      case "within-region":
      case "avoid-region":
        break;
      case "overlap-policy":
        break;
      case "topology-admissibility":
        break;
    }
  }
}

function positionRangeSatisfied(
  rect: Pick<RectPlacementInput, "x" | "y">,
  constraint: PositionRangeConstraint,
): boolean {
  return (
    (constraint.minX === undefined || rect.x >= constraint.minX) &&
    (constraint.maxX === undefined || rect.x <= constraint.maxX) &&
    (constraint.minY === undefined || rect.y >= constraint.minY) &&
    (constraint.maxY === undefined || rect.y <= constraint.maxY)
  );
}

function extentRangeSatisfied(
  rect: Pick<RectPlacementInput, "width" | "height">,
  constraint: ExtentRangeConstraint,
): boolean {
  return (
    (constraint.minWidth === undefined || rect.width >= constraint.minWidth) &&
    (constraint.maxWidth === undefined || rect.width <= constraint.maxWidth) &&
    (constraint.minHeight === undefined || rect.height >= constraint.minHeight) &&
    (constraint.maxHeight === undefined || rect.height <= constraint.maxHeight)
  );
}

function subjectOverlapRatio(subject: RectPlacement, peer: RectPlacementInput): number {
  const peerRect = placement(peer.x, peer.y, peer.width, peer.height);

  const width = Math.max(
    0,
    Math.min(subject.bounds.maxX, peerRect.bounds.maxX) -
      Math.max(subject.bounds.minX, peerRect.bounds.minX),
  );
  const height = Math.max(
    0,
    Math.min(subject.bounds.maxY, peerRect.bounds.maxY) -
      Math.max(subject.bounds.minY, peerRect.bounds.minY),
  );

  const subjectArea = subject.width * subject.height;
  return subjectArea <= 0 ? 0 : (width * height) / subjectArea;
}

function placement(x: number, y: number, width: number, height: number): RectPlacement {
  return {
    x,
    y,
    width,
    height,
    bounds: {
      minX: x,
      minY: y,
      maxX: x + width,
      maxY: y + height,
    },
  };
}

function inputRegion(environment: SpatialEnvironment, id: RegionId) {
  const region = environment.getRegion(id);
  if (!region) throw new Error("Unknown constraint region: " + id);
  return region;
}

function assertRectInput(rect: RectPlacementInput): void {
  if (
    !Number.isFinite(rect.x) ||
    !Number.isFinite(rect.y) ||
    !Number.isFinite(rect.width) ||
    !Number.isFinite(rect.height) ||
    rect.width <= 0 ||
    rect.height <= 0
  ) {
    throw new Error("Constraint subject must be finite with positive extent");
  }
}

function assertOptionalFinite(...values: Array<number | undefined>): void {
  for (const value of values) {
    if (value !== undefined && !Number.isFinite(value)) {
      throw new Error("Constraint bounds must be finite when present");
    }
  }
}

function maxDefined(values: readonly (number | undefined)[], fallback: number): number {
  const defined = values.filter((value): value is number => value !== undefined);
  return defined.length > 0 ? Math.max(...defined) : fallback;
}

function minDefined(values: readonly (number | undefined)[], fallback: number): number {
  const defined = values.filter((value): value is number => value !== undefined);
  return defined.length > 0 ? Math.min(...defined) : fallback;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function clampFinite(value: number, min: number, max: number): number {
  let next = value;
  if (Number.isFinite(min)) next = Math.max(next, min);
  if (Number.isFinite(max)) next = Math.min(next, max);
  return next;
}

function addFinite(target: Set<number>, value: number): void {
  if (Number.isFinite(value)) target.add(value);
}

function squaredDistance(ax: number, ay: number, bx: number, by: number): number {
  const dx = ax - bx;
  const dy = ay - by;
  return dx * dx + dy * dy;
}

function uniqueIds(ids: readonly ConstraintId[]): ConstraintId[] {
  return [...new Set(ids)];
}

function uniqueConflicts(conflicts: readonly ConstraintConflict[]): ConstraintConflict[] {
  const seen = new Set<string>();
  const result: ConstraintConflict[] = [];

  for (const conflict of conflicts) {
    const key = conflict.reason + "\u0000" + conflict.constraintIds.join("\u0000");
    if (seen.has(key)) continue;
    seen.add(key);
    result.push(conflict);
  }

  return result;
}
