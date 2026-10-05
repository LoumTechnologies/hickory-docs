import { defineConfig, devices } from "@playwright/test";
export default defineConfig({
  testDir: "./e2e", testMatch: "browserEmbedding.spec.ts", timeout: 45_000,
  use: { baseURL: "http://127.0.0.1:4178", trace: "retain-on-failure", screenshot: "only-on-failure" },
  reporter: [["list"], ["json", { outputFile: "test-results/browser-embedding.json" }]],
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } },
  ],
  webServer: { command: "npm run build:site && npx vite preview --config vite.site.config.ts --host 127.0.0.1 --port 4178", url: "http://127.0.0.1:4178", timeout: 120_000, reuseExistingServer: false },
});
