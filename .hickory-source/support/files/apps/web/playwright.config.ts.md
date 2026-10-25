<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/playwright.config.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `5acb88438e2bf509237bf150d36dfce6c691c97effcb3f2d8dd35f1b4191a571`.

<book:entry path="apps/web/playwright.config.ts">
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
</book:entry>

</book:doc>
