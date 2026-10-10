import { describe, expect, it } from "vitest";
import sceneFixture from "../../experiments/blobin-next-01/CP02-VULKAN/fixtures/scene-stream.v1.json";
import type { RendererFrame } from "../../src/renderer/contract";
import { applyRendererDelta } from "../../src/renderer/delta";
import { asEntityId } from "../../src/algebra/laws";
import { Oracle05SemanticSession } from "../../src/interaction/wayland-optical-live";

let current = sceneFixture.initial as unknown as RendererFrame;
for (const delta of sceneFixture.deltas) {
  current = applyRendererDelta(current, delta as never);
}
const seq = current.sequence;
function session() {
  return new Oracle05SemanticSession(current, asEntityId("cavity-control"));
}
function hello(surface_token = "gpu05-owned-fixture-session-10") {
  return {
    type: "hello",
    schema: "blobin.gpu05.semantic-witness.v1",
    surface_token,
    scene_frame_sequence: seq,
    scene_source: "in.gnu6-verified-scene-stream-v1",
    coordinate_space: "surface_logical_pixels",
    consumer_authority: "read_only_observation",
    calibration_status: "fixture_declared_not_physically_verified",
  };
}
function button(sequence: number, x = 1280, y = 720) {
  return {
    type: "wayland_button",
    button: "BTN_LEFT",
    timing_class: "unattested_wayland_software_input",
    scene_frame_sequence: seq,
    wl_surface_token: "gpu05-owned-fixture-session-10",
    provider_id: "wayland-gpu05-owned-surface",
    pointer_id: "primary",
    provider_sequence: sequence,
    pressed: true,
    surface_x: x,
    surface_y: y,
    coordinates_space: "surface_logical_pixels",
  };
}
describe("ORACLE-05 — live observation remains read-only", () => {
  it("accepts bound sequence from actual fixture final frame", () => {
    expect(seq).toBe(13);
    const s = session();
    expect(s.accept(hello()).type).toBe("observer_ready");
    const result = s.accept(button(1));
    expect(result.type).toBe("semantic_pointer_observation");
    expect(result.native_wayland_input_observed).toBe(false);
    expect(result.sender_claimed_wayland_input).toBe(true);
    expect(result.semantic_projection_executed).toBe(true);
    expect(result.cp03_pass).toBe(false);
    expect(result.mutation_authority_granted).toBe(false);
    expect(result.physical_chart_qualified).toBe(false);
    expect(s.received).toBe(1);
  });
  it("rejects replayed pointer sequence", () => {
    const s = session();
    s.accept(hello());
    s.accept(button(2));
    expect(() => s.accept(button(2))).toThrow();
    expect(s.received).toBe(1);
  });
  it("rejects wrong frame version and scene state", () => {
    const s = session();
    expect(() => s.accept({ ...hello(), scene_frame_sequence: 10 })).toThrow("SCENE");
    expect(() => s.accept(button(1))).toThrow("BEFORE_HANDSHAKE");
    s.accept(hello());
    expect(() => s.accept({ ...button(1), scene_frame_sequence: 11 })).toThrow("FRAME");
  });
  it("rejects foreign surface, provider and invalid geometry", () => {
    const s = session();
    s.accept(hello());
    expect(() => s.accept({ ...button(1), wl_surface_token: "foreign" })).toThrow("BINDING");
    expect(() => s.accept({ ...button(1), provider_id: "fake" })).toThrow("PROVIDER");
    expect(() => s.accept(button(1, -1, 50))).toThrow("OUTSIDE");
  });
  it("rejects duplicate hello, forbidden rights, fake calibration", () => {
    const s = session();
    s.accept(hello());
    expect(() => s.accept(hello())).toThrow("DUPLICATE");
    const other = session();
    expect(() => other.accept({ ...hello(), consumer_authority: "mutation" })).toThrow("AUTHORITY");
    expect(() => other.accept({ ...hello(), calibration_status: "physical_pass" })).toThrow(
      "AUTHORITY",
    );
  });
  it("rejects odd packet types and foreign events", () => {
    const s = session();
    expect(() => s.accept(null)).toThrow("NONOBJECT");
    s.accept(hello());
    expect(() => s.accept({ ...button(1), type: "command" })).toThrow("UNAUTHORIZED");
    expect(() => s.accept({ ...button(1), button: "BTN_RIGHT" })).toThrow("UNAUTHORIZED");
  });
  it("a software scene target is never an optical qualification", () => {
    const s = session();
    s.accept(hello());
    const center = s.accept(button(1, 1280, 720));
    expect(center.geometric_target_candidate).toBeTypeOf("boolean");
    expect(center.photodiode_qualified).toBe(false);
    expect(center.scene_render_pose_verified).toBe(false);
    expect(center.cp03_pass).toBe(false);
  });
});
