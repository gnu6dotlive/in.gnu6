import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { cpus, totalmem } from "node:os";
import { dirname, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { dockSurfaceFrame, layoutDocklabInteractiveFan } from "../../consumers/docklab/harness";
import { createSpjWorld, layoutSpjHand } from "../../consumers/spj-deck/harness";
import { asEntityId } from "../../src/algebra/laws";
import type { EntityId, RadialField, SceneEntity } from "../../src/algebra/types";
import { sumFields } from "../../src/dynamics/field";
import { zeroVelocity } from "../../src/dynamics/math";
import { buildSceneReceipt } from "../../src/introspection/frame";
import { SceneEngine } from "../../src/scene/engine";
import { pickTopmost, type SpatialOccupant } from "../../src/spatial/environment";
import { SpatialFrameGraph, asFrameId } from "../../src/spatial/frame";
import {
  createDeformableContourTrack,
  retargetDeformableContour,
  stepDeformableContour,
} from "../../src/topology/deformation";

interface Stats {
  meanMs: number;
  medianMs: number;
  p95Ms: number;
  minMs: number;
  maxMs: number;
}

interface ScalePoint extends Stats {
  count: number;
}

let sink = 0;

function measure(operation: () => void, iterations: number, batches = 9): Stats {
  for (let i = 0; i < Math.max(4, Math.floor(iterations / 10)); i += 1) {
    operation();
  }

  const samples: number[] = [];
  for (let batch = 0; batch < batches; batch += 1) {
    const start = performance.now();
    for (let iteration = 0; iteration < iterations; iteration += 1) {
      operation();
    }
    samples.push((performance.now() - start) / iterations);
  }

  samples.sort((a, b) => a - b);
  const mean = samples.reduce((sum, value) => sum + value, 0) / samples.length;
  const median = samples[Math.floor(samples.length / 2)]!;
  const p95 = samples[Math.min(samples.length - 1, Math.ceil(samples.length * 0.95) - 1)]!;

  return {
    meanMs: mean,
    medianMs: median,
    p95Ms: p95,
    minMs: samples[0]!,
    maxMs: samples.at(-1)!,
  };
}

function entity(id: EntityId, index: number): SceneEntity {
  return {
    id,
    role: "card",
    pose: {
      position: {
        x: (index % 50) * 16,
        y: Math.floor(index / 50) * 20,
      },
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    velocity: zeroVelocity(),
    shape: {
      kind: "rect",
      width: 72,
      height: 112,
      radius: 10,
    },
    z: index,
    visible: true,
    parent: null,
  };
}

function targetedScene(count: number): SceneEngine {
  const engine = new SceneEngine();
  const entities = Array.from({ length: count }, (_, index) =>
    entity(asEntityId("entity-" + index), index),
  );

  engine.mutate({
    add: entities,
    targets: entities.map((item, index) => ({
      entityId: item.id,
      pose: {
        position: {
          x: item.pose.position.x + 120 + (index % 7),
          y: item.pose.position.y - 40,
        },
        rotation: 0.2,
        scale: { x: 1.08, y: 0.94 },
      },
      spring: {
        stiffness: 180,
        damping: 24,
        mass: 1,
      },
    })),
  });

  return engine;
}

function occupantsFor(engine: SceneEngine): SpatialOccupant[] {
  return [...engine.snapshot().entities.values()].map((item) => ({
    id: item.id,
    visible: item.visible,
    bounds: {
      minX: item.pose.position.x - 36,
      minY: item.pose.position.y - 56,
      maxX: item.pose.position.x + 36,
      maxY: item.pose.position.y + 56,
    },
  }));
}

function processMemoryObservation(entityCount: number) {
  const before = process.memoryUsage();
  const engine = targetedScene(entityCount);
  const checksum = engine.snapshot().entities.size + engine.snapshot().targets.size;
  const after = process.memoryUsage();

  return {
    entityCount,
    checksum,
    heapUsedBytes: after.heapUsed,
    rssBytes: after.rss,
    heapDeltaBytes: after.heapUsed - before.heapUsed,
    rssDeltaBytes: after.rss - before.rss,
  };
}

describe("MQ-14 core performance envelope", () => {
  it("characterizes current core scaling without asserting product targets", () => {
    const entityCounts = [24, 128, 512, 2048];

    const fixedStep: ScalePoint[] = entityCounts.map((count) => {
      const engine = targetedScene(count);
      return {
        count,
        ...measure(
          () => {
            engine.step(1 / 60);
            sink += engine.snapshot().time;
          },
          count >= 1024 ? 12 : 30,
        ),
      };
    });

    const fieldsByCount: ScalePoint[] = [8, 32, 128].map((count) => {
      const fields: RadialField[] = Array.from({ length: count }, (_, index) => ({
        id: "field-" + index,
        kind: "radial",
        center: {
          x: (index % 16) * 20,
          y: Math.floor(index / 16) * 24,
        },
        radius: 180,
        strength: 12,
        falloff: 2.2,
      }));

      return {
        count,
        ...measure(() => {
          const vector = sumFields(fields, { x: 137, y: 91 });
          sink += vector.x + vector.y;
        }, 400),
      };
    });

    const contourNodes: ScalePoint[] = [16, 64, 256, 1024].map((count) => {
      let track = createDeformableContourTrack(
        asEntityId("morph-" + count),
        {
          kind: "rect",
          width: 420,
          height: 260,
          radius: 24,
        },
        {
          kind: "rect",
          width: 110,
          height: 180,
          radius: 18,
        },
        {
          stiffness: 180,
          damping: 24,
          mass: 1,
        },
        count,
      );

      return {
        count,
        ...measure(
          () => {
            track = stepDeformableContour(track, 1 / 60);
            sink += track.nodes[0]!.position.x;
          },
          count >= 512 ? 60 : 160,
        ),
      };
    });

    const contourRetarget: ScalePoint[] = [16, 64, 256, 1024].map((count) => {
      let track = createDeformableContourTrack(
        asEntityId("retarget-" + count),
        {
          kind: "rect",
          width: 420,
          height: 260,
          radius: 24,
        },
        {
          kind: "rect",
          width: 110,
          height: 180,
          radius: 18,
        },
        {
          stiffness: 180,
          damping: 24,
          mass: 1,
        },
        count,
      );
      let toggle = false;

      return {
        count,
        ...measure(
          () => {
            track = retargetDeformableContour(
              track,
              toggle
                ? {
                    kind: "rect",
                    width: 420,
                    height: 260,
                    radius: 24,
                  }
                : {
                    kind: "polygon",
                    closed: true,
                    vertices: [
                      { x: 0, y: -130 },
                      { x: 170, y: 0 },
                      { x: 0, y: 130 },
                      { x: -170, y: 0 },
                    ],
                  },
            );
            toggle = !toggle;
            sink += track.target.vertices.length;
          },
          count >= 512 ? 30 : 80,
        ),
      };
    });

    const picking: ScalePoint[] = [100, 500, 2000, 5000].map((count) => {
      const occupants: SpatialOccupant[] = Array.from({ length: count }, (_, index) => ({
        id: asEntityId("pick-" + index),
        visible: true,
        bounds: {
          minX: index * 4,
          minY: 0,
          maxX: index * 4 + 2,
          maxY: 2,
        },
      }));

      return {
        count,
        ...measure(() => {
          const hit = pickTopmost(occupants, { x: -100, y: -100 });
          sink += hit === null ? 1 : 0;
        }, 400),
      };
    });

    const frames = new SpatialFrameGraph();
    const dockFrame = asFrameId("perf:dock");
    frames.add(dockSurfaceFrame(dockFrame, "bottom", { width: 1440, height: 900 }));
    const spjFrames = createSpjWorld(frames, { width: 1280, height: 800 });

    const dockIds = Array.from({ length: 24 }, (_, index) => asEntityId("dock-perf-" + index));
    const spjIds = Array.from({ length: 12 }, (_, index) => asEntityId("spj-perf-" + index));

    const representative = {
      docklab24: measure(() => {
        const poses = layoutDocklabInteractiveFan(frames, dockFrame, dockIds, { x: 180, y: 44 });
        sink += poses[0]!.pose.position.x;
      }, 250),
      spj12: measure(() => {
        const poses = layoutSpjHand(frames, spjFrames.handFrameId, spjIds, spjFrames.viewport);
        sink += poses[0]!.pose.position.x;
      }, 300),
    };

    const observerSceneBaseline = targetedScene(256);
    const observerSceneInstrumented = targetedScene(256);
    const observerOccupants = occupantsFor(observerSceneInstrumented);

    const observerBaseline = measure(() => {
      observerSceneBaseline.step(1 / 60);
      sink += observerSceneBaseline.snapshot().time;
    }, 40);

    const observerInstrumented = measure(() => {
      observerSceneInstrumented.step(1 / 60);
      const receipt = buildSceneReceipt({
        scene: observerSceneInstrumented.snapshot(),
        topologies: [],
        ownership: [],
        constraints: new Map(),
        occupants: observerOccupants,
      });
      sink += receipt.entities.length;
    }, 40);

    const sourceHead = execFileSync("git", ["rev-parse", "HEAD"], {
      encoding: "utf8",
    }).trim();

    const report = {
      schemaVersion: 1,
      observedAt: new Date().toISOString(),
      source: {
        gitHead: sourceHead,
        treeStatus: execFileSync("git", ["status", "--short"], {
          encoding: "utf8",
        }).trim(),
      },
      instrument: {
        node: process.version,
        platform: process.platform,
        arch: process.arch,
        cpuModel: cpus()[0]?.model ?? "unknown",
        logicalCpuCount: cpus().length,
        totalMemoryBytes: totalmem(),
      },
      methodology: {
        unit: "milliseconds-per-operation",
        note: "Development-host characterization only. No hard performance target is implied.",
      },
      fixedStep,
      fieldsByCount,
      contourNodes,
      contourRetarget,
      picking,
      representative,
      observer: {
        entityCount: 256,
        baseline: observerBaseline,
        withSceneReceipt: observerInstrumented,
        medianOverheadRatio: observerInstrumented.medianMs / observerBaseline.medianMs,
      },
      memory: [24, 128, 512, 2048].map(processMemoryObservation),
    };

    const outputPath = resolve(
      process.cwd(),
      process.env.IN_GNU6_MQ14_CORE_OUTPUT ?? "docs/evidence/assets/mq-14-core-envelope.json",
    );
    mkdirSync(dirname(outputPath), { recursive: true });
    writeFileSync(outputPath, JSON.stringify(report, null, 2) + "\n");

    process.stdout.write(
      "IN_GNU6_MQ14_CORE=" +
        JSON.stringify({
          outputPath,
          sourceHead,
          representative,
          observerOverheadRatio: report.observer.medianOverheadRatio,
        }) +
        "\n",
    );

    expect(Number.isFinite(sink)).toBe(true);
    for (const series of [fixedStep, fieldsByCount, contourNodes, contourRetarget, picking]) {
      for (const point of series) {
        expect(point.medianMs).toBeGreaterThanOrEqual(0);
        expect(point.p95Ms).toBeGreaterThanOrEqual(point.medianMs);
      }
    }
    expect(report.observer.medianOverheadRatio).toBeGreaterThan(0);
  });
});
