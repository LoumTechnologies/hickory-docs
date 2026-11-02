import wasm from '../apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm?url';
import { loadHickLangSync } from '../apps/web/src/editor/hickLang.ts';
import { compileHighlights } from './static-highlights.mjs';
loadHickLangSync(Buffer.from(wasm.split(',')[1], 'base64'));
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
process.stdout.write(JSON.stringify(compileHighlights(JSON.parse(Buffer.concat(chunks).toString('utf8')))));
