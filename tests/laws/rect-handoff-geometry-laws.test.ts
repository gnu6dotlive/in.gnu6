import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import {
  planRectHandoff,
  rectBoundsToGeometry,
  rectGeometryToBounds,
} from "../../src/topology/handoff";

describe("renderer-neutral rectangular handoff geometry", () => {
  it("round-trips world bounds without moving the subject center", () => {
    const bounds = { x: 120, y: 80, width: 640, height: 360, radius: 14 };
    const geometry = rectBoundsToGeometry(bounds);

    expect(geometry.center).toEqual({ x: 440, y: 260 });
    expect(geometry.shape).toEqual({ kind: "rect", width: 640, height: 360, radius: 14 });
    expect(rectGeometryToBounds(geometry)).toEqual(bounds);
  });

  it("reshapes at the source before translating the same subject", () => {
    const plan = planRectHandoff({
      subjectId: asEntityId("subject:scratch-2"),
      sequence: "reshape-then-translate",
      source: { x: 100, y: 80, width: 900, height: 600, radius: 12 },
      target: { x: 24, y: 700, width: 92, height: 132, radius: 18 },
      transferShape: { width: 92, height: 132, radius: 18 },
    });
    expect(plan.subjectId).toBe(asEntityId("subject:scratch-2"));
    expect(rectGeometryToBounds(plan.source)).toEqual({
      x: 100,
      y: 80,
      width: 900,
      height: 600,
      radius: 12,
    });
    expect(rectGeometryToBounds(plan.transfer!)).toEqual({
      x: 504,
      y: 314,
      width: 92,
      height: 132,
      radius: 18,
    });
    expect(rectGeometryToBounds(plan.target)).toEqual({
      x: 24,
      y: 700,
      width: 92,
      height: 132,
      radius: 18,
    });
    expect(plan.vector).toEqual({ x: -480, y: 386 });
  });

  it("translates a transfer shape before expanding at the destination", () => {
    const plan = planRectHandoff({
      subjectId: asEntityId("subject:scratch-2"),
      sequence: "translate-then-reshape",
      source: { x: 24, y: 700, width: 92, height: 132, radius: 18 },
      target: { x: 100, y: 80, width: 900, height: 600, radius: 12 },
      transferShape: { width: 92, height: 132, radius: 18 },
    });
    expect(rectGeometryToBounds(plan.transfer!)).toEqual({
      x: 504,
      y: 314,
      width: 92,
      height: 132,
      radius: 18,
    });
    expect(plan.vector).toEqual({ x: 480, y: -386 });
  });

  it("supports direct handoff without manufacturing an intermediate representation", () => {
    const plan = planRectHandoff({
      subjectId: asEntityId("subject:direct"),
      sequence: "direct",
      source: { x: 0, y: 0, width: 80, height: 120, radius: 16 },
      target: { x: 400, y: 240, width: 700, height: 480, radius: 12 },
    });

    expect(plan.transfer).toBeNull();
    expect(plan.vector).toEqual({ x: 710, y: 420 });
  });

  it("rejects non-finite or non-positive geometry before projection", () => {
    expect(() =>
      planRectHandoff({
        subjectId: asEntityId("subject:bad"),
        sequence: "direct",
        source: { x: 0, y: 0, width: 0, height: 20, radius: 4 },
        target: { x: 10, y: 10, width: 20, height: 20, radius: 4 },
      }),
    ).toThrow("positive");

    expect(() =>
      rectBoundsToGeometry({ x: Number.NaN, y: 0, width: 20, height: 20, radius: 4 }),
    ).toThrow("finite");
  });
});
