import type { EntityId, Vec2 } from "../../src/algebra/types";
import {
  applyDeckPointerField,
  layoutTarotFan,
  previewDeckOrder,
  projectDeckCentersToWorld,
  type DeckCardPose,
  type DeckSpec,
} from "../../src/deck/deck";
import {
  asFrameId,
  type FrameId,
  type SpatialFrame,
  type SpatialFrameGraph,
} from "../../src/spatial/frame";

export type DockEdge = "top" | "right" | "bottom" | "left";

export interface ViewportExtent {
  width: number;
  height: number;
}

/**
 * Docklab donor translation.
 *
 * Reusable capital:
 *   local U = edge tangent
 *   local V = inward normal
 *
 * Edge identity itself remains consumer policy.
 */
export function dockSurfaceFrame(
  id: FrameId,
  edge: DockEdge,
  viewport: ViewportExtent,
): SpatialFrame {
  switch (edge) {
    case "top":
      return {
        id,
        parent: null,
        origin: { x: 0, y: 0 },
        basisU: { x: 1, y: 0 },
        basisV: { x: 0, y: 1 },
      };
    case "right":
      return {
        id,
        parent: null,
        origin: { x: viewport.width, y: 0 },
        basisU: { x: 0, y: 1 },
        basisV: { x: -1, y: 0 },
      };
    case "bottom":
      return {
        id,
        parent: null,
        origin: { x: 0, y: viewport.height },
        basisU: { x: 1, y: 0 },
        basisV: { x: 0, y: -1 },
      };
    case "left":
      return {
        id,
        parent: null,
        origin: { x: 0, y: 0 },
        basisU: { x: 0, y: 1 },
        basisV: { x: 1, y: 0 },
      };
  }
}

export function dockEdgeFrame(edge: DockEdge, viewport: ViewportExtent): SpatialFrame {
  return dockSurfaceFrame(asFrameId("dock:" + edge), edge, viewport);
}

export function updateDockSurfaceFrame(
  frames: SpatialFrameGraph,
  frameId: FrameId,
  edge: DockEdge,
  viewport: ViewportExtent,
): SpatialFrame {
  const next = dockSurfaceFrame(frameId, edge, viewport);
  return frames.update(frameId, {
    parent: next.parent,
    origin: next.origin,
    basisU: next.basisU,
    basisV: next.basisV,
  });
}

export const DOCKLAB_DECK_SPEC: DeckSpec = {
  center: { x: 180, y: 44 },
  cardWidth: 72,
  cardHeight: 82,
  overlap: 58,
  fanAngle: Math.PI / 18,
  fanDepth: 10,
  hoverRadius: 160,
  hoverScale: 1.055,
  hoverLift: 18,
  liftDirection: { x: 0, y: 1 },
  neighborYield: 18,
  neighborFalloff: 2.4,
};

export function layoutDocklabFanInFrame(
  frames: SpatialFrameGraph,
  frameId: FrameId,
  ids: readonly EntityId[],
): DeckCardPose[] {
  return projectDeckCentersToWorld(
    layoutTarotFan(ids, DOCKLAB_DECK_SPEC),
    frames.snapshot(),
    frameId,
  );
}

export function layoutDocklabInteractiveFan(
  frames: SpatialFrameGraph,
  frameId: FrameId,
  ids: readonly EntityId[],
  pointerLocal: Vec2 | null,
): DeckCardPose[] {
  const local = applyDeckPointerField(
    layoutTarotFan(ids, DOCKLAB_DECK_SPEC),
    pointerLocal,
    DOCKLAB_DECK_SPEC,
  );

  return projectDeckCentersToWorld(local, frames.snapshot(), frameId);
}

/**
 * Reorder stays on the deck tangent coordinate U.
 * World orientation does not change the permutation law.
 */
export function previewDocklabReorder(
  order: readonly EntityId[],
  dragId: EntityId,
  tangentCoordinate: number,
  previousIndex: number | null,
): { order: EntityId[]; index: number } {
  const staticOrder = order.filter((id) => id !== dragId);
  const slots = layoutTarotFan(staticOrder, DOCKLAB_DECK_SPEC).map((card) => card.pose.position.x);

  return previewDeckOrder(order, dragId, tangentCoordinate, slots, previousIndex);
}

export function layoutDocklabFan(
  frames: SpatialFrameGraph,
  ids: readonly EntityId[],
  edge: DockEdge = "bottom",
): DeckCardPose[] {
  return layoutDocklabFanInFrame(frames, asFrameId("dock:" + edge), ids);
}
