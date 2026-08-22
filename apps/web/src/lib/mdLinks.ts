// Markdown links and images, as text — found, made, and pointed at the right
// file.
//
// Everything here is a pure function over a string. The CodeMirror side
// (editor/mdLinks.ts) decorates and opens what this finds, and the paste side
// (editor/mdPaste.ts) writes what this makes; keeping the string work here is
// what lets both be tested without an editor.
//
// ## Why the link syntax is markdown and not a hick tag
//
// A `.hick` document is markdown first (`bare-documents.md`), and a link is
// the one piece of markup every reader of the woven `.md` already
// understands. A `<hick:link>` would have to weave to `[…](…)` anyway, and
// would be unreadable in the source in the meantime. So a cross-document link
// is written the way a reader would write it, and the WEAVER is what makes it
// land — see `crates/hick-literate/src/links.rs`, which rewrites a `.hick`
// target to the `.md` that document weaves.

/** A markdown inline link or image, located in some text. */
export interface MdLink {
  /** An image (`![…](…)`) rather than a link (`[…](…)`). */
  image: boolean;
  /** The whole construct, brackets and parentheses included. */
  from: number;
  to: number;
  /** The label between the brackets. */
  textFrom: number;
  textTo: number;
  /** The destination inside the parentheses, title excluded. */
  targetFrom: number;
  targetTo: number;
  /** The destination, as written. */
  target: string;
  /** The label, as written. */
  text: string;
}

/**
 * Every inline link and image in `text`.
 *
 * A deliberately small scanner rather than a markdown parser: it matches the
 * bracket/parenthesis pair, balances nested parentheses inside the
 * destination (Wikipedia URLs have them), and stops at a newline, because an
 * unclosed `[` at the end of a paragraph is a typo and not a link that runs
 * to the end of the document.
 */
export function findLinks(text: string): MdLink[] {
  const found: MdLink[] = [];
  for (let i = 0; i < text.length; i++) {
    if (text[i] !== "[") continue;
    // An escaped bracket is a bracket.
    if (i > 0 && text[i - 1] === "\\") continue;
    const image = i > 0 && text[i - 1] === "!" && !(i > 1 && text[i - 2] === "\\");
    const textFrom = i + 1;
    const textTo = closingBracket(text, textFrom);
    if (textTo < 0) continue;
    if (text[textTo + 1] !== "(") continue;
    const targetFrom = textTo + 2;
    const close = closingParen(text, targetFrom);
    if (close < 0) continue;
    // `[a](b "title")` — the title is not part of the destination.
    const inner = text.slice(targetFrom, close);
    const titleAt = titleStart(inner);
    const targetTo = targetFrom + (titleAt < 0 ? inner.length : titleAt);
    found.push({
      image,
      from: image ? i - 1 : i,
      to: close + 1,
      textFrom,
      textTo,
      targetFrom,
      targetTo,
      target: text.slice(targetFrom, targetTo).trim(),
      text: text.slice(textFrom, textTo),
    });
    i = close;
  }
  return found;
}

/** Index of the `]` closing a `[` at `from`, or -1. Nesting counts. */
function closingBracket(text: string, from: number): number {
  let depth = 0;
  for (let i = from; i < text.length; i++) {
    const ch = text[i];
    if (ch === "\n") return -1;
    if (ch === "\\") {
      i++;
      continue;
    }
    if (ch === "[") depth++;
    else if (ch === "]") {
      if (depth === 0) return i;
      depth--;
    }
  }
  return -1;
}

/** Index of the `)` closing a `(` before `from`, or -1. */
function closingParen(text: string, from: number): number {
  let depth = 0;
  for (let i = from; i < text.length; i++) {
    const ch = text[i];
    if (ch === "\n") return -1;
    if (ch === "\\") {
      i++;
      continue;
    }
    if (ch === "(") depth++;
    else if (ch === ")") {
      if (depth === 0) return i;
      depth--;
    }
  }
  return -1;
}

/** Where a `"…"` / `'…'` title begins inside a destination, or -1. */
function titleStart(inner: string): number {
  const match = /\s+["']/.exec(inner);
  return match ? match.index : -1;
}

/**
 * Does this text name somewhere to go?
 *
 * Used to decide whether a paste over a selection is a link being applied to
 * words rather than text replacing them, so it is deliberately strict: a
 * scheme and something after it, on one line, with no whitespace. "See
 * https://example.com for more" is prose being pasted, not a URL, and
 * swallowing it into a link would be the wrong guess.
 */
export function isUrl(text: string): boolean {
  const trimmed = text.trim();
  if (trimmed.length === 0 || /\s/.test(trimmed)) return false;
  if (/^(https?|ftp|mailto|tel|file):/i.test(trimmed)) return true;
  // `www.example.com` with no scheme is the other thing people copy out of a
  // browser's address bar and out of an email.
  return /^www\.[^.]+\.[^.]+$/i.test(trimmed);
}

/** A URL that a markdown destination can hold: schemeless `www.` gets one,
 * and spaces (which would end the destination) get encoded. */
export function normalizeUrl(text: string): string {
  const trimmed = text.trim();
  const url = /^www\./i.test(trimmed) ? `https://${trimmed}` : trimmed;
  return url.replace(/ /g, "%20").replace(/\(/g, "%28").replace(/\)/g, "%29");
}

/**
 * The markdown for pasting `url` over `selected`, or null when this paste is
 * not that act.
 *
 * Null — rather than a link with an empty label — when nothing is selected:
 * pasting a URL at the caret should give you the URL you copied, which is
 * what every other editor does and what you get by doing nothing here.
 */
export function linkOverSelection(selected: string, pasted: string): string | null {
  if (!isUrl(pasted)) return null;
  if (selected.length === 0 || selected.includes("\n")) return null;
  // Selecting a link and pasting a URL over it means "point this somewhere
  // else", not "nest a link inside a label".
  const links = findLinks(selected);
  const whole = links.find((l) => l.from === 0 && l.to === selected.length);
  const label = whole ? whole.text : selected;
  return `[${label}](${normalizeUrl(pasted)})`;
}

/** Characters a markdown destination cannot hold bare. */
export function encodeTarget(path: string): string {
  return path.replace(/ /g, "%20").replace(/\(/g, "%28").replace(/\)/g, "%29");
}

/** The markdown for an image at `path`, labelled `alt`. */
export function imageMarkdown(path: string, alt: string): string {
  return `![${alt.replace(/[[\]]/g, "")}](${encodeTarget(path)})`;
}

/**
 * Is this destination somewhere in the notes folder rather than on the web?
 *
 * A fragment alone (`#section`) is inside the document that holds it, which
 * is neither — it is already correct in the woven markdown and wants no
 * rewriting and no file to open.
 */
export function isLocalTarget(target: string): boolean {
  if (target.length === 0) return false;
  if (target.startsWith("#")) return false;
  if (/^[a-z][a-z0-9+.-]*:/i.test(target)) return false;
  return !target.startsWith("//");
}

/** A destination split into the path and the `#fragment` after it. */
export function splitFragment(target: string): { path: string; fragment: string } {
  const hash = target.indexOf("#");
  return hash < 0
    ? { path: target, fragment: "" }
    : { path: target.slice(0, hash), fragment: target.slice(hash) };
}

/** The destination as it should read in woven markdown: a link to a document
 * points at the markdown that document weaves, because that is the file the
 * reader of the weave actually has. Anything else is left alone. */
export function wovenTarget(target: string): string {
  if (!isLocalTarget(target)) return target;
  const { path, fragment } = splitFragment(target);
  if (!path.toLowerCase().endsWith(".hick")) return target;
  return `${path.slice(0, -".hick".length)}.md${fragment}`;
}

/**
 * Where a link in the document at `docPath` points, as a folder-relative
 * path — or null when it points nowhere this app opens.
 *
 * Relative destinations resolve against the LINKING document's directory,
 * which is what a relative link means everywhere else and what keeps a folder
 * of notes movable.
 */
export function resolveTarget(docPath: string | null, target: string): string | null {
  if (!isLocalTarget(target)) return null;
  const { path } = splitFragment(decodeURI(target));
  if (path.length === 0) return null;
  const base = path.startsWith("/")
    ? []
    : (docPath ?? "").split("/").slice(0, -1).filter(Boolean);
  const parts = path.replace(/^\//, "").split("/");
  const out = [...base];
  for (const part of parts) {
    if (part === "" || part === ".") continue;
    if (part === "..") out.pop();
    else out.push(part);
  }
  return out.length > 0 ? out.join("/") : null;
}
