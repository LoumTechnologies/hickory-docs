import { parseHickDoc, languageForBlock, codeRangesOf, proseCodeFences } from '../apps/web/src/editor/hickDoc.ts';
import { highlightCode } from '../apps/web/src/editor/embedded.ts';
import { languageFromPath } from '../apps/web/src/editor/languages.ts';

export function compileHighlights(input) {
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
  const origins = new Map();
  for (const file of input.files ?? []) {
    const lang = languageFromPath(file.path);
    if (!lang) continue;
    for (const origin of file.origins ?? []) {
      const found = origins.get(origin.document) ?? [];
      found.push({ from: origin.source[0], to: origin.source[1], lang });
      origins.set(origin.document, found);
    }
  }
  const output = {};
  for (const document of input.documents) {
    const text = document.source, structure = parseHickDoc(text), ranges = [];
    for (const block of input.files ? [] : structure.blocks) {
      const inferred = languages.get(block.attrs.id);
      const lang = languageForBlock(structure, block) ?? (block.name === "entry" && block.attrs.encoding !== "base64" ? languageFromPath(block.attrs.path) : null) ?? (inferred?.size === 1 ? [...inferred][0] : null);
      if (lang) for (const [from, to] of codeRangesOf(structure, block)) ranges.push({ from, to, lang, semantic: true });
    }
    if (input.files) {
      const bytes = Buffer.from(text, 'utf8'), found = origins.get(document.path) ?? [];
      const endpoints = [...new Set(found.flatMap(range => [range.from, range.to]))]
        .filter(at => at >= 0 && at <= bytes.length && (at === bytes.length || (bytes[at] & 0xc0) !== 0x80)).sort((a, b) => a-b);
      const offsets = new Map();
      let previous = 0, offset = 0;
      for (const at of endpoints) {
        offset += bytes.subarray(previous, at).toString('utf8').length;
        offsets.set(at, offset); previous = at;
      }
      const selections = new Map();
      for (const range of found) {
        const from = offsets.get(range.from), to = offsets.get(range.to);
        if (from === undefined || to === undefined || to <= from) continue;
        const key = `${from}:${to}`, selection = selections.get(key) ?? { from, to, languages: new Set() };
        selection.languages.add(range.lang); selections.set(key, selection);
      }
      for (const range of selections.values()) if (range.languages.size === 1)
        ranges.push({ from: range.from, to: range.to, lang: [...range.languages][0], semantic: true });
    }
    for (const fence of proseCodeFences(structure, text)) {
      const from = text.indexOf('\n', fence.from) + 1;
      if (from > 0) ranges.push({ from, to: from + fence.body.length, lang: fence.info.split(/\s/)[0], semantic: false });
    }
    const spans = [], positions = new Map();
    for (const range of ranges) {
      const code = text.slice(range.from, range.to);
      const tokens = highlightCode(code, range.lang);
      for (const span of tokens) spans.push({ from: range.from + span.from, to: range.from + span.to, class: span.cls });
      if (!range.semantic || input.capturePositions === false) continue;
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
    output[document.path] = { spans, positions: occurrences, semantic: ranges.some(range => range.semantic) };
  }
  return output;
}
