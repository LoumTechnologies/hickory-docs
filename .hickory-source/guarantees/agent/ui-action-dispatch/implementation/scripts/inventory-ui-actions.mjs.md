<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/inventory-ui-actions.mjs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-209db9f9ea87"></a>
## `scripts/inventory-ui-actions.mjs`

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../contract.md).

<a id="f-209db9f9ea87-hash_bang_line"></a>
### `hash_bang_line`

<storycode:copy id="f-209db9f9ea87-hash_bang_line">
#!/usr/bin/env node
</storycode:copy>
<a id="f-209db9f9ea87-import_statement"></a>
### `import_statement`

An auditable index of control declarations, not proof of semantic parity.

<storycode:copy id="f-209db9f9ea87-import_statement">
// An auditable index of control declarations, not proof of semantic parity.
import { execFileSync } from "node:child_process";
</storycode:copy>
<a id="f-209db9f9ea87-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-209db9f9ea87-import_statement-2">
import { createRequire } from "node:module";
</storycode:copy>
<a id="f-209db9f9ea87-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-209db9f9ea87-import_statement-3">
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
</storycode:copy>
<a id="f-209db9f9ea87-import_statement-4"></a>
### `import_statement`

<storycode:copy id="f-209db9f9ea87-import_statement-4">
import { fileURLToPath } from "node:url";
</storycode:copy>
<a id="f-209db9f9ea87-import_statement-5"></a>
### `import_statement`

<storycode:copy id="f-209db9f9ea87-import_statement-5">
import { uiReachability } from "./ui-reachability.mjs";
</storycode:copy>
<a id="f-209db9f9ea87-import_statement-6"></a>
### `import_statement`

<storycode:copy id="f-209db9f9ea87-import_statement-6">
import { createHash } from "node:crypto";

</storycode:copy>
<a id="f-209db9f9ea87-root"></a>
### `root`

<storycode:copy id="f-209db9f9ea87-root">
const root = fileURLToPath(new URL("../", import.meta.url));
</storycode:copy>
<a id="f-209db9f9ea87-require"></a>
### `require`

<storycode:copy id="f-209db9f9ea87-require">
const require = createRequire(new URL("../apps/web/package.json", import.meta.url));
</storycode:copy>
<a id="f-209db9f9ea87-ts"></a>
### `ts`

<storycode:copy id="f-209db9f9ea87-ts">
const ts = require("typescript");
</storycode:copy>
<a id="f-209db9f9ea87-allFiles"></a>
### `allFiles`

<storycode:copy id="f-209db9f9ea87-allFiles">
const allFiles = execFileSync("rg", ["--files", "apps/web/src", "apps/desktop/src-tauri/src", "apps/desktop/fskit", "crates/hickory-cli/src/serve/shell.rs", "crates/hickory-cli/src/serve/reveal.rs"], { cwd: root, encoding: "utf8" }).trim().split("\n").sort();
</storycode:copy>
<a id="f-209db9f9ea87-files"></a>
### `files`

<storycode:copy id="f-209db9f9ea87-files">
const files = allFiles.filter(path => path.startsWith("apps/web/src/"))
  .filter(path => /\.tsx?$/.test(path) && !/\.(test|spec)\./.test(path)).sort();
</storycode:copy>
<a id="f-209db9f9ea87-native"></a>
### `native`

<storycode:copy id="f-209db9f9ea87-native">
const native = new Set(["button", "input", "select", "textarea", "summary", "details", "a", "form", "dialog"]);
</storycode:copy>
<a id="f-209db9f9ea87-rows"></a>
### `rows`

<storycode:copy id="f-209db9f9ea87-rows">
const rows = [];
</storycode:copy>
<a id="f-209db9f9ea87-moduleSources"></a>
### `moduleSources`

<storycode:copy id="f-209db9f9ea87-moduleSources">
const moduleSources = {};
</storycode:copy>
<a id="f-209db9f9ea87-for_in_statement"></a>
### `for_in_statement`

<storycode:copy id="f-209db9f9ea87-for_in_statement">
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
</storycode:copy>
<a id="f-209db9f9ea87-nativeRows"></a>
### `nativeRows`

<storycode:copy id="f-209db9f9ea87-nativeRows">
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
</storycode:copy>
<a id="f-209db9f9ea87-entrypoints"></a>
### `entrypoints`

<storycode:copy id="f-209db9f9ea87-entrypoints">
const entrypoints = {};
</storycode:copy>
<a id="f-209db9f9ea87-for_in_statement-2"></a>
### `for_in_statement`

<storycode:copy id="f-209db9f9ea87-for_in_statement-2">
for (const filename of readdirSync(new URL("../apps/web/", import.meta.url)).filter(name => name.endsWith(".html")).sort()) {
  const html = readFileSync(new URL(`../apps/web/${filename}`, import.meta.url), "utf8");
  entrypoints[filename] = [...html.matchAll(/<script\b[^>]*\bsrc=["']([^"']+)["'][^>]*>/g)].map(match => match[1])
    .filter(src => /^(?:\.\/|\/)src\//.test(src)).map(src => `apps/web/${src.replace(/^(?:\.\/|\/)/, "").split("?")[0]}`);
}
</storycode:copy>
<a id="f-209db9f9ea87-reachability"></a>
### `reachability`

<storycode:copy id="f-209db9f9ea87-reachability">
const reachability = uiReachability(moduleSources, entrypoints);
</storycode:copy>
<a id="f-209db9f9ea87-for_in_statement-3"></a>
### `for_in_statement`

<storycode:copy id="f-209db9f9ea87-for_in_statement-3">
for (const row of rows) row.possible_entrypoints = reachability.roots[row.path];
</storycode:copy>
<a id="f-209db9f9ea87-header"></a>
### `header`

<storycode:copy id="f-209db9f9ea87-header">
const header = { version: 3, generator: "scripts/inventory-ui-actions.mjs", coverage: "All non-test web TS/TSX; desktop Rust/FSKit Swift; host shell/reveal Rust. Declarations, adapter hook calls and compile guards are review aids, not proof of registration, reachability, parity or platform support. Rendered values and dynamic controls still require the companion manual inventory." };
</storycode:copy>
<a id="f-209db9f9ea87-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-209db9f9ea87-expression_statement">
header.reachability = { scope: "Conservative local module graph from every top-level web HTML script, including type-only, lazy, worker/raw-query imports and re-exports. Presence is potential import reachability, not rendered visibility; absence is not a proof of dead code or exemption from parity. Review unresolved paths and runtime injection before excluding anything.", entrypoints, unresolved: reachability.unresolved };
</storycode:copy>
<a id="f-209db9f9ea87-result"></a>
### `result`

<storycode:copy id="f-209db9f9ea87-result">
const result = JSON.stringify(header, null, 2).slice(0, -2) + ',\n  "files": [\n' + rows.map(row => `    ${JSON.stringify(row)}`).join(",\n") + "\n  ],\n  \"native_files\": [\n" + nativeRows.map(row => `    ${JSON.stringify(row)}`).join(",\n") + "\n  ]\n}\n";
</storycode:copy>
<a id="f-209db9f9ea87-target"></a>
### `target`

<storycode:copy id="f-209db9f9ea87-target">
const target = new URL("../docs/developers/ui-control-inventory.json", import.meta.url);
</storycode:copy>
<a id="f-209db9f9ea87-if_statement"></a>
### `if_statement`

<storycode:copy id="f-209db9f9ea87-if_statement">
if (process.argv.includes("--check")) {
  if (readFileSync(target, "utf8") !== result) { process.stderr.write("UI declaration inventory changed; regenerate and review semantic coverage.\n"); process.exitCode = 1; }
} else writeFileSync(target, result);
</storycode:copy>

### Assemble `scripts/inventory-ui-actions.mjs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/inventory-ui-actions.mjs"><storycode:paste select="#f-209db9f9ea87-hash_bang_line"/><storycode:paste select="#f-209db9f9ea87-import_statement"/><storycode:paste select="#f-209db9f9ea87-import_statement-2"/><storycode:paste select="#f-209db9f9ea87-import_statement-3"/><storycode:paste select="#f-209db9f9ea87-import_statement-4"/><storycode:paste select="#f-209db9f9ea87-import_statement-5"/><storycode:paste select="#f-209db9f9ea87-import_statement-6"/><storycode:paste select="#f-209db9f9ea87-root"/><storycode:paste select="#f-209db9f9ea87-require"/><storycode:paste select="#f-209db9f9ea87-ts"/><storycode:paste select="#f-209db9f9ea87-allFiles"/><storycode:paste select="#f-209db9f9ea87-files"/><storycode:paste select="#f-209db9f9ea87-native"/><storycode:paste select="#f-209db9f9ea87-rows"/><storycode:paste select="#f-209db9f9ea87-moduleSources"/><storycode:paste select="#f-209db9f9ea87-for_in_statement"/><storycode:paste select="#f-209db9f9ea87-nativeRows"/><storycode:paste select="#f-209db9f9ea87-entrypoints"/><storycode:paste select="#f-209db9f9ea87-for_in_statement-2"/><storycode:paste select="#f-209db9f9ea87-reachability"/><storycode:paste select="#f-209db9f9ea87-for_in_statement-3"/><storycode:paste select="#f-209db9f9ea87-header"/><storycode:paste select="#f-209db9f9ea87-expression_statement"/><storycode:paste select="#f-209db9f9ea87-result"/><storycode:paste select="#f-209db9f9ea87-target"/><storycode:paste select="#f-209db9f9ea87-if_statement"/></storycode:file>

</storycode:doc>
