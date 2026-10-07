import type { EntityId } from "../../src/algebra/types";
import {
  layoutTarotFan,
  projectDeckCentersToWorld,
  type DeckCardPose,
  type DeckSpec,
} from "../../src/deck/deck";
import { asFrameId, type SpatialFrame, type SpatialFrameGraph } from "../../src/spatial/frame";

export type DockEdge = "top" | "right" | "bottom" | "left";

export interface ViewportExtent {
  width: number;
  height: number;
}

/**
 * Docklab donor translation.
 *
 * Docklab's reusable fact is not "there are four special Dock modes".
 * It is that an environment surface defines a local basis:
 *
 *   u = tangent to the surface
 *   v = inward normal
 *
 * This harness expresses that fact as an ordinary SpatialFrame.
 */
export function dockEdgeFrame(edge: DockEdge, viewport: ViewportExtent): SpatialFrame {
  switch (edge) {
    case "top":
      return {
        id: asFrameId("dock:top"),
        parent: null,
        origin: { x: 0, y: 0 },
        basisU: { x: 1, y: 0 },
        basisV: { x: 0, y: 1 },
      };
    case "right":
      return {
        id: asFrameId("dock:right"),
        parent: null,
        origin: { x: viewport.width, y: 0 },
        basisU: { x: 0, y: 1 },
        basisV: { x: -1, y: 0 },
      };
    case "bottom":
      return {
        id: asFrameId("dock:bottom"),
        parent: null,
        origin: { x: 0, y: viewport.height },
        basisU: { x: 1, y: 0 },
        basisV: { x: 0, y: -1 },
      };
    case "left":
      return {
        id: asFrameId("dock:left"),
        parent: null,
        origin: { x: 0, y: 0 },
        basisU: { x: 0, y: 1 },
        basisV: { x: 1, y: 0 },
      };
  }
}

const DOCKLAB_DECK_SPEC: DeckSpec = {
  center: { x: 180, y: 44 },
  cardWidth: 72,
  cardHeight: 82,
  overlap: 58,
  fanAngle: Math.PI / 18,
  fanDepth: 10,
  hoverRadius: 160,
  hoverScale: 1.055,
  hoverLift: 18,
  neighborYield: 18,
  neighborFalloff: 2.4,
};

/**
 * The generic deck solver produces local poses.
 * SpatialFrame projects them into World coordinates.
 *
 * deck topology != environment orientation
 */
export function layoutDocklabFan(
  frames: SpatialFrameGraph,
  ids: readonly EntityId[],
  edge: DockEdge = "bottom",
): DeckCardPose[] {
  const frameId = asFrameId("dock:" + edge);
  const local = layoutTarotFan(ids, DOCKLAB_DECK_SPEC);

  return projectDeckCentersToWorld(local, frames.snapshot(), frameId);
}
