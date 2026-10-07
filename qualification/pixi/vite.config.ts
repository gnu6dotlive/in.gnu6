import { defineConfig } from "vite";

const qualificationPort = Number(
  process.env.IN_GNU6_QUALIFICATION_PORT ?? "4176",
);

export default defineConfig({
  root: "qualification/pixi",
  server: {
    host: "127.0.0.1",
    port: qualificationPort,
    strictPort: true,
    fs: {
      allow: ["."],
    },
  },
  build: {
    outDir: "../../.tmp/pixi-qualification-dist",
    emptyOutDir: true,
  },
});
