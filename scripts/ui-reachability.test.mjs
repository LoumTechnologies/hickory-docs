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
test("reports nonliteral/glob/unresolved imports and missing entries instead of silently declaring coverage", () => {
  const result = uiReachability({ "src/main.ts": 'void import(target); import.meta.glob("./panels/*.tsx"); import "./missing"; import "./style.css"; import "react";' }, { "index.html": ["src/main.ts"], "broken.html": ["src/nope.ts"] });
  assert.deepEqual(result.unresolved.map(item => item.kind).sort(), ["glob-requires-review", "missing-entry", "nonliteral-import", "unresolved-local-import"]);
});
