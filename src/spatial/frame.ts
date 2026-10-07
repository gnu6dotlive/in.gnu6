import type { Vec2 } from "../algebra/types";
import { EPSILON } from "../dynamics/math";

export type FrameId = string & { readonly __frameId: unique symbol };

export interface SpatialFrame {
  id: FrameId;
  parent: FrameId | null;
  origin: Vec2;
  basisU: Vec2;
  basisV: Vec2;
}

export type SpatialFrameSnapshot = ReadonlyMap<FrameId, SpatialFrame>;

export function asFrameId(value: string): FrameId {
  if (!value) throw new Error("Frame identity must be non-empty");
  return value as FrameId;
}

/**
 * A SpatialFrame is an affine coordinate chart over the same World.
 *
 * For local coordinates p = (u,v):
 *
 *   worldLike(p) = O + u·basisU + v·basisV
 *
 * The basis matrix is:
 *
 *        [ Ux  Vx ]
 *   B =  [ Uy  Vy ]
 *
 * and is admissible iff:
 *
 *   det(B) = Ux·Vy - Uy·Vx != 0
 *
 * Non-zero determinant is the exact condition required for an inverse map.
 * We intentionally do not force frames to be axis-aligned or orthonormal.
 */
export class SpatialFrameGraph {
  private readonly frames = new Map<FrameId, SpatialFrame>();

  snapshot(): SpatialFrameSnapshot {
    return new Map(this.frames);
  }

  add(frame: SpatialFrame): SpatialFrame {
    if (this.frames.has(frame.id)) {
      throw new Error("Duplicate spatial frame identity: " + frame.id);
    }
    assertInvertible(frame);
    if (frame.parent && !this.frames.has(frame.parent)) {
      throw new Error("Missing parent spatial frame: " + frame.parent);
    }

    this.frames.set(frame.id, cloneFrame(frame));
    this.assertAcyclic();
    return cloneFrame(frame);
  }

  reparent(id: FrameId, parent: FrameId | null): SpatialFrame {
    const current = this.frames.get(id);
    if (!current) throw new Error("Unknown spatial frame: " + id);
    if (parent && !this.frames.has(parent)) {
      throw new Error("Missing parent spatial frame: " + parent);
    }

    const previous = current.parent;
    const next = { ...current, parent };
    this.frames.set(id, next);

    try {
      this.assertAcyclic();
    } catch (error) {
      this.frames.set(id, { ...current, parent: previous });
      throw error;
    }

    return cloneFrame(next);
  }

  private assertAcyclic(): void {
    for (const id of this.frames.keys()) {
      const seen = new Set<FrameId>();
      let cursor: FrameId | null = id;

      while (cursor) {
        if (seen.has(cursor)) {
          throw new Error("Spatial frame graph contains cycle at " + cursor);
        }
        seen.add(cursor);
        cursor = this.frames.get(cursor)?.parent ?? null;
      }
    }
  }
}

export function localToWorld(frames: SpatialFrameSnapshot, frameId: FrameId, point: Vec2): Vec2 {
  const frame = requireFrame(frames, frameId);
  const parentPoint = {
    x: frame.origin.x + frame.basisU.x * point.x + frame.basisV.x * point.y,
    y: frame.origin.y + frame.basisU.y * point.x + frame.basisV.y * point.y,
  };

  return frame.parent ? localToWorld(frames, frame.parent, parentPoint) : parentPoint;
}

export function worldToLocal(frames: SpatialFrameSnapshot, frameId: FrameId, point: Vec2): Vec2 {
  const frame = requireFrame(frames, frameId);
  const parentPoint = frame.parent ? worldToLocal(frames, frame.parent, point) : point;

  const dx = parentPoint.x - frame.origin.x;
  const dy = parentPoint.y - frame.origin.y;
  const determinant = frame.basisU.x * frame.basisV.y - frame.basisU.y * frame.basisV.x;

  return {
    x: (dx * frame.basisV.y - dy * frame.basisV.x) / determinant,
    y: (frame.basisU.x * dy - frame.basisU.y * dx) / determinant,
  };
}

function requireFrame(frames: SpatialFrameSnapshot, id: FrameId): SpatialFrame {
  const frame = frames.get(id);
  if (!frame) throw new Error("Unknown spatial frame: " + id);
  return frame;
}

function assertInvertible(frame: SpatialFrame): void {
  const determinant = frame.basisU.x * frame.basisV.y - frame.basisU.y * frame.basisV.x;
  if (!Number.isFinite(determinant) || Math.abs(determinant) <= EPSILON) {
    throw new Error("Spatial frame basis must be invertible: " + frame.id);
  }
}

function cloneFrame(frame: SpatialFrame): SpatialFrame {
  return {
    ...frame,
    origin: { ...frame.origin },
    basisU: { ...frame.basisU },
    basisV: { ...frame.basisV },
  };
}
