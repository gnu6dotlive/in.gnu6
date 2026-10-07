import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import type { SceneEntity, SceneState } from "../../src/algebra/types";
import { zeroVelocity } from "../../src/dynamics/math";
import { queryEntityIds, queryRelations } from "../../src/scene/query";
import { diffSceneLifecycle } from "../../src/scene/lifecycle";
import { SceneEngine } from "../../src/scene/engine";
import { OwnershipRegistry, asTopologyId } from "../../src/topology/instance";

function item(
  idValue: string,
  role: SceneEntity["role"],
  visible: boolean,
  parent: string | null = null,
): SceneEntity {
  return {
    id: asEntityId(idValue),
    role,
    visible,
    parent: parent === null ? null : asEntityId(parent),
    pose: {
      position: { x: 0, y: 0 },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    velocity: zeroVelocity(),
    shape: {
      kind: "rect",
      width: 40,
      height: 40,
      radius: 4,
    },
    z: 0,
  };
}

function fixture(): SceneState {
  const root = item("root", "surface", true);
  const a = item("a", "card", true, "root");
  const b = item("b", "card", false, "root");
  const c = item("c", "control", true);

  return new SceneEngine({
    entities: new Map([
      [root.id, root],
      [a.id, a],
      [b.id, b],
      [c.id, c],
    ]),
    // Deliberately non-lexicographic: query results must preserve scene order.
    order: [c.id, root.id, b.id, a.id],
    relations: [
      {
        kind: "attached",
        from: a.id,
        to: root.id,
      },
      {
        kind: "ext:depends-on",
        from: c.id,
        to: a.id,
      },
    ],
  }).snapshot();
}

describe("MQ-01 explicit scene query algebra", () => {
  it("implements set intersection, union and complement over the finite entity set", () => {
    const state = fixture();

    expect(
      queryEntityIds(state, {
        kind: "and",
        terms: [
          { kind: "role", roles: ["card"] },
          { kind: "visible", value: true },
        ],
      }),
    ).toEqual([asEntityId("a")]);

    expect(
      queryEntityIds(state, {
        kind: "or",
        terms: [
          { kind: "role", roles: ["control"] },
          { kind: "ids", ids: [asEntityId("b")] },
        ],
      }),
    ).toEqual([asEntityId("c"), asEntityId("b")]);

    expect(
      queryEntityIds(state, {
        kind: "not",
        term: { kind: "role", roles: ["card"] },
      }),
    ).toEqual([asEntityId("c"), asEntityId("root")]);
  });

  it("queries hierarchy and relation incidence without consumer concepts", () => {
    const state = fixture();

    expect(
      queryEntityIds(state, {
        kind: "parent",
        parent: asEntityId("root"),
      }),
    ).toEqual([asEntityId("b"), asEntityId("a")]);

    expect(
      queryEntityIds(state, {
        kind: "related",
        relationKinds: ["attached"],
        direction: "outgoing",
        counterpart: asEntityId("root"),
      }),
    ).toEqual([asEntityId("a")]);

    expect(
      queryEntityIds(state, {
        kind: "related",
        relationKinds: ["ext:depends-on"],
        direction: "incoming",
        counterpart: asEntityId("c"),
      }),
    ).toEqual([asEntityId("a")]);
  });

  it("returns relations in canonical deterministic order", () => {
    const state = fixture();

    expect(
      queryRelations(state, {
        kinds: ["attached", "ext:depends-on"],
      }).map((relation) => [relation.kind, relation.from, relation.to].join(":")),
    ).toEqual(["attached:a:root", "ext:depends-on:c:a"]);
  });
});

describe("MQ-01 lifecycle set semantics", () => {
  it("classifies birth, persistence and termination as disjoint set partitions", () => {
    const beforeEngine = new SceneEngine();
    const a = item("a", "card", true);
    const b = item("b", "card", true);
    beforeEngine.mutate({ add: [a, b] });

    const before = beforeEngine.snapshot();
    beforeEngine.mutate({
      remove: [a.id],
      add: [item("c", "card", true)],
    });
    const after = beforeEngine.snapshot();

    expect(diffSceneLifecycle(before, after)).toEqual({
      born: [asEntityId("c")],
      persisted: [asEntityId("b")],
      terminated: [asEntityId("a")],
    });
  });

  it("treats authority transfer over a persistent subject as neither birth nor termination", () => {
    const engine = new SceneEngine();
    const a = item("a", "card", true);
    engine.mutate({ add: [a] });

    const ownership = new OwnershipRegistry();
    const layout = asTopologyId("layout");
    const drag = asTopologyId("drag");
    ownership.claim(layout, a.id, ["position.x", "position.y", "rotation"]);

    const before = engine.snapshot();

    ownership.transfer(layout, drag, a.id, ["position.x", "position.y", "rotation"]);

    const after = engine.snapshot();

    expect(diffSceneLifecycle(before, after)).toEqual({
      born: [],
      persisted: [a.id],
      terminated: [],
    });
    expect(ownership.ownerOf(a.id, "position.x")).toBe(drag);
  });

  it("keeps a failed lifecycle mutation fully transactional", () => {
    const root = item("root", "surface", true);
    const child = item("child", "card", true, "root");
    const engine = new SceneEngine();
    engine.mutate({ add: [root, child] });

    const before = engine.snapshot();

    expect(() =>
      engine.mutate({
        remove: [root.id],
      }),
    ).toThrow("PARENT_BELONGS_TO_ENTITY_SET");

    expect(engine.snapshot()).toBe(before);
  });
});
