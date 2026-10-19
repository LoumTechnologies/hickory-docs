#!/usr/bin/env node
// An auditable index of control declarations, not proof of semantic parity.
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";

const root = fileURLToPath(new URL("../", import.meta.url));
const require = createRequire(new URL("../apps/web/package.json", import.meta.url));
const ts = require("typescript");
const files = execFileSync("rg", ["--files", "apps/web/src"], { cwd: root, encoding: "utf8" }).trim().split("\n")
  .filter(path => /\.tsx?$/.test(path) && !/\.(test|spec)\./.test(path)).sort();
const native = new Set(["button", "input", "select", "textarea", "summary", "details", "a", "form", "dialog"]);
const rows = [];
for (const path of files) {
  const text = readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
  const source = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true, path.endsWith("tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  const controls = [];
  const imperative = [];
  function visit(node) {
    if (ts.isJsxOpeningElement(node) || ts.isJsxSelfClosingElement(node)) {
      const tag = node.tagName.getText(source);
      const attrs = node.attributes.properties.filter(ts.isJsxAttribute);
      const events = attrs.filter(attr => /^on[A-Z]/.test(attr.name.getText(source))).map(attr => ({ name: attr.name.getText(source), handler: attr.initializer?.getText(source).slice(0, 240) ?? null }));
      if (native.has(tag) || events.length) {
        const label = attrs.find(attr => ["aria-label", "label", "title"].includes(attr.name.getText(source)));
        controls.push({ line: source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1, tag, label: label?.initializer?.getText(source) ?? null, events, spread: node.attributes.properties.some(ts.isJsxSpreadAttribute) });
      }
    }
    if (ts.isCallExpression(node)) {
      const expression = node.expression.getText(source);
      if (/\.(addEventListener|createElement|dispatch|dispatchEvent)$/.test(expression) || /(^|\.)(keymap\.of|invoke)$/.test(expression)) {
        imperative.push({ line: source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1, call: expression, arguments: node.arguments.map(arg => arg.getText(source).slice(0, 160)) });
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  // Include files with no declarations: this makes scan coverage reviewable.
  rows.push({ path, sha256: createHash("sha256").update(text).digest("hex"), controls, imperative });
}
const header = { version: 1, generator: "scripts/inventory-ui-actions.mjs", coverage: "All non-test TS/TSX under apps/web/src; native Rust/Swift and rendered values require the companion manual inventory. This declaration scan does not establish registration, reachability or parity." };
const result = JSON.stringify(header, null, 2).slice(0, -2) + ',\n  "files": [\n' + rows.map(row => `    ${JSON.stringify(row)}`).join(",\n") + "\n  ]\n}\n";
const target = new URL("../docs/developers/ui-control-inventory.json", import.meta.url);
if (process.argv.includes("--check")) {
  if (readFileSync(target, "utf8") !== result) { process.stderr.write("UI declaration inventory changed; regenerate and review semantic coverage.\n"); process.exitCode = 1; }
} else writeFileSync(target, result);
