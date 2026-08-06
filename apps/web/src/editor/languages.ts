import type { Extension } from "@codemirror/state";
import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
import { python } from "@codemirror/lang-python";
import { javascript } from "@codemirror/lang-javascript";
import { rust } from "@codemirror/lang-rust";
import { json } from "@codemirror/lang-json";
import { markdown } from "@codemirror/lang-markdown";

/** Syntax highlighting for an output file's declared language. */
export function languageExtensions(language: string): Extension[] {
  const lang = language.toLowerCase();
  const exts: Extension[] = [syntaxHighlighting(defaultHighlightStyle, { fallback: true })];
  switch (lang) {
    case "python":
    case "py":
      exts.push(python());
      break;
    case "javascript":
    case "js":
    case "jsx":
      exts.push(javascript({ jsx: lang === "jsx" }));
      break;
    case "typescript":
    case "ts":
    case "tsx":
      exts.push(javascript({ typescript: true, jsx: lang === "tsx" }));
      break;
    case "rust":
    case "rs":
      exts.push(rust());
      break;
    case "json":
      exts.push(json());
      break;
    case "markdown":
    case "md":
      exts.push(markdown());
      break;
    default:
      // Unknown language: plain text, still readable.
      break;
  }
  return exts;
}
