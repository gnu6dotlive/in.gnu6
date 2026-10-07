import type { EntityId } from "../algebra/types";

export type RegionId = string & { readonly __regionId: unique symbol };

export type RegionKind = "admissible" | "exclusion" | "container";

export interface Aabb {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export interface WorldRegion {
  id: RegionId;
  kind: RegionKind;
  bounds: Aabb;
}

export interface RectPlacementInput {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RectPlacement extends RectPlacementInput {
  bounds: Aabb;
}

export interface PlacementRequest {
  desired: RectPlacementInput;
  within: RegionId;
  avoid?: readonly RegionId[];
}

export interface PlacementResult {
  status: "admissible" | "adjusted" | "rejected";
  rect: RectPlacement | null;
  displacement: { x: number; y: number } | null;
  reasons: readonly string[];
}

export interface SpatialOccupant {
  id: EntityId;
  bounds: Aabb;
  visible: boolean;
}

export interface OcclusionPair {
  occluder: EntityId;
  occluded: EntityId;
}

export interface NearestRegionResult {
  region: WorldRegion;
  distance: number;
}

export interface EnvironmentObservation {
  revision: number;
  regions: readonly WorldRegion[];
}

export function asRegionId(value: string): RegionId {
  if (!value) throw new Error("Region identity must be non-empty");
  return value as RegionId;
}

/**
 * Axis-aligned world bounds are the first admissibility primitive.
 *
 * MQ-04 deliberately starts from AABB truth because the reference consumers
 * all need a common answer to:
 *
 *   "Can this extent exist here without leaving its allowed region or
 *    intersecting reserved space?"
 *
 * Richer arbitrary-shape constraints belong to MQ-07. The engine must not
 * pretend this AABB solver proves arbitrary polygonal admissibility.
 */
export class SpatialEnvironment {
  private readonly regions = new Map<RegionId, WorldRegion>();
  private revision = 0;

  addRegion(region: WorldRegion): WorldRegion {
    if (this.regions.has(region.id)) {
      throw new Error("Duplicate region identity: " + region.id);
    }
    assertAabb(region.bounds);

    const stored = cloneRegion(region);
    this.regions.set(region.id, stored);
    this.revision += 1;
    return cloneRegion(stored);
  }

  removeRegion(id: RegionId): WorldRegion {
    const region = this.regions.get(id);
    if (!region) throw new Error("Unknown region identity: " + id);

    this.regions.delete(id);
    this.revision += 1;
    return cloneRegion(region);
  }

  getRegion(id: RegionId): WorldRegion | null {
    const region = this.regions.get(id);
    return region ? cloneRegion(region) : null;
  }

  observation(): EnvironmentObservation {
    return {
      revision: this.revision,
      regions: [...this.regions.values()].map(cloneRegion).sort((a, b) => a.id.localeCompare(b.id)),
    };
  }

  regionsAt(point: { x: number; y: number }, kinds?: readonly RegionKind[]): WorldRegion[] {
    assertPoint(point);
    const allowed = kinds ? new Set(kinds) : null;

    return [...this.regions.values()]
      .filter((region) => !allowed || allowed.has(region.kind))
      .filter((region) => containsPoint(region.bounds, point))
      .map(cloneRegion)
      .sort((a, b) => a.id.localeCompare(b.id));
  }

  nearestRegion(
    point: { x: number; y: number },
    kinds?: readonly RegionKind[],
  ): NearestRegionResult | null {
    assertPoint(point);
    const allowed = kinds ? new Set(kinds) : null;

    const candidates = [...this.regions.values()]
      .filter((region) => !allowed || allowed.has(region.kind))
      .map((region) => ({
        region,
        distance: pointAabbDistance(point, region.bounds),
      }))
      .sort((a, b) => {
        if (a.distance !== b.distance) return a.distance - b.distance;
        return a.region.id.localeCompare(b.region.id);
      });

    const nearest = candidates[0];
    return nearest ? { region: cloneRegion(nearest.region), distance: nearest.distance } : null;
  }

  /**
   * Find the nearest admissible top-left placement.
   *
   * For axis-aligned rectangles, the nearest collision-free solution can be
   * searched over a finite candidate set composed from:
   * - the clamped desired coordinate;
   * - admissible-region edges;
   * - exclusion-region edges.
   *
   * Candidate score:
   *
   *   d² = (x - desired.x)² + (y - desired.y)²
   *
   * Ties are broken by y then x, making the solver deterministic.
   */
  place(request: PlacementRequest): PlacementResult {
    const within = this.regions.get(request.within);
    if (!within) {
      throw new Error("Unknown placement region: " + request.within);
    }
    if (within.kind === "exclusion") {
      throw new Error("Placement region cannot be an exclusion region");
    }

    assertRectInput(request.desired);

    const regionWidth = widthOf(within.bounds);
    const regionHeight = heightOf(within.bounds);
    if (request.desired.width > regionWidth || request.desired.height > regionHeight) {
      return {
        status: "rejected",
        rect: null,
        displacement: null,
        reasons: ["subject-does-not-fit-within-region"],
      };
    }

    const obstacles = this.resolveObstacles(request);
    const clampedX = clamp(
      request.desired.x,
      within.bounds.minX,
      within.bounds.maxX - request.desired.width,
    );
    const clampedY = clamp(
      request.desired.y,
      within.bounds.minY,
      within.bounds.maxY - request.desired.height,
    );

    const xCandidates = new Set<number>([
      clampedX,
      within.bounds.minX,
      within.bounds.maxX - request.desired.width,
    ]);
    const yCandidates = new Set<number>([
      clampedY,
      within.bounds.minY,
      within.bounds.maxY - request.desired.height,
    ]);

    for (const obstacle of obstacles) {
      xCandidates.add(obstacle.bounds.minX - request.desired.width);
      xCandidates.add(obstacle.bounds.maxX);
      yCandidates.add(obstacle.bounds.minY - request.desired.height);
      yCandidates.add(obstacle.bounds.maxY);
    }

    const candidates: RectPlacement[] = [];

    for (const x of xCandidates) {
      for (const y of yCandidates) {
        const rect = rectPlacement(x, y, request.desired.width, request.desired.height);

        if (!containsAabb(within.bounds, rect.bounds)) continue;
        if (obstacles.some((obstacle) => intersectsAabb(rect.bounds, obstacle.bounds))) {
          continue;
        }

        candidates.push(rect);
      }
    }

    if (candidates.length === 0) {
      return {
        status: "rejected",
        rect: null,
        displacement: null,
        reasons: ["no-admissible-placement"],
      };
    }

    candidates.sort((a, b) => {
      const scoreA = squaredDisplacement(a, request.desired);
      const scoreB = squaredDisplacement(b, request.desired);
      if (scoreA !== scoreB) return scoreA - scoreB;
      if (a.y !== b.y) return a.y - b.y;
      return a.x - b.x;
    });

    const rect = candidates[0]!;
    const displacement = {
      x: rect.x - request.desired.x,
      y: rect.y - request.desired.y,
    };
    const adjusted = displacement.x !== 0 || displacement.y !== 0;

    const reasons: string[] = [];
    const desiredRect = rectPlacement(
      request.desired.x,
      request.desired.y,
      request.desired.width,
      request.desired.height,
    );

    if (!containsAabb(within.bounds, desiredRect.bounds)) {
      reasons.push("outside-admissible-region");
    }
    if (obstacles.some((obstacle) => intersectsAabb(desiredRect.bounds, obstacle.bounds))) {
      reasons.push("intersects-exclusion-region");
    }

    return {
      status: adjusted ? "adjusted" : "admissible",
      rect,
      displacement,
      reasons,
    };
  }

  private resolveObstacles(request: PlacementRequest): WorldRegion[] {
    const ids =
      request.avoid ??
      [...this.regions.values()]
        .filter((region) => region.kind === "exclusion")
        .map((region) => region.id);

    const obstacles = ids.map((id) => {
      const region = this.regions.get(id);
      if (!region) throw new Error("Unknown exclusion region: " + id);
      if (region.kind !== "exclusion") {
        throw new Error("Avoided region is not exclusion: " + id);
      }
      return region;
    });

    return obstacles.sort((a, b) => a.id.localeCompare(b.id));
  }
}

export function containsAabb(outer: Aabb, inner: Aabb): boolean {
  assertAabb(outer);
  assertAabb(inner);

  return (
    inner.minX >= outer.minX &&
    inner.minY >= outer.minY &&
    inner.maxX <= outer.maxX &&
    inner.maxY <= outer.maxY
  );
}

export function intersectsAabb(a: Aabb, b: Aabb): boolean {
  assertAabb(a);
  assertAabb(b);

  return a.minX < b.maxX && a.maxX > b.minX && a.minY < b.maxY && a.maxY > b.minY;
}

/**
 * Euclidean gap between two AABBs.
 *
 * Overlapping or edge-touching bounds have distance zero.
 */
export function aabbDistance(a: Aabb, b: Aabb): number {
  assertAabb(a);
  assertAabb(b);

  const dx = Math.max(a.minX - b.maxX, b.minX - a.maxX, 0);
  const dy = Math.max(a.minY - b.maxY, b.minY - a.maxY, 0);
  return Math.hypot(dx, dy);
}

export function containsPoint(bounds: Aabb, point: { x: number; y: number }): boolean {
  assertAabb(bounds);
  assertPoint(point);

  return (
    point.x >= bounds.minX &&
    point.x <= bounds.maxX &&
    point.y >= bounds.minY &&
    point.y <= bounds.maxY
  );
}

export function pointAabbDistance(point: { x: number; y: number }, bounds: Aabb): number {
  assertAabb(bounds);
  assertPoint(point);

  const dx = Math.max(bounds.minX - point.x, point.x - bounds.maxX, 0);
  const dy = Math.max(bounds.minY - point.y, point.y - bounds.maxY, 0);
  return Math.hypot(dx, dy);
}

/**
 * Renderer-neutral spatial pick.
 *
 * Input order is explicit back-to-front scene order.
 * The last visible occupant containing the point is the spatially topmost hit.
 *
 * This function answers only geometric identity. MQ-08 owns the separate
 * question of whether a hit receives interaction authority.
 */
export function pickTopmost(
  backToFront: readonly SpatialOccupant[],
  point: { x: number; y: number },
): EntityId | null {
  assertPoint(point);

  for (let index = backToFront.length - 1; index >= 0; index -= 1) {
    const occupant = backToFront[index]!;
    if (!occupant.visible) continue;
    if (containsPoint(occupant.bounds, point)) return occupant.id;
  }

  return null;
}

/**
 * Occlusion is derived from explicit back-to-front ordering.
 *
 * For ordered occupants e0 ... en:
 *
 *   j > i and intersects(ej, ei) => ej occludes ei
 *
 * Invisible occupants contribute no occlusion relation.
 */
export function computeOcclusion(backToFront: readonly SpatialOccupant[]): OcclusionPair[] {
  const pairs: OcclusionPair[] = [];

  for (let backIndex = 0; backIndex < backToFront.length; backIndex += 1) {
    const back = backToFront[backIndex]!;
    if (!back.visible) continue;

    for (let frontIndex = backIndex + 1; frontIndex < backToFront.length; frontIndex += 1) {
      const front = backToFront[frontIndex]!;
      if (!front.visible) continue;
      if (!intersectsAabb(front.bounds, back.bounds)) continue;

      pairs.push({
        occluder: front.id,
        occluded: back.id,
      });
    }
  }

  return pairs;
}

function rectPlacement(x: number, y: number, width: number, height: number): RectPlacement {
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

function squaredDisplacement(rect: RectPlacement, desired: RectPlacementInput): number {
  const dx = rect.x - desired.x;
  const dy = rect.y - desired.y;
  return dx * dx + dy * dy;
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
    throw new Error("Placement rectangle must be finite with positive extent");
  }
}

function assertPoint(point: { x: number; y: number }): void {
  if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) {
    throw new Error("Point must be finite");
  }
}

function assertAabb(bounds: Aabb): void {
  const values = [bounds.minX, bounds.minY, bounds.maxX, bounds.maxY];
  if (!values.every(Number.isFinite) || bounds.maxX < bounds.minX || bounds.maxY < bounds.minY) {
    throw new Error("AABB must be finite and ordered");
  }
}

function widthOf(bounds: Aabb): number {
  return bounds.maxX - bounds.minX;
}

function heightOf(bounds: Aabb): number {
  return bounds.maxY - bounds.minY;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function cloneRegion(region: WorldRegion): WorldRegion {
  return {
    ...region,
    bounds: { ...region.bounds },
  };
}
