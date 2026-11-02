<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Run the installed grammar without a checkout

The downloadable CLI needs no Vite installation or Hickory source checkout. Bundle the shared planner, grammars and parser bytes into a Node entry. Standard input and output carry only finite JSON; nothing in this build-time worker becomes a running website server.

<static:copy id="static-scripts-static-highlights-entry-mjs-0">
import wasm from '../apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm?url';
import { loadHickLangSync } from '../apps/web/src/editor/hickLang.ts';
import { compileHighlights } from './static-highlights.mjs';
loadHickLangSync(Buffer.from(wasm.split(',')[1], 'base64'));
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
process.stdout.write(JSON.stringify(compileHighlights(JSON.parse(Buffer.concat(chunks).toString('utf8')))));
</static:copy>

<static:file path="scripts/static-highlights-entry.mjs"><static:paste select="#static-scripts-static-highlights-entry-mjs-0"/></static:file>
</static:doc>
