import { defineConfig } from "@playwright/test";

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
    baseURL: "http://127.0.0.1:4176",
    viewport: { width: 1280, height: 900 },
    trace: "off",
    screenshot: "only-on-failure",
    video: "off",
  },
  webServer: {
    command: "mise exec node@24 -- corepack npx vite --config qualification/pixi/vite.config.ts",
    url: "http://127.0.0.1:4176",
    reuseExistingServer: true,
    gracefulShutdown: {
      signal: "SIGTERM",
      timeout: 2_000,
    },
    timeout: 120_000,
  },
});
