import type { EntityId } from "../../src/algebra/types";
import {
  layoutTarotFan,
  previewDeckOrder,
  projectDeckCentersToWorld,
  type DeckCardPose,
  type DeckSpec,
} from "../../src/deck/deck";
import { asFrameId, type FrameId, type SpatialFrameGraph } from "../../src/spatial/frame";

export interface ViewportExtent {
  width: number;
  height: number;
}

export interface SpjWorldFrames {
  viewportFrameId: FrameId;
  handFrameId: FrameId;
  playzoneFrameId: FrameId;
  viewport: ViewportExtent;
}

/**
 * SP Joaillerie donor translation.
 *
 * Responsive sizing is consumer policy. The engine receives only the resulting
 * deck spec and spatial frames.
 */
export function createSpjWorld(
  frames: SpatialFrameGraph,
  viewport: ViewportExtent,
): SpjWorldFrames {
  const viewportFrameId = asFrameId("spj:viewport");
  const handFrameId = asFrameId("spj:hand");
  const playzoneFrameId = asFrameId("spj:playzone");

  frames.add({
    id: viewportFrameId,
    parent: null,
    origin: { x: 0, y: 0 },
    basisU: { x: 1, y: 0 },
    basisV: { x: 0, y: 1 },
  });

  frames.add({
    id: handFrameId,
    parent: viewportFrameId,
    origin: { x: viewport.width / 2, y: viewport.height * 0.78 },
    basisU: { x: 1, y: 0 },
    basisV: { x: 0, y: 1 },
  });

  frames.add({
    id: playzoneFrameId,
    parent: viewportFrameId,
    origin: { x: viewport.width / 2, y: viewport.height * 0.28 },
    basisU: { x: 1, y: 0 },
    basisV: { x: 0, y: 1 },
  });

  return {
    viewportFrameId,
    handFrameId,
    playzoneFrameId,
    viewport: { ...viewport },
  };
}

export function spjDeckSpec(viewport: ViewportExtent, cardCount: number): DeckSpec {
  const viewportScale = clamp(Math.min(viewport.width / 1280, viewport.height / 800), 0.65, 1.2);
  const baseOverlap = Math.max(42, Math.min(86, 340 / Math.max(1, cardCount - 1)));

  return {
    center: { x: 0, y: 0 },
    cardWidth: 120 * viewportScale,
    cardHeight: 190 * viewportScale,
    overlap: baseOverlap * viewportScale,
    fanAngle: Math.PI / 7,
    fanDepth: 34 * viewportScale,
    hoverRadius: 190 * viewportScale,
    hoverScale: 1.09,
    hoverLift: 28 * viewportScale,
    liftDirection: { x: 0, y: -1 },
    neighborYield: 20 * viewportScale,
    neighborFalloff: 2.2,
  };
}

export function layoutSpjHand(
  frames: SpatialFrameGraph,
  handFrameId: FrameId,
  ids: readonly EntityId[],
  viewport: ViewportExtent = { width: 1280, height: 800 },
): DeckCardPose[] {
  const local = layoutTarotFan(ids, spjDeckSpec(viewport, ids.length));

  return projectDeckCentersToWorld(local, frames.snapshot(), handFrameId);
}

export function previewSpjReorder(
  order: readonly EntityId[],
  dragId: EntityId,
  tangentCoordinate: number,
  viewport: ViewportExtent,
  previousIndex: number | null,
): { order: EntityId[]; index: number } {
  const staticOrder = order.filter((id) => id !== dragId);
  const spec = spjDeckSpec(viewport, staticOrder.length);
  const slots = layoutTarotFan(staticOrder, spec).map((card) => card.pose.position.x);

  return previewDeckOrder(order, dragId, tangentCoordinate, slots, previousIndex);
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}
