import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { chromium } from "playwright";
import { createServer } from "vite";

const outputPath = resolve(
  process.cwd(),
  process.argv[2] ?? "docs/evidence/assets/mq-14-browser-envelope.json",
);

const port = Number(process.env.IN_GNU6_MQ14_BENCH_PORT ?? "4194");
if (!Number.isInteger(port) || port <= 0) {
  throw new Error("Invalid MQ-14 browser benchmark port");
}

const head = execFileSync("git", ["rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();

const server = await createServer({
  configFile: resolve(process.cwd(), "qualification/pixi/vite.config.ts"),
  server: {
    host: "127.0.0.1",
    port,
    strictPort: true,
  },
  logLevel: "error",
});

let browser = null;

try {
  await server.listen();

  browser = await chromium.launch({
    headless: true,
    args: ["--enable-precise-memory-info"],
  });

  const page = await browser.newPage({
    viewport: { width: 1280, height: 900 },
  });

  await page.goto("http://127.0.0.1:" + port + "/");
  await page.waitForFunction(() => {
    return window.__IN_GNU6_PIXI_QUALIFICATION__?.ready === true;
  });

  const userAgent = await page.evaluate(() => navigator.userAgent);
  const results = [];

  for (const entityCount of [16, 64, 256, 512]) {
    results.push(
      await page.evaluate(
        ({ count, iterations }) => {
          return window.__IN_GNU6_PIXI_QUALIFICATION__.benchmarkProjection(count, iterations);
        },
        { count: entityCount, iterations: 60 },
      ),
    );
  }

  const report = {
    schemaVersion: 1,
    observedAt: new Date().toISOString(),
    source: {
      gitHead: head,
      treeStatus: execFileSync("git", ["status", "--short"], {
        encoding: "utf8",
      }).trim(),
    },
    instrument: {
      browserVersion: browser.version(),
      userAgent,
      pixiVersion: "8.22.0",
      backend: results[0]?.diagnostics.rendererName ?? "unknown",
      tickerStarted: results[0]?.diagnostics.tickerStarted ?? null,
      resolution: 1,
      antialias: false,
      viewport: { width: 1280, height: 900 },
    },
    methodology: {
      entityCounts: [16, 64, 256, 512],
      iterationsPerCount: 60,
      warmupRenders: 10,
      note: "Headless Chrome reference-envelope observation. JS heap is browser-observed heap, not GPU memory.",
    },
    projection: results,
  };

  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, JSON.stringify(report, null, 2) + "\n");

  process.stdout.write(
    JSON.stringify(
      {
        outputPath,
        browserVersion: report.instrument.browserVersion,
        backend: report.instrument.backend,
        projection: results.map((result) => ({
          entityCount: result.entityCount,
          frameBuildMeanMs: result.frameBuild.meanMs,
          pixiRenderMeanMs: result.pixiRender.meanMs,
          pixiRenderP95Ms: result.pixiRender.p95Ms,
          jsHeapUsedBytes: result.jsHeap?.usedBytes ?? null,
        })),
      },
      null,
      2,
    ) + "\n",
  );
} finally {
  await browser?.close();
  await server.close();
}
