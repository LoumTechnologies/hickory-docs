<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/playwright.config.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1ccf286d0c56"></a>
## `apps/web/playwright.config.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-1ccf286d0c56-outputDir"></a>
### `outputDir`

<storycode:copy id="f-1ccf286d0c56-outputDir">
import { defineConfig } from "@playwright/test";

// Concurrent agent/CI runs must never clean up another run’s traces.
const outputDir = process.env.HICKORY_E2E_OUTPUT_DIR ?? `test-results/browser-${process.pid}`;

</storycode:copy>
<a id="f-1ccf286d0c56-export_statement"></a>
### `export_statement`

<storycode:copy id="f-1ccf286d0c56-export_statement">
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
</storycode:copy>

### Assemble `apps/web/playwright.config.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/playwright.config.ts"><storycode:paste select="#f-1ccf286d0c56-outputDir"/><storycode:paste select="#f-1ccf286d0c56-export_statement"/></storycode:file>

</storycode:doc>
