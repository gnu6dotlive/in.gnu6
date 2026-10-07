import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));

function path(relative) {
  return join(root, relative);
}

const failures = [];
const passes = [];

function gate(name, condition, detail) {
  if (condition) {
    passes.push({ name, detail });
  } else {
    failures.push({ name, detail });
  }
}

function text(relative) {
  return readFileSync(path(relative), "utf8");
}

function json(relative) {
  return JSON.parse(text(relative));
}

const questlog = text("QUESTLOG.md");
const readme = text("README.md");
const contract = text("CONTRACT.md");
const pkg = json("package.json");

for (let index = 0; index <= 15; index += 1) {
  const id = String(index).padStart(2, "0");
  const block = questBlock(questlog, id);

  gate(
    "MQ-" + id + " ratified",
    block.includes("State: **RATIFIED"),
    block ? "state line must begin with RATIFIED" : "quest block missing",
  );
}

function questBlock(source, id) {
  const heading = "## MQ-" + id;
  const start = source.indexOf(heading);
  if (start < 0) return "";

  const nextQuest = source.indexOf("\n## MQ-", start + heading.length);
  const sideQuestRegistry = source.indexOf("\n# Side Quest Registry", start + heading.length);
  const candidates = [nextQuest, sideQuestRegistry].filter((position) => position >= 0);
  const end = candidates.length > 0 ? Math.min(...candidates) : source.length;

  return source.slice(start, end);
}

const requiredEvidence = [
  "bootstrap-2026-10-06.md",
  "mq-01-formal-scene-ontology-2026-10-07.md",
  "mq-02-spatial-topology-2026-10-06.md",
  "mq-03-deterministic-time-2026-10-06.md",
  "mq-04-environment-space-2026-10-06.md",
  "mq-05-topology-closure-2026-10-07.md",
  "mq-06-deck-topology-2026-10-06.md",
  "mq-07-constraints-admissibility-2026-10-06.md",
  "mq-08-interaction-authority-2026-10-06.md",
  "mq-09-introspection-2026-10-06.md",
  "mq-10-multi-consumer-2026-10-06.md",
  "mq-11-renderer-contract-2026-10-06.md",
  "mq-12-pixi-browser-2026-10-07.md",
  "mq-13-motion-continuity-2026-10-07.md",
  "mq-14-performance-envelope-2026-10-07.md",
  "mq-15-completeness-evaluation-2026-10-07.md",
];

for (const receipt of requiredEvidence) {
  const relative = "docs/evidence/" + receipt;
  gate("evidence " + receipt, existsSync(path(relative)), "required durable Main Quest receipt");

  if (receipt.startsWith("mq-") && existsSync(path(relative))) {
    const receiptText = text(relative);
    gate(
      "receipt ratified " + receipt,
      receiptText.includes("Status: **RATIFIED**") &&
        receiptText.includes("Disposition: **RATIFIED**"),
      "ratified Main Quest receipt must state status and disposition",
    );
  }
}

const requiredAssets = [
  "mq-12-pixi-reference-browser.png",
  "mq-14-core-envelope.json",
  "mq-14-browser-envelope.json",
];

for (const asset of requiredAssets) {
  gate(
    "evidence asset " + asset,
    existsSync(path("docs/evidence/assets/" + asset)),
    "required reference/measurement asset",
  );
}

gate(
  "repository remains private",
  pkg.private === true && pkg.version === "0.0.0-private" && pkg.license === "UNLICENSED",
  "package.json must not imply a public API/release",
);

gate(
  "reference renderer version",
  pkg.dependencies?.["pixi.js"] === "8.22.0",
  "MQ-12 ratified PixiJS 8.22.0 as reference provider",
);

for (const script of [
  "check",
  "test:browser",
  "test:performance:core",
  "test:performance:browser",
]) {
  gate(
    "deterministic script " + script,
    typeof pkg.scripts?.[script] === "string",
    "required qualification mechanic",
  );
}

const requiredCoreSurfaces = [
  "src/scene/engine.ts",
  "src/scene/query.ts",
  "src/scene/lifecycle.ts",
  "src/spatial/frame.ts",
  "src/spatial/environment.ts",
  "src/topology/instance.ts",
  "src/interaction/projection.ts",
  "src/introspection/frame.ts",
  "src/renderer/contract.ts",
  "src/renderer/continuity.ts",
];

for (const surface of requiredCoreSurfaces) {
  gate("core surface " + surface, existsSync(path(surface)), "required ratified semantic surface");
}

for (const consumer of ["consumers/docklab/harness.ts", "consumers/spj-deck/harness.ts"]) {
  gate(
    "consumer proof " + consumer,
    existsSync(path(consumer)),
    "MQ-10 requires two distinct consumers",
  );
}

for (const referenceSurface of [
  "adapters/pixi.ts",
  "qualification/pixi/main.ts",
  "tests/browser/pixi-reference.pw.ts",
]) {
  gate(
    "reference browser surface " + referenceSurface,
    existsSync(path(referenceSurface)),
    "MQ-12/MQ-13 browser proof surface",
  );
}

const srcFiles = walk(path("src"));
const srcText = srcFiles
  .filter((file) => file.endsWith(".ts"))
  .map((file) => readFileSync(file, "utf8"))
  .join("\n");

const forbiddenCoreNames = [
  /pixi(?:js|\.js)?/i,
  /docklab/i,
  /sp\s*joaillerie/i,
  /spj:/i,
  /react/i,
  /playwright/i,
  /vite/i,
];

for (const pattern of forbiddenCoreNames) {
  gate(
    "core neutral " + pattern,
    !pattern.test(srcText),
    "provider/consumer names must stay outside src ontology",
  );
}

const engineDeclarations = srcText.match(/export\s+class\s+\w*Engine\b/g) ?? [];

gate(
  "single semantic engine class",
  engineDeclarations.length === 1 && engineDeclarations[0] === "export class SceneEngine",
  "exactly one exported semantic Engine class is allowed under src",
);

gate(
  "README renderer truth",
  readme.includes("PixiJS 8.22.0 / WebGL") && !readme.includes("Renderer choice: **UNDECIDED**"),
  "README must describe the ratified MQ-12 renderer reference",
);

gate(
  "README consumer truth",
  !readme.includes("first planned falsification consumers") &&
    readme.includes("Docklab") &&
    readme.includes("SP Joaillerie"),
  "README must describe consumers as proven rather than merely planned",
);

gate(
  "contract keeps renderer replaceable",
  contract.includes(
    "DOM, SVG, Canvas, PixiJS, WebGL, WebGPU or later renderers are replaceable providers.",
  ),
  "renderer selection must not become ontology authority",
);

const coreEnvelope = json("docs/evidence/assets/mq-14-core-envelope.json");
const browserEnvelope = json("docs/evidence/assets/mq-14-browser-envelope.json");

gate(
  "performance core evidence clean",
  coreEnvelope.source?.treeStatus === "" &&
    coreEnvelope.fixedStep?.some((point) => point.count === 2048),
  "MQ-14 core envelope must come from clean source and include high-cardinality point",
);

gate(
  "performance browser evidence clean",
  browserEnvelope.source?.treeStatus === "" &&
    browserEnvelope.instrument?.pixiVersion === "8.22.0" &&
    browserEnvelope.instrument?.backend === "webgl" &&
    browserEnvelope.instrument?.tickerStarted === false &&
    browserEnvelope.projection?.some((point) => point.entityCount === 512),
  "MQ-14 browser envelope must preserve MQ-12 provider boundary and measured scale",
);

gate(
  "browser continuity proof named",
  text("tests/browser/pixi-reference.pw.ts").includes("MQ-13 no-cutoff ledger"),
  "MQ-13 browser continuity gate must remain addressable",
);

gate(
  "current program is complete for current mandate",
  questlog.includes("MQ-15: RATIFIED COMPLETE FOR CURRENT MANDATE") &&
    questlog.includes("No mandatory Main Quest remains under the current ratified mandate."),
  "Questlog must describe the completed Main Questline truthfully",
);

const result = {
  status: failures.length === 0 ? "PASS" : "FAIL",
  passed: passes.length,
  failed: failures.length,
  failures,
};

process.stdout.write(JSON.stringify(result, null, 2) + "\n");

if (failures.length > 0) {
  process.exit(1);
}

function walk(directory) {
  const files = [];

  for (const entry of readdirSync(directory, {
    withFileTypes: true,
  })) {
    const child = join(directory, entry.name);

    if (entry.isDirectory()) {
      files.push(...walk(child));
    } else {
      files.push(child);
    }
  }

  return files;
}
