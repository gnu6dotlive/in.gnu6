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
      const calibration: WaylandSurfaceCalibration = {
        id: "oracle05-test-declared-viewport-geometry",
        surfaceToken: this.surfaceToken,
        viewportWidth: 2560,
        viewportHeight: 1440,
        worldOrigin: { x: -200, y: -150 },
        worldUPerPixel: { x: 400 / 2560, y: 0 },
        worldVPerPixel: { x: 0, y: 300 / 1440 },
        sceneFrameSequence: this.frame.sequence,
      };
      this.projection = new WaylandOpticalSemanticProjector(this.frame, calibration, this.targetId);
      return {
        type: "observer_ready",
        schema: "blobin.gpu05.semantic-observation.v1",
        scene_sequence: this.frame.sequence,
        calibration: "fixture_declared_not_physically_verified",
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
      photodiode_qualified: false,
      mutation_authority_granted: false,
      cp03_pass: false,
    };
  }
}
