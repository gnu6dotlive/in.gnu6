import { defineConfig } from "vite";

export default defineConfig({
  root: "qualification/pixi",
  server: {
    host: "127.0.0.1",
    port: 4176,
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
