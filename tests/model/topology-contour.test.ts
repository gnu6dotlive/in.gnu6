import { describe, expect, it } from "vitest";
import {
  alignCyclicContours,
  ensureClockwise,
  interpolateContours,
  prepareShapeMorph,
  resampleClosedContour,
  signedArea,
} from "../../src/topology/contour";
import type { PolygonShape, RectShape } from "../../src/algebra/types";

const square: PolygonShape = {
  kind: "polygon",
  closed: true,
  vertices: [
    { x: -10, y: -10 },
    { x: 10, y: -10 },
    { x: 10, y: 10 },
    { x: -10, y: 10 },
  ],
};

describe("Experience contour topology", () => {
  it("normalizes contour winding deterministically", () => {
    const clockwise = ensureClockwise(square.vertices);
    expect(signedArea(clockwise)).toBeLessThanOrEqual(0);
    expect(ensureClockwise([...square.vertices].reverse())).toEqual(clockwise);
  });

  it("resamples a closed boundary to exact requested cardinality", () => {
    const samples = resampleClosedContour(square.vertices, 32);
    expect(samples).toHaveLength(32);
    expect(new Set(samples.map((point) => point.x + ":" + point.y)).size).toBeGreaterThan(4);
  });

  it("aligns cyclically shifted contours without changing their point set", () => {
    const source = resampleClosedContour(square.vertices, 8);
    const shifted = [...source.slice(3), ...source.slice(0, 3)];
    const aligned = alignCyclicContours(source, shifted);

    expect(aligned).toEqual(source);
    expect(new Set(aligned.map((p) => p.x + ":" + p.y))).toEqual(
      new Set(source.map((p) => p.x + ":" + p.y)),
    );
  });

  it("interpolates every point continuously and preserves endpoint truth", () => {
    const source = resampleClosedContour(square.vertices, 8);
    const target = source.map((point) => ({ x: point.x * 2, y: point.y * 0.5 }));

    expect(interpolateContours(source, target, 0)).toEqual(source);
    expect(interpolateContours(source, target, 1)).toEqual(target);

    const half = interpolateContours(source, target, 0.5);
    expect(half[0]!.x).toBeCloseTo((source[0]!.x + target[0]!.x) / 2, 8);
    expect(half[0]!.y).toBeCloseTo((source[0]!.y + target[0]!.y) / 2, 8);
  });

  it("normalizes a window rect and tarot-card rect into morph-compatible cycles", () => {
    const windowShape: RectShape = {
      kind: "rect",
      width: 1120,
      height: 620,
      radius: 14,
    };
    const cardShape: RectShape = {
      kind: "rect",
      width: 72,
      height: 82,
      radius: 16,
    };

    const morph = prepareShapeMorph(windowShape, cardShape, 48);

    expect(morph.source.vertices).toHaveLength(48);
    expect(morph.target.vertices).toHaveLength(48);
    expect(signedArea(morph.source.vertices)).toBeLessThanOrEqual(0);
    expect(signedArea(morph.target.vertices)).toBeLessThanOrEqual(0);
  });
});
