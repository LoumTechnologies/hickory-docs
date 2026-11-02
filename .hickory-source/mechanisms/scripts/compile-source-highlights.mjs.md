<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Compile source colours with the editor grammar

At publication time we need the same block boundaries, language routing and token classes as the editor. Load its WASM parser and grammar registry in Node; only the resulting UTF-16 spans enter the website. Cross-page fragments use the committed construction map, and ambiguous reuse has no inferred language. Identifier occurrences become a finite capture plan for the language server; prose words do not become LSP requests.

Scan sorted grammar tokens and source lines once rather than searching all tokens and re-splitting the document for every identifier. The raw JSON construction inventories are large enough to make that repeated work quadratic.

The complete publication is larger than an OS pipe buffer. Consume stdin as an asynchronous stream so a temporary empty read waits for the Python publisher instead of failing with `EAGAIN`. A large-input export witness exercises this transport boundary.

<static:copy id="static-scripts-compile-source-highlights-mjs-0">
/** Build-time only: load the same parser and syntax planner as the installed CLI. */
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createServer } from '../apps/web/node_modules/vite/dist/node/index.js';
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString('utf8'));
const server = await createServer({ root: 'apps/web', server: { middlewareMode: true }, appType: 'custom' });
try {
  const parser = await server.ssrLoadModule('/src/editor/hickLang.ts');
  parser.loadHickLangSync(readFileSync('apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm'));
  const { compileHighlights } = await server.ssrLoadModule(fileURLToPath(new URL('./static-highlights.mjs', import.meta.url)));
  process.stdout.write(JSON.stringify(compileHighlights(input)));
} finally { await server.close(); }
</static:copy>

<static:file path="scripts/compile-source-highlights.mjs"><static:paste select="#static-scripts-compile-source-highlights-mjs-0"/></static:file>
</static:doc>
