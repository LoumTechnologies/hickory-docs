import { defineConfig, mergeConfig } from "vitest/config";
import base from "./vite.config";
export default defineConfig(env => mergeConfig(base(env), {
  build: { outDir: "dist-simple", rollupOptions: { input: ["index.html", "simple.html"] } },
}));
