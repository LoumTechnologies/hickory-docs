import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [react()],
  base: "./",
  publicDir: false,
  build: {
    outDir: "dist-ci",
    rollupOptions: { input: resolve(import.meta.dirname, "ci.html") },
  },
});
