import { clamp, distance, mul, sub, vec } from "./math";
import type { RadialField, Vec2 } from "../algebra/types";

/**
 * Continuous radial kernel.
 *
 * Let d = distance(p,c) / radius.
 *
 * We use:
 *   k(d) = (1 - clamp(d,0,1))^falloff
 *
 * Properties:
 *   k(0) = 1
 *   k(d >= 1) = 0
 *   k is continuous on [0,∞)
 *
 * Continuity matters more than the exact kernel family: if pointer motion is
 * continuous, field influence should not create discontinuous card motion.
 */
export function radialWeight(field: RadialField, point: Vec2): number {
  if (field.radius <= 0) return 0;
  const normalized = clamp(distance(point, field.center) / field.radius, 0, 1);
  return Math.pow(1 - normalized, Math.max(0.01, field.falloff));
}

/**
 * Directional field vector.
 *
 * strength > 0 attracts toward the center.
 * strength < 0 repels away from the center.
 */
export function radialVector(field: RadialField, point: Vec2): Vec2 {
  const delta = sub(field.center, point);
  const d = Math.hypot(delta.x, delta.y);
  if (d < 1e-6) return vec();

  const unit = mul(delta, 1 / d);
  return mul(unit, field.strength * radialWeight(field, point));
}

/**
 * Superposition:
 *
 *   F_total(p) = Σ Fi(p)
 *
 * This lets several cards, a pointer and an envelope boundary all contribute
 * without introducing state-machine branches for every pairwise interaction.
 */
export function sumFields(fields: readonly RadialField[], point: Vec2): Vec2 {
  return fields.reduce((sum, field) => {
    const next = radialVector(field, point);
    return { x: sum.x + next.x, y: sum.y + next.y };
  }, vec());
}
