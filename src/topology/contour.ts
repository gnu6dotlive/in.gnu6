import { add, distance, mul, sub } from "../dynamics/math";
import type { PolygonShape, RectShape, Shape2, Vec2 } from "../algebra/types";

/**
 * Signed polygon area:
 *
 *   A = 1/2 Σ (xi yi+1 - xi+1 yi)
 *
 * sign(A) encodes winding. We normalize winding before correspondence because
 * clockwise and counter-clockwise cycles describe opposite traversals of the
 * same geometric boundary.
 */
export function signedArea(vertices: readonly Vec2[]): number {
  let twiceArea = 0;
  for (let i = 0; i < vertices.length; i += 1) {
    const a = vertices[i]!;
    const b = vertices[(i + 1) % vertices.length]!;
    twiceArea += a.x * b.y - b.x * a.y;
  }
  return twiceArea / 2;
}

export function ensureClockwise(vertices: readonly Vec2[]): Vec2[] {
  if (vertices.length < 3) return [...vertices];
  return signedArea(vertices) > 0 ? [...vertices].reverse() : [...vertices];
}

/**
 * Arc-length resampling.
 *
 * A closed contour C is treated as a periodic map:
 *
 *   C : [0,L) → R²
 *
 * We sample N equally spaced arc-length parameters:
 *
 *   sk = kL/N
 *
 * This converts arbitrary source/target vertex counts into equal-cardinality
 * cycles without inventing product semantics.
 */
export function resampleClosedContour(vertices: readonly Vec2[], sampleCount: number): Vec2[] {
  if (sampleCount < 3) throw new Error("Closed contour requires at least 3 samples");
  if (vertices.length < 3) throw new Error("Closed contour requires at least 3 vertices");

  const points = ensureClockwise(vertices);
  const segments = points.map((point, index) => {
    const next = points[(index + 1) % points.length]!;
    return { from: point, to: next, length: distance(point, next) };
  });

  const perimeter = segments.reduce((sum, segment) => sum + segment.length, 0);
  if (perimeter <= 1e-6) throw new Error("Closed contour perimeter must be positive");

  const samples: Vec2[] = [];
  let segmentIndex = 0;
  let segmentStartDistance = 0;

  for (let k = 0; k < sampleCount; k += 1) {
    const targetDistance = (k * perimeter) / sampleCount;

    while (
      segmentIndex < segments.length - 1 &&
      segmentStartDistance + segments[segmentIndex]!.length < targetDistance
    ) {
      segmentStartDistance += segments[segmentIndex]!.length;
      segmentIndex += 1;
    }

    const segment = segments[segmentIndex]!;
    const localDistance = targetDistance - segmentStartDistance;
    const t = segment.length <= 1e-6 ? 0 : localDistance / segment.length;
    samples.push(add(segment.from, mul(sub(segment.to, segment.from), t)));
  }

  return samples;
}

/**
 * Cyclic correspondence problem.
 *
 * After equal-cardinality resampling, source and target still have no canonical
 * index zero. We choose the cyclic shift j minimizing total squared distance:
 *
 *   j* = argmin_j Σi ||source[i] - target[(i+j) mod N]||²
 *
 * This is O(N²), intentionally simple and deterministic. N is expected to be
 * small for UI contours. A faster solver is not justified until profiling says
 * topology matching is a bottleneck.
 */
export function alignCyclicContours(source: readonly Vec2[], target: readonly Vec2[]): Vec2[] {
  if (source.length !== target.length) {
    throw new Error("Contour cardinalities must match before alignment");
  }

  let bestShift = 0;
  let bestCost = Number.POSITIVE_INFINITY;

  for (let shift = 0; shift < target.length; shift += 1) {
    let cost = 0;
    for (let i = 0; i < source.length; i += 1) {
      const a = source[i]!;
      const b = target[(i + shift) % target.length]!;
      const dx = a.x - b.x;
      const dy = a.y - b.y;
      cost += dx * dx + dy * dy;
    }
    if (cost < bestCost) {
      bestCost = cost;
      bestShift = shift;
    }
  }

  return target.map((_, index) => target[(index + bestShift) % target.length]!);
}

/**
 * Linear contour interpolation after topological normalization.
 *
 * For corresponding points ai, bi:
 *
 *   pi(t) = (1-t) ai + t bi,  t ∈ [0,1]
 *
 * The interpolation is geometric, not semantic. The semantic decision that a
 * window may morph into a card belongs to a higher adapter; this kernel merely
 * guarantees a continuous path once two shapes are declared corresponding.
 */
export function interpolateContours(
  source: readonly Vec2[],
  target: readonly Vec2[],
  progress: number,
): Vec2[] {
  if (source.length !== target.length) {
    throw new Error("Contour cardinalities must match for interpolation");
  }

  const t = Math.min(1, Math.max(0, progress));
  return source.map((point, index) => {
    const next = target[index]!;
    return {
      x: point.x + (next.x - point.x) * t,
      y: point.y + (next.y - point.y) * t,
    };
  });
}

/**
 * Rounded rectangle to closed contour.
 *
 * Each corner contributes arcSamples points, so:
 *
 *   |V| = 4 * arcSamples
 *
 * Radius is clamped to the half-extent to preserve a simple boundary.
 */
export function roundedRectContour(shape: RectShape, arcSamples = 6): PolygonShape {
  const count = Math.max(2, Math.floor(arcSamples));
  const halfW = shape.width / 2;
  const halfH = shape.height / 2;
  const radius = Math.min(Math.max(0, shape.radius), halfW, halfH);
  const centers = [
    { x: halfW - radius, y: halfH - radius, start: 0 },
    { x: -halfW + radius, y: halfH - radius, start: Math.PI / 2 },
    { x: -halfW + radius, y: -halfH + radius, start: Math.PI },
    { x: halfW - radius, y: -halfH + radius, start: (3 * Math.PI) / 2 },
  ];

  const vertices: Vec2[] = [];
  for (const corner of centers) {
    for (let i = 0; i < count; i += 1) {
      const angle = corner.start + (i / (count - 1)) * (Math.PI / 2);
      vertices.push({
        x: corner.x + Math.cos(angle) * radius,
        y: corner.y + Math.sin(angle) * radius,
      });
    }
  }

  return { kind: "polygon", vertices: ensureClockwise(vertices), closed: true };
}

export function shapeToContour(shape: Shape2, sampleCount = 32): PolygonShape {
  const source = shape.kind === "polygon" ? shape.vertices : roundedRectContour(shape).vertices;
  return {
    kind: "polygon",
    vertices: resampleClosedContour(source, sampleCount),
    closed: true,
  };
}

/**
 * Full shape morph normalization:
 *
 *   normalize : Shape × Shape → (V,V)
 *
 * where both returned cycles have equal cardinality, equal winding and a
 * minimum-distance cyclic correspondence.
 */
export function prepareShapeMorph(
  source: Shape2,
  target: Shape2,
  sampleCount = 32,
): { source: PolygonShape; target: PolygonShape } {
  const a = shapeToContour(source, sampleCount);
  const b = shapeToContour(target, sampleCount);

  return {
    source: a,
    target: {
      kind: "polygon",
      vertices: alignCyclicContours(a.vertices, b.vertices),
      closed: true,
    },
  };
}
