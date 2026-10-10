import { describe, expect, it } from "vitest";
import sceneFixture from "../../experiments/blobin-next-01/CP02-VULKAN/fixtures/scene-stream.v1.json";
import { asEntityId } from "../../src/algebra/laws";
import type { RendererFrame } from "../../src/renderer/contract";
import { applyRendererDelta } from "../../src/renderer/delta";
import { Oracle05SemanticSession } from "../../src/interaction/wayland-optical-live";

let frame = sceneFixture.initial as unknown as RendererFrame;
for (const delta of sceneFixture.deltas) frame = applyRendererDelta(frame, delta as never);
function start() {
  return new Oracle05SemanticSession(frame, asEntityId("cavity-control"));
}
const marker = {
  kind: "scene_control_2d_overlay",
  entity_id: "cavity-control",
  scene_frame_sequence: 13,
  rect_uv: [0.47, 0.46, 0.53, 0.54],
  viewport_width: 2560,
  viewport_height: 1440,
  render_origin: "rust_scene_stream_projection",
  independent_pixel_capture: false,
};
function hello(visual = marker) {
  return {
    type: "hello",
    schema: "blobin.gpu05.semantic-witness.v1",
    surface_token: "gpu05-reference-owned-test",
    scene_frame_sequence: 13,
    scene_source: "in.gnu6-verified-scene-stream-v1",
    coordinate_space: "surface_logical_pixels",
    consumer_authority: "read_only_observation",
    calibration_status: "fixture_declared_not_physically_verified",
    visual_marker: visual,
  };
}
function button(sequence: number, x: number, y: number) {
  return {
    type: "wayland_button",
    button: "BTN_LEFT",
    provider_id: "wayland-gpu05-owned-surface",
    pointer_id: "primary",
    provider_sequence: sequence,
    scene_frame_sequence: 13,
    pressed: true,
    wl_surface_token: "gpu05-reference-owned-test",
    surface_x: x,
    surface_y: y,
    coordinates_space: "surface_logical_pixels",
    timing_class: "unattested_wayland_software_input",
  };
}
describe("GPU05 ORACLE-06 — 2D scene-owned screen overlay contracts", () => {
  it("valid marker and actual scene geometry agree; centre is target", () => {
    const s = start();
    const ready = s.accept(hello());
    expect(ready.scene_visual_marker_contract_consistent).toBe(true);
    const hit = s.accept(button(1, 1280, 720));
    expect(hit.semantic_entity_id).toBe("cavity-control");
    expect(hit.geometric_target_candidate).toBe(true);
    expect(hit.scene_visual_marker_contract_consistent).toBe(true);
    expect(hit.independently_verified_screen_pixel).toBe(false);
    expect(hit.photodiode_qualified).toBe(false);
    expect(hit.cp03_pass).toBe(false);
  });
  it("outside target rectangle remains stage", () => {
    const s = start();
    s.accept(hello());
    const miss = s.accept(button(1, 400, 1100));
    expect(miss.semantic_entity_id).toBe("stage");
    expect(miss.geometric_target_candidate).toBe(false);
  });
  it("renderer's altered rect cannot become scene truth", () => {
    const s = start();
    expect(() => s.accept(hello({ ...marker, rect_uv: [0.1, 0.1, 0.3, 0.3] }))).toThrow(
      "CONTRADICTS",
    );
  });
  it("wrong frame sequence is rejected before projection", () => {
    const s = start();
    expect(() => s.accept(hello({ ...marker, scene_frame_sequence: 12 }))).toThrow(
      "MARKER_BINDING",
    );
  });
  it("marker must never attest display pixel capture", () => {
    const s = start();
    expect(() => s.accept(hello({ ...marker, independent_pixel_capture: true }))).toThrow(
      "MARKER_BINDING",
    );
  });
  it("nonfinite or malformed visible coordinates fail closed", () => {
    const s = start();
    expect(() => s.accept(hello({ ...marker, rect_uv: [0.47, NaN, 0.53, 0.54] }))).toThrow(
      "MARKER_BINDING",
    );
    expect(() => s.accept(hello({ ...marker, rect_uv: [0.47, 0.46] }))).toThrow("MARKER_BINDING");
  });
  it("cross-surface pointer cannot claim target", () => {
    const s = start();
    s.accept(hello());
    expect(() => s.accept({ ...button(1, 1280, 720), wl_surface_token: "another" })).toThrow(
      "BINDING",
    );
  });
  it("without visual marker, older ORACLE-05 output cannot inherit its gate", () => {
    const s = start();
    const readiness = s.accept({ ...hello(), visual_marker: null });
    expect(readiness.scene_visual_marker_contract_consistent).toBe(false);
    const result = s.accept(button(1, 1280, 720));
    expect(result.scene_visual_marker_contract_consistent).toBe(false);
  });
});
