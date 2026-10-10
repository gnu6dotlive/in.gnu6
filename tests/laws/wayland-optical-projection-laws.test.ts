import sceneFixture from "../../experiments/blobin-next-01/CP02-VULKAN/fixtures/scene-stream.v1.json";
import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import type { RendererFrame } from "../../src/renderer/contract";
import {
  WaylandOpticalSemanticProjector,
  type CapturedWaylandPointerButton,
  type WaylandSurfaceCalibration,
} from "../../src/interaction/wayland-optical-projection";

const frame = sceneFixture.initial as unknown as RendererFrame;
const target = asEntityId("cavity-control");
const calibration: WaylandSurfaceCalibration = {
  id: "oracle03-fixture-stage-chart",
  surfaceToken: "s1",
  viewportWidth: 2560,
  viewportHeight: 1440,
  worldOrigin: { x: -200, y: -150 },
  worldUPerPixel: { x: 400 / 2560, y: 0 },
  worldVPerPixel: { x: 0, y: 300 / 1440 },
  sceneFrameSequence: 10,
};
function captured(
  sequence: number,
  x: number,
  y: number,
  pressed = true,
): CapturedWaylandPointerButton {
  return {
    provider_id: "wayland-gpu05-owned-surface",
    pointer_id: "primary",
    provider_sequence: sequence,
    wl_surface_token: calibration.surfaceToken,
    pressed,
    surface_x: x,
    surface_y: y,
    coordinates_space: "surface_logical_pixels",
  };
}
function projector(binding = calibration, source = frame) {
  return new WaylandOpticalSemanticProjector(source, binding, target);
}

describe("GPU05 ORACLE-03 — bounded read-only semantic Wayland observation", () => {
  it("reuses the in.gnu6 spatial projector: center maps to exact fixture control", () => {
    const observation = projector().observe(captured(1, 1280, 720));
    expect(observation.projection.semantic.subjectId).toBe(target);
    expect(observation.projection.semantic.worldPoint).toEqual({ x: 0, y: 0 });
    expect(observation.projection.provenance).toEqual({
      providerId: "wayland-gpu05-owned-surface",
      coordinateSpace: "frame",
    });
    expect(observation.exactTargetObserved).toBe(true);
    expect(observation.photodiodeQualified).toBe(false);
    expect(observation.mutationAuthorityGranted).toBe(false);
    expect(observation.sceneSequence).toBe(10);
  });

  it("a miss cannot be promoted to the optical target", () => {
    const observation = projector().observe(captured(1, 100, 100));
    expect(observation.projection.semantic.subjectId).toBe(asEntityId("stage"));
    expect(observation.exactTargetObserved).toBe(false);
  });

  it("a released button is an observation, not a new optical trial", () => {
    const adapter = projector();
    adapter.observe(captured(1, 1280, 720, true));
    expect(adapter.observe(captured(2, 1280, 720, false)).exactTargetObserved).toBe(false);
  });

  it("rejects non-monotonic and replayed Wayland provider sequences", () => {
    const adapter = projector();
    adapter.observe(captured(4, 1280, 720));
    expect(() => adapter.observe(captured(4, 1280, 720))).toThrow("monotonic");
    expect(() => adapter.observe(captured(3, 1280, 720))).toThrow("monotonic");
  });

  it("rejects cross-surface and foreign-provider observations", () => {
    const adapter = projector();
    expect(() => adapter.observe({ ...captured(1, 1280, 720), wl_surface_token: "other" })).toThrow(
      "PROVIDER_OR_SURFACE_MISMATCH",
    );
    expect(() =>
      adapter.observe({ ...captured(2, 1280, 720), provider_id: "screenshot" as never }),
    ).toThrow("PROVIDER_OR_SURFACE_MISMATCH");
  });

  it("rejects an uncalibrated, stale or noninvertible viewport", () => {
    expect(() => projector({ ...calibration, sceneFrameSequence: 11 })).toThrow("STALE");
    expect(() => projector({ ...calibration, viewportWidth: 0 })).toThrow("VIEWPORT");
    expect(() => projector({ ...calibration, worldVPerPixel: { x: 0, y: 0 } })).toThrow(
      "NONINVERTIBLE",
    );
    expect(() => projector({ ...calibration, worldOrigin: { x: Number.NaN, y: 0 } })).toThrow(
      "NONFINITE",
    );
  });

  it("rejects pointer coordinates beyond the calibrated surface extent", () => {
    const adapter = projector();
    for (const [x, y] of [
      [-1, 720],
      [1280, 1441],
      [Number.NaN, 720],
      [4097, 720],
    ]) {
      expect(() => adapter.observe(captured(1, x!, y!))).toThrow("OUTSIDE_CALIBRATED_VIEWPORT");
    }
  });

  it("refuses hidden, absent and wrongly typed scene-control targets", () => {
    expect(
      () => new WaylandOpticalSemanticProjector(frame, calibration, asEntityId("does-not-exist")),
    ).toThrow("TARGET_NOT_VISIBLE");
    const hidden = {
      ...frame,
      entities: frame.entities.map((e) => (e.entityId === target ? { ...e, visible: false } : e)),
    };
    expect(() => projector(calibration, hidden)).toThrow("TARGET_NOT_VISIBLE");
    const role = {
      ...frame,
      entities: frame.entities.map((e) =>
        e.entityId === target ? { ...e, role: "decoration" as const } : e,
      ),
    };
    expect(() => projector(calibration, role)).toThrow("TARGET_NOT_VISIBLE");
  });

  it("rejects rotated/nonrectangular pick geometry rather than approximating", () => {
    const rotated = {
      ...frame,
      entities: frame.entities.map((e) =>
        e.entityId === target ? { ...e, worldPose: { ...e.worldPose, rotation: 0.4 } } : e,
      ),
    };
    expect(() => projector(calibration, rotated)).toThrow("SCENE_GEOMETRY_UNSUPPORTED");
  });

  it("retains the calibration identity as evidence, not a mutation grant", () => {
    const result = projector().observe(captured(1, 1280, 720));
    expect(result.calibrationId).toBe("oracle03-fixture-stage-chart");
    expect("execute" in result).toBe(false);
    expect("sceneMutation" in result).toBe(false);
  });
});
