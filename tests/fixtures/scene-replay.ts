import type { EntityId, PoseTarget, SceneState } from "../../src/algebra/types";
import { FixedStepClock } from "../../src/dynamics/clock";
import { SceneEngine } from "../../src/scene/engine";

export type ReplayEvent =
  | {
      tick: number;
      kind: "set-target";
      target: PoseTarget;
    }
  | {
      tick: number;
      kind: "clear-target";
      entityId: EntityId;
    }
  | {
      tick: number;
      kind: "settle-target";
      entityId: EntityId;
    };

export interface ReplayResult {
  state: SceneState;
  clock: ReturnType<FixedStepClock["observation"]>;
}

/**
 * Deterministic semantic replay fixture.
 *
 * Events are indexed by fixed simulation tick, not renderer frame.
 * Therefore a replay means:
 *
 *   initial state
 *   + ordered semantic events
 *   + fixed step law
 *   = deterministic final observation
 *
 * Renderer cadence is intentionally absent from this fixture.
 */
export function replayScene(
  engine: SceneEngine,
  events: readonly ReplayEvent[],
  totalTicks: number,
  hz = 60,
): ReplayResult {
  if (!Number.isInteger(totalTicks) || totalTicks < 0) {
    throw new Error("Replay totalTicks must be a non-negative integer");
  }

  const ordered = [...events].sort((a, b) => a.tick - b.tick);
  let eventIndex = 0;
  const clock = new FixedStepClock({ hz, maxStepsPerAdvance: 1 });

  for (let tick = 0; tick < totalTicks; tick += 1) {
    while (ordered[eventIndex]?.tick === tick) {
      const event = ordered[eventIndex]!;
      applyReplayEvent(engine, event);
      eventIndex += 1;
    }

    clock.advance(engine, 1 / hz);
  }

  return {
    state: engine.snapshot(),
    clock: clock.observation(),
  };
}

function applyReplayEvent(engine: SceneEngine, event: ReplayEvent): void {
  switch (event.kind) {
    case "set-target":
      engine.setTarget(event.target);
      break;
    case "clear-target":
      engine.clearTarget(event.entityId);
      break;
    case "settle-target":
      engine.settleTarget(event.entityId);
      break;
  }
}
