<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/playwright.embed.config.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6fcf90daa368"></a>
## `apps/web/playwright.embed.config.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Observations read beside their captured source](../../../guarantees/debugging/source-anchored-observation-reading/contract.md).

<a id="f-6fcf90daa368-outputDir"></a>
### `outputDir`

<storycode:copy id="f-6fcf90daa368-outputDir">
import { defineConfig, devices } from "@playwright/test";

// Concurrent agent/CI runs must never clean up another run’s traces.
const outputDir = process.env.HICKORY_E2E_OUTPUT_DIR ?? `test-results/browser-embedding-${process.pid}`;
</storycode:copy>
<a id="f-6fcf90daa368-export_statement"></a>
### `export_statement`

<storycode:copy id="f-6fcf90daa368-export_statement">
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
</storycode:copy>

### Assemble `apps/web/playwright.embed.config.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/playwright.embed.config.ts"><storycode:paste select="#f-6fcf90daa368-outputDir"/><storycode:paste select="#f-6fcf90daa368-export_statement"/></storycode:file>

</storycode:doc>
