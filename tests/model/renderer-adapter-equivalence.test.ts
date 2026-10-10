import { describe, expect, it } from "vitest";
import { asEntityId } from "../../src/algebra/laws";
import { zeroVelocity } from "../../src/dynamics/math";
import { buildRendererFrame, validateRendererResult } from "../../src/renderer/contract";
import { SceneEngine } from "../../src/scene/engine";
import { CommandRenderer } from "../../adapters/nonvisual";
import { SvgStringRenderer } from "../../adapters/svg";

describe("MQ-11 replaceable renderer adapters", () => {
  it("projects one render frame through nonvisual and SVG adapters without changing identity", () => {
    const visible = asEntityId("visible");
    const hidden = asEntityId("hidden");
    const scene = new SceneEngine();

    scene.mutate({
      add: [
        {
          id: visible,
          role: "card",
          pose: {
            position: { x: 100, y: 120 },
            rotation: 0.1,
            scale: { x: 1, y: 1 },
          },
          velocity: zeroVelocity(),
          shape: { kind: "rect", width: 80, height: 120, radius: 12 },
          z: 0,
          visible: true,
          parent: null,
        },
        {
          id: hidden,
          role: "decoration",
          pose: {
            position: { x: 0, y: 0 },
            rotation: 0,
            scale: { x: 1, y: 1 },
          },
          velocity: zeroVelocity(),
          shape: { kind: "rect", width: 10, height: 10, radius: 0 },
          z: 1,
          visible: false,
          parent: null,
        },
      ],
    });

    const frame = buildRendererFrame(scene.snapshot(), {
      sequence: 12,
      motionMode: "normal",
      accessibility: [
        {
          entityId: visible,
          role: "button",
          label: "Visible card",
        },
      ],
    });

    const commands = new CommandRenderer().render(frame);
    const svg = new SvgStringRenderer({
      width: 800,
      height: 600,
    }).render(frame);

    validateRendererResult(frame, commands);
    validateRendererResult(frame, svg);

    expect(commands.emittedEntityIds).toEqual([visible]);
    expect(svg.emittedEntityIds).toEqual([visible]);
    expect(commands.omitted).toEqual([{ entityId: hidden, reason: "not-visible" }]);
    expect(svg.omitted).toEqual([{ entityId: hidden, reason: "not-visible" }]);

    expect(commands.output[0]?.entityId).toBe(visible);
    expect(svg.output).toContain('data-entity-id="visible"');
    expect(svg.output).toContain('role="button"');
    expect(svg.output).toContain('aria-label="Visible card"');

    expect(scene.snapshot().entities.get(visible)?.pose.position).toEqual({
      x: 100,
      y: 120,
    });
  });

  it("produces deterministic adapter output for the same frame", () => {
    const id = asEntityId("card");
    const scene = new SceneEngine();

    scene.mutate({
      add: [
        {
          id,
          role: "card",
          pose: {
            position: { x: 50, y: 70 },
            rotation: 0,
            scale: { x: 1, y: 1 },
          },
          velocity: zeroVelocity(),
          shape: { kind: "rect", width: 72, height: 112, radius: 8 },
          z: 0,
          visible: true,
          parent: null,
        },
      ],
    });

    const frame = buildRendererFrame(scene.snapshot(), {
      sequence: 1,
      motionMode: "reduced",
      accessibility: [],
    });

    const commandRenderer = new CommandRenderer();
    const svgRenderer = new SvgStringRenderer({
      width: 400,
      height: 300,
    });

    expect(commandRenderer.render(frame)).toEqual(commandRenderer.render(frame));
    expect(svgRenderer.render(frame)).toEqual(svgRenderer.render(frame));
  });
});
