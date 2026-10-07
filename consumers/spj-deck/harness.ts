import type { EntityId, Pose2 } from "../../src/algebra/types";
import { layoutTarotFan, type DeckSpec } from "../../src/deck/deck";
import {
  asFrameId,
  localToWorld,
  type FrameId,
  type SpatialFrameGraph,
} from "../../src/spatial/frame";

export interface ViewportExtent {
  width: number;
  height: number;
}

export interface SpjWorldFrames {
  viewportFrameId: FrameId;
  handFrameId: FrameId;
  playzoneFrameId: FrameId;
}

/**
 * SP Joaillerie donor translation.
 *
 * Responsive sizing remains consumer policy. The engine receives spatial
 * declarations: viewport, hand and playzone are ordinary frames.
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

  return { viewportFrameId, handFrameId, playzoneFrameId };
}

function spjDeckSpec(cardCount: number): DeckSpec {
  const overlap = Math.max(42, Math.min(86, 340 / Math.max(1, cardCount - 1)));

  return {
    center: { x: 0, y: 0 },
    cardWidth: 120,
    cardHeight: 190,
    overlap,
    fanAngle: Math.PI / 7,
    fanDepth: 34,
    hoverRadius: 190,
    hoverScale: 1.09,
    hoverLift: 28,
    neighborYield: 20,
    neighborFalloff: 2.2,
  };
}

export function layoutSpjHand(
  frames: SpatialFrameGraph,
  handFrameId: FrameId,
  ids: readonly EntityId[],
): Array<{ id: EntityId; pose: Pose2; z: number }> {
  const local = layoutTarotFan(ids, spjDeckSpec(ids.length));

  return local.map((card) => ({
    id: card.id,
    z: card.z,
    pose: {
      ...card.pose,
      position: localToWorld(frames.snapshot(), handFrameId, card.pose.position),
    },
  }));
}
