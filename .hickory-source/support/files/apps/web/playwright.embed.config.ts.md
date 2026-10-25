<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/playwright.embed.config.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Observations read beside their captured source](../../../../guarantees/debugging/source-anchored-observation-reading/contract.md)

Captured SHA-256: `1129232ccb1343648a15f85699ff5e158d892cb3edd786c2f74912909ae6a3b0`.

<book:entry path="apps/web/playwright.embed.config.ts">
import { defineConfig, devices } from "@playwright/test";

// Concurrent agent/CI runs must never clean up another run’s traces.
const outputDir = process.env.HICKORY_E2E_OUTPUT_DIR ?? `test-results/browser-embedding-${process.pid}`;
export default defineConfig({
  outputDir,
  testDir: "./e2e", testMatch: "browserEmbedding.spec.ts", timeout: 45_000,
  use: { baseURL: "http://127.0.0.1:4178", trace: "retain-on-failure", screenshot: "only-on-failure" },
  reporter: [["list"], ["json", { outputFile: `${outputDir}/report.json` }]],
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } },
  ],
  webServer: { command: "npm run build:site && npx vite preview --config vite.site.config.ts --host 127.0.0.1 --port 4178", url: "http://127.0.0.1:4178", timeout: 120_000, reuseExistingServer: false },
});
</book:entry>

</book:doc>
