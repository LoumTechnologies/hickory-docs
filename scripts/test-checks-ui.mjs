// Contract: .hickory-source/mechanisms/optional-checks.md
import { chromium } from "../apps/web/node_modules/playwright-core/index.mjs";
import { execFileSync } from "node:child_process";
const [url, provider] = process.argv.slice(2);
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage();
  const response = await page.request.get(`${url}/api/projects/local/docs`);
  const docs = await response.json();
  const doc = docs.find(doc => doc.path === "note.md");
  if (!doc) throw new Error("Missing scratch note");
  await page.goto(`${url}/#/docs/${doc.id}`);
  await page.getByRole("button", { name: "Checks", exact: true }).click();
  const pane = page.getByRole("region", { name: "Checks", exact: true });
  await pane.getByText("passed (0)", { exact: true }).first().waitFor();
  execFileSync(provider, ["service", "stop"], { env: process.env });
  await pane.getByText(/Last results below may be stale/).waitFor({ timeout: 20000 });
  if (await pane.getByText("passed (0)", { exact: true }).count() === 0) throw new Error("Lost stale results");
  await page.screenshot({ path: process.env.HICKORY_CHECKS_SCREENSHOT ?? "/tmp/hickory-checks.png", fullPage: true });
  console.log("Checks button, live results and stale provider recovery passed");
} finally {
  await browser.close();
}
