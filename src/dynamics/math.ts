import type { Pose2, Vec2, Velocity2 } from "../algebra/types";

export const EPSILON = 1e-6;

export const vec = (x = 0, y = 0): Vec2 => ({ x, y });

export function add(a: Vec2, b: Vec2): Vec2 {
  return { x: a.x + b.x, y: a.y + b.y };
}

export function sub(a: Vec2, b: Vec2): Vec2 {
  return { x: a.x - b.x, y: a.y - b.y };
}

export function mul(v: Vec2, scalar: number): Vec2 {
  return { x: v.x * scalar, y: v.y * scalar };
}

export function magnitude(v: Vec2): number {
  return Math.hypot(v.x, v.y);
}

export function distance(a: Vec2, b: Vec2): number {
  return magnitude(sub(a, b));
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function zeroVelocity(): Velocity2 {
  return { linear: vec(), angular: 0, scale: vec() };
}

export function validPose(pose: Pose2): boolean {
  const values = [pose.position.x, pose.position.y, pose.rotation, pose.scale.x, pose.scale.y];
  return values.every(Number.isFinite) && pose.scale.x > 0 && pose.scale.y > 0;
}

/**
 * Solve acceleration from the damped spring equation:
 *
 *   m x'' + c x' + k(x - target) = 0
 *
 * therefore:
 *
 *   x'' = (-c x' - k(x - target)) / m
 */
export function springAcceleration(
  position: number,
  velocity: number,
  target: number,
  stiffness: number,
  damping: number,
  mass: number,
): number {
  const safeMass = Math.max(EPSILON, mass);
  return (-damping * velocity - stiffness * (position - target)) / safeMass;
}

/**
 * Semi-implicit Euler:
 *
 *   v[n+1] = v[n] + a[n] dt
 *   x[n+1] = x[n] + v[n+1] dt
 *
 * Updating velocity first is materially more stable for damped springs than
 * explicit Euler at interactive frame rates. The blob.in specimen uses the
 * same useful idea.
 */
export function integrateScalar(
  position: number,
  velocity: number,
  target: number,
  dt: number,
  stiffness: number,
  damping: number,
  mass: number,
): { position: number; velocity: number } {
  const acceleration = springAcceleration(position, velocity, target, stiffness, damping, mass);
  const nextVelocity = velocity + acceleration * dt;
  return {
    velocity: nextVelocity,
    position: position + nextVelocity * dt,
  };
}
