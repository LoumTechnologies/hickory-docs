<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# scripts/inventory-ui-actions.mjs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../contract.md)

Captured SHA-256: `88c999e865b0a439e111e0107a629a349662eddd2cc119789c77e918f0a987f7`.

<book:entry path="scripts/inventory-ui-actions.mjs">
#!/usr/bin/env node
// An auditable index of control declarations, not proof of semantic parity.
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { uiReachability } from "./ui-reachability.mjs";
import { createHash } from "node:crypto";

const root = fileURLToPath(new URL("../", import.meta.url));
const require = createRequire(new URL("../apps/web/package.json", import.meta.url));
const ts = require("typescript");
const allFiles = execFileSync("rg", ["--files", "apps/web/src", "apps/desktop/src-tauri/src", "apps/desktop/fskit", "crates/hickory-cli/src/serve/shell.rs", "crates/hickory-cli/src/serve/reveal.rs"], { cwd: root, encoding: "utf8" }).trim().split("\n").sort();
const files = allFiles.filter(path => path.startsWith("apps/web/src/"))
  .filter(path => /\.tsx?$/.test(path) && !/\.(test|spec)\./.test(path)).sort();
const native = new Set(["button", "input", "select", "textarea", "summary", "details", "a", "form", "dialog"]);
const rows = [];
const moduleSources = {};
for (const path of files) {
  const text = readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
  moduleSources[path] = text;
  const source = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true, path.endsWith("tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  const controls = [];
  const imperative = [];
  const registrations = [];
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
      if (/^(useUiCommands|use[A-Z]\w*Actions)$/.test(expression)) registrations.push({ line: source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1, call: expression, arguments: node.arguments.map(arg => arg.getText(source).slice(0, 160)) });
      if (/\.(addEventListener|createElement|dispatch|dispatchEvent)$/.test(expression) || /(^|\.)(keymap\.of|invoke)$/.test(expression)) {
        imperative.push({ line: source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1, call: expression, arguments: node.arguments.map(arg => arg.getText(source).slice(0, 160)) });
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  // Include files with no declarations: this makes scan coverage reviewable.
  rows.push({ path, sha256: createHash("sha256").update(text).digest("hex"), controls, imperative, registrations });
}
const nativeRows = allFiles.filter(path => /\.(rs|swift)$/.test(path)).map(path => {
  const text = readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
  const declarations = [];
  const lines = text.split("\n");
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index].trim();
    let kind;
    if (/\b(?:fn|func)\s+\w+/.test(line)) kind = "function";
    else if (/pub\s+\w+\s*:\s*Arc<dyn Fn/.test(line)) kind = "host-callback";
    else if (/\.route\(|MenuItem::|PredefinedMenuItem::|\.on_menu_event\(/.test(line)) kind = "route-or-menu";
    else if (/^#\[(?:cfg|tauri::command)|^#(?:if|elseif|else|endif)\b/.test(line)) kind = "platform-or-command-attribute";
    if (kind) declarations.push({ line: index + 1, kind, declaration: lines.slice(index, index + 3).join("\n").slice(0, 320) });
  }
  return { path, sha256: createHash("sha256").update(text).digest("hex"), follow_up: [74, 75], declarations };
});
const entrypoints = {};
for (const filename of readdirSync(new URL("../apps/web/", import.meta.url)).filter(name => name.endsWith(".html")).sort()) {
  const html = readFileSync(new URL(`../apps/web/${filename}`, import.meta.url), "utf8");
  entrypoints[filename] = [...html.matchAll(/<script\b[^>]*\bsrc=["']([^"']+)["'][^>]*>/g)].map(match => match[1])
    .filter(src => /^(?:\.\/|\/)src\//.test(src)).map(src => `apps/web/${src.replace(/^(?:\.\/|\/)/, "").split("?")[0]}`);
}
const reachability = uiReachability(moduleSources, entrypoints);
for (const row of rows) row.possible_entrypoints = reachability.roots[row.path];
const header = { version: 3, generator: "scripts/inventory-ui-actions.mjs", coverage: "All non-test web TS/TSX; desktop Rust/FSKit Swift; host shell/reveal Rust. Declarations, adapter hook calls and compile guards are review aids, not proof of registration, reachability, parity or platform support. Rendered values and dynamic controls still require the companion manual inventory." };
header.reachability = { scope: "Conservative local module graph from every top-level web HTML script, including type-only, lazy, worker/raw-query imports and re-exports. Presence is potential import reachability, not rendered visibility; absence is not a proof of dead code or exemption from parity. Review unresolved paths and runtime injection before excluding anything.", entrypoints, unresolved: reachability.unresolved };
const result = JSON.stringify(header, null, 2).slice(0, -2) + ',\n  "files": [\n' + rows.map(row => `    ${JSON.stringify(row)}`).join(",\n") + "\n  ],\n  \"native_files\": [\n" + nativeRows.map(row => `    ${JSON.stringify(row)}`).join(",\n") + "\n  ]\n}\n";
const target = new URL("../docs/developers/ui-control-inventory.json", import.meta.url);
if (process.argv.includes("--check")) {
  if (readFileSync(target, "utf8") !== result) { process.stderr.write("UI declaration inventory changed; regenerate and review semantic coverage.\n"); process.exitCode = 1; }
} else writeFileSync(target, result);
</book:entry>

</book:doc>
