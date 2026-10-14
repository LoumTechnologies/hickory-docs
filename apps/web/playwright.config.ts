import { defineConfig } from "@playwright/test";

// Concurrent agent/CI runs must never clean up another run’s traces.
const outputDir = process.env.HICKORY_E2E_OUTPUT_DIR ?? `test-results/browser-${process.pid}`;

export default defineConfig({
  outputDir,
  testDir: "./e2e",
  timeout: 20_000,
  use: {
    baseURL: process.env.HICKORY_E2E_URL,
    channel: process.env.HICKORY_E2E_CHANNEL ?? "chrome",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
});
