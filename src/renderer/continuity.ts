export type ContinuityTermination = "continuous" | "semantic-termination";

export interface ContinuitySubjectObservation {
  semanticExists: boolean;
  handleId: string | null;
  mounted: boolean;
  visible: boolean;
  renderable: boolean;
  alpha: number;
  finiteGeometry: boolean;
}

export interface ContinuitySample {
  index: number;
  sceneTime: number;
  subject: ContinuitySubjectObservation;
}

export interface ContinuityTrace {
  entityId: string;
  termination: ContinuityTermination;
  samples: readonly ContinuitySample[];
}

export type ContinuityLaw =
  | "TRACE_INDEX_IS_STRICTLY_INCREASING"
  | "TRACE_SCENE_TIME_IS_MONOTONIC"
  | "PROVIDER_ABSENT_BEFORE_SEMANTIC_BIRTH"
  | "PROVIDER_HANDLE_PRESENT_WHILE_SEMANTIC_IDENTITY_LIVES"
  | "HANDLE_STABLE_WHILE_SEMANTIC_IDENTITY_LIVES"
  | "PROVIDER_MOUNTED_WHILE_SEMANTIC_IDENTITY_LIVES"
  | "PROVIDER_VISIBLE_WHILE_SEMANTIC_IDENTITY_LIVES"
  | "PROVIDER_RENDERABLE_WHILE_SEMANTIC_IDENTITY_LIVES"
  | "PROVIDER_NONZERO_ALPHA_WHILE_SEMANTIC_IDENTITY_LIVES"
  | "PROVIDER_GEOMETRY_FINITE"
  | "SEMANTIC_IDENTITY_PERSISTS_FOR_CONTINUOUS_TRANSITION"
  | "SEMANTIC_TERMINATION_IS_ONE_WAY"
  | "PROVIDER_DOES_NOT_OUTLIVE_SEMANTIC_TERMINATION";

export interface ContinuityViolation {
  law: ContinuityLaw;
  sampleIndex: number;
  message: string;
}

/**
 * Pure no-cutoff oracle.
 *
 * Let E(t) be semantic identity membership and H(t) the provider handle.
 *
 * For a continuous transition over interval I:
 *
 *   e ∈ E(t) for every t ∈ I
 *   H_e(t) = constant != ∅
 *
 * and every provider projection must remain:
 *
 *   mounted ∧ visible ∧ renderable ∧ alpha > 0 ∧ finiteGeometry
 *
 * For explicit semantic termination at τ:
 *
 *   e ∈ E(t) for t < τ
 *   e ∉ E(t) for t >= τ
 *
 * Once termination occurs, resurrection inside the same transition trace is
 * illegal. A new semantic birth must be represented by a new lifecycle trace,
 * not hidden inside the old one.
 *
 * The function only observes. It never repairs provider state.
 */
export function inspectRendererContinuity(trace: ContinuityTrace): ContinuityViolation[] {
  const violations: ContinuityViolation[] = [];

  let born = false;
  let terminated = false;
  let stableHandle: string | null = null;
  let previousIndex = Number.NEGATIVE_INFINITY;
  let previousSceneTime = Number.NEGATIVE_INFINITY;

  for (const sample of trace.samples) {
    if (sample.index <= previousIndex) {
      violations.push({
        law: "TRACE_INDEX_IS_STRICTLY_INCREASING",
        sampleIndex: sample.index,
        message: "Continuity sample indices must be strictly increasing",
      });
    }
    if (sample.sceneTime < previousSceneTime) {
      violations.push({
        law: "TRACE_SCENE_TIME_IS_MONOTONIC",
        sampleIndex: sample.index,
        message: "Continuity scene time must be monotonic",
      });
    }

    previousIndex = sample.index;
    previousSceneTime = sample.sceneTime;

    const subject = sample.subject;

    if (!subject.semanticExists) {
      if (!born) {
        if (
          subject.handleId !== null ||
          subject.mounted ||
          subject.visible ||
          subject.renderable ||
          subject.alpha > 0
        ) {
          violations.push({
            law: "PROVIDER_ABSENT_BEFORE_SEMANTIC_BIRTH",
            sampleIndex: sample.index,
            message: "Provider projection exists before semantic identity birth",
          });
        }
        continue;
      }

      if (trace.termination === "continuous") {
        violations.push({
          law: "SEMANTIC_IDENTITY_PERSISTS_FOR_CONTINUOUS_TRANSITION",
          sampleIndex: sample.index,
          message: "Semantic identity disappeared during a transition declared continuous",
        });
      }

      terminated = true;

      if (
        subject.handleId !== null ||
        subject.mounted ||
        subject.visible ||
        subject.renderable ||
        subject.alpha > 0
      ) {
        violations.push({
          law: "PROVIDER_DOES_NOT_OUTLIVE_SEMANTIC_TERMINATION",
          sampleIndex: sample.index,
          message: "Provider projection remained after semantic identity termination",
        });
      }

      continue;
    }

    if (terminated) {
      violations.push({
        law: "SEMANTIC_TERMINATION_IS_ONE_WAY",
        sampleIndex: sample.index,
        message: "Semantic identity reappeared after termination inside one lifecycle trace",
      });
    }

    if (!born) {
      born = true;
      stableHandle = subject.handleId;
    }

    if (subject.handleId === null) {
      violations.push({
        law: "PROVIDER_HANDLE_PRESENT_WHILE_SEMANTIC_IDENTITY_LIVES",
        sampleIndex: sample.index,
        message: "Semantic identity exists but has no provider projection handle",
      });
    } else if (stableHandle !== null && subject.handleId !== stableHandle) {
      violations.push({
        law: "HANDLE_STABLE_WHILE_SEMANTIC_IDENTITY_LIVES",
        sampleIndex: sample.index,
        message: "Provider handle changed while semantic identity remained alive",
      });
    }

    if (!subject.mounted) {
      violations.push({
        law: "PROVIDER_MOUNTED_WHILE_SEMANTIC_IDENTITY_LIVES",
        sampleIndex: sample.index,
        message: "Provider handle is detached while semantic identity remains alive",
      });
    }

    if (!subject.visible) {
      violations.push({
        law: "PROVIDER_VISIBLE_WHILE_SEMANTIC_IDENTITY_LIVES",
        sampleIndex: sample.index,
        message: "Provider handle is hidden while semantic identity remains alive",
      });
    }

    if (!subject.renderable) {
      violations.push({
        law: "PROVIDER_RENDERABLE_WHILE_SEMANTIC_IDENTITY_LIVES",
        sampleIndex: sample.index,
        message: "Provider handle is non-renderable while semantic identity remains alive",
      });
    }

    if (!Number.isFinite(subject.alpha) || subject.alpha <= 0) {
      violations.push({
        law: "PROVIDER_NONZERO_ALPHA_WHILE_SEMANTIC_IDENTITY_LIVES",
        sampleIndex: sample.index,
        message: "Provider alpha is zero/non-finite while semantic identity remains alive",
      });
    }

    if (!subject.finiteGeometry) {
      violations.push({
        law: "PROVIDER_GEOMETRY_FINITE",
        sampleIndex: sample.index,
        message: "Provider geometry is non-finite while semantic identity remains alive",
      });
    }
  }

  return violations;
}
