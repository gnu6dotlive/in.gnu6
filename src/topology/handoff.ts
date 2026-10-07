import type { EntityId, RectShape, Vec2 } from "../algebra/types";

export type RectHandoffSequence = "reshape-then-translate" | "translate-then-reshape" | "direct";

export interface RectBounds {
  x: number;
  y: number;
  width: number;
  height: number;
  radius: number;
}

export interface RectSurfaceGeometry {
  center: Vec2;
  shape: RectShape;
}

export interface RectHandoffPlan {
  subjectId: EntityId;
  sequence: RectHandoffSequence;
  source: RectSurfaceGeometry;
  transfer: RectSurfaceGeometry | null;
  target: RectSurfaceGeometry;
  vector: Vec2;
  generation: number;
  retargetCount: number;
}
export function rectBoundsToGeometry(bounds: RectBounds): RectSurfaceGeometry {
  assertRectBounds(bounds);
  const radius = normalizeRadius(bounds.radius, bounds.width, bounds.height);
  return {
    center: {
      x: bounds.x + bounds.width / 2,
      y: bounds.y + bounds.height / 2,
    },
    shape: {
      kind: "rect",
      width: bounds.width,
      height: bounds.height,
      radius,
    },
  };
}

export function rectGeometryToBounds(geometry: RectSurfaceGeometry): RectBounds {
  assertGeometry(geometry);
  return {
    x: geometry.center.x - geometry.shape.width / 2,
    y: geometry.center.y - geometry.shape.height / 2,
    width: geometry.shape.width,
    height: geometry.shape.height,
    radius: geometry.shape.radius,
  };
}
/**
 * Plan a rectangular representation handoff without acquiring product semantics.
 *
 * The caller chooses whether the same subject reshapes before translation,
 * translates before reshaping, or moves directly. The engine owns only the
 * geometric consequence of that sequence.
 */
export function planRectHandoff({
  subjectId,
  sequence,
  source,
  target,
  transferShape,
  generation = 0,
}: {
  subjectId: EntityId;
  sequence: RectHandoffSequence;
  source: RectBounds;
  target: RectBounds;
  transferShape?: Omit<RectShape, "kind">;
  generation?: number;
}): RectHandoffPlan {
  assertGeneration(generation);
  const sourceGeometry = rectBoundsToGeometry(source);
  const targetGeometry = rectBoundsToGeometry(target);

  const transfer =
    sequence === "direct"
      ? null
      : transferGeometry(
          sequence === "reshape-then-translate" ? sourceGeometry.center : targetGeometry.center,
          transferShape,
        );
  return {
    subjectId,
    sequence,
    source: sourceGeometry,
    transfer,
    target: targetGeometry,
    vector: {
      x: targetGeometry.center.x - sourceGeometry.center.x,
      y: targetGeometry.center.y - sourceGeometry.center.y,
    },
    generation,
    retargetCount: 0,
  };
}

export function retargetRectHandoff({
  plan,
  target,
  transferShape,
  generation,
}: {
  plan: RectHandoffPlan;
  target: RectBounds;
  transferShape?: Omit<RectShape, "kind">;
  generation: number;
}): RectHandoffPlan {
  assertGeneration(generation);
  if (generation <= plan.generation) {
    throw new Error("Rectangular handoff retarget generation must be newer than the current plan");
  }

  const targetGeometry = rectBoundsToGeometry(target);
  const transfer =
    plan.sequence === "direct"
      ? null
      : transferGeometry(
          plan.sequence === "reshape-then-translate" ? plan.source.center : targetGeometry.center,
          transferShape ??
            (plan.transfer
              ? {
                  width: plan.transfer.shape.width,
                  height: plan.transfer.shape.height,
                  radius: plan.transfer.shape.radius,
                }
              : undefined),
        );

  return {
    subjectId: plan.subjectId,
    sequence: plan.sequence,
    source: {
      center: { ...plan.source.center },
      shape: { ...plan.source.shape },
    },
    transfer,
    target: targetGeometry,
    vector: {
      x: targetGeometry.center.x - plan.source.center.x,
      y: targetGeometry.center.y - plan.source.center.y,
    },
    generation,
    retargetCount: plan.retargetCount + 1,
  };
}

function transferGeometry(
  center: Vec2,
  shape: Omit<RectShape, "kind"> | undefined,
): RectSurfaceGeometry {
  if (!shape) {
    throw new Error("Non-direct rectangular handoff requires a transfer shape");
  }
  assertPositiveFinite(shape.width, "transfer width");
  assertPositiveFinite(shape.height, "transfer height");
  assertFiniteNonNegative(shape.radius, "transfer radius");

  return {
    center: { ...center },
    shape: {
      kind: "rect",
      width: shape.width,
      height: shape.height,
      radius: normalizeRadius(shape.radius, shape.width, shape.height),
    },
  };
}
function assertRectBounds(bounds: RectBounds): void {
  for (const [name, value] of [
    ["x", bounds.x],
    ["y", bounds.y],
    ["width", bounds.width],
    ["height", bounds.height],
    ["radius", bounds.radius],
  ] as const) {
    if (!Number.isFinite(value)) {
      throw new Error(`Rectangular handoff ${name} must be finite`);
    }
  }
  assertPositiveFinite(bounds.width, "width");
  assertPositiveFinite(bounds.height, "height");
  assertFiniteNonNegative(bounds.radius, "radius");
}

function assertGeometry(geometry: RectSurfaceGeometry): void {
  if (!Number.isFinite(geometry.center.x) || !Number.isFinite(geometry.center.y)) {
    throw new Error("Rectangular handoff center must be finite");
  }
  assertPositiveFinite(geometry.shape.width, "width");
  assertPositiveFinite(geometry.shape.height, "height");
  assertFiniteNonNegative(geometry.shape.radius, "radius");
}
function assertPositiveFinite(value: number, label: string): void {
  if (!Number.isFinite(value)) {
    throw new Error(`Rectangular handoff ${label} must be finite`);
  }
  if (value <= 0) {
    throw new Error(`Rectangular handoff ${label} must be positive`);
  }
}

function assertFiniteNonNegative(value: number, label: string): void {
  if (!Number.isFinite(value)) {
    throw new Error(`Rectangular handoff ${label} must be finite`);
  }
  if (value < 0) {
    throw new Error(`Rectangular handoff ${label} must be non-negative`);
  }
}

function assertGeneration(generation: number): void {
  if (!Number.isSafeInteger(generation) || generation < 0) {
    throw new Error("Rectangular handoff generation must be a non-negative safe integer");
  }
}

function normalizeRadius(radius: number, width: number, height: number): number {
  return Math.min(radius, width / 2, height / 2);
}
