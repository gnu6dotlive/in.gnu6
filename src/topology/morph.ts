import { interpolateContours, prepareShapeMorph } from "./contour";
import { integrateScalar } from "../dynamics/math";
import type { EntityId, PolygonShape, Shape2, SpringLaw } from "../algebra/types";

export interface ShapeMorphTrack {
  subjectId: EntityId;
  source: PolygonShape;
  target: PolygonShape;
  progress: number;
  velocity: number;
  spring: SpringLaw;
}

/**
 * Create a topology-normalized shape track.
 *
 * The relation is:
 *
 *   M : subject × sourceShape × targetShape → track
 *
 * subject identity is independent of representation. A window becoming a card
 * therefore remains the same subject while its boundary representation moves
 * through shape space.
 */
export function createShapeMorphTrack(
  subjectId: EntityId,
  source: Shape2,
  target: Shape2,
  spring: SpringLaw,
  sampleCount = 48,
): ShapeMorphTrack {
  const prepared = prepareShapeMorph(source, target, sampleCount);
  return {
    subjectId,
    source: prepared.source,
    target: prepared.target,
    progress: 0,
    velocity: 0,
    spring,
  };
}

/**
 * Progress itself is spring-driven:
 *
 *   m p'' + c p' + k(p - 1) = 0
 *
 * Shape is then C(p), where C is the normalized contour interpolation.
 *
 * This separation is useful:
 * - physics decides *when* progress moves;
 * - topology decides *what geometry* progress means.
 */
export function stepShapeMorph(track: ShapeMorphTrack, dtSeconds: number): ShapeMorphTrack {
  const dt = Math.min(1 / 15, Math.max(0, dtSeconds));
  if (dt === 0) return track;

  const next = integrateScalar(
    track.progress,
    track.velocity,
    1,
    dt,
    track.spring.stiffness,
    track.spring.damping,
    track.spring.mass,
  );

  return {
    ...track,
    progress: next.position,
    velocity: next.velocity,
  };
}

export function currentShape(track: ShapeMorphTrack): PolygonShape {
  return {
    kind: "polygon",
    closed: true,
    vertices: interpolateContours(track.source.vertices, track.target.vertices, track.progress),
  };
}

export function shapeMorphSettled(
  track: ShapeMorphTrack,
  positionEpsilon = 0.001,
  velocityEpsilon = 0.001,
): boolean {
  return (
    Math.abs(1 - track.progress) <= positionEpsilon && Math.abs(track.velocity) <= velocityEpsilon
  );
}
