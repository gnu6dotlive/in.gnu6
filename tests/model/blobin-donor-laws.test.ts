import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { integrateScalar } from "../../src/dynamics/math";
import { createShapeMorphTrack, currentShape } from "../../src/topology/morph";
import { signedArea } from "../../src/topology/contour";
import { BLOBIN_DONOR_FIXTURE } from "../fixtures/blobin-donor";

describe("blob.in donor laws", () => {
  it("preserves the donor's damped semi-implicit spring step at its substep quantum", () => {
    const fixture = BLOBIN_DONOR_FIXTURE.spring;
    const next = integrateScalar(
      fixture.startPosition,
      fixture.startVelocity,
      fixture.target,
      fixture.dtSeconds,
      fixture.stiffness,
      fixture.damping,
      fixture.mass,
    );

    expect(next.velocity).toBeCloseTo(fixture.expectedVelocityAfterOneSubstep, 12);
    expect(next.position).toBeCloseTo(fixture.expectedPositionAfterOneSubstep, 12);
  });

  it("harvests shape continuity as stable subject identity plus normalized closed contours", () => {
    const fixture = BLOBIN_DONOR_FIXTURE.morph;
    const subjectId = asEntityId(fixture.subjectId);
    const track = createShapeMorphTrack(
      subjectId,
      fixture.source,
      fixture.target,
      { stiffness: 170, damping: 22, mass: 1 },
      fixture.sampleCount,
    );

    expect(track.subjectId).toBe(subjectId);
    expect(track.source.vertices).toHaveLength(fixture.sampleCount);
    expect(track.target.vertices).toHaveLength(fixture.sampleCount);
    expect(signedArea(track.source.vertices)).toBeLessThanOrEqual(0);
    expect(signedArea(track.target.vertices)).toBeLessThanOrEqual(0);
    expect(currentShape(track).vertices).toEqual(track.source.vertices);
  });
});
