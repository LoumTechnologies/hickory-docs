// Language registry shared by the Output view (full CodeMirror language
// extensions) and the Document view (embedded highlighting of block bodies).
//
// Colors are NOT defined here: highlighting emits stable `tok-*` classes and
// styles.css maps them onto the hickory palette for both light and dark, so
// the two views and both themes stay in tune.

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
import { shell } from "@codemirror/legacy-modes/mode/shell";

/** Canonical language ids the app understands. */
export type LanguageId =
  | "python"
  | "javascript"
  | "typescript"
  | "jsx"
  | "tsx"
  | "rust"
  | "json"
  | "markdown"
  | "html"
  | "css"
  | "sql"
  | "shell";

const ALIASES: Record<string, LanguageId> = {
  python: "python",
  py: "python",
  javascript: "javascript",
  js: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  jsx: "jsx",
  typescript: "typescript",
  ts: "typescript",
  tsx: "tsx",
  rust: "rust",
  rs: "rust",
  json: "json",
  markdown: "markdown",
  md: "markdown",
  html: "html",
  htm: "html",
  css: "css",
  sql: "sql",
  shell: "shell",
  sh: "shell",
  bash: "shell",
  zsh: "shell",
};

/** Canonicalise a language name or file extension; null when unknown. */
export function normalizeLanguage(name: string | undefined | null): LanguageId | null {
  if (!name) return null;
  return ALIASES[name.toLowerCase()] ?? null;
}

/** Language inferred from a file path's extension; null when unknown. */
export function languageFromPath(path: string | undefined | null): LanguageId | null {
  if (!path) return null;
  const dot = path.lastIndexOf(".");
  if (dot < 0 || dot === path.length - 1) return null;
  return normalizeLanguage(path.slice(dot + 1));
}

// One language instance per id, built lazily (the shell stream parser and the
// big lezer grammars are only paid for when a doc actually uses them).
const parserCache = new Map<LanguageId, Parser | null>();

function buildParser(lang: LanguageId): Parser {
  switch (lang) {
    case "python":
      return python().language.parser;
    case "javascript":
      return javascript().language.parser;
    case "jsx":
      return javascript({ jsx: true }).language.parser;
    case "typescript":
      return javascript({ typescript: true }).language.parser;
    case "tsx":
      return javascript({ typescript: true, jsx: true }).language.parser;
    case "rust":
      return rust().language.parser;
    case "json":
      return json().language.parser;
    case "markdown":
      return markdown().language.parser;
    case "html":
      return html().language.parser;
    case "css":
      return css().language.parser;
    case "sql":
      return sql().language.parser;
    case "shell":
      return StreamLanguage.define(shell).parser;
  }
}

/** The lezer parser for a language id (or alias/extension); null if unknown. */
export function parserForLanguage(name: string | undefined | null): Parser | null {
  const lang = normalizeLanguage(name);
  if (!lang) return null;
  let p = parserCache.get(lang);
  if (p === undefined) {
    try {
      p = buildParser(lang);
    } catch {
      p = null;
    }
    parserCache.set(lang, p);
  }
  return p;
}

/**
 * Class-based highlight style: token colors live in styles.css (`.tok-*`),
 * tuned for the hickory palette in both light and dark themes.
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

function languageSupport(lang: LanguageId): Extension {
  switch (lang) {
    case "python":
      return python();
    case "javascript":
      return javascript();
    case "jsx":
      return javascript({ jsx: true });
    case "typescript":
      return javascript({ typescript: true });
    case "tsx":
      return javascript({ typescript: true, jsx: true });
    case "rust":
      return rust();
    case "json":
      return json();
    case "markdown":
      return markdown();
    case "html":
      return html();
    case "css":
      return css();
    case "sql":
      return sql();
    case "shell":
      return StreamLanguage.define(shell);
  }
}

/**
 * Syntax highlighting for an output file's declared language (a name, alias,
 * extension, or full path). Unknown languages degrade to plain text.
 */
export function languageExtensions(language: string): Extension[] {
  const lang = normalizeLanguage(language) ?? languageFromPath(language);
  const exts: Extension[] = [syntaxHighlighting(hickoryHighlightStyle, { fallback: true })];
  if (lang) exts.push(languageSupport(lang));
  return exts;
}
