/** Build-time only: share the editor's parser and grammar registry. */
import { readFileSync } from 'node:fs';
import { createServer } from '../apps/web/node_modules/vite/dist/node/index.js';
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString('utf8'));
const server = await createServer({ root: 'apps/web', server: { middlewareMode: true }, appType: 'custom' });
try {
  const parser = await server.ssrLoadModule('/src/editor/hickLang.ts');
  parser.loadHickLangSync(readFileSync('apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm'));
  const { parseHickDoc, languageForBlock, codeRangesOf, proseCodeFences } = await server.ssrLoadModule('/src/editor/hickDoc.ts');
  const { highlightCode } = await server.ssrLoadModule('/src/editor/embedded.ts');
  const { languageFromPath } = await server.ssrLoadModule('/src/editor/languages.ts');
  // A fragment's construction can live on another page. Infer its language
  // only when every selected output agrees; mixed-language reuse stays plain.
  const languages = new Map();
  for (const [path, owner] of Object.entries(input.constructors)) {
    const lang = languageFromPath(path);
    if (!lang) continue;
    for (const id of owner.fragments ?? []) {
      const found = languages.get(id) ?? new Set(); found.add(lang); languages.set(id, found);
    }
  }
  const output = {};
  for (const document of input.documents) {
    const text = document.source, structure = parseHickDoc(text), ranges = [];
    for (const block of structure.blocks) {
      const inferred = languages.get(block.attrs.id);
      const lang = languageForBlock(structure, block) ?? (block.name === "entry" && block.attrs.encoding !== "base64" ? languageFromPath(block.attrs.path) : null) ?? (inferred?.size === 1 ? [...inferred][0] : null);
      if (lang) for (const [from, to] of codeRangesOf(structure, block)) ranges.push({ from, to, lang });
    }
    for (const fence of proseCodeFences(structure, text)) {
      const from = text.indexOf('\n', fence.from) + 1;
      if (from > 0) ranges.push({ from, to: from + fence.body.length, lang: fence.info.split(/\s/)[0] });
    }
    const spans = [], positions = new Map();
    for (const range of ranges) {
      const code = text.slice(range.from, range.to);
      const tokens = highlightCode(code, range.lang);
      for (const span of tokens) spans.push({ from: range.from + span.from, to: range.from + span.to, class: span.cls });
      // Capture once per identifier occurrence, in UTF-16 source coordinates.
      let tokenIndex = 0;
      for (const match of code.matchAll(/[\p{L}_$][\p{L}\p{N}_$]*/gu)) {
        while (tokenIndex < tokens.length && tokens[tokenIndex].to <= match.index) tokenIndex++;
        const token = tokens[tokenIndex];
        if (!token || token.from > match.index || token.to < match.index + match[0].length || !/tok-(variableName|propertyName|typeName|definition|function|namespace|attributeName|macroName|labelName)/.test(token.cls)) continue;
        const from = range.from + match.index, to = from + match[0].length;
        positions.set(from, { from, to });
      }
    }
    const occurrences = [...positions.values()].sort((a, b) => a.from - b.from);
    let line = 0, lineStart = 0, nextBreak = text.indexOf('\n');
    for (const occurrence of occurrences) {
      while (nextBreak >= 0 && nextBreak < occurrence.from) {
        line++; lineStart = nextBreak + 1; nextBreak = text.indexOf('\n', lineStart);
      }
      occurrence.position = { line, character: occurrence.from - lineStart };
    }
    output[document.path] = { spans, positions: occurrences };
  }
  process.stdout.write(JSON.stringify(output));
} finally { await server.close(); }
