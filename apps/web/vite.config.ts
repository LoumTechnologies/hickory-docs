import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Dev-server env (no @types/node dependency; Vite runs under Node).
const env: Record<string, string | undefined> =
  (globalThis as { process?: { env: Record<string, string | undefined> } }).process?.env ?? {};

export default defineConfig({
  plugins: [react()],
  server: {
    // Ports come from the environment (scripts/dev.sh exports them from
    // the gitignored .env); the values here are only last-resort defaults.
    port: Number(env.WEB_PORT ?? 5173),
    proxy: {
      "/api": {
        target: `http://localhost:${env.HICKORY_SERVER_PORT ?? 8080}`,
        changeOrigin: true,
        ws: true,
      },
    },
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
  },
});
