import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["tests/performance/**/*.perf.ts"],
    pool: "forks",
    maxWorkers: 1,
    minWorkers: 1,
  },
});
