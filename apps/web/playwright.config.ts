import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 20_000,
  use: {
    baseURL: process.env.HICKORY_E2E_URL,
    channel: process.env.HICKORY_E2E_CHANNEL ?? "chrome",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
});
