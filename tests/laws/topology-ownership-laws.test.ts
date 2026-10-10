import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { SpatialFrameGraph, asFrameId } from "../../src/spatial/frame";
import {
  OwnershipRegistry,
  TopologyRegistry,
  asTopologyId,
  defineTopology,
} from "../../src/topology/instance";

describe("MQ-02 topology ownership laws", () => {
  it("allows concurrent topology owners when degrees of freedom are disjoint", () => {
    const entity = asEntityId("card-1");
    const registry = new OwnershipRegistry();

    registry.claim(asTopologyId("layout"), entity, ["position.x", "position.y"]);
    registry.claim(asTopologyId("deformation"), entity, ["shape"]);

    expect(registry.ownerOf(entity, "position.x")).toBe(asTopologyId("layout"));
    expect(registry.ownerOf(entity, "shape")).toBe(asTopologyId("deformation"));
  });

  it("rejects silent double ownership of the same degree of freedom", () => {
    const entity = asEntityId("card-1");
    const registry = new OwnershipRegistry();
    registry.claim(asTopologyId("fan"), entity, ["rotation"]);

    expect(() => registry.claim(asTopologyId("drag"), entity, ["rotation"])).toThrow(
      "already owned",
    );
  });

  it("transfers authority atomically without changing subject identity", () => {
    const entity = asEntityId("card-7");
    const registry = new OwnershipRegistry();
    const from = asTopologyId("tarot-hand");
    const to = asTopologyId("drag");

    registry.claim(from, entity, ["position.x", "position.y", "rotation"]);
    registry.transfer(from, to, entity, ["position.x", "position.y", "rotation"]);

    expect(registry.ownerOf(entity, "position.x")).toBe(to);
    expect(registry.ownerOf(entity, "position.y")).toBe(to);
    expect(registry.ownerOf(entity, "rotation")).toBe(to);
  });

  it("defines topology membership independently from its ownership claims", () => {
    const topology = defineTopology({
      id: asTopologyId("tarot-hand"),
      frameId: asFrameId("viewport"),
      members: [asEntityId("a"), asEntityId("b")],
      solverKind: "tarot-fan",
      constraints: [{ id: "hand-bounds", kind: "bounds" }],
      claims: [
        {
          entityId: asEntityId("a"),
          degrees: ["position.x", "position.y"],
        },
      ],
    });

    expect(topology.members).toEqual([asEntityId("a"), asEntityId("b")]);
    expect(topology.claims).toHaveLength(1);
  });

  it("registers multiple topology instances in one frame graph without conflating them", () => {
    const frames = new SpatialFrameGraph();
    frames.add({
      id: asFrameId("world"),
      parent: null,
      origin: { x: 0, y: 0 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });

    const registry = new TopologyRegistry();
    registry.add(
      defineTopology({
        id: asTopologyId("dock"),
        frameId: asFrameId("world"),
        members: [asEntityId("dock-card")],
        solverKind: "edge-fan",
        constraints: [],
        claims: [],
      }),
      frames.snapshot(),
    );
    registry.add(
      defineTopology({
        id: asTopologyId("spj"),
        frameId: asFrameId("world"),
        members: [asEntityId("spj-card")],
        solverKind: "tarot-hand",
        constraints: [],
        claims: [],
      }),
      frames.snapshot(),
    );

    expect(registry.snapshot().size).toBe(2);
    expect(registry.get(asTopologyId("dock"))?.solverKind).toBe("edge-fan");
    expect(registry.get(asTopologyId("spj"))?.solverKind).toBe("tarot-hand");
  });

  it("observes declared constraints and effective authority after handoff", () => {
    const frames = new SpatialFrameGraph();
    frames.add({
      id: asFrameId("world"),
      parent: null,
      origin: { x: 0, y: 0 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });

    const card = asEntityId("card");
    const hand = asTopologyId("hand");
    const drag = asTopologyId("drag");
    const registry = new TopologyRegistry();

    registry.add(
      defineTopology({
        id: hand,
        frameId: asFrameId("world"),
        members: [card],
        solverKind: "tarot-hand",
        constraints: [{ id: "world-bounds", kind: "bounds" }],
        claims: [
          {
            entityId: card,
            degrees: ["position.x", "position.y", "rotation"],
          },
        ],
      }),
      frames.snapshot(),
    );

    registry.ownership.transfer(hand, drag, card, ["position.x", "position.y", "rotation"]);

    const observation = registry.observe(hand);
    expect(observation.constraints).toEqual([{ id: "world-bounds", kind: "bounds" }]);
    expect(observation.authority).toEqual([
      {
        entityId: card,
        degree: "position.x",
        ownerId: drag,
        heldByTopology: false,
      },
      {
        entityId: card,
        degree: "position.y",
        ownerId: drag,
        heldByTopology: false,
      },
      {
        entityId: card,
        degree: "rotation",
        ownerId: drag,
        heldByTopology: false,
      },
    ]);
  });

  it("rolls back earlier claims when a later claim conflicts", () => {
    const frames = new SpatialFrameGraph();
    frames.add({
      id: asFrameId("world"),
      parent: null,
      origin: { x: 0, y: 0 },
      basisU: { x: 1, y: 0 },
      basisV: { x: 0, y: 1 },
    });

    const occupied = asEntityId("occupied");
    const transient = asEntityId("transient");
    const registry = new TopologyRegistry();

    registry.ownership.claim(asTopologyId("existing"), occupied, ["position.x"]);

    expect(() =>
      registry.add(
        defineTopology({
          id: asTopologyId("candidate"),
          frameId: asFrameId("world"),
          members: [transient, occupied],
          solverKind: "candidate",
          constraints: [],
          claims: [
            { entityId: transient, degrees: ["shape"] },
            { entityId: occupied, degrees: ["position.x"] },
          ],
        }),
        frames.snapshot(),
      ),
    ).toThrow("already owned");

    expect(registry.get(asTopologyId("candidate"))).toBeNull();
    expect(registry.ownership.ownerOf(transient, "shape")).toBeNull();
    expect(registry.ownership.ownerOf(occupied, "position.x")).toBe(asTopologyId("existing"));
  });
});
