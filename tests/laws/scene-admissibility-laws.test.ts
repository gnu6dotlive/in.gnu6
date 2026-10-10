import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import type { SceneEntity, SceneRelationKind } from "../../src/algebra/types";
import { zeroVelocity } from "../../src/dynamics/math";
import { SceneEngine } from "../../src/scene/engine";

function entity(idValue: string, parentValue: string | null = null): SceneEntity {
  return {
    id: asEntityId(idValue),
    role: "surface",
    pose: {
      position: { x: 0, y: 0 },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    velocity: zeroVelocity(),
    shape: {
      kind: "rect",
      width: 100,
      height: 100,
      radius: 8,
    },
    z: 0,
    visible: true,
    parent: parentValue === null ? null : asEntityId(parentValue),
  };
}

describe("MQ-01 scene admissibility laws", () => {
  it("rejects a parent cycle at scene admission rather than only during world-pose query", () => {
    const a = entity("a", "b");
    const b = entity("b", "a");

    expect(
      () =>
        new SceneEngine({
          entities: new Map([
            [a.id, a],
            [b.id, b],
          ]),
          order: [a.id, b.id],
        }),
    ).toThrow("HIERARCHY_IS_ACYCLIC");
  });

  it("treats scene relations as a mathematical set, not a multiset", () => {
    const a = entity("a");
    const b = entity("b");
    const engine = new SceneEngine();

    expect(() =>
      engine.mutate({
        add: [a, b],
        relations: [
          {
            kind: "attached",
            from: b.id,
            to: a.id,
          },
          {
            kind: "attached",
            from: b.id,
            to: a.id,
          },
        ],
      }),
    ).toThrow("RELATIONS_ARE_SET");
  });

  it("rejects unnamed relation semantics instead of accepting an opaque empty string", () => {
    const a = entity("a");
    const b = entity("b");
    const engine = new SceneEngine();

    expect(() =>
      engine.mutate({
        add: [a, b],
        relations: [
          {
            kind: "" as unknown as SceneRelationKind,
            from: b.id,
            to: a.id,
          },
        ],
      }),
    ).toThrow("RELATION_KIND_IS_EXPLICIT");
  });
});
