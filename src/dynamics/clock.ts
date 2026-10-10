import type { SceneEngine } from "../scene/engine";

export interface FixedStepClockConfig {
  hz: number;
  maxStepsPerAdvance: number;
}

export interface FixedStepAdvanceResult {
  executedSteps: number;
  droppedSteps: number;
  droppedSeconds: number;
  phase: number;
}

export interface FixedStepClockObservation {
  hz: number;
  stepSeconds: number;
  maxStepsPerAdvance: number;
  phase: number;
  totalExecutedSteps: number;
  totalDroppedSteps: number;
  totalDroppedSeconds: number;
}

/**
 * Renderer-neutral fixed-step scheduler.
 *
 * Renderer cadence provides elapsed wall time.
 * Scene integration advances only in fixed quanta:
 *
 *   dt = 1 / hz
 *
 * The accumulator is expressed in "step units":
 *
 *   phase += elapsedSeconds * hz
 *
 * Whole units become semantic simulation steps.
 * The fractional remainder is preserved for the next observation.
 *
 * A long wall-clock pause may expose an arbitrarily large elapsed value.
 * To prevent one renderer callback from injecting unbounded work:
 *
 *   executed = min(floor(phase), maxStepsPerAdvance)
 *
 * Excess whole steps are explicitly recorded as dropped time rather than
 * silently replayed or hidden.
 *
 * This means:
 * - normal renderer cadence cannot change fixed-step integration order;
 * - one stalled callback cannot force unbounded catch-up;
 * - dropped time is observable instead of being mistaken for scene time.
 */
export class FixedStepClock {
  readonly hz: number;
  readonly stepSeconds: number;
  readonly maxStepsPerAdvance: number;

  private phase = 0;
  private totalExecutedSteps = 0;
  private totalDroppedSteps = 0;

  constructor(config: FixedStepClockConfig) {
    if (!Number.isFinite(config.hz) || config.hz <= 0) {
      throw new Error("FixedStepClock hz must be finite and > 0");
    }
    if (!Number.isInteger(config.maxStepsPerAdvance) || config.maxStepsPerAdvance <= 0) {
      throw new Error("FixedStepClock maxStepsPerAdvance must be a positive integer");
    }

    this.hz = config.hz;
    this.stepSeconds = 1 / config.hz;
    this.maxStepsPerAdvance = config.maxStepsPerAdvance;
  }

  advance(engine: SceneEngine, elapsedSeconds: number): FixedStepAdvanceResult {
    if (!Number.isFinite(elapsedSeconds) || elapsedSeconds < 0) {
      throw new Error("FixedStepClock elapsed time must be finite and >= 0");
    }

    this.phase += elapsedSeconds * this.hz;

    // Floating-point renderer deltas such as 1/60 * 60 should count as one
    // whole fixed step. The epsilon corrects representation noise, not model
    // semantics.
    const availableWholeSteps = Math.floor(this.phase + 1e-9);
    const executedSteps = Math.min(availableWholeSteps, this.maxStepsPerAdvance);
    const droppedSteps = Math.max(0, availableWholeSteps - executedSteps);

    for (let index = 0; index < executedSteps; index += 1) {
      engine.step(this.stepSeconds);
    }

    // Consume every whole step represented by this observation. Executed
    // steps became scene time; dropped steps become explicit epistemic facts.
    this.phase -= availableWholeSteps;
    if (Math.abs(this.phase) < 1e-12) this.phase = 0;

    this.totalExecutedSteps += executedSteps;
    this.totalDroppedSteps += droppedSteps;

    return {
      executedSteps,
      droppedSteps,
      droppedSeconds: droppedSteps * this.stepSeconds,
      phase: this.phase,
    };
  }

  observation(): FixedStepClockObservation {
    return {
      hz: this.hz,
      stepSeconds: this.stepSeconds,
      maxStepsPerAdvance: this.maxStepsPerAdvance,
      phase: this.phase,
      totalExecutedSteps: this.totalExecutedSteps,
      totalDroppedSteps: this.totalDroppedSteps,
      totalDroppedSeconds: this.totalDroppedSteps * this.stepSeconds,
    };
  }
}
