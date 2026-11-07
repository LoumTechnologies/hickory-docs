import { defineConfig, mergeConfig } from "vitest/config";
import base from "./vite.config";
export default mergeConfig(base, defineConfig({
  build: { outDir: "dist-simple", rollupOptions: { input: "simple.html" } },
}));
