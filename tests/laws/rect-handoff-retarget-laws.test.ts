import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import {
  planRectHandoff,
  rectGeometryToBounds,
  retargetRectHandoff,
} from "../../src/topology/handoff";

describe("renderer-neutral rectangular handoff retargeting", () => {
  it("retargets the same subject without changing the original source", () => {
    const initial = planRectHandoff({
      subjectId: asEntityId("subject:workspace-1"),
      sequence: "reshape-then-translate",
      source: { x: 100, y: 80, width: 900, height: 600, radius: 12 },
      target: { x: 24, y: 700, width: 92, height: 132, radius: 18 },
      transferShape: { width: 92, height: 132, radius: 18 },
      generation: 4,
    });

    const next = retargetRectHandoff({
      plan: initial,
      target: { x: 1080, y: 280, width: 104, height: 144, radius: 20 },
      transferShape: { width: 104, height: 144, radius: 20 },
      generation: 5,
    });

    expect(next.subjectId).toBe(initial.subjectId);
    expect(next.sequence).toBe(initial.sequence);
    expect(next.generation).toBe(5);
    expect(next.retargetCount).toBe(1);
    expect(rectGeometryToBounds(next.source)).toEqual(rectGeometryToBounds(initial.source));
    expect(rectGeometryToBounds(next.transfer!)).toEqual({
      x: 498,
      y: 308,
      width: 104,
      height: 144,
      radius: 20,
    });
    expect(rectGeometryToBounds(next.target)).toEqual({
      x: 1080,
      y: 280,
      width: 104,
      height: 144,
      radius: 20,
    });
    expect(next.vector).toEqual({ x: 582, y: -28 });
  });

  it("moves the destination-anchored transfer representation when restore target changes", () => {
    const initial = planRectHandoff({
      subjectId: asEntityId("subject:workspace-1"),
      sequence: "translate-then-reshape",
      source: { x: 24, y: 700, width: 92, height: 132, radius: 18 },
      target: { x: 100, y: 80, width: 900, height: 600, radius: 12 },
      transferShape: { width: 92, height: 132, radius: 18 },
      generation: 8,
    });

    const next = retargetRectHandoff({
      plan: initial,
      target: { x: 220, y: 120, width: 720, height: 480, radius: 12 },
      generation: 9,
    });

    expect(rectGeometryToBounds(next.transfer!)).toEqual({
      x: 534,
      y: 294,
      width: 92,
      height: 132,
      radius: 18,
    });
    expect(next.subjectId).toBe(initial.subjectId);
    expect(next.retargetCount).toBe(1);
  });

  it("requires a strictly newer geometry generation", () => {
    const initial = planRectHandoff({
      subjectId: asEntityId("subject:workspace-1"),
      sequence: "direct",
      source: { x: 0, y: 0, width: 80, height: 120, radius: 16 },
      target: { x: 400, y: 240, width: 700, height: 480, radius: 12 },
      generation: 3,
    });

    expect(() =>
      retargetRectHandoff({
        plan: initial,
        target: { x: 420, y: 260, width: 700, height: 480, radius: 12 },
        generation: 3,
      }),
    ).toThrow("newer");
  });
});
