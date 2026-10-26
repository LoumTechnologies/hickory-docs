<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/ui-reachability.mjs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2536103e1457"></a>
## `scripts/ui-reachability.mjs`

<a id="f-2536103e1457-require"></a>
### `require`

Conservative import reachability, not runtime control or semantic coverage.

<storycode:copy id="f-2536103e1457-require">
// Conservative import reachability, not runtime control or semantic coverage.
import { createRequire } from "node:module";
import path from "node:path";
const require = createRequire(new URL("../apps/web/package.json", import.meta.url));
</storycode:copy>
<a id="f-2536103e1457-ts"></a>
### `ts`

<storycode:copy id="f-2536103e1457-ts">
const ts = require("typescript");
</storycode:copy>
<a id="f-2536103e1457-uiReachability"></a>
### `uiReachability`

<storycode:copy id="f-2536103e1457-uiReachability">
export function uiReachability(sources, entries) {
  const paths = new Set(Object.keys(sources));
  const edges = new Map(), unresolved = [];
  function resolve(from, specifier) {
    if (!specifier.startsWith(".")) return null;
    const target = path.posix.normalize(path.posix.join(path.posix.dirname(from), specifier.split("?")[0].split("#")[0]));
    const base = target.replace(/\.jsx?$/, "");
    return [target, `${target}.ts`, `${target}.tsx`, `${target}/index.ts`, `${target}/index.tsx`, `${base}.ts`, `${base}.tsx`].find(candidate => paths.has(candidate)) ?? null;
  }
  for (const [filename, text] of Object.entries(sources)) {
    const source = ts.createSourceFile(filename, text, ts.ScriptTarget.Latest, true, filename.endsWith("tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
    const imports = new Set();
    function record(node, specifier) {
      const line = source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1;
      if (specifier === null) { unresolved.push({ path: filename, line, kind: "nonliteral-import" }); return; }
      const resolved = resolve(filename, specifier);
      if (resolved) imports.add(resolved);
      else if (specifier.startsWith(".") && !/\.(css|svg|png|jpe?g|webp|gif|json|wasm|txt|md)([?#].*)?$/.test(specifier))
        unresolved.push({ path: filename, line, kind: "unresolved-local-import", specifier });
    }
    function visit(node) {
      if ((ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) && node.moduleSpecifier && ts.isStringLiteralLike(node.moduleSpecifier)) record(node, node.moduleSpecifier.text);
      if (ts.isImportTypeNode(node) && ts.isLiteralTypeNode(node.argument) && ts.isStringLiteralLike(node.argument.literal)) record(node, node.argument.literal.text);
      if (ts.isCallExpression(node) && (node.expression.kind === ts.SyntaxKind.ImportKeyword || node.expression.getText(source) === "require"))
        record(node, node.arguments.length === 1 && ts.isStringLiteralLike(node.arguments[0]) ? node.arguments[0].text : null);
      if (ts.isCallExpression(node) && /^import\.meta\.glob(?:Eager)?$/.test(node.expression.getText(source))) unresolved.push({ path: filename, line: source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1, kind: "glob-requires-review" });
      ts.forEachChild(node, visit);
    }
    visit(source); edges.set(filename, imports);
  }
  const roots = Object.fromEntries(Object.keys(sources).map(filename => [filename, []]));
  for (const [entry, initial] of Object.entries(entries)) {
    const pending = [...initial], seen = new Set();
    while (pending.length) {
      const filename = pending.pop();
      if (seen.has(filename)) continue;
      seen.add(filename);
      if (!paths.has(filename)) { unresolved.push({ path: filename, kind: "missing-entry", entry }); continue; }
      roots[filename].push(entry);
      pending.push(...edges.get(filename));
    }
  }
  for (const values of Object.values(roots)) values.sort();
  return { roots, unresolved: unresolved.sort((a, b) => a.path.localeCompare(b.path) || (a.line ?? 0) - (b.line ?? 0)) };
}
</storycode:copy>

### Assemble `scripts/ui-reachability.mjs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/ui-reachability.mjs"><storycode:paste select="#f-2536103e1457-require"/><storycode:paste select="#f-2536103e1457-ts"/><storycode:paste select="#f-2536103e1457-uiReachability"/></storycode:file>

</storycode:doc>
