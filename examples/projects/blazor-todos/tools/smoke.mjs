import { randomBytes } from "node:crypto";
import { createRequire } from "node:module";
import assert from "node:assert/strict";

const require = createRequire(new URL("../../../../apps/web/package.json", import.meta.url));
const { chromium } = require("playwright");
const base = new URL(process.argv[2]);
assert.ok(["127.0.0.1", "localhost"].includes(base.hostname), "Use a local demo database.");

const password = "Todo!" + randomBytes(12).toString("hex") + "Aa1";
const runId = randomBytes(8).toString("hex");
const account = (label) => `${label}-${runId}@example.test`;
const goto = (page, path) => page.goto(new URL(path, base).href);
const button = (page, name) => page.getByRole("button", { name, exact: true });
async function ready(page) {
    await page.getByRole("heading", { name: "My todos", exact: true }).waitFor();
    await button(page, "Add todo").click({ trial: true });
}

async function register(page, email) {
    await goto(page, "/Account/Register?ReturnUrl=%2Ftodos");
    await page.getByLabel("Email", { exact: true }).fill(email);
    await page.getByLabel("Password", { exact: true }).fill(password);
    await page.getByLabel("Confirm Password").fill(password);
    await button(page, "Register").click();
    await ready(page);
}

const browser = await chromium.launch();
try {
    const first = await browser.newPage();
    await goto(first, "/todos");
    await first.waitForURL((url) => url.pathname === "/Account/Login");

    await register(first, account("first"));
    await button(first, "Add todo").click();
    await first.getByText("The Title field is required.").waitFor();
    await first.getByLabel("Title", { exact: true }).fill("Learn literate patches");
    await button(first, "Add todo").click();
    let row = first.getByRole("row", { name: /Learn literate patches/ });
    await row.getByRole("button", { name: "Edit" }).click();
    await button(first, "Save todo").waitFor();
    await first.getByLabel("Title", { exact: true }).fill("Learn the C# code model");
    await first.getByLabel("Completed", { exact: true }).check();
    await button(first, "Save todo").click();
    row = first.getByRole("row", { name: /Learn the C# code model/ });
    await row.getByRole("cell", { name: "Yes", exact: true }).waitFor();

    const second = await browser.newPage();
    await register(second, account("second"));
    await second.getByText("No todos yet.").waitFor();
    assert.equal(await second.getByRole("row", { name: /Learn the C# code model/ }).count(), 0);

    await button(first, "Sign out").click();
    await first.getByRole("link", { name: "Log in", exact: true }).waitFor();
    await goto(first, "/Account/Login?ReturnUrl=%2Ftodos");
    await first.getByLabel("Email", { exact: true }).fill(account("first"));
    await first.getByLabel("Password", { exact: true }).fill(password);
    await button(first, "Log in").click();
    await ready(first);
    await row.getByRole("cell", { name: "Yes", exact: true }).waitFor();
    await row.getByRole("button", { name: "Delete" }).click();
    await first.getByText("No todos yet.").waitFor();
    await first.reload();
    await ready(first);
    await first.getByText("No todos yet.").waitFor();
    console.log("Password login, todo validation and CRUD, persistence, and account isolation passed.");
} finally {
    await browser.close();
}
