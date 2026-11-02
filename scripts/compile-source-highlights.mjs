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
