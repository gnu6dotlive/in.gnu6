import type {
  RendererAdapter,
  RendererEntityFrame,
  RendererFrame,
  RendererOmission,
  RendererResult,
} from "../src/renderer/contract";
import type { EntityId, Pose2, Shape2 } from "../src/algebra/types";

export interface RenderCommand {
  entityId: EntityId;
  role: RendererEntityFrame["role"];
  worldPose: Pose2;
  shape: Shape2;
}

/**
 * Deterministic nonvisual renderer.
 *
 * Useful as an oracle because it converts a RendererFrame into explicit draw
 * commands without browser, DOM, Canvas or GPU behavior.
 */
export class CommandRenderer implements RendererAdapter<readonly RenderCommand[]> {
  readonly id = "nonvisual-command";

  render(frame: RendererFrame): RendererResult<readonly RenderCommand[]> {
    const output: RenderCommand[] = [];
    const emittedEntityIds: EntityId[] = [];
    const omitted: RendererOmission[] = [];

    for (const entity of frame.entities) {
      if (!entity.visible) {
        omitted.push({
          entityId: entity.entityId,
          reason: "not-visible" as const,
        });
        continue;
      }

      emittedEntityIds.push(entity.entityId);
      output.push({
        entityId: entity.entityId,
        role: entity.role,
        worldPose: clonePose(entity.worldPose),
        shape: cloneShape(entity.shape),
      });
    }

    return {
      adapterId: this.id,
      sequence: frame.sequence,
      emittedEntityIds,
      omitted,
      output,
    };
  }
}

function clonePose(pose: Pose2): Pose2 {
  return {
    position: { ...pose.position },
    rotation: pose.rotation,
    scale: { ...pose.scale },
  };
}

function cloneShape(shape: Shape2): Shape2 {
  if (shape.kind === "rect") return { ...shape };

  return {
    kind: "polygon",
    closed: true,
    vertices: shape.vertices.map((point) => ({ ...point })),
  };
}
