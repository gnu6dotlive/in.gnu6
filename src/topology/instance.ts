import type { EntityId } from "../algebra/types";
import type { FrameId, SpatialFrameSnapshot } from "../spatial/frame";

export type TopologyId = string & { readonly __topologyId: unique symbol };

export type DegreeOfFreedom =
  | "position.x"
  | "position.y"
  | "rotation"
  | "scale.x"
  | "scale.y"
  | "shape";

export interface OwnershipClaim {
  entityId: EntityId;
  degrees: readonly DegreeOfFreedom[];
}

/**
 * Constraints are declared before MQ-07 decides how each family is solved.
 *
 * Keeping identity + kind explicit now lets topology instances state the
 * constraints they depend on without smuggling consumer-specific solver
 * objects into the core.
 */
export interface TopologyConstraint {
  id: string;
  kind: string;
}

export interface TopologyInstance {
  id: TopologyId;
  frameId: FrameId;
  members: readonly EntityId[];
  solverKind: string;
  constraints: readonly TopologyConstraint[];
  claims: readonly OwnershipClaim[];
}

export function asTopologyId(value: string): TopologyId {
  if (!value) throw new Error("Topology identity must be non-empty");
  return value as TopologyId;
}

/**
 * A topology is a relation between a member set, a frame, constraints and a
 * solver. Membership and authority are intentionally distinct.
 *
 *   members(T) ⊆ E
 *   claims(T)  ⊆ members(T) × Q
 *
 * where Q is the finite set of scene degrees of freedom currently modeled.
 *
 * An entity may belong to a topology without that topology owning every
 * property of the entity. This is what allows, for example:
 *
 *   fan topology owns position + rotation
 *   deformation topology owns shape
 *
 * at the same time.
 */
export function defineTopology(input: TopologyInstance): TopologyInstance {
  if (!input.id) throw new Error("Topology identity must be non-empty");
  if (!input.solverKind) throw new Error("Topology solver kind must be non-empty");

  const members = new Set(input.members);
  if (members.size !== input.members.length) {
    throw new Error("Topology members must be injective");
  }

  const constraintIds = new Set<string>();
  for (const constraint of input.constraints) {
    if (!constraint.id || !constraint.kind) {
      throw new Error("Topology constraints require non-empty id and kind");
    }
    if (constraintIds.has(constraint.id)) {
      throw new Error("Topology constraint identities must be injective");
    }
    constraintIds.add(constraint.id);
  }

  for (const claim of input.claims) {
    if (!members.has(claim.entityId)) {
      throw new Error("Topology claim subject must belong to topology members");
    }
    if (new Set(claim.degrees).size !== claim.degrees.length) {
      throw new Error("Topology claim degrees must be injective");
    }
  }

  return {
    ...input,
    members: [...input.members],
    constraints: input.constraints.map((constraint) => ({ ...constraint })),
    claims: input.claims.map((claim) => ({
      entityId: claim.entityId,
      degrees: [...claim.degrees],
    })),
  };
}

/**
 * Ownership is a partial function:
 *
 *   owner : E × Q ⇀ T
 *
 * Therefore for every entity e and degree q:
 *
 *   |owner(e,q)| <= 1
 *
 * This registry is deliberately independent from the topology solver.
 * Solvers propose targets; ownership decides whether they are authorized to
 * control a degree of freedom.
 */
export interface OwnershipObservation {
  entityId: EntityId;
  degree: DegreeOfFreedom;
  ownerId: TopologyId;
}

export class OwnershipRegistry {
  private readonly owners = new Map<string, OwnershipObservation>();

  ownerOf(entityId: EntityId, degree: DegreeOfFreedom): TopologyId | null {
    return this.owners.get(key(entityId, degree))?.ownerId ?? null;
  }

  observation(): readonly OwnershipObservation[] {
    return [...this.owners.values()]
      .map((entry) => ({ ...entry }))
      .sort((a, b) => {
        const entityOrder = a.entityId.localeCompare(b.entityId);
        return entityOrder !== 0 ? entityOrder : a.degree.localeCompare(b.degree);
      });
  }

  claim(ownerId: TopologyId, entityId: EntityId, degrees: readonly DegreeOfFreedom[]): void {
    assertUniqueDegrees(degrees);

    for (const degree of degrees) {
      const current = this.ownerOf(entityId, degree);
      if (current && current !== ownerId) {
        throw new Error(entityId + "." + degree + " is already owned by " + current);
      }
    }

    for (const degree of degrees) {
      this.owners.set(key(entityId, degree), { entityId, degree, ownerId });
    }
  }

  release(ownerId: TopologyId, entityId: EntityId, degrees: readonly DegreeOfFreedom[]): void {
    assertUniqueDegrees(degrees);

    for (const degree of degrees) {
      const current = this.ownerOf(entityId, degree);
      if (current !== ownerId) {
        throw new Error("Cannot release " + entityId + "." + degree + " from non-owner " + ownerId);
      }
    }

    for (const degree of degrees) {
      this.owners.delete(key(entityId, degree));
    }
  }

  /**
   * Atomic authority handoff:
   *
   *   owner_before(e,q) = from
   *   owner_after(e,q)  = to
   *
   * No intermediate unowned scene state is externally observable.
   * Entity identity, pose and velocity live elsewhere and are not rewritten by
   * an ownership transfer.
   */
  transfer(
    from: TopologyId,
    to: TopologyId,
    entityId: EntityId,
    degrees: readonly DegreeOfFreedom[],
  ): void {
    assertUniqueDegrees(degrees);

    for (const degree of degrees) {
      const current = this.ownerOf(entityId, degree);
      if (current !== from) {
        throw new Error(
          "Cannot transfer " + entityId + "." + degree + " because " + from + " is not the owner",
        );
      }
    }

    for (const degree of degrees) {
      this.owners.set(key(entityId, degree), { entityId, degree, ownerId: to });
    }
  }
}

function key(entityId: EntityId, degree: DegreeOfFreedom): string {
  return entityId + "\u0000" + degree;
}

function assertUniqueDegrees(degrees: readonly DegreeOfFreedom[]): void {
  if (new Set(degrees).size !== degrees.length) {
    throw new Error("Ownership degree set must be injective");
  }
}

export interface TopologyAuthorityObservation {
  entityId: EntityId;
  degree: DegreeOfFreedom;
  ownerId: TopologyId | null;
  heldByTopology: boolean;
}

export interface TopologyObservation {
  id: TopologyId;
  frameId: FrameId;
  solverKind: string;
  members: readonly EntityId[];
  constraints: readonly TopologyConstraint[];
  authority: readonly TopologyAuthorityObservation[];
}

/**
 * Registry of concurrently existing topology instances.
 *
 * This registry does not solve topology. It records:
 *   T = finite set of topology identities
 *   frame : T -> F
 *   members : T -> P(E)
 *   claims : T -> P(E × Q)
 *
 * Adding T_i is transactional with respect to ownership:
 * either every claim is admissible and T_i becomes observable,
 * or neither the topology nor its claims are committed.
 */
export class TopologyRegistry {
  private readonly topologies = new Map<TopologyId, TopologyInstance>();
  readonly ownership = new OwnershipRegistry();

  snapshot(): ReadonlyMap<TopologyId, TopologyInstance> {
    return new Map(this.topologies);
  }

  get(id: TopologyId): TopologyInstance | null {
    return this.topologies.get(id) ?? null;
  }

  /**
   * Deterministic observation of declared topology + effective authority.
   *
   * Declaration and authority are intentionally distinct:
   * a topology may still list a subject among its members after authority has
   * been handed to another topology such as Drag or Playzone.
   */
  observe(id: TopologyId): TopologyObservation {
    const topology = this.topologies.get(id);
    if (!topology) throw new Error("Unknown topology identity: " + id);

    const authority = topology.claims
      .flatMap((claim) =>
        claim.degrees.map((degree) => {
          const ownerId = this.ownership.ownerOf(claim.entityId, degree);
          return {
            entityId: claim.entityId,
            degree,
            ownerId,
            heldByTopology: ownerId === topology.id,
          };
        }),
      )
      .sort((a, b) => {
        const entityOrder = a.entityId.localeCompare(b.entityId);
        return entityOrder !== 0 ? entityOrder : a.degree.localeCompare(b.degree);
      });

    return {
      id: topology.id,
      frameId: topology.frameId,
      solverKind: topology.solverKind,
      members: [...topology.members],
      constraints: topology.constraints.map((constraint) => ({ ...constraint })),
      authority,
    };
  }

  add(topology: TopologyInstance, frames: SpatialFrameSnapshot): TopologyInstance {
    if (this.topologies.has(topology.id)) {
      throw new Error("Duplicate topology identity: " + topology.id);
    }
    if (!frames.has(topology.frameId)) {
      throw new Error("Topology frame does not exist: " + topology.frameId);
    }

    const normalized = defineTopology(topology);
    const committedClaims: Array<{
      entityId: EntityId;
      degrees: readonly DegreeOfFreedom[];
    }> = [];

    try {
      for (const claim of normalized.claims) {
        this.ownership.claim(normalized.id, claim.entityId, claim.degrees);
        committedClaims.push(claim);
      }
    } catch (error) {
      for (const claim of committedClaims.reverse()) {
        this.ownership.release(normalized.id, claim.entityId, claim.degrees);
      }
      throw error;
    }

    this.topologies.set(normalized.id, normalized);
    return normalized;
  }

  remove(id: TopologyId): TopologyInstance {
    const topology = this.topologies.get(id);
    if (!topology) throw new Error("Unknown topology identity: " + id);

    for (const claim of topology.claims) {
      this.ownership.release(id, claim.entityId, claim.degrees);
    }

    this.topologies.delete(id);
    return topology;
  }
}
