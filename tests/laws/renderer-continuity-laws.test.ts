import { describe, expect, it } from "vitest";
import { inspectRendererContinuity, type ContinuityTrace } from "../../src/renderer/continuity";

function sample(
  index: number,
  semanticExists: boolean,
  handleId: string | null,
  overrides: Partial<ContinuityTrace["samples"][number]["subject"]> = {},
) {
  return {
    index,
    sceneTime: index / 60,
    subject: {
      semanticExists,
      handleId,
      mounted: semanticExists,
      visible: semanticExists,
      renderable: semanticExists,
      alpha: semanticExists ? 1 : 0,
      finiteGeometry: semanticExists,
      ...overrides,
    },
  };
}

describe("MQ-13 renderer continuity laws", () => {
  it("accepts stable provider continuity across a semantic transition", () => {
    const trace: ContinuityTrace = {
      entityId: "card",
      termination: "continuous",
      samples: [sample(0, true, "h1"), sample(1, true, "h1"), sample(2, true, "h1")],
    };

    expect(inspectRendererContinuity(trace)).toEqual([]);
  });

  it("rejects provider handle replacement while semantic identity survives", () => {
    const trace: ContinuityTrace = {
      entityId: "card",
      termination: "continuous",
      samples: [sample(0, true, "h1"), sample(1, true, "h2")],
    };

    expect(inspectRendererContinuity(trace).map((v) => v.law)).toContain(
      "HANDLE_STABLE_WHILE_SEMANTIC_IDENTITY_LIVES",
    );
  });

  it("rejects mount, visibility, alpha and finite-geometry cut-offs", () => {
    const trace: ContinuityTrace = {
      entityId: "card",
      termination: "continuous",
      samples: [
        sample(0, true, "h1"),
        sample(1, true, "h1", { mounted: false }),
        sample(2, true, "h1", { visible: false }),
        sample(3, true, "h1", { alpha: 0 }),
        sample(4, true, "h1", { finiteGeometry: false }),
      ],
    };

    const laws = inspectRendererContinuity(trace).map((v) => v.law);
    expect(laws).toContain("PROVIDER_MOUNTED_WHILE_SEMANTIC_IDENTITY_LIVES");
    expect(laws).toContain("PROVIDER_VISIBLE_WHILE_SEMANTIC_IDENTITY_LIVES");
    expect(laws).toContain("PROVIDER_NONZERO_ALPHA_WHILE_SEMANTIC_IDENTITY_LIVES");
    expect(laws).toContain("PROVIDER_GEOMETRY_FINITE");
  });

  it("allows absence before semantic birth, then requires uninterrupted continuity", () => {
    const trace: ContinuityTrace = {
      entityId: "card",
      termination: "continuous",
      samples: [sample(0, false, null), sample(1, true, "h1"), sample(2, true, "h1")],
    };

    expect(inspectRendererContinuity(trace)).toEqual([]);
  });

  it("allows one-way semantic termination but rejects resurrection", () => {
    const legal: ContinuityTrace = {
      entityId: "card",
      termination: "semantic-termination",
      samples: [
        sample(0, true, "h1"),
        sample(1, true, "h1"),
        sample(2, false, null),
        sample(3, false, null),
      ],
    };
    expect(inspectRendererContinuity(legal)).toEqual([]);

    const resurrection: ContinuityTrace = {
      ...legal,
      samples: [sample(0, true, "h1"), sample(1, false, null), sample(2, true, "h2")],
    };

    expect(inspectRendererContinuity(resurrection).map((v) => v.law)).toContain(
      "SEMANTIC_TERMINATION_IS_ONE_WAY",
    );
  });

  it("rejects disappearance when a transition is declared continuous", () => {
    const trace: ContinuityTrace = {
      entityId: "card",
      termination: "continuous",
      samples: [sample(0, true, "h1"), sample(1, false, null)],
    };

    expect(inspectRendererContinuity(trace).map((v) => v.law)).toContain(
      "SEMANTIC_IDENTITY_PERSISTS_FOR_CONTINUOUS_TRANSITION",
    );
  });
});
