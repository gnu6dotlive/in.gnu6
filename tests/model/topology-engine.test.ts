import { describe, expect, it } from "vitest";
import {
  applyDeckPointerField,
  assertPermutation,
  type DeckSpec,
  layoutTarotFan,
  previewDeckOrder,
} from "../../src/deck/deck";
import { radialWeight, sumFields } from "../../src/dynamics/field";
import { asEntityId, inspectSceneLaws } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { SceneEngine } from "../../src/scene/engine";
import type { EntityId, RadialField, SceneEntity, SceneState } from "../../src/algebra/types";

const ids = (...values: string[]): EntityId[] => values.map(asEntityId);

const deckSpec: DeckSpec = {
  center: { x: 500, y: 700 },
  cardWidth: 140,
  cardHeight: 220,
  overlap: 72,
  fanAngle: Math.PI / 8,
  fanDepth: 36,
  hoverRadius: 180,
  hoverScale: 1.14,
  hoverLift: 42,
  liftDirection: { x: 0, y: -1 },
  neighborYield: 24,
  neighborFalloff: 2.6,
};

function card(id: string, x = 0): SceneEntity {
  return {
    id: asEntityId(id),
    role: "card",
    pose: {
      position: { x, y: 0 },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    velocity: zeroVelocity(),
    shape: { kind: "rect", width: 100, height: 160, radius: 12 },
    z: 0,
    visible: true,
    parent: null,
  };
}

describe("Experience scenic/topological kernel", () => {
  it("treats deck order as a permutation, never a multiset", () => {
    expect(() => assertPermutation(ids("a", "b", "c"))).not.toThrow();
    expect(() => assertPermutation(ids("a", "b", "a"))).toThrow("Deck order must be injective");
  });

  it("lays out an odd tarot hand symmetrically around its scene center", () => {
    const poses = layoutTarotFan(ids("a", "b", "c", "d", "e"), deckSpec);

    const left = poses[0]!;
    const middle = poses[2]!;
    const right = poses[4]!;

    expect(middle.pose.position.x).toBe(deckSpec.center.x);
    expect(middle.pose.rotation).toBe(0);

    expect(left.pose.position.x + right.pose.position.x).toBeCloseTo(2 * deckSpec.center.x, 8);
    expect(left.pose.rotation).toBeCloseTo(-right.pose.rotation, 8);
    expect(left.pose.position.y).toBeCloseTo(right.pose.position.y, 8);
  });

  it("produces a continuous pointer field instead of a hovered boolean discontinuity", () => {
    const base = layoutTarotFan(ids("a", "b", "c"), deckSpec);
    const center = base[1]!.pose.position;

    const atCenter = applyDeckPointerField(base, center, deckSpec);
    const onePixelAway = applyDeckPointerField(base, { x: center.x + 1, y: center.y }, deckSpec);

    expect(atCenter[1]!.influence).toBeCloseTo(1, 8);
    expect(onePixelAway[1]!.influence).toBeLessThan(1);
    expect(onePixelAway[1]!.influence).toBeGreaterThan(0.999);
    expect(Math.abs(atCenter[1]!.pose.scale.x - onePixelAway[1]!.pose.scale.x)).toBeLessThan(0.001);
  });

  it("uses hysteresis so pointer noise cannot thrash reorder topology", () => {
    const order = ids("a", "b", "c", "d");
    const slots = [100, 200, 300];

    const initial = previewDeckOrder(order, asEntityId("b"), 230, slots, null, 12);
    const noisyLeft = previewDeckOrder(order, asEntityId("b"), 194, slots, initial.index, 12);
    const crossedLeft = previewDeckOrder(order, asEntityId("b"), 180, slots, noisyLeft.index, 12);

    expect(noisyLeft.index).toBe(initial.index);
    expect(crossedLeft.index).toBeLessThan(noisyLeft.index);
    expect(new Set(crossedLeft.order)).toEqual(new Set(order));
  });

  it("superposes spatial fields algebraically", () => {
    const fields: RadialField[] = [
      {
        id: "left",
        kind: "radial",
        center: { x: -10, y: 0 },
        radius: 100,
        strength: 5,
        falloff: 2,
      },
      {
        id: "right",
        kind: "radial",
        center: { x: 10, y: 0 },
        radius: 100,
        strength: 5,
        falloff: 2,
      },
    ];

    const origin = { x: 0, y: 0 };
    expect(radialWeight(fields[0]!, origin)).toBeGreaterThan(0);
    const sum = sumFields(fields, origin);
    expect(sum.x).toBeCloseTo(0, 8);
    expect(sum.y).toBeCloseTo(0, 8);
  });

  it("rejects scene states whose traversal order is not a permutation of E", () => {
    const a = card("a");
    const b = card("b");

    const invalid: SceneState = {
      time: 0,
      entities: new Map([
        [a.id, a],
        [b.id, b],
      ]),
      relations: [],
      order: [a.id, a.id],
      targets: new Map(),
      fields: [],
    };

    expect(inspectSceneLaws(invalid).map((v) => v.law)).toContain("ORDER_IS_INJECTIVE");
    expect(inspectSceneLaws(invalid).map((v) => v.law)).toContain(
      "ORDER_IS_PERMUTATION_OF_ENTITIES",
    );
  });

  it("commits scene mutations transactionally and forbids duplicate identity", () => {
    const engine = new SceneEngine();
    engine.mutate({ add: [card("a"), card("b")] });

    expect(engine.snapshot().order).toEqual(ids("a", "b"));

    expect(() => engine.mutate({ add: [card("a")] })).toThrow("Duplicate scene identity");

    expect(engine.snapshot().order).toEqual(ids("a", "b"));
  });

  it("moves targeted poses continuously under one spring owner", () => {
    const engine = new SceneEngine();
    const a = card("a", 0);
    engine.mutate({ add: [a] });

    engine.setTarget({
      entityId: a.id,
      pose: {
        position: { x: 100, y: 0 },
        rotation: 0,
        scale: { x: 1.2, y: 1.2 },
      },
      spring: { stiffness: 180, damping: 22, mass: 1 },
    });

    const first = engine.step(1 / 60).entities.get(a.id)!;
    const second = engine.step(1 / 60).entities.get(a.id)!;

    expect(first.pose.position.x).toBeGreaterThan(0);
    expect(first.pose.position.x).toBeLessThan(100);
    expect(second.pose.position.x).toBeGreaterThan(first.pose.position.x);
    expect(first.pose.scale.x).toBeGreaterThan(1);
    expect(first.pose.scale.x).toBeLessThan(1.2);
  });

  it("preserves finite scene state after a very large wall-clock gap", () => {
    const engine = new SceneEngine();
    const a = card("a", 0);
    engine.mutate({ add: [a] });
    engine.setTarget({
      entityId: a.id,
      pose: {
        position: { x: 1000, y: -500 },
        rotation: 1,
        scale: { x: 1.5, y: 0.8 },
      },
      spring: { stiffness: 220, damping: 28, mass: 0.8 },
    });

    const state = engine.step(30);
    expect(inspectSceneLaws(state)).toEqual([]);
    expect(Number.isFinite(state.entities.get(a.id)!.pose.position.x)).toBe(true);
  });
});
