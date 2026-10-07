import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import {
  InteractionProjector,
  TopologyHandoffAuthority,
  createTopologyHandoffRequest,
} from "../../src/interaction/projection";
import { OwnershipRegistry, asTopologyId } from "../../src/topology/instance";

const card = asEntityId("card");
const occupants = [
  {
    id: card,
    bounds: { minX: 0, minY: 0, maxX: 100, maxY: 140 },
    visible: true,
  },
];

function projection() {
  return new InteractionProjector(new Map()).project(
    {
      providerId: "renderer",
      pointerId: "primary",
      sequence: 1,
      phase: "down",
      coordinates: { space: "world", point: { x: 50, y: 60 } },
    },
    occupants,
  );
}

describe("MQ-08 bounded topology handoff authority", () => {
  it("projection and request creation do not mutate topology ownership", () => {
    const ownership = new OwnershipRegistry();
    const hand = asTopologyId("hand");
    const drag = asTopologyId("drag");
    ownership.claim(hand, card, ["position.x", "position.y", "rotation"]);

    const projected = projection();
    const request = createTopologyHandoffRequest(projected, {
      requestId: "r1",
      from: hand,
      to: drag,
      degrees: ["position.x", "position.y", "rotation"],
    });

    expect(request.subjectId).toBe(card);
    expect(ownership.ownerOf(card, "position.x")).toBe(hand);
    expect(ownership.ownerOf(card, "rotation")).toBe(hand);
  });

  it("executes only an explicitly allowed authority transfer", () => {
    const ownership = new OwnershipRegistry();
    const hand = asTopologyId("hand");
    const drag = asTopologyId("drag");
    ownership.claim(hand, card, ["position.x", "position.y", "rotation"]);

    const authority = new TopologyHandoffAuthority([
      {
        from: hand,
        to: drag,
        allowedDegrees: ["position.x", "position.y", "rotation"],
      },
    ]);

    const request = createTopologyHandoffRequest(projection(), {
      requestId: "r2",
      from: hand,
      to: drag,
      degrees: ["position.x", "position.y", "rotation"],
    });

    const receipt = authority.execute(request, ownership);

    expect(receipt.subjectId).toBe(card);
    expect(receipt.from).toBe(hand);
    expect(receipt.to).toBe(drag);
    expect(ownership.ownerOf(card, "position.x")).toBe(drag);
    expect(ownership.ownerOf(card, "rotation")).toBe(drag);
  });

  it("rejects authority amplification beyond the allowlisted degree set", () => {
    const ownership = new OwnershipRegistry();
    const hand = asTopologyId("hand");
    const drag = asTopologyId("drag");
    ownership.claim(hand, card, ["position.x", "position.y", "rotation", "shape"]);

    const authority = new TopologyHandoffAuthority([
      {
        from: hand,
        to: drag,
        allowedDegrees: ["position.x", "position.y", "rotation"],
      },
    ]);

    const request = createTopologyHandoffRequest(projection(), {
      requestId: "r3",
      from: hand,
      to: drag,
      degrees: ["position.x", "position.y", "rotation", "shape"],
    });

    expect(() => authority.execute(request, ownership)).toThrow("not allowed");
    expect(ownership.ownerOf(card, "position.x")).toBe(hand);
    expect(ownership.ownerOf(card, "shape")).toBe(hand);
  });

  it("rejects transfer routes absent from authority policy", () => {
    const ownership = new OwnershipRegistry();
    const hand = asTopologyId("hand");
    const playzone = asTopologyId("playzone");
    ownership.claim(hand, card, ["position.x", "position.y"]);

    const authority = new TopologyHandoffAuthority([]);

    const request = createTopologyHandoffRequest(projection(), {
      requestId: "r4",
      from: hand,
      to: playzone,
      degrees: ["position.x", "position.y"],
    });

    expect(() => authority.execute(request, ownership)).toThrow("handoff route");
    expect(ownership.ownerOf(card, "position.x")).toBe(hand);
  });

  it("cannot construct a subject handoff request from a miss", () => {
    const projector = new InteractionProjector(new Map());
    const miss = projector.project(
      {
        providerId: "renderer",
        pointerId: "p",
        sequence: 1,
        phase: "down",
        coordinates: { space: "world", point: { x: 500, y: 500 } },
      },
      occupants,
    );

    expect(() =>
      createTopologyHandoffRequest(miss, {
        requestId: "miss",
        from: asTopologyId("hand"),
        to: asTopologyId("drag"),
        degrees: ["position.x"],
      }),
    ).toThrow("subject");
  });
});
