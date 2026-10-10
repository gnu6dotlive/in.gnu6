import type { EntityId } from "../src/algebra/types";
import type {
  RendererAdapter,
  RendererEntityFrame,
  RendererFrame,
  RendererOmission,
  RendererResult,
} from "../src/renderer/contract";

export interface SvgRendererOptions {
  width: number;
  height: number;
}

/**
 * Pure SVG-string projection.
 *
 * This is a contract-qualification adapter, not yet the MQ-12 reference
 * browser renderer. It performs no DOM mutation and owns no scene authority.
 */
export class SvgStringRenderer implements RendererAdapter<string> {
  readonly id = "svg-string";

  constructor(private readonly options: SvgRendererOptions) {
    if (
      !Number.isFinite(options.width) ||
      !Number.isFinite(options.height) ||
      options.width <= 0 ||
      options.height <= 0
    ) {
      throw new Error("SVG renderer extent must be finite and positive");
    }
  }

  render(frame: RendererFrame): RendererResult<string> {
    const emittedEntityIds: EntityId[] = [];
    const omitted: RendererOmission[] = [];
    const body: string[] = [];

    for (const entity of frame.entities) {
      if (!entity.visible) {
        omitted.push({
          entityId: entity.entityId,
          reason: "not-visible",
        });
        continue;
      }

      emittedEntityIds.push(entity.entityId);
      body.push(renderEntity(entity));
    }

    return {
      adapterId: this.id,
      sequence: frame.sequence,
      emittedEntityIds,
      omitted,
      output:
        '<svg xmlns="http://www.w3.org/2000/svg" ' +
        'width="' +
        numberText(this.options.width) +
        '" height="' +
        numberText(this.options.height) +
        '" data-motion-mode="' +
        frame.motionMode +
        '">' +
        body.join("") +
        "</svg>",
    };
  }
}

function renderEntity(entity: RendererEntityFrame): string {
  const pose = entity.worldPose;
  const rotationDegrees = (pose.rotation * 180) / Math.PI;
  const accessibility = accessibilityAttributes(entity);
  const transform =
    "translate(" +
    numberText(pose.position.x) +
    " " +
    numberText(pose.position.y) +
    ") rotate(" +
    numberText(rotationDegrees) +
    ") scale(" +
    numberText(pose.scale.x) +
    " " +
    numberText(pose.scale.y) +
    ")";

  const shape = entity.shape.kind === "rect" ? renderRect(entity) : renderPolygon(entity);

  return (
    '<g data-entity-id="' +
    escapeXml(entity.entityId) +
    '" transform="' +
    transform +
    '"' +
    accessibility +
    ">" +
    shape +
    "</g>"
  );
}

function renderRect(entity: RendererEntityFrame): string {
  if (entity.shape.kind !== "rect") {
    throw new Error("Expected rect shape");
  }

  return (
    '<rect x="' +
    numberText(-entity.shape.width / 2) +
    '" y="' +
    numberText(-entity.shape.height / 2) +
    '" width="' +
    numberText(entity.shape.width) +
    '" height="' +
    numberText(entity.shape.height) +
    '" rx="' +
    numberText(entity.shape.radius) +
    '" />'
  );
}

function renderPolygon(entity: RendererEntityFrame): string {
  if (entity.shape.kind !== "polygon") {
    throw new Error("Expected polygon shape");
  }

  const points = entity.shape.vertices
    .map((point) => numberText(point.x) + "," + numberText(point.y))
    .join(" ");

  return '<polygon points="' + points + '" />';
}

function accessibilityAttributes(entity: RendererEntityFrame): string {
  const accessibility = entity.accessibility;
  if (!accessibility) return "";

  let attributes = ' role="' + escapeXml(accessibility.role) + '"';

  if (accessibility.label !== undefined) {
    attributes += ' aria-label="' + escapeXml(accessibility.label) + '"';
  }

  if (accessibility.hidden) {
    attributes += ' aria-hidden="true"';
  }

  return attributes;
}

function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function numberText(value: number): string {
  if (!Number.isFinite(value)) {
    throw new Error("SVG renderer received non-finite number");
  }
  return String(value);
}
