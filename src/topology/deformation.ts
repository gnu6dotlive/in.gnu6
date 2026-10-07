import type {
  EntityId,
  PolygonShape,
  RadialField,
  Shape2,
  SpringLaw,
  Vec2,
} from "../algebra/types";
import { sumFields } from "../dynamics/field";
import { integrateScalar } from "../dynamics/math";
import type { Aabb, RegionId } from "../spatial/environment";
import { containsPoint, type SpatialEnvironment } from "../spatial/environment";
import { type FrameId, localToWorld, type SpatialFrameSnapshot } from "../spatial/frame";
import { alignCyclicContours, shapeToContour } from "./contour";

export interface ContourNodeState {
  position: Vec2;
  velocity: Vec2;
}

export interface DeformableContourTrack {
  subjectId: EntityId;
  nodes: readonly ContourNodeState[];
  target: PolygonShape;
  spring: SpringLaw;
}

export interface ContourEnvironmentObservation {
  contained: boolean;
  collisions: readonly RegionId[];
  admissible: boolean;
}

/**
 * Deformable contour dynamics.
 *
 * A shape is no longer represented only by a scalar morph progress.
 * Each contour node carries its own state:
 *
 *   node_i = (position_i, velocity_i)
 *
 * and converges toward a corresponding target vertex under the same damped
 * spring law used elsewhere in the engine.
 *
 * This gives retargeting a real dynamic state to preserve.
 */
export function createDeformableContourTrack(
  subjectId: EntityId,
  source: Shape2,
  target: Shape2,
  spring: SpringLaw,
  sampleCount = 48,
): DeformableContourTrack {
  const sourceContour = shapeToContour(source, sampleCount);
  const targetContour = shapeToContour(target, sampleCount);
  const alignedTarget = alignCyclicContours(sourceContour.vertices, targetContour.vertices);

  return {
    subjectId,
    nodes: sourceContour.vertices.map((position) => ({
      position: { ...position },
      velocity: { x: 0, y: 0 },
    })),
    target: {
      kind: "polygon",
      closed: true,
      vertices: alignedTarget.map((point) => ({ ...point })),
    },
    spring: { ...spring },
  };
}

/**
 * Retargeting preserves live node position and velocity.
 *
 * Let X(t) be the current node state and Y be a new target contour.
 * Retarget changes only the target correspondence:
 *
 *   (X(t), X'(t), Y_old) -> (X(t), X'(t), Y_new)
 *
 * No node is recreated and no velocity is reset.
 */
export function retargetDeformableContour(
  track: DeformableContourTrack,
  target: Shape2,
): DeformableContourTrack {
  const currentVertices = track.nodes.map((node) => node.position);
  const targetContour = shapeToContour(target, track.nodes.length);
  const alignedTarget = alignCyclicContours(currentVertices, targetContour.vertices);

  return {
    ...track,
    nodes: track.nodes.map((node) => ({
      position: { ...node.position },
      velocity: { ...node.velocity },
    })),
    target: {
      kind: "polygon",
      closed: true,
      vertices: alignedTarget.map((point) => ({ ...point })),
    },
  };
}

export function stepDeformableContour(
  track: DeformableContourTrack,
  dtSeconds: number,
): DeformableContourTrack {
  const dt = Math.min(1 / 15, Math.max(0, dtSeconds));
  if (dt === 0) return track;

  const nodes = track.nodes.map((node, index) => {
    const target = track.target.vertices[index];
    if (!target) {
      throw new Error("Deformable contour target cardinality mismatch");
    }

    const x = integrateScalar(
      node.position.x,
      node.velocity.x,
      target.x,
      dt,
      track.spring.stiffness,
      track.spring.damping,
      track.spring.mass,
    );
    const y = integrateScalar(
      node.position.y,
      node.velocity.y,
      target.y,
      dt,
      track.spring.stiffness,
      track.spring.damping,
      track.spring.mass,
    );

    return {
      position: { x: x.position, y: y.position },
      velocity: { x: x.velocity, y: y.velocity },
    };
  });

  return {
    ...track,
    nodes,
  };
}

export function currentDeformableShape(track: DeformableContourTrack): PolygonShape {
  return {
    kind: "polygon",
    closed: true,
    vertices: track.nodes.map((node) => ({ ...node.position })),
  };
}

export function deformContourByFields(
  contour: PolygonShape,
  fields: readonly RadialField[],
): PolygonShape {
  return {
    kind: "polygon",
    closed: true,
    vertices: contour.vertices.map((point) => {
      const influence = sumFields(fields, point);
      return {
        x: point.x + influence.x,
        y: point.y + influence.y,
      };
    }),
  };
}

export function deformableContourSettled(
  track: DeformableContourTrack,
  positionEpsilon = 0.001,
  velocityEpsilon = 0.001,
): boolean {
  return track.nodes.every((node, index) => {
    const target = track.target.vertices[index];
    if (!target) return false;

    return (
      Math.hypot(node.position.x - target.x, node.position.y - target.y) <= positionEpsilon &&
      Math.hypot(node.velocity.x, node.velocity.y) <= velocityEpsilon
    );
  });
}

/**
 * Project a local contour into World coordinates through the full frame chain.
 *
 * Topology remains local; environment truth is evaluated in World space.
 */
export function transformContourToWorld(
  contour: PolygonShape,
  frames: SpatialFrameSnapshot,
  frameId: FrameId,
): PolygonShape {
  return {
    kind: "polygon",
    closed: true,
    vertices: contour.vertices.map((point) => localToWorld(frames, frameId, point)),
  };
}

export function contourBounds(contour: PolygonShape): Aabb {
  if (contour.vertices.length < 3) {
    throw new Error("Contour bounds require at least 3 vertices");
  }

  let minX = Number.POSITIVE_INFINITY;
  let minY = Number.POSITIVE_INFINITY;
  let maxX = Number.NEGATIVE_INFINITY;
  let maxY = Number.NEGATIVE_INFINITY;

  for (const point of contour.vertices) {
    if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) {
      throw new Error("Contour vertices must be finite");
    }
    minX = Math.min(minX, point.x);
    minY = Math.min(minY, point.y);
    maxX = Math.max(maxX, point.x);
    maxY = Math.max(maxY, point.y);
  }

  return { minX, minY, maxX, maxY };
}

export function inspectContourEnvironment(
  contour: PolygonShape,
  environment: SpatialEnvironment,
  within: RegionId,
  avoid?: readonly RegionId[],
): ContourEnvironmentObservation {
  const withinRegion = environment.getRegion(within);
  if (!withinRegion) throw new Error("Unknown contour region: " + within);
  if (withinRegion.kind === "exclusion") {
    throw new Error("Contour containing region cannot be exclusion");
  }

  const contained = contour.vertices.every((point) => containsPoint(withinRegion.bounds, point));

  const avoidIds =
    avoid ??
    environment
      .observation()
      .regions.filter((region) => region.kind === "exclusion")
      .map((region) => region.id);

  const collisions = avoidIds
    .filter((id) => {
      const region = environment.getRegion(id);
      if (!region) throw new Error("Unknown contour exclusion region: " + id);
      if (region.kind !== "exclusion") {
        throw new Error("Contour avoided region is not exclusion: " + id);
      }
      return contourIntersectsAabb(contour, region.bounds);
    })
    .sort((a, b) => a.localeCompare(b));

  return {
    contained,
    collisions,
    admissible: contained && collisions.length === 0,
  };
}

/**
 * Exact polygon-vs-AABB intersection for simple closed contours.
 *
 * Intersection exists if any of these hold:
 * - a contour vertex lies inside the AABB;
 * - an AABB corner lies inside the polygon;
 * - a contour edge intersects an AABB edge.
 */
export function contourIntersectsAabb(contour: PolygonShape, bounds: Aabb): boolean {
  if (contour.vertices.some((point) => containsPoint(bounds, point))) {
    return true;
  }

  const corners: Vec2[] = [
    { x: bounds.minX, y: bounds.minY },
    { x: bounds.maxX, y: bounds.minY },
    { x: bounds.maxX, y: bounds.maxY },
    { x: bounds.minX, y: bounds.maxY },
  ];

  if (corners.some((corner) => pointInPolygon(corner, contour.vertices))) {
    return true;
  }

  const boxEdges = corners.map(
    (corner, index) => [corner, corners[(index + 1) % corners.length]!] as const,
  );

  for (let index = 0; index < contour.vertices.length; index += 1) {
    const a = contour.vertices[index]!;
    const b = contour.vertices[(index + 1) % contour.vertices.length]!;

    for (const [c, d] of boxEdges) {
      if (segmentsIntersect(a, b, c, d)) return true;
    }
  }

  return false;
}

function pointInPolygon(point: Vec2, vertices: readonly Vec2[]): boolean {
  let inside = false;

  for (let i = 0, previous = vertices.length - 1; i < vertices.length; previous = i, i += 1) {
    const a = vertices[i]!;
    const b = vertices[previous]!;

    if (pointOnSegment(point, a, b)) return true;

    const crosses =
      a.y > point.y !== b.y > point.y &&
      point.x < ((b.x - a.x) * (point.y - a.y)) / (b.y - a.y) + a.x;

    if (crosses) inside = !inside;
  }

  return inside;
}

function segmentsIntersect(a: Vec2, b: Vec2, c: Vec2, d: Vec2): boolean {
  const o1 = orientation(a, b, c);
  const o2 = orientation(a, b, d);
  const o3 = orientation(c, d, a);
  const o4 = orientation(c, d, b);

  if (o1 !== o2 && o3 !== o4) return true;

  if (o1 === 0 && pointOnSegment(c, a, b)) return true;
  if (o2 === 0 && pointOnSegment(d, a, b)) return true;
  if (o3 === 0 && pointOnSegment(a, c, d)) return true;
  if (o4 === 0 && pointOnSegment(b, c, d)) return true;

  return false;
}

function orientation(a: Vec2, b: Vec2, c: Vec2): -1 | 0 | 1 {
  const cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);

  if (Math.abs(cross) <= 1e-9) return 0;
  return cross > 0 ? 1 : -1;
}

function pointOnSegment(point: Vec2, a: Vec2, b: Vec2): boolean {
  if (orientation(a, b, point) !== 0) return false;

  return (
    point.x >= Math.min(a.x, b.x) - 1e-9 &&
    point.x <= Math.max(a.x, b.x) + 1e-9 &&
    point.y >= Math.min(a.y, b.y) - 1e-9 &&
    point.y <= Math.max(a.y, b.y) + 1e-9
  );
}
