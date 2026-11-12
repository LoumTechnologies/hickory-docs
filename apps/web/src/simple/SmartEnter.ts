import registry from "../../../simple-contract/enter-rules.json";
export type EnterMode = "smart" | "copy" | "none";
export type EnterPlan = { from: number; to: number; text: string; caret: number };
type Profile = { id: string; names: string[]; comments: string[]; blocks: string[][]; multiline: string[]; quotes: string; open: string; close: string; middle: string; next: string; regexLiterals: boolean; after: string; literal: string; tags: boolean; voidTags: string[]; foldCase: boolean; offside: boolean };
const compiled = new Map<string, RegExp>();
function matches(profile: Profile, pattern: string, text: string): boolean {
  if (!pattern) return false;
  const key = `${profile.foldCase}:${pattern}`;
  let regex = compiled.get(key);
  if (!regex) { regex = new RegExp(pattern, profile.foldCase ? "i" : ""); compiled.set(key, regex); }
  return regex.test(text);
}
function profileFor(path: string): Profile | undefined {
  const name = path.split(/[\\/]/).pop()!.toLowerCase();
  const ext = name.includes(".") ? name.slice(name.lastIndexOf(".") + 1) : name;
  return (registry.profiles as Profile[]).find(profile => profile.names.includes(ext) && profile.blocks.every(pair => pair.length === 2 && pair.every(Boolean)) && profile.comments.every(Boolean) && profile.multiline.every(Boolean));
}
function outdent(prefix: string, unit: string): string {
  if (prefix.endsWith("\t")) return prefix.slice(0, -1);
  let count = 0; while (count < unit.length && prefix.endsWith(" ")) { prefix = prefix.slice(0, -1); count++; }
  return prefix;
}
export function enterMode(): EnterMode {
  try { const mode = localStorage.getItem("editor.enterIndent"); return mode === "copy" || mode === "none" ? mode : "smart"; }
  catch { return "smart"; }
}
// The scanner retains only lexical state and opening indentation, never an AST.
class EnterScanner {
  block = "";
  quoted = false;
  brackets: { token: string; indent: string; line: number }[] = [];
  keywords: string[] = [];
  logical = "";
  logicalIndent = "";
  literal: string | null = null;
  singleIndent: string | null = null;
  clean(row: string, profile: Profile): string {
    let code = "", index = 0; this.quoted = false;
    while (index < row.length) {
      if (this.block) {
        if (row.startsWith(this.block, index) && (index === 0 || row[index - 1] !== "\\")) { index += this.block.length; this.block = ""; }
        else index++;
        code += " "; continue;
      }
      const block = profile.blocks.find(pair => row.startsWith(pair[0], index));
      const multi = profile.multiline.find(mark => row.startsWith(mark, index));
      if (block || multi) { this.block = block ? block[1] : multi!; index += block ? block[0].length : multi!.length; code += " "; continue; }
      if (profile.comments.some(mark => row.startsWith(mark, index))) break;
      if (profile.regexLiterals && row[index] === "/" && /(?:^|\b(?:return|throw|yield|case)|[=(:,!&|?;{}\[>])\s*$/.test(code)) {
        index++; let characterClass = false, closed = false;
        while (index < row.length) {
          const next = row[index++];
          if (next === "\\") { index++; continue; }
          if (next === "[") characterClass = true; if (next === "]") characterClass = false;
          if (next === "/" && !characterClass) { closed = true; break; }
        }
        if (!closed) { this.quoted = true; this.block = "/"; }
        code += "~"; continue;
      }
      const char = row[index++];
      if (profile.quotes.includes(char)) {
        if (char === "'" && profile.id === "rust" && !row.slice(index).includes("'")) { code += char; continue; }
        let closed = false;
        while (index < row.length) { const next = row[index++]; if (next === "\\") { index++; continue; } if (next === char) { closed = true; break; } }
        if (!closed) { this.quoted = true; this.block = char; }
        code += "~"; continue;
      }
      code += char;
    }
    return code.trim();
  }
  line(row: string, profile: Profile, line: number, unit: string): { indent: string; split: string; literal: boolean } {
    const prefix = row.match(/^[ \t]*/)![0];
    if (this.literal !== null && (!row.trim() || prefix.length > this.literal.length)) return { indent: prefix, split: "", literal: true };
    this.literal = null;
    const wasBlock = !!this.block, code = this.clean(row, profile), startDepth = this.brackets.length;
    if (wasBlock || this.block || this.quoted || !code) return { indent: prefix, split: "", literal: true };
    if (!startDepth) { this.logical = code; this.logicalIndent = prefix; }
    let base = prefix;
    if (matches(profile, profile.close, code)) base = this.keywords.pop() ?? outdent(prefix, unit);
    const middle = matches(profile, profile.middle, code);
    if (middle && !profile.offside && this.keywords.length) base = this.keywords[this.keywords.length - 1];
    if (profile.tags) {
      for (const match of code.matchAll(/<(\/?)([A-Za-z][\w:.-]*)\b[^>]*>/g)) {
        const tag = profile.foldCase ? match[2].toLowerCase() : match[2];
        if (match[1]) { if (this.brackets.at(-1)?.token === `</${tag}>`) base = this.brackets.pop()!.indent; }
        else if (!match[0].endsWith("/>") && !profile.voidTags.includes(tag)) this.brackets.push({ token: `</${tag}>`, indent: base, line });
      }
    } else for (const char of code) {
      if ("{[(".includes(char)) this.brackets.push({ token: char, indent: base, line });
      const opening = "}])".indexOf(char);
      if (opening >= 0 && this.brackets.at(-1)?.token === "{[("[opening]) base = this.brackets.pop()!.indent;
    }
    const top = this.brackets.at(-1);
    if (top) return { indent: top.line === line || base.length <= top.indent.length ? top.indent + unit : base, split: top.token.startsWith("</") ? top.token : "}])"["{[(".indexOf(top.token)], literal: false };
    if (matches(profile, profile.next, code)) {
      this.singleIndent ??= base; return { indent: base + unit, split: "", literal: false };
    }
    if (matches(profile, profile.open, code) || middle) {
      if (!middle) this.keywords.push(base);
      return { indent: base + unit, split: "", literal: false };
    }
    if (matches(profile, profile.literal, code)) { this.literal = base; return { indent: base + unit, split: "", literal: false }; }
    if (matches(profile, profile.after, this.logical)) base = outdent(this.logicalIndent, unit);
    if (this.singleIndent !== null) { base = this.singleIndent; this.singleIndent = null; }
    return { indent: base, split: "", literal: false };
  }
}
// Return one ordinary source replacement and a caret within it. No save or format request.
export function smartEnter(source: string, from: number, to: number, path: string, unit: string, mode: EnterMode): EnterPlan {
  const start = from === 0 ? 0 : source.lastIndexOf("\n", from - 1) + 1;
  const next = source.indexOf("\n", from);
  const eol = (next >= 0 ? next > 0 && source[next - 1] === "\r" : start >= 2 && source[start - 2] === "\r") ? "\r\n" : "\n";
  const prefix = source.slice(start, from).match(/^[ \t]*/)![0];
  const fallback = (indent: string): EnterPlan => ({ from, to, text: eol + indent, caret: from + eol.length + indent.length });
  if (mode === "none") return fallback("");
  if (mode === "copy" || from > registry.maxContext || from - start > registry.maxLine) return fallback(prefix);
  const rows = source.slice(0, from).split("\n");
  let profile = profileFor(path), scanner = new EnterScanner(), fence = "", payload = "";
  const markdown = /\.(md|markdown)$/i.test(path);
  if (!markdown && !profile) return fallback(prefix);
  let result = { indent: prefix, split: "", literal: true };
  for (let line = 0; line < rows.length; line++) {
    const row = rows[line].replace(/\r$/, ""), trimmed = row.trim();
    if (row.length > registry.maxLine) return fallback(prefix);
    if (markdown) {
      const marker = trimmed.match(/^(`{3,}|~{3,})([^ ]*)/);
      if (fence) {
        if (trimmed.startsWith(fence) && /^(`+|~+)\s*$/.test(trimmed)) { fence = ""; profile = undefined; scanner = new EnterScanner(); continue; }
      } else if (!payload && marker) {
        fence = marker[1]; profile = profileFor(marker[2]); scanner = new EnterScanner(); continue;
      } else if (!fence) {
        const open = trimmed.match(/^<[\w.-]+:(file|copy|cut|exec)\b[^>]*>/);
        if (open) {
          payload = open[1]; const hint = open[0].match(/\b(?:path|language|lang)\s*=\s*["']([^"']+)["']/);
          profile = hint ? profileFor(hint[1]) : undefined; scanner = new EnterScanner(); continue;
        }
        if (payload && trimmed.startsWith("</") && trimmed.includes(`:${payload}>`)) { payload = ""; profile = undefined; scanner = new EnterScanner(); continue; }
      }
    }
    if (profile) result = scanner.line(row, profile, line, unit);
  }
  if (!profile || result.literal) return fallback(prefix);
  const suffix = source.slice(to, source.indexOf("\n", to) < 0 ? source.length : source.indexOf("\n", to)).replace(/\r$/, "");
  const whitespace = suffix.match(/^[ \t]*/)![0].length;
  const top = scanner.brackets.at(-1);
  if (result.split && (profile.foldCase ? suffix.trimStart().toLowerCase() : suffix.trimStart()).startsWith(result.split) && top) {
    const openingHere = top.line === rows.length - 1;
    const text = openingHere ? eol + result.indent + eol + top.indent : eol + top.indent;
    return { from, to: to + whitespace, text, caret: from + eol.length + (openingHere ? result.indent.length : top.indent.length) };
  }
  return fallback(result.indent);
}
