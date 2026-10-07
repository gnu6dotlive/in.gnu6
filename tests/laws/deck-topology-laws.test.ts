import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import {
  applyDeckFocusField,
  deckOcclusionPairs,
  insertDeckCard,
  layoutCardSpread,
  layoutCardStack,
  planDeckOrderTransition,
  removeDeckCard,
  tiltFromVelocity,
  transferDeckCard,
} from "../../src/deck/deck";

const ids = (...values: string[]) => values.map(asEntityId);

describe("MQ-06 deck topology family", () => {
  it("lays out a deterministic stack with increasing z-order", () => {
    const poses = layoutCardStack(ids("a", "b", "c"), {
      origin: { x: 100, y: 200 },
      offset: { x: 8, y: -6 },
      rotationStep: 0,
      scaleStep: 0.02,
    });

    expect(poses.map((pose) => pose.z)).toEqual([0, 1, 2]);
    expect(poses[0]!.pose.position).toEqual({ x: 100, y: 200 });
    expect(poses[2]!.pose.position).toEqual({ x: 116, y: 188 });
    expect(poses[2]!.pose.scale.x).toBeGreaterThan(poses[0]!.pose.scale.x);
  });

  it("lays out a symmetric linear spread independently from tarot fan", () => {
    const poses = layoutCardSpread(ids("a", "b", "c", "d", "e"), {
      center: { x: 500, y: 300 },
      gap: 90,
    });

    expect(poses[2]!.pose.position).toEqual({ x: 500, y: 300 });
    expect(poses[0]!.pose.position.x + poses[4]!.pose.position.x).toBe(1000);
    expect(poses.every((pose) => pose.pose.rotation === 0)).toBe(true);
  });

  it("derives stack occlusion from card geometry and z-order", () => {
    const poses = layoutCardStack(ids("a", "b", "c"), {
      origin: { x: 0, y: 0 },
      offset: { x: 18, y: 0 },
      rotationStep: 0,
      scaleStep: 0,
    });

    expect(deckOcclusionPairs(poses, { width: 72, height: 112 })).toEqual([
      { occluder: asEntityId("b"), occluded: asEntityId("a") },
      { occluder: asEntityId("c"), occluded: asEntityId("a") },
      { occluder: asEntityId("c"), occluded: asEntityId("b") },
    ]);
  });

  it("plans insertion and removal without changing surviving identities", () => {
    const before = ids("a", "b", "c");
    const inserted = insertDeckCard(before, asEntityId("x"), 1);
    const insertionPlan = planDeckOrderTransition(before, inserted);

    expect(inserted).toEqual(ids("a", "x", "b", "c"));
    expect(insertionPlan.entering).toEqual([asEntityId("x")]);
    expect(insertionPlan.leaving).toEqual([]);
    expect(insertionPlan.moved).toContainEqual({
      id: asEntityId("b"),
      fromIndex: 1,
      toIndex: 2,
    });

    const removed = removeDeckCard(inserted, asEntityId("b"));
    const removalPlan = planDeckOrderTransition(inserted, removed);

    expect(removed).toEqual(ids("a", "x", "c"));
    expect(removalPlan.leaving).toEqual([asEntityId("b")]);
  });

  it("transfers one persistent identity between decks", () => {
    const result = transferDeckCard(ids("a", "b", "c"), ids("x", "y"), asEntityId("b"), 1);

    expect(result.source).toEqual(ids("a", "c"));
    expect(result.destination).toEqual(ids("x", "b", "y"));
    expect(
      [...result.source, ...result.destination].filter((id) => id === asEntityId("b")),
    ).toHaveLength(1);
  });

  it("applies focus influence without requiring a pointer observation", () => {
    const poses = layoutCardSpread(ids("a", "b", "c"), {
      center: { x: 0, y: 0 },
      gap: 80,
    });

    const focused = applyDeckFocusField(poses, asEntityId("b"), {
      hoverRadius: 180,
      hoverScale: 1.12,
      hoverLift: 30,
      liftDirection: { x: 0, y: -1 },
      neighborYield: 18,
      neighborFalloff: 2.5,
    });

    expect(focused[1]!.influence).toBeCloseTo(1, 8);
    expect(focused[1]!.pose.scale.x).toBeCloseTo(1.12, 8);
    expect(focused[0]!.influence).toBeGreaterThan(0);
    expect(focused[2]!.influence).toBeGreaterThan(0);
  });

  it("treats lift direction as consumer-provided local geometry", () => {
    const poses = layoutCardSpread(ids("a", "b", "c"), {
      center: { x: 0, y: 0 },
      gap: 80,
    });

    const upward = applyDeckFocusField(poses, asEntityId("b"), {
      hoverRadius: 180,
      hoverScale: 1,
      hoverLift: 30,
      liftDirection: { x: 0, y: -1 },
      neighborYield: 0,
      neighborFalloff: 2.5,
    });
    const inward = applyDeckFocusField(poses, asEntityId("b"), {
      hoverRadius: 180,
      hoverScale: 1,
      hoverLift: 30,
      liftDirection: { x: 1, y: 0 },
      neighborYield: 0,
      neighborFalloff: 2.5,
    });

    expect(upward[1]!.pose.position).toEqual({ x: 0, y: -30 });
    expect(inward[1]!.pose.position).toEqual({ x: 30, y: 0 });
  });

  it("derives bounded optional tilt from velocity", () => {
    expect(tiltFromVelocity({ x: 400, y: 0 }, { gain: 0.001, maxTilt: 0.18 })).toBe(0.18);
    expect(tiltFromVelocity({ x: -400, y: 0 }, { gain: 0.001, maxTilt: 0.18 })).toBe(-0.18);
    expect(tiltFromVelocity({ x: 50, y: 999 }, { gain: 0.001, maxTilt: 0.18 })).toBeCloseTo(
      0.05,
      8,
    );
  });
});
