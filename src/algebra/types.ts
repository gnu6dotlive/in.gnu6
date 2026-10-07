/**
 * Headless scenic/topological vocabulary.
 *
 * Let E be the finite set of scene entities.
 * Let R be a set of directed relations, R ⊆ E × E.
 * Let P be planar pose space:
 *
 *   P = R²(position) × R(rotation) × R²(scale)
 *
 * A renderer is only a projection:
 *
 *   render : SceneState → Pixels
 *
 * Therefore product identity and motion meaning must exist before rendering.
 */
export type EntityId = string & { readonly __entityId: unique symbol };

export interface Vec2 {
  x: number;
  y: number;
}

export interface Pose2 {
  position: Vec2;
  rotation: number;
  scale: Vec2;
}

/**
 * Velocity is the first derivative of pose with respect to scene time.
 *
 *   v = dx/dt
 *   ω = dθ/dt
 *   σ = ds/dt
 *
 * Scale velocity is explicit because magnification must approach a target;
 * an instantaneous scale assignment is the cut-off we are trying to forbid.
 */
export interface Velocity2 {
  linear: Vec2;
  angular: number;
  scale: Vec2;
}

export interface RectShape {
  kind: "rect";
  width: number;
  height: number;
  radius: number;
}

/**
 * Polygon topology is an ordered cycle, not a bag of points.
 *
 * For V = {v0 ... vn-1}, boundary adjacency is:
 *
 *   B = {(vi, v(i+1 mod n)) | 0 <= i < n}
 *
 * Same points + different B may represent a different surface.
 */
export interface PolygonShape {
  kind: "polygon";
  vertices: readonly Vec2[];
  closed: true;
}

export type Shape2 = RectShape | PolygonShape;

export type SceneRole = "card" | "surface" | "control" | "proxy" | "anchor" | "decoration";

/**
 * Identity law:
 *
 *   for all a,b in E:
 *     a.id = b.id  implies  a = b
 *
 * Pixels may be duplicated by a renderer. Semantic identity may not.
 */
export interface SceneEntity {
  id: EntityId;
  role: SceneRole;
  pose: Pose2;
  velocity: Velocity2;
  shape: Shape2;
  z: number;
  visible: boolean;
  parent: EntityId | null;
}

/**
 * General directed relation over identities.
 *
 * Examples:
 *   follows(A,B)
 *   attachedTo(submenu,invoker)
 *   occludes(topCard,bottomCard)
 *
 * Relations encode meaning that would otherwise degrade into local booleans.
 */
export interface SceneRelation {
  kind: string;
  from: EntityId;
  to: EntityId;
}

/**
 * Radial field φ centered at c.
 *
 * influence(p) = strength * kernel(distance(p,c) / radius)
 *
 * Multiple fields compose by superposition:
 *
 *   F_total(p) = Σ Fi(p)
 *
 * This is the basis for continuous deck magnification and neighbor yield.
 */
export interface RadialField {
  id: string;
  kind: "radial";
  center: Vec2;
  radius: number;
  strength: number;
  falloff: number;
}

/**
 * Classical damped spring:
 *
 *   m x'' + c x' + k(x - x*) = 0
 *
 * damping ratio:
 *
 *   ζ = c / (2 sqrt(km))
 *
 * ζ < 1 expressive, ζ = 1 critical, ζ > 1 heavy.
 */
export interface SpringLaw {
  stiffness: number;
  damping: number;
  mass: number;
}

/**
 * Ownership law for animated degrees of freedom:
 *
 *   for every entity e and property q:
 *     |Owner(e,q)| <= 1
 *
 * This formalizes the SP Joaillerie donor lesson:
 * one animation clock per property, per node.
 */
export interface PoseTarget {
  entityId: EntityId;
  pose: Pose2;
  spring: SpringLaw;
}

/**
 * SceneState is a finite attributed graph G = (E,R).
 *
 * order must be a permutation of E:
 *
 *   set(order) = E
 *   |order| = |E|
 *
 * That gives deterministic traversal without conflating traversal order
 * with product identity.
 */
export interface SceneState {
  time: number;
  entities: ReadonlyMap<EntityId, SceneEntity>;
  relations: readonly SceneRelation[];
  order: readonly EntityId[];
  targets: ReadonlyMap<EntityId, PoseTarget>;
  fields: readonly RadialField[];
}
