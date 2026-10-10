import { Application, Graphics } from "pixi.js";
import type { EntityId, Shape2 } from "../src/algebra/types";
import type {
  RendererAdapter,
  RendererEntityFrame,
  RendererFrame,
  RendererOmission,
  RendererResult,
} from "../src/renderer/contract";

export interface PixiBrowserRendererOptions {
  mount: HTMLElement;
  semanticMount?: HTMLElement;
  width: number;
  height: number;
  resolution?: number;
  antialias?: boolean;
  preference?: "webgl" | "webgpu";
}

export interface PixiBrowserRendererDiagnostics {
  rendererName: string;
  tickerStarted: boolean;
  graphicsHandles: number;
  semanticNodes: number;
  canvasWidth: number;
  canvasHeight: number;
}

export interface PixiProviderObservation {
  handlePresent: boolean;
  mounted: boolean;
  visible: boolean;
  renderable: boolean;
  alpha: number;
  finiteGeometry: boolean;
}

/**
 * PixiJS projection adapter for the MQ-11 RendererFrame contract.
 *
 * Authority boundary:
 *
 *   [in.gnu6] scene -> RendererFrame -> Pixi Graphics
 *
 * PixiJS owns GPU projection only.
 *
 * It does NOT own:
 * - semantic identity;
 * - scene time;
 * - topology;
 * - interaction authority;
 * - simulation cadence.
 *
 * Pixi's ticker is stopped permanently. A caller explicitly submits a
 * RendererFrame and this adapter performs exactly one manual render.
 *
 * Pixi event handling is disabled on every Graphics object. Future pointer
 * observations must return through the MQ-08 provider projection contract
 * rather than mutating scene state from Pixi events.
 */
export class PixiBrowserRenderer implements RendererAdapter<HTMLCanvasElement> {
  readonly id = "pixi-browser";

  private readonly app: Application;
  private readonly mount: HTMLElement;
  private readonly semanticMount: HTMLElement | null;
  private readonly graphics = new Map<EntityId, Graphics>();
  private readonly semanticNodes = new Map<EntityId, HTMLElement>();

  private constructor(app: Application, options: PixiBrowserRendererOptions) {
    this.app = app;
    this.mount = options.mount;
    this.semanticMount = options.semanticMount ?? null;

    this.app.stage.sortableChildren = true;
    this.mount.replaceChildren(this.app.canvas);
  }

  static async create(options: PixiBrowserRendererOptions): Promise<PixiBrowserRenderer> {
    if (
      !Number.isFinite(options.width) ||
      !Number.isFinite(options.height) ||
      options.width <= 0 ||
      options.height <= 0
    ) {
      throw new Error("Pixi browser renderer extent must be finite and positive");
    }

    const app = new Application();

    await app.init({
      width: options.width,
      height: options.height,
      preference: options.preference ?? "webgl",
      antialias: options.antialias ?? true,
      autoDensity: true,
      resolution: options.resolution ?? Math.max(1, globalThis.devicePixelRatio || 1),
      backgroundAlpha: 0,
      autoStart: false,
    });

    // The semantic scene clock lives in [in.gnu6], never in Pixi.
    app.stop();

    return new PixiBrowserRenderer(app, options);
  }

  render(frame: RendererFrame): RendererResult<HTMLCanvasElement> {
    const frameIds = new Set(frame.entities.map((entity) => entity.entityId));

    // Entity removal is semantic truth from RendererFrame. Destroying the
    // provider handle is downstream cleanup, not scene lifecycle authority.
    for (const [entityId, graphics] of this.graphics) {
      if (frameIds.has(entityId)) continue;
      graphics.removeFromParent();
      graphics.destroy();
      this.graphics.delete(entityId);
    }

    for (const [entityId, node] of this.semanticNodes) {
      if (frameIds.has(entityId)) continue;
      node.remove();
      this.semanticNodes.delete(entityId);
    }

    const emittedEntityIds: EntityId[] = [];
    const omitted: RendererOmission[] = [];

    for (const entity of frame.entities) {
      if (!entity.visible) {
        const graphics = this.graphics.get(entity.entityId);
        if (graphics) graphics.visible = false;

        const semanticNode = this.semanticNodes.get(entity.entityId);
        if (semanticNode) semanticNode.hidden = true;

        omitted.push({
          entityId: entity.entityId,
          reason: "not-visible",
        });
        continue;
      }

      const graphics = this.ensureGraphics(entity.entityId);
      updateGraphics(graphics, entity);
      graphics.visible = true;
      emittedEntityIds.push(entity.entityId);

      this.updateSemanticProjection(entity);
    }

    this.app.render();

    return {
      adapterId: this.id,
      sequence: frame.sequence,
      emittedEntityIds,
      omitted,
      output: this.app.canvas,
    };
  }

  resize(width: number, height: number): void {
    this.app.renderer.resize(width, height);
  }

  getHandle(entityId: EntityId): Graphics | null {
    return this.graphics.get(entityId) ?? null;
  }

  observeProvider(entityId: EntityId): PixiProviderObservation {
    const graphics = this.graphics.get(entityId);
    if (!graphics) {
      return {
        handlePresent: false,
        mounted: false,
        visible: false,
        renderable: false,
        alpha: 0,
        finiteGeometry: false,
      };
    }

    const bounds = graphics.getBounds();
    const finiteGeometry = [
      graphics.position.x,
      graphics.position.y,
      graphics.scale.x,
      graphics.scale.y,
      graphics.rotation,
      bounds.minX,
      bounds.minY,
      bounds.maxX,
      bounds.maxY,
    ].every(Number.isFinite);

    return {
      handlePresent: true,
      mounted: graphics.parent !== null,
      visible: graphics.visible,
      renderable: graphics.renderable,
      alpha: graphics.alpha,
      finiteGeometry,
    };
  }

  diagnostics(): PixiBrowserRendererDiagnostics {
    return {
      rendererName: this.app.renderer.name,
      tickerStarted: this.app.ticker.started,
      graphicsHandles: this.graphics.size,
      semanticNodes: this.semanticNodes.size,
      canvasWidth: this.app.canvas.width,
      canvasHeight: this.app.canvas.height,
    };
  }

  destroy(): void {
    for (const graphics of this.graphics.values()) {
      graphics.removeFromParent();
      graphics.destroy();
    }
    this.graphics.clear();

    for (const node of this.semanticNodes.values()) {
      node.remove();
    }
    this.semanticNodes.clear();

    this.app.destroy({ removeView: true }, { children: true });
  }

  private ensureGraphics(entityId: EntityId): Graphics {
    const existing = this.graphics.get(entityId);
    if (existing) return existing;

    const graphics = new Graphics({
      label: "in.gnu6:" + entityId,
    });

    // Pixi is not an interaction authority.
    graphics.eventMode = "none";

    this.graphics.set(entityId, graphics);
    this.app.stage.addChild(graphics);
    return graphics;
  }

  private updateSemanticProjection(entity: RendererEntityFrame): void {
    if (!this.semanticMount) return;

    const accessibility = entity.accessibility;
    if (!accessibility || accessibility.hidden) {
      const existing = this.semanticNodes.get(entity.entityId);
      if (existing) existing.hidden = true;
      return;
    }

    const existing = this.semanticNodes.get(entity.entityId);
    const requiresButton = accessibility.role === "button";

    let node = existing;
    if (
      !node ||
      (requiresButton && node.tagName !== "BUTTON") ||
      (!requiresButton && node.tagName === "BUTTON")
    ) {
      existing?.remove();
      node = requiresButton ? document.createElement("button") : document.createElement("div");

      node.dataset.entityId = entity.entityId;
      node.style.position = "absolute";
      node.style.opacity = "0.001";
      node.style.pointerEvents = "none";
      node.style.margin = "0";
      node.style.padding = "0";
      node.style.border = "0";
      node.style.background = "transparent";

      this.semanticMount.appendChild(node);
      this.semanticNodes.set(entity.entityId, node);
    }

    node.hidden = false;
    node.setAttribute("role", accessibility.role);

    if (accessibility.label !== undefined) {
      node.setAttribute("aria-label", accessibility.label);
    } else {
      node.removeAttribute("aria-label");
    }

    if (node instanceof HTMLButtonElement) {
      node.type = "button";
      node.tabIndex = 0;
    }

    const bounds = projectedBounds(entity);
    node.style.left = bounds.x + "px";
    node.style.top = bounds.y + "px";
    node.style.width = Math.max(1, bounds.width) + "px";
    node.style.height = Math.max(1, bounds.height) + "px";
  }
}

function updateGraphics(graphics: Graphics, entity: RendererEntityFrame): void {
  graphics.clear();
  drawShape(graphics, entity.shape, fillForRole(entity.role));

  graphics.position.set(entity.worldPose.position.x, entity.worldPose.position.y);
  graphics.rotation = entity.worldPose.rotation;
  graphics.scale.set(entity.worldPose.scale.x, entity.worldPose.scale.y);
  graphics.zIndex = entity.z;
}

function drawShape(graphics: Graphics, shape: Shape2, color: number): void {
  if (shape.kind === "rect") {
    graphics
      .roundRect(-shape.width / 2, -shape.height / 2, shape.width, shape.height, shape.radius)
      .fill({
        color,
        alpha: 0.92,
      });
    return;
  }

  graphics
    .poly(
      shape.vertices.map((point) => ({
        x: point.x,
        y: point.y,
      })),
      true,
    )
    .fill({
      color,
      alpha: 0.92,
    });
}

function fillForRole(role: RendererEntityFrame["role"]): number {
  switch (role) {
    case "card":
      return 0xf4eee2;
    case "surface":
      return 0x28323f;
    case "control":
      return 0xf1893d;
    case "proxy":
      return 0x8d7fbf;
    case "anchor":
      return 0xd0a14a;
    case "decoration":
      return 0x61826c;
  }
}

function projectedBounds(entity: RendererEntityFrame): {
  x: number;
  y: number;
  width: number;
  height: number;
} {
  let width: number;
  let height: number;

  if (entity.shape.kind === "rect") {
    width = entity.shape.width;
    height = entity.shape.height;
  } else {
    const xs = entity.shape.vertices.map((point) => point.x);
    const ys = entity.shape.vertices.map((point) => point.y);
    width = Math.max(...xs) - Math.min(...xs);
    height = Math.max(...ys) - Math.min(...ys);
  }

  width *= Math.abs(entity.worldPose.scale.x);
  height *= Math.abs(entity.worldPose.scale.y);

  return {
    x: entity.worldPose.position.x - width / 2,
    y: entity.worldPose.position.y - height / 2,
    width,
    height,
  };
}
