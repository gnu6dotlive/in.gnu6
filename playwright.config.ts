import { defineConfig } from "@playwright/test";

const qualificationPort = Number(
  process.env.IN_GNU6_QUALIFICATION_PORT ?? "4176",
);
const qualificationUrl =
  "http://127.0.0.1:" + qualificationPort;

export default defineConfig({
  testDir: "./tests/browser",
  testMatch: "**/*.pw.ts",
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  fullyParallel: false,
  workers: 1,
  reporter: [["line"]],
  use: {
    baseURL: qualificationUrl,
    viewport: { width: 1280, height: 900 },
    trace: "off",
    screenshot: "only-on-failure",
    video: "off",
  },
  webServer: {
    command:
      "IN_GNU6_QUALIFICATION_PORT=" +
      qualificationPort +
      " mise exec node@24 -- node node_modules/vite/bin/vite.js --config qualification/pixi/vite.config.ts",
    url: qualificationUrl,
    reuseExistingServer: false,
    gracefulShutdown: {
      signal: "SIGTERM",
      timeout: 2_000,
    },
    timeout: 120_000,
  },
});
