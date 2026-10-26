<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/ui-reachability.test.mjs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fe2b8d41f747"></a>
## `scripts/ui-reachability.test.mjs`

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../contract.md).

<a id="f-fe2b8d41f747-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-fe2b8d41f747-expression_statement">
import { test } from "node:test";
import assert from "node:assert/strict";
import { uiReachability } from "./ui-reachability.mjs";
test("conservatively follows lazy, type, re-export, worker/query and JS-suffix references without inventing callers", () => {
  const files = {
    "src/main.tsx": 'import type { T } from "./types"; export * from "./barrel"; void import("./lazy"); import worker from "./worker.ts?worker&url";',
    "src/types.ts": "export type T = string;", "src/barrel.ts": 'export { X } from "./dir/index.js";',
    "src/dir/index.tsx": 'export const X = 1; import "../main";', "src/lazy.ts": 'export type P = import("./types").T;',
    "src/worker.ts": "export {};", "src/unused.tsx": "export const Unused = () => <button>Unused</button>;",
    "src/site.tsx": 'import "./types";',
  };
  const result = uiReachability(files, { "index.html": ["src/main.tsx"], "site.html": ["src/site.tsx"] });
  assert.deepEqual(result.roots["src/types.ts"], ["index.html", "site.html"]);
  for (const filename of ["src/barrel.ts", "src/dir/index.tsx", "src/lazy.ts", "src/worker.ts"]) assert.deepEqual(result.roots[filename], ["index.html"]);
  assert.deepEqual(result.roots["src/unused.tsx"], []);
  assert.deepEqual(result.unresolved, []);
});
</storycode:copy>
<a id="f-fe2b8d41f747-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-fe2b8d41f747-expression_statement-2">
test("reports nonliteral/glob/unresolved imports and missing entries instead of silently declaring coverage", () => {
  const result = uiReachability({ "src/main.ts": 'void import(target); import.meta.glob("./panels/*.tsx"); import "./missing"; import "./style.css"; import "react";' }, { "index.html": ["src/main.ts"], "broken.html": ["src/nope.ts"] });
  assert.deepEqual(result.unresolved.map(item => item.kind).sort(), ["glob-requires-review", "missing-entry", "nonliteral-import", "unresolved-local-import"]);
});
</storycode:copy>

### Assemble `scripts/ui-reachability.test.mjs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/ui-reachability.test.mjs"><storycode:paste select="#f-fe2b8d41f747-expression_statement"/><storycode:paste select="#f-fe2b8d41f747-expression_statement-2"/></storycode:file>

</storycode:doc>
