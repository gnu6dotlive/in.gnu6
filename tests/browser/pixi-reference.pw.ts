import { expect, test } from "@playwright/test";
type Snapshot = {
  diagnostics: {
    rendererName: string;
    tickerStarted: boolean;
    graphicsHandles: number;
    semanticNodes: number;
    canvasWidth: number;
    canvasHeight: number;
  };
  sceneTime: number;
  motionMode: "normal" | "reduced";
  entities: Array<{
    id: string;
    pose: {
      position: { x: number; y: number };
      rotation: number;
      scale: { x: number; y: number };
    };
    shapeKind: "rect" | "polygon";
    z: number;
    handleToken: string | null;
  }>;
  ownership: ReadonlyArray<{
    entityId: string;
    degree: string;
    ownerId: string;
  }>;
  topologies: Array<{
    id: string;
    frameId: string;
    members: string[];
    solverKind: string;
  }>;
  frames: Array<{
    id: string;
    parent: string | null;
    origin: { x: number; y: number };
  }>;
  semanticNodes: Array<{
    id: string;
    role: string | null;
    label: string | null;
    hidden: boolean;
    tag: string;
  }>;
};

function snapshot(page: import("@playwright/test").Page): Promise<Snapshot> {
  return page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.snapshot();
  });
}

function entity(state: Snapshot, id: string) {
  const found = state.entities.find((candidate) => candidate.id === id);
  if (!found) throw new Error("Missing entity " + id);
  return found;
}

test("PixiJS 8.22 reference renderer preserves [in.gnu6] scene truth in a real browser", async ({
  page,
}) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));

  await page.goto("/");
  await expect(page.locator("#status")).toContainText("ready");

  const initial = await snapshot(page);

  expect(initial.diagnostics.rendererName.toLowerCase()).toContain("webgl");
  expect(initial.diagnostics.tickerStarted).toBe(false);
  expect(initial.diagnostics.graphicsHandles).toBe(10);
  expect(initial.diagnostics.semanticNodes).toBe(10);
  expect(initial.diagnostics.canvasWidth).toBe(1200);
  expect(initial.diagnostics.canvasHeight).toBe(720);

  expect(initial.topologies.map((topology) => topology.id)).toEqual(["dock", "hand", "playzone"]);
  expect(initial.frames.find((frame) => frame.id === "hand-frame")?.parent).toBe("world");
  expect(initial.frames.find((frame) => frame.id === "playzone-frame")?.parent).toBe("world");

  const dockBInitial = entity(initial, "dock-b");
  const dockCInitial = entity(initial, "dock-c");
  const initialHandle = dockCInitial.handleToken;

  // Overlap is real: center spacing is smaller than the 132px card width.
  expect(Math.abs(dockCInitial.pose.position.x - dockBInitial.pose.position.x)).toBeLessThan(132);

  // Continuous magnification / neighbor-yield path.
  await page.evaluate(async () => {
    await window.__IN_GNU6_PIXI_QUALIFICATION__.focusDock(2);
  });

  const focused = await snapshot(page);
  const dockBFocused = entity(focused, "dock-b");
  const dockCFocused = entity(focused, "dock-c");

  expect(dockCFocused.handleToken).toBe(initialHandle);
  expect(dockCFocused.pose.scale.x).toBeGreaterThan(dockCInitial.pose.scale.x);
  expect(dockBFocused.pose.position.x).not.toBe(dockBInitial.pose.position.x);

  // Shape topology morph keeps the exact same Pixi Graphics handle.
  const morphBefore = entity(focused, "morph-surface");
  await page.evaluate(async () => {
    await window.__IN_GNU6_PIXI_QUALIFICATION__.morphToDiamond();
  });
  const diamond = await snapshot(page);
  const morphDiamond = entity(diamond, "morph-surface");

  expect(morphDiamond.handleToken).toBe(morphBefore.handleToken);
  expect(morphDiamond.shapeKind).toBe("polygon");

  await page.evaluate(async () => {
    await window.__IN_GNU6_PIXI_QUALIFICATION__.morphToCard();
  });
  const cardMorph = await snapshot(page);
  expect(entity(cardMorph, "morph-surface").handleToken).toBe(morphBefore.handleToken);

  // Insertion/removal occurs downstream without replacing unrelated handles.
  const stableDockHandle = entity(cardMorph, "dock-a").handleToken;

  await page.evaluate(async () => {
    await window.__IN_GNU6_PIXI_QUALIFICATION__.insertCard();
  });
  const inserted = await snapshot(page);
  expect(entity(inserted, "inserted-card").handleToken).not.toBeNull();
  expect(entity(inserted, "dock-a").handleToken).toBe(stableDockHandle);

  await page.evaluate(async () => {
    await window.__IN_GNU6_PIXI_QUALIFICATION__.removeCard();
  });
  const removed = await snapshot(page);
  expect(removed.entities.some((candidate) => candidate.id === "inserted-card")).toBe(false);
  expect(entity(removed, "dock-a").handleToken).toBe(stableDockHandle);

  // Topology authority moves hand -> playzone while visual identity survives.
  const handBHandle = entity(removed, "hand-b").handleToken;

  await page.evaluate(async () => {
    await window.__IN_GNU6_PIXI_QUALIFICATION__.handoffToPlayzone();
  });
  const handedOff = await snapshot(page);

  expect(entity(handedOff, "hand-b").handleToken).toBe(handBHandle);
  expect(
    handedOff.ownership.find(
      (entry) => entry.entityId === "hand-b" && entry.degree === "position.x",
    )?.ownerId,
  ).toBe("playzone");

  // Accessibility is a sibling semantic projection, not canvas guesswork.
  const semanticHandB = handedOff.semanticNodes.find((node) => node.id === "hand-b");
  expect(semanticHandB).toMatchObject({
    role: "button",
    label: "Scene entity hand-b",
    hidden: false,
    tag: "BUTTON",
  });

  const parity = await page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.compareMotionParity();
  });

  expect(parity.semanticDestinationEqual).toBe(true);
  expect(parity.normalTarget).toEqual(parity.reducedTarget);
  expect(parity.reducedPose).toEqual(parity.reducedTarget);
  expect(parity.normalPose.position.x).toBeCloseTo(parity.normalTarget.position.x, 3);
  expect(parity.normalPose.position.y).toBeCloseTo(parity.normalTarget.position.y, 3);
  expect(parity.normalMotionMode).toBe("normal");
  expect(parity.reducedMotionMode).toBe("reduced");

  await page.screenshot({
    path: "docs/evidence/assets/mq-12-pixi-reference-browser.png",
    fullPage: true,
  });

  expect(pageErrors).toEqual([]);
});

test("MQ-13 no-cutoff ledger proves continuous and terminal browser transitions", async ({
  page,
}) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));

  await page.goto("/");
  await expect(page.locator("#status")).toContainText("ready");

  const focus = await page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.traceFocusDock();
  });
  expect(focus.violations).toEqual([]);
  expect(focus.traces[0]!.samples.length).toBeGreaterThan(10);
  expect(
    new Set(
      focus.traces[0]!.samples.filter((sample) => sample.subject.semanticExists).map(
        (sample) => sample.subject.handleId,
      ),
    ).size,
  ).toBe(1);

  const reflow = await page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.traceDeckReflow();
  });
  expect(reflow.violations).toEqual([]);
  expect(reflow.traces).toHaveLength(5);
  expect(reflow.traces.every((trace) => trace.samples.length > 10)).toBe(true);

  const morph = await page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.traceWindowLikeToCard();
  });
  expect(morph.violations).toEqual([]);
  expect(morph.traces[0]!.samples.length).toBeGreaterThan(20);

  const insertion = await page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.traceInsertCard();
  });
  expect(insertion.violations).toEqual([]);
  expect(insertion.traces[0]!.samples[0]!.subject.semanticExists).toBe(false);
  expect(insertion.traces[0]!.samples.at(-1)!.subject.semanticExists).toBe(true);

  const removal = await page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.traceRemoveCard();
  });
  expect(removal.violations).toEqual([]);
  expect(removal.traces[0]!.samples[0]!.subject.semanticExists).toBe(true);
  expect(removal.traces[0]!.samples.at(-1)!.subject.semanticExists).toBe(false);
  expect(removal.traces[0]!.samples.at(-1)!.subject.handleId).toBeNull();

  const handoff = await page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.traceHandoffToPlayzone();
  });
  expect(handoff.violations).toEqual([]);
  expect(handoff.traces[0]!.samples.length).toBeGreaterThan(20);

  const context = await page.evaluate(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__.traceContextSurfaceLifecycle();
  });
  expect(context.violations).toEqual([]);
  expect(context.result.attachmentObserved).toBe(true);

  const contextSamples = context.traces[0]!.samples;
  expect(contextSamples[0]!.subject.semanticExists).toBe(false);
  expect(contextSamples.some((sample) => sample.subject.semanticExists)).toBe(true);
  expect(contextSamples.at(-1)!.subject.semanticExists).toBe(false);
  expect(contextSamples.at(-1)!.subject.handleId).toBeNull();

  // Every live semantic sample across every continuous/terminal transition
  // remained projected, mounted, visible, renderable, opaque and finite.
  const allTraces = [
    ...focus.traces,
    ...reflow.traces,
    ...morph.traces,
    ...insertion.traces,
    ...removal.traces,
    ...handoff.traces,
    ...context.traces,
  ];

  for (const trace of allTraces) {
    for (const sample of trace.samples) {
      if (!sample.subject.semanticExists) continue;
      expect(sample.subject.handleId).not.toBeNull();
      expect(sample.subject.mounted).toBe(true);
      expect(sample.subject.visible).toBe(true);
      expect(sample.subject.renderable).toBe(true);
      expect(sample.subject.alpha).toBeGreaterThan(0);
      expect(sample.subject.finiteGeometry).toBe(true);
    }
  }

  expect(pageErrors).toEqual([]);
});
