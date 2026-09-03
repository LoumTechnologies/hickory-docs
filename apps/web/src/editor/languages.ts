// Language registry shared by the Output view (full CodeMirror language
// extensions), the plain-file pane, and the Document view (embedded
// highlighting of block bodies).
//
// WHICH languages exist is not decided here. It is generated from the
// server's own routing table (`crates/hick-lsp/src/lang_detect.rs`) by
// `just codegen`, because this file used to keep a second list and the two
// drifted apart in both directions at once: Go, Java, Kotlin, Scala, C, C++,
// Ruby, Swift, Lua, TOML and YAML were routed and reported as Bronze — whose
// definition is "the text is right: routed, highlighted, runnable" — while
// opening as undifferentiated grey text, and SQL and XML highlighted here
// while being routed nowhere at all. What is decided here is only HOW each
// language is drawn, and `languages.test.ts` fails if the server expects a
// grammar this file has not bound.
//
// Colors are NOT defined here: highlighting emits stable `tok-*` classes and
// styles.css maps them onto the hick palette for both light and dark, so
// every view and both themes stay in tune.

import type { Extension } from "@codemirror/state";
import { syntaxHighlighting, HighlightStyle, StreamLanguage } from "@codemirror/language";
import type { Parser } from "@lezer/common";
import { tags as t } from "@lezer/highlight";
import { python } from "@codemirror/lang-python";
import { javascript } from "@codemirror/lang-javascript";
import { rust } from "@codemirror/lang-rust";
import { json } from "@codemirror/lang-json";
import { markdown } from "@codemirror/lang-markdown";
import { html } from "@codemirror/lang-html";
import { css } from "@codemirror/lang-css";
import { sql } from "@codemirror/lang-sql";
import { php } from "@codemirror/lang-php";
import { shell } from "@codemirror/legacy-modes/mode/shell";
// Most of the JetBrains pack comes from the legacy modes rather than its own
// lezer grammar: `@codemirror/legacy-modes` is already here for the shell, so
// Go, Java, C, C++, C#, Kotlin, Scala, Dart, Ruby, Swift, Lua, Groovy, R, F#,
// Visual Basic, TOML, YAML and XML cost no new dependency — which matters for
// a product that must stay copyleft-free and ships every byte it builds.
import { c, cpp, csharp, dart, java, kotlin, scala } from "@codemirror/legacy-modes/mode/clike";
import { xml } from "@codemirror/legacy-modes/mode/xml";
import { go } from "@codemirror/legacy-modes/mode/go";
import { ruby } from "@codemirror/legacy-modes/mode/ruby";
import { swift } from "@codemirror/legacy-modes/mode/swift";
import { lua } from "@codemirror/legacy-modes/mode/lua";
import { groovy } from "@codemirror/legacy-modes/mode/groovy";
import { r } from "@codemirror/legacy-modes/mode/r";
import { fSharp } from "@codemirror/legacy-modes/mode/mllike";
import { vb } from "@codemirror/legacy-modes/mode/vb";
import { toml } from "@codemirror/legacy-modes/mode/toml";
import { yaml } from "@codemirror/legacy-modes/mode/yaml";

import {
  EXTENSION_TO_LANGUAGE,
  HIGHLIGHTED_LANGUAGES,
  type RoutedLanguageId,
} from "./generated/languages";

/** Canonical language ids the app understands — the server's own list. */
export type LanguageId = RoutedLanguageId;

/**
 * Spellings that are not file extensions: what a person writes in a
 * `language=` attribute or a markdown fence. Extensions come from the
 * generated table and are never repeated here.
 */
const SYNONYMS: Record<string, RoutedLanguageId> = {
  "c++": "cpp",
  "c#": "csharp",
  "f#": "fsharp",
  "objective-c": "c",
  bash: "shellscript",
  sh: "shellscript",
  shell: "shellscript",
  zsh: "shellscript",
  console: "shellscript",
  jsx: "javascriptreact",
  tsx: "typescriptreact",
  "javascript-react": "javascriptreact",
  "typescript-react": "typescriptreact",
  js: "javascript",
  ts: "typescript",
  py: "python",
  rs: "rust",
  md: "markdown",
  htm: "html",
  cs: "csharp",
  kt: "kotlin",
  rb: "ruby",
  yml: "yaml",
  golang: "go",
  "visual-basic": "vb",
  vbnet: "vb",
  dotnet: "csharp",
  postgres: "sql",
  postgresql: "sql",
  mysql: "sql",
  sqlite: "sql",
  plsql: "sql",
  tsql: "sql",
};

/** Canonicalise a language name or file extension; null when unknown. */
export function normalizeLanguage(name: string | undefined | null): LanguageId | null {
  if (!name) return null;
  const key = name.toLowerCase();
  // The language's own id always wins, then a file extension, then a synonym.
  if ((HIGHLIGHTED_LANGUAGES as readonly string[]).includes(key)) return key as LanguageId;
  const routed = EXTENSION_TO_LANGUAGE[key];
  if (routed) return routed;
  return SYNONYMS[key] ?? null;
}

/** Language inferred from a file path's extension; null when unknown. */
export function languageFromPath(path: string | undefined | null): LanguageId | null {
  if (!path) return null;
  const dot = path.lastIndexOf(".");
  if (dot < 0 || dot === path.length - 1) return null;
  return normalizeLanguage(path.slice(dot + 1));
}

/**
 * How each language is drawn. A language the server routes but that has no
 * grammar anywhere (Nix, Zig) is deliberately absent: it opens as plain text,
 * and `hick lang` reports that rather than promising otherwise.
 */
const GRAMMARS: Partial<Record<LanguageId, () => Extension>> = {
  python: () => python(),
  javascript: () => javascript(),
  javascriptreact: () => javascript({ jsx: true }),
  typescript: () => javascript({ typescript: true }),
  typescriptreact: () => javascript({ typescript: true, jsx: true }),
  rust: () => rust(),
  json: () => json(),
  markdown: () => markdown(),
  html: () => html(),
  css: () => css(),
  sql: () => sql(),
  php: () => php(),
  shellscript: () => StreamLanguage.define(shell),
  csharp: () => StreamLanguage.define(csharp),
  xml: () => StreamLanguage.define(xml),
  go: () => StreamLanguage.define(go),
  java: () => StreamLanguage.define(java),
  c: () => StreamLanguage.define(c),
  cpp: () => StreamLanguage.define(cpp),
  kotlin: () => StreamLanguage.define(kotlin),
  scala: () => StreamLanguage.define(scala),
  dart: () => StreamLanguage.define(dart),
  ruby: () => StreamLanguage.define(ruby),
  swift: () => StreamLanguage.define(swift),
  lua: () => StreamLanguage.define(lua),
  groovy: () => StreamLanguage.define(groovy),
  r: () => StreamLanguage.define(r),
  fsharp: () => StreamLanguage.define(fSharp),
  vb: () => StreamLanguage.define(vb),
  toml: () => StreamLanguage.define(toml),
  yaml: () => StreamLanguage.define(yaml),
};

/** Language ids this file can actually draw. Read by the drift test. */
export function boundGrammars(): LanguageId[] {
  return (Object.keys(GRAMMARS) as LanguageId[]).sort();
}

// One language instance per id, built lazily (the stream parsers and the big
// lezer grammars are only paid for when something actually uses them).
const supportCache = new Map<LanguageId, Extension | null>();
const parserCache = new Map<LanguageId, Parser | null>();

function languageSupportFor(lang: LanguageId): Extension | null {
  let ext = supportCache.get(lang);
  if (ext === undefined) {
    const build = GRAMMARS[lang];
    try {
      ext = build ? build() : null;
    } catch {
      ext = null;
    }
    supportCache.set(lang, ext);
  }
  return ext;
}

/** The lezer parser for a language id (or alias/extension); null if unknown. */
export function parserForLanguage(name: string | undefined | null): Parser | null {
  const lang = normalizeLanguage(name);
  if (!lang) return null;
  let p = parserCache.get(lang);
  if (p === undefined) {
    // A lezer package returns a LanguageSupport, whose parser is one level
    // down under `.language`; `StreamLanguage.define` returns a Language,
    // which carries `.parser` itself. Reading only the first shape silently
    // returned null for every legacy mode — the shell, C#, XML and the whole
    // JetBrains half of the table — which is a blank block, not an error.
    const support = languageSupportFor(lang) as
      | { language?: { parser?: Parser }; parser?: Parser }
      | null;
    p = support?.language?.parser ?? support?.parser ?? null;
    parserCache.set(lang, p);
  }
  return p;
}

/**
 * Class-based highlight style: token colors live in styles.css (`.tok-*`),
 * tuned for the hick palette in both light and dark themes.
 */
export const hickoryHighlightStyle = HighlightStyle.define([
  { tag: [t.keyword, t.modifier, t.operatorKeyword, t.controlKeyword, t.definitionKeyword], class: "tok-keyword" },
  { tag: [t.string, t.special(t.string), t.regexp], class: "tok-string" },
  { tag: [t.comment, t.lineComment, t.blockComment, t.docComment], class: "tok-comment" },
  { tag: [t.number, t.integer, t.float, t.bool, t.atom, t.null], class: "tok-literal" },
  { tag: [t.typeName, t.className, t.namespace], class: "tok-type" },
  { tag: [t.function(t.variableName), t.function(t.propertyName), t.macroName], class: "tok-function" },
  { tag: [t.definition(t.variableName), t.definition(t.propertyName)], class: "tok-definition" },
  { tag: [t.propertyName, t.attributeName, t.labelName], class: "tok-property" },
  { tag: [t.tagName, t.heading], class: "tok-tag" },
  { tag: [t.meta, t.processingInstruction, t.punctuation], class: "tok-meta" },
  { tag: [t.operator, t.derefOperator, t.arithmeticOperator, t.logicOperator, t.compareOperator], class: "tok-operator" },
  { tag: [t.link, t.url], class: "tok-link" },
  { tag: t.invalid, class: "tok-invalid" },
]);

/**
 * Syntax highlighting for an output file's declared language (a name, alias,
 * extension, or full path). Unknown languages degrade to plain text.
 */
export function languageExtensions(language: string): Extension[] {
  const lang = normalizeLanguage(language) ?? languageFromPath(language);
  const exts: Extension[] = [syntaxHighlighting(hickoryHighlightStyle, { fallback: true })];
  const support = lang ? languageSupportFor(lang) : null;
  if (support) exts.push(support);
  return exts;
}
