import type { EntityId } from "../algebra/types";
import { InteractionProjector, type InteractionProjection } from "./projection";
import type { RendererEntityFrame, RendererFrame } from "../renderer/contract";
import type { SpatialOccupant } from "../spatial/environment";
import { SpatialFrameGraph, asFrameId } from "../spatial/frame";

/**
 * Experimental ORACLE-03 *read-only* input provenance adapter.
 *
 * A native wl_pointer provides surface-local pixels, NOT scene coordinates.
 * Only a separately supplied, explicit affine calibration can map these pixels
 * into the [in.gnu6] world. The renderer may observe an exact subject identity
 * but acquires no scene, topology, ownership or product mutation authority.
 */
export interface WaylandSurfaceCalibration {
  id: string;
  surfaceToken: string;
  viewportWidth: number;
  viewportHeight: number;
  worldOrigin: { x: number; y: number };
  worldUPerPixel: { x: number; y: number };
  worldVPerPixel: { x: number; y: number };
  sceneFrameSequence: number;
}

export interface CapturedWaylandPointerButton {
  provider_id: "wayland-gpu05-owned-surface";
  pointer_id: "primary";
  provider_sequence: number;
  wl_surface_token: string;
  pressed: boolean;
  surface_x: number;
  surface_y: number;
  coordinates_space: "surface_logical_pixels";
}

export interface OpticalSemanticObservation {
  readonly projection: InteractionProjection;
  readonly sceneSequence: number;
  readonly calibrationId: string;
  readonly targetId: EntityId;
  /** A geometric match, not a product action, mutation grant or optical PASS. */
  readonly exactTargetObserved: boolean;
  readonly photodiodeQualified: false;
  readonly mutationAuthorityGranted: false;
}

/**
 * AABB in world coordinates is exact only for axis-aligned rectangular
 * entities. Any rotation, polygon, or invalid pose fails closed rather
 * than inventing a hit candidate from an approximate projection.
 */
function exactOccupants(frame: RendererFrame): SpatialOccupant[] {
  return frame.entities.map((entity: RendererEntityFrame) => {
    const pose = entity.worldPose;
    const shape = entity.shape;
    if (
      shape.kind !== "rect" ||
      pose.rotation !== 0 ||
      !Number.isFinite(pose.position.x) ||
      !Number.isFinite(pose.position.y) ||
      !Number.isFinite(pose.scale.x) ||
      !Number.isFinite(pose.scale.y) ||
      pose.scale.x <= 0 ||
      pose.scale.y <= 0 ||
      !Number.isFinite(shape.width) ||
      !Number.isFinite(shape.height) ||
      shape.width <= 0 ||
      shape.height <= 0
    ) {
      throw new Error("ORACLE03_SCENE_GEOMETRY_UNSUPPORTED_FOR_EXACT_PICK");
    }
    const halfW = (shape.width * pose.scale.x) / 2;
    const halfH = (shape.height * pose.scale.y) / 2;
    return {
      id: entity.entityId,
      visible: entity.visible,
      bounds: {
        minX: pose.position.x - halfW,
        minY: pose.position.y - halfH,
        maxX: pose.position.x + halfW,
        maxY: pose.position.y + halfH,
      },
    };
  });
}

function assertCalibration(binding: WaylandSurfaceCalibration, frame: RendererFrame): void {
  if (!binding.id || !binding.surfaceToken || !Number.isSafeInteger(binding.sceneFrameSequence)) {
    throw new Error("ORACLE03_MISSING_CALIBRATION_IDENTITY");
  }
  if (binding.sceneFrameSequence !== frame.sequence) {
    throw new Error("ORACLE03_STALE_SCENE_FRAME_BINDING");
  }
  if (
    ![binding.viewportWidth, binding.viewportHeight].every(
      (value) => Number.isSafeInteger(value) && value > 0 && value <= 4096,
    )
  ) {
    throw new Error("ORACLE03_INVALID_VIEWPORT");
  }
  const values = [
    ...Object.values(binding.worldOrigin),
    ...Object.values(binding.worldUPerPixel),
    ...Object.values(binding.worldVPerPixel),
  ];
  if (!values.every(Number.isFinite)) {
    throw new Error("ORACLE03_NONFINITE_CALIBRATION");
  }
  const u = binding.worldUPerPixel;
  const v = binding.worldVPerPixel;
  const determinant = u.x * v.y - u.y * v.x;
  if (!Number.isFinite(determinant) || Math.abs(determinant) <= 1e-12) {
    throw new Error("ORACLE03_NONINVERTIBLE_CALIBRATION");
  }
}

export class WaylandOpticalSemanticProjector {
  private readonly projector: InteractionProjector;
  private readonly occupants: readonly SpatialOccupant[];

  constructor(
    private readonly frame: RendererFrame,
    private readonly calibration: WaylandSurfaceCalibration,
    private readonly targetId: EntityId,
  ) {
    assertCalibration(calibration, frame);
    const target = frame.entities.find((entity) => entity.entityId === targetId);
    if (!target || target.role !== "control" || !target.visible) {
      throw new Error("ORACLE03_TARGET_NOT_VISIBLE_SCENE_CONTROL");
    }
    this.occupants = exactOccupants(frame);
    const frames = new SpatialFrameGraph();
    frames.add({
      id: asFrameId(calibration.id),
      parent: null,
      origin: { ...calibration.worldOrigin },
      basisU: { ...calibration.worldUPerPixel },
      basisV: { ...calibration.worldVPerPixel },
    });
    this.projector = new InteractionProjector(frames.snapshot());
  }

  observe(captured: CapturedWaylandPointerButton): OpticalSemanticObservation {
    if (
      captured.provider_id !== "wayland-gpu05-owned-surface" ||
      captured.pointer_id !== "primary" ||
      captured.wl_surface_token !== this.calibration.surfaceToken ||
      captured.coordinates_space !== "surface_logical_pixels"
    ) {
      throw new Error("ORACLE03_POINTER_PROVIDER_OR_SURFACE_MISMATCH");
    }
    if (
      !Number.isSafeInteger(captured.provider_sequence) ||
      captured.provider_sequence <= 0 ||
      typeof captured.pressed !== "boolean"
    ) {
      throw new Error("ORACLE03_INVALID_POINTER_SEQUENCE_OR_PHASE");
    }
    const { surface_x: x, surface_y: y } = captured;
    if (
      !Number.isFinite(x) ||
      !Number.isFinite(y) ||
      x < 0 ||
      y < 0 ||
      x > this.calibration.viewportWidth ||
      y > this.calibration.viewportHeight
    ) {
      throw new Error("ORACLE03_POINTER_OUTSIDE_CALIBRATED_VIEWPORT");
    }
    const projection = this.projector.project(
      {
        providerId: captured.provider_id,
        pointerId: captured.pointer_id,
        sequence: captured.provider_sequence,
        phase: captured.pressed ? "down" : "up",
        coordinates: {
          space: "frame",
          frameId: asFrameId(this.calibration.id),
          point: { x, y },
        },
      },
      this.occupants,
    );
    return {
      projection,
      sceneSequence: this.frame.sequence,
      calibrationId: this.calibration.id,
      targetId: this.targetId,
      exactTargetObserved: captured.pressed && projection.semantic.subjectId === this.targetId,
      photodiodeQualified: false,
      mutationAuthorityGranted: false,
    };
  }
}
