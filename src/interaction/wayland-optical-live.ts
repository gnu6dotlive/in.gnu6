import type { EntityId } from "../algebra/types";
import type { RendererFrame } from "../renderer/contract";
import {
  WaylandOpticalSemanticProjector,
  type CapturedWaylandPointerButton,
  type WaylandSurfaceCalibration,
} from "./wayland-optical-projection";

/**
 * One-shot read-only live observer. The *input* comes from the owned
 * wl_surface; this is not a product intent, scene mutation, or optical sensor.
 * The reference chart is explicitly fixture-declared, NOT calibrated against
 * display photons or the independently animated Vulkan geometry.
 */
export class Oracle05SemanticSession {
  private projection: WaylandOpticalSemanticProjector | undefined;
  private surfaceToken: string | undefined;
  private visualMarkerContractConsistent = false;
  private readonly frame: RendererFrame;
  private readonly targetId: EntityId;
  private count = 0;

  constructor(frame: RendererFrame, targetId: EntityId) {
    this.frame = frame;
    this.targetId = targetId;
  }

  get received(): number {
    return this.count;
  }

  accept(input: unknown): Record<string, unknown> {
    if (typeof input !== "object" || input === null || Array.isArray(input)) {
      throw new Error("ORACLE05_NONOBJECT_PACKET");
    }
    const row = input as Record<string, unknown>;
    if (row.type === "hello") {
      if (this.projection) throw new Error("ORACLE05_DUPLICATE_HANDSHAKE");
      if (
        row.schema !== "blobin.gpu05.semantic-witness.v1" ||
        row.consumer_authority !== "read_only_observation" ||
        row.scene_source !== "in.gnu6-verified-scene-stream-v1" ||
        row.coordinate_space !== "surface_logical_pixels" ||
        row.calibration_status !== "fixture_declared_not_physically_verified" ||
        typeof row.surface_token !== "string" ||
        row.surface_token.length < 12 ||
        row.surface_token.length > 160 ||
        !Number.isSafeInteger(row.scene_frame_sequence) ||
        row.scene_frame_sequence !== this.frame.sequence
      ) {
        throw new Error("ORACLE05_HANDSHAKE_SCENE_OR_AUTHORITY_MISMATCH");
      }
      this.surfaceToken = row.surface_token;
      const visual = row.visual_marker;
      let viewportWidth = 2560;
      let viewportHeight = 1440;
      if (visual !== undefined && visual !== null) {
        if (typeof visual !== "object" || Array.isArray(visual)) {
          throw new Error("ORACLE06_INVALID_VISUAL_MARKER");
        }
        const marker = visual as Record<string, unknown>;
        if (
          marker.kind !== "scene_control_2d_overlay" ||
          marker.entity_id !== this.targetId ||
          marker.scene_frame_sequence !== this.frame.sequence ||
          marker.render_origin !== "rust_scene_stream_projection" ||
          marker.independent_pixel_capture !== false ||
          !Number.isSafeInteger(marker.viewport_width) ||
          !Number.isSafeInteger(marker.viewport_height) ||
          !(Number(marker.viewport_width) > 0 && Number(marker.viewport_width) <= 4096) ||
          !(Number(marker.viewport_height) > 0 && Number(marker.viewport_height) <= 4096) ||
          !Array.isArray(marker.rect_uv) ||
          marker.rect_uv.length !== 4 ||
          !marker.rect_uv.every((v: unknown) => typeof v === "number" && Number.isFinite(v))
        ) {
          throw new Error("ORACLE06_INVALID_MARKER_BINDING_OR_AUTHORITY");
        }
        viewportWidth = marker.viewport_width as number;
        viewportHeight = marker.viewport_height as number;
        const stage = this.frame.entities.find((e) => e.entityId === "stage");
        const control = this.frame.entities.find((e) => e.entityId === this.targetId);
        if (
          !stage ||
          !control ||
          !stage.visible ||
          !control.visible ||
          stage.role !== "surface" ||
          control.role !== "control" ||
          stage.shape.kind !== "rect" ||
          control.shape.kind !== "rect" ||
          stage.worldPose.rotation !== 0 ||
          control.worldPose.rotation !== 0
        ) {
          throw new Error("ORACLE06_UNSUPPORTED_SCENE_VISUAL_BINDING");
        }
        const sw = stage.shape.width * stage.worldPose.scale.x;
        const sh = stage.shape.height * stage.worldPose.scale.y;
        const cw = control.shape.width * control.worldPose.scale.x;
        const ch = control.shape.height * control.worldPose.scale.y;
        const ux = (control.worldPose.position.x - stage.worldPose.position.x) / sw + 0.5;
        const uy = (control.worldPose.position.y - stage.worldPose.position.y) / sh + 0.5;
        const expected = [ux - cw / sw / 2, uy - ch / sh / 2, ux + cw / sw / 2, uy + ch / sh / 2];
        if (
          expected.some((x) => !Number.isFinite(x) || x < 0 || x > 1) ||
          expected.some((x, i) => Math.abs(x - (marker.rect_uv as number[])[i]!) > 0.0000002)
        ) {
          throw new Error("ORACLE06_VISUAL_GEOMETRY_CONTRADICTS_SEMANTIC_FRAME");
        }
        this.visualMarkerContractConsistent = true;
      }
      const stageForCalibration = this.frame.entities.find((e) => e.entityId === "stage");
      if (
        !stageForCalibration ||
        stageForCalibration.shape.kind !== "rect" ||
        stageForCalibration.worldPose.rotation !== 0
      ) {
        throw new Error("ORACLE06_STAGE_CHART_UNAVAILABLE");
      }
      const stageW = stageForCalibration.shape.width * stageForCalibration.worldPose.scale.x;
      const stageH = stageForCalibration.shape.height * stageForCalibration.worldPose.scale.y;
      const stageX = stageForCalibration.worldPose.position.x;
      const stageY = stageForCalibration.worldPose.position.y;
      const calibration: WaylandSurfaceCalibration = {
        id: "oracle05-test-declared-viewport-geometry",
        surfaceToken: this.surfaceToken,
        viewportWidth,
        viewportHeight,
        worldOrigin: { x: stageX - stageW / 2, y: stageY - stageH / 2 },
        worldUPerPixel: { x: stageW / viewportWidth, y: 0 },
        worldVPerPixel: { x: 0, y: stageH / viewportHeight },
        sceneFrameSequence: this.frame.sequence,
      };
      this.projection = new WaylandOpticalSemanticProjector(this.frame, calibration, this.targetId);
      return {
        type: "observer_ready",
        schema: "blobin.gpu05.semantic-observation.v1",
        scene_sequence: this.frame.sequence,
        calibration: "fixture_declared_not_physically_verified",
        scene_visual_marker_contract_consistent: this.visualMarkerContractConsistent,
        independent_display_pixel_capture: false,
        cp03_pass: false,
      };
    }
    if (!this.projection || !this.surfaceToken) {
      throw new Error("ORACLE05_EVENT_BEFORE_HANDSHAKE");
    }
    if (
      row.type !== "wayland_button" ||
      row.button !== "BTN_LEFT" ||
      row.timing_class !== "unattested_wayland_software_input" ||
      row.scene_frame_sequence !== this.frame.sequence ||
      row.wl_surface_token !== this.surfaceToken
    ) {
      throw new Error("ORACLE05_UNAUTHORIZED_EVENT_OR_FRAME_BINDING");
    }
    const pointer: CapturedWaylandPointerButton = {
      provider_id: row.provider_id as CapturedWaylandPointerButton["provider_id"],
      pointer_id: row.pointer_id as CapturedWaylandPointerButton["pointer_id"],
      provider_sequence: row.provider_sequence as number,
      wl_surface_token: row.wl_surface_token as string,
      pressed: row.pressed as boolean,
      surface_x: row.surface_x as number,
      surface_y: row.surface_y as number,
      coordinates_space: row.coordinates_space as CapturedWaylandPointerButton["coordinates_space"],
    };
    const result = this.projection.observe(pointer);
    this.count += 1;
    return {
      type: "semantic_pointer_observation",
      schema: "blobin.gpu05.semantic-observation.v1",
      provider_sequence: pointer.provider_sequence,
      phase: pointer.pressed ? "down" : "up",
      scene_sequence: result.sceneSequence,
      semantic_entity_id: result.projection.semantic.subjectId ?? null,
      geometric_target_candidate: result.exactTargetObserved,
      calibration_id: result.calibrationId,
      physical_chart_qualified: false,
      scene_render_pose_verified: false,
      native_wayland_input_observed: false,
      sender_claimed_wayland_input: true,
      semantic_projection_executed: true,
      scene_visual_marker_contract_consistent: this.visualMarkerContractConsistent,
      independently_verified_screen_pixel: false,
      photodiode_qualified: false,
      mutation_authority_granted: false,
      cp03_pass: false,
    };
  }
}
