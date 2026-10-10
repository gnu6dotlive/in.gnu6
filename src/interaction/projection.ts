import type { EntityId, Vec2 } from "../algebra/types";
import { pickTopmost, type SpatialOccupant } from "../spatial/environment";
import { type FrameId, localToWorld, type SpatialFrameSnapshot } from "../spatial/frame";
import { type DegreeOfFreedom, OwnershipRegistry, type TopologyId } from "../topology/instance";

export type PointerPhase = "down" | "move" | "up" | "cancel";

export type PointerCoordinates =
  | {
      space: "world";
      point: Vec2;
    }
  | {
      space: "frame";
      frameId: FrameId;
      point: Vec2;
    };

export interface RendererPointerObservation {
  providerId: string;
  pointerId: string;
  sequence: number;
  phase: PointerPhase;
  coordinates: PointerCoordinates;
}

export interface InteractionProvenance {
  providerId: string;
  coordinateSpace: PointerCoordinates["space"];
}

export interface SemanticPointerObservation {
  pointerId: string;
  sequence: number;
  phase: PointerPhase;
  worldPoint: Vec2;
  delta: Vec2;
  travelDistance: number;
  subjectId: EntityId | null;
}

export interface InteractionProjection {
  provenance: InteractionProvenance;
  semantic: SemanticPointerObservation;
}

interface PointerTrack {
  sequence: number;
  point: Vec2;
  travelDistance: number;
}

/**
 * Renderer observations are facts, not authority.
 *
 * Projection performs three deterministic operations:
 *
 *   provider coordinates -> World coordinates
 *   ordered spatial occupants -> exact subject identity
 *   ordered pointer samples -> trajectory observation
 *
 * No scene or topology mutation capability is reachable from this class.
 * Provider provenance is retained separately from semantic output so two
 * providers can produce the same semantic observation without becoming the
 * ontology.
 */
export class InteractionProjector {
  private readonly tracks = new Map<string, PointerTrack>();

  constructor(private readonly frames: SpatialFrameSnapshot) {}

  project(
    observation: RendererPointerObservation,
    occupants: readonly SpatialOccupant[],
  ): InteractionProjection {
    assertObservation(observation);

    const key = pointerKey(observation.providerId, observation.pointerId);
    const previous = this.tracks.get(key);

    if (previous && observation.sequence <= previous.sequence) {
      throw new Error("Pointer observation sequence must be strictly monotonic");
    }

    const worldPoint = normalizePoint(this.frames, observation.coordinates);

    // A new down phase starts a new trajectory while sequence monotonicity still
    // protects provenance ordering for this provider/pointer pair.
    const trajectoryPrevious = observation.phase === "down" ? undefined : previous;

    const delta = trajectoryPrevious
      ? {
          x: worldPoint.x - trajectoryPrevious.point.x,
          y: worldPoint.y - trajectoryPrevious.point.y,
        }
      : { x: 0, y: 0 };

    const travelDistance = (trajectoryPrevious?.travelDistance ?? 0) + Math.hypot(delta.x, delta.y);

    const semantic: SemanticPointerObservation = {
      pointerId: observation.pointerId,
      sequence: observation.sequence,
      phase: observation.phase,
      worldPoint,
      delta,
      travelDistance,
      subjectId: pickTopmost(occupants, worldPoint),
    };

    this.tracks.set(key, {
      sequence: observation.sequence,
      point: { ...worldPoint },
      travelDistance,
    });

    return {
      provenance: {
        providerId: observation.providerId,
        coordinateSpace: observation.coordinates.space,
      },
      semantic,
    };
  }
}

export interface TopologyHandoffRequest {
  requestId: string;
  subjectId: EntityId;
  from: TopologyId;
  to: TopologyId;
  degrees: readonly DegreeOfFreedom[];
  provenance: {
    providerId: string;
    pointerId: string;
    pointerSequence: number;
  };
}

export interface TopologyHandoffRequestInput {
  requestId: string;
  from: TopologyId;
  to: TopologyId;
  degrees: readonly DegreeOfFreedom[];
}

export interface TopologyHandoffRule {
  from: TopologyId;
  to: TopologyId;
  allowedDegrees: readonly DegreeOfFreedom[];
}

export interface TopologyHandoffReceipt {
  requestId: string;
  subjectId: EntityId;
  from: TopologyId;
  to: TopologyId;
  degrees: readonly DegreeOfFreedom[];
}

/**
 * A projection can identify a subject and carry provenance, but request
 * construction itself still does not mutate authority.
 */
export function createTopologyHandoffRequest(
  projection: InteractionProjection,
  input: TopologyHandoffRequestInput,
): TopologyHandoffRequest {
  if (!input.requestId) {
    throw new Error("Handoff request identity must be non-empty");
  }
  if (!projection.semantic.subjectId) {
    throw new Error("Handoff request requires an exact subject");
  }
  assertUniqueDegrees(input.degrees);

  return {
    requestId: input.requestId,
    subjectId: projection.semantic.subjectId,
    from: input.from,
    to: input.to,
    degrees: [...input.degrees],
    provenance: {
      providerId: projection.provenance.providerId,
      pointerId: projection.semantic.pointerId,
      pointerSequence: projection.semantic.sequence,
    },
  };
}

/**
 * Explicit handoff authority.
 *
 * Renderer facts do not carry authority. A transfer can occur only when:
 *
 *   route = (from,to) is allowlisted
 *   requested degree set D is a subset of allowed degrees A
 *   current ownership for every d in D is exactly from
 *
 * Therefore:
 *
 *   D subseteq A
 *
 * and authority cannot amplify because a request names more degrees.
 */
export class TopologyHandoffAuthority {
  private readonly rules = new Map<string, Set<DegreeOfFreedom>>();

  constructor(rules: readonly TopologyHandoffRule[]) {
    for (const rule of rules) {
      assertUniqueDegrees(rule.allowedDegrees);
      const key = routeKey(rule.from, rule.to);
      if (this.rules.has(key)) {
        throw new Error("Duplicate handoff route policy");
      }
      this.rules.set(key, new Set(rule.allowedDegrees));
    }
  }

  execute(request: TopologyHandoffRequest, ownership: OwnershipRegistry): TopologyHandoffReceipt {
    const allowed = this.rules.get(routeKey(request.from, request.to));
    if (!allowed) {
      throw new Error("Topology handoff route is not allowed");
    }

    for (const degree of request.degrees) {
      if (!allowed.has(degree)) {
        throw new Error("Requested topology handoff degree is not allowed: " + degree);
      }
    }

    ownership.transfer(request.from, request.to, request.subjectId, request.degrees);

    return {
      requestId: request.requestId,
      subjectId: request.subjectId,
      from: request.from,
      to: request.to,
      degrees: [...request.degrees],
    };
  }
}

function normalizePoint(frames: SpatialFrameSnapshot, coordinates: PointerCoordinates): Vec2 {
  assertFinitePoint(coordinates.point);

  return coordinates.space === "world"
    ? { ...coordinates.point }
    : localToWorld(frames, coordinates.frameId, coordinates.point);
}

function assertObservation(observation: RendererPointerObservation): void {
  if (!observation.providerId) {
    throw new Error("Pointer provider identity must be non-empty");
  }
  if (!observation.pointerId) {
    throw new Error("Pointer identity must be non-empty");
  }
  if (!Number.isInteger(observation.sequence) || observation.sequence < 0) {
    throw new Error("Pointer observation sequence must be a non-negative integer");
  }
}

function assertFinitePoint(point: Vec2): void {
  if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) {
    throw new Error("Pointer coordinates must be finite");
  }
}

function assertUniqueDegrees(degrees: readonly DegreeOfFreedom[]): void {
  if (degrees.length === 0) {
    throw new Error("Handoff degree set must be non-empty");
  }
  if (new Set(degrees).size !== degrees.length) {
    throw new Error("Handoff degree set must be injective");
  }
}

function pointerKey(providerId: string, pointerId: string): string {
  return providerId + "\u0000" + pointerId;
}

function routeKey(from: TopologyId, to: TopologyId): string {
  return from + "\u0000" + to;
}
