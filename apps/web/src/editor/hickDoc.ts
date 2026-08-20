// Hand-rolled incremental-friendly structure parser for .hick source, used by
// the Typora-style Document view to compute decorations.
//
// The hick no-escaping invariant is sacred: ONLY namespace-prefixed tags
// (`<hick:...>` / `</hick:...>`) are structure. Every other character —
// including raw `<`, `>`, `&`, and non-hick tags like `<div>` — is plain
// text, byte for byte. This parser therefore recognises hick: tags with a
// single regex and treats absolutely everything else as prose/markdown.
//
// It must NEVER throw, whatever the input: malformed tags simply aren't tags,
// unclosed blocks extend to the end of the document, stray closers are
// rendered as tag chrome without forming a block.

import { languageFromPath, normalizeLanguage } from "./languages";
import type { LanguageId } from "./languages";

export interface HickTag {
  from: number;
  to: number;
  /** Tag name without the `hick:` prefix, e.g. "exec". */
  name: string;
  closing: boolean;
  selfClosing: boolean;
  attrs: Record<string, string>;
  /** Offsets of each attribute name within the doc (for chrome styling). */
  attrNames: { from: number; to: number }[];
}

export interface HickBlock {
  /** Tag name without prefix ("exec", "file", "copy", …). */
  name: string;
  from: number;
  to: number;
  open: HickTag;
  /** Missing when the block is unclosed (runs to end of doc). */
  close?: HickTag;
  attrs: Record<string, string>;
  /** Content range between open and close tag (equal when self-closing). */
  contentFrom: number;
  contentTo: number;
}

export interface Heading {
  level: number;
  /** Whole line range. */
  from: number;
  to: number;
  /** The `#…# ` marker (including the trailing space). */
  markFrom: number;
  markTo: number;
}

/** One line of a `>` block quote. Quotes are scanned per LINE rather than
 * per paragraph: the decoration that tints a quote is a line decoration, and
 * a reader who splits a quote in half wants both halves to stay quoted while
 * they type — which per-paragraph grouping would fight. */
export interface QuoteLine {
  /** Whole line range. */
  from: number;
  to: number;
  /** How many `>` markers open the line; nesting tints deeper. */
  depth: number;
  /** The `> > ` marker run, trailing space included. */
  markFrom: number;
  markTo: number;
}

/** A `- [ ]` / `- [x]` task-list item. */
export interface TaskItem {
  /** Whole line range. */
  from: number;
  to: number;
  checked: boolean;
  /** The three-character box, brackets included — what a click replaces. */
  boxFrom: number;
  boxTo: number;
}

export interface InlineMark {
  kind: "strong" | "em" | "code";
  /** Content (between the delimiters). */
  from: number;
  to: number;
  /** Delimiter ranges (before and after the content). */
  openFrom: number;
  openTo: number;
  closeFrom: number;
  closeTo: number;
}

export interface HickDocStructure {
  tags: HickTag[];
  blocks: HickBlock[];
  headings: Heading[];
  inline: InlineMark[];
  quotes: QuoteLine[];
  tasks: TaskItem[];
}

// Open/close/self-closing hick: tags including quoted attributes. Quoted
// strings may contain `>`; unquoted attr chars may not.
const TAG_RE = /<(\/?)hick:([\w.-]+)((?:\s+(?:"[^"]*"|'[^']*'|[^<>"'])*)?)\s*(\/?)>/g;
const ATTR_RE = /([\w.:-]+)\s*=\s*("[^"]*"|'[^']*'|[^\s/>]+)/g;

export function parseTags(text: string): HickTag[] {
  const tags: HickTag[] = [];
  TAG_RE.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = TAG_RE.exec(text)) !== null) {
    const [whole, slash, name, attrText] = m;
    // The attr chunk can swallow a trailing `/`; detect self-closing from the
    // tag text itself.
    const selfClosing = slash !== "/" && whole.endsWith("/>");
    const attrs: Record<string, string> = {};
    const attrNames: { from: number; to: number }[] = [];
    if (attrText) {
      const attrBase = m.index + whole.indexOf(attrText, 1 + slash.length + 5 + name.length);
      ATTR_RE.lastIndex = 0;
      let a: RegExpExecArray | null;
      while ((a = ATTR_RE.exec(attrText)) !== null) {
        let value = a[2];
        if (
          (value.startsWith('"') && value.endsWith('"')) ||
          (value.startsWith("'") && value.endsWith("'"))
        ) {
          value = value.slice(1, -1);
        }
        attrs[a[1]] = value;
        attrNames.push({ from: attrBase + a.index, to: attrBase + a.index + a[1].length });
      }
    }
    tags.push({
      from: m.index,
      to: m.index + whole.length,
      name,
      closing: slash === "/",
      selfClosing,
      attrs,
      attrNames,
    });
  }
  return tags;
}

/** Pair open/close tags into blocks. Unclosed blocks run to end of doc. */
export function buildBlocks(text: string, tags: HickTag[]): HickBlock[] {
  const blocks: HickBlock[] = [];
  const stack: HickTag[] = [];
  for (const tag of tags) {
    if (tag.closing) {
      // Find the nearest matching open tag on the stack; ignore stray closers.
      for (let i = stack.length - 1; i >= 0; i--) {
        if (stack[i].name === tag.name) {
          const open = stack[i];
          stack.length = i;
          blocks.push({
            name: open.name,
            from: open.from,
            to: tag.to,
            open,
            close: tag,
            attrs: open.attrs,
            contentFrom: open.to,
            contentTo: tag.from,
          });
          break;
        }
      }
    } else if (tag.selfClosing) {
      blocks.push({
        name: tag.name,
        from: tag.from,
        to: tag.to,
        open: tag,
        close: undefined,
        attrs: tag.attrs,
        contentFrom: tag.to,
        contentTo: tag.to,
      });
    } else {
      stack.push(tag);
    }
  }
  // Unclosed opens: block extends to end of doc (never throw on malformed docs).
  for (const open of stack) {
    blocks.push({
      name: open.name,
      from: open.from,
      to: text.length,
      open,
      attrs: open.attrs,
      contentFrom: open.to,
      contentTo: text.length,
    });
  }
  blocks.sort((a, b) => a.from - b.from || b.to - a.to);
  return blocks;
}

/**
 * Elements whose content is PROSE — markdown styling applies inside them.
 * Everything else (exec commands, file bodies, copy/cut fragments, expect
 * output, …) is verbatim payload that markdown must not touch. `doc` is the
 * whole-document wrapper, `when` gates prose sections, and session elements
 * hold conversational prose.
 */
const PROSE_CONTAINERS = new Set([
  "doc",
  "when",
  "session",
  "user",
  "assistant",
  "observation",
]);

/** Ranges whose content is verbatim (exec commands, file bodies, copy slots) —
 * markdown styling must not apply inside them. Prose containers (doc, when,
 * session, …) are excluded so nested prose still styles; their verbatim
 * children contribute their own ranges. */
function verbatimRanges(blocks: HickBlock[]): [number, number][] {
  const ranges: [number, number][] = [];
  for (const b of blocks) {
    if (PROSE_CONTAINERS.has(b.name)) continue;
    if (b.contentTo > b.contentFrom) ranges.push([b.contentFrom, b.contentTo]);
  }
  ranges.sort((a, b) => a[0] - b[0]);
  return ranges;
}

function inRanges(pos: number, ranges: [number, number][]): boolean {
  for (const [from, to] of ranges) {
    if (pos >= from && pos < to) return true;
    if (from > pos) break;
  }
  return false;
}

const HEADING_RE = /^(#{1,6})[ \t]+\S?/;
// A quote prefix is one or more `>`, each allowed one space after it. Up to
// three leading spaces, the same indent CommonMark lets any block start with.
const QUOTE_RE = /^ {0,3}(?:>[ \t]?)+/;
// A task box belongs to a list item, so the bullet is part of the match: a
// bare `[ ]` in prose is text about brackets, not a checkbox.
const TASK_RE = /^([ \t]*(?:[-*+]|\d+[.)])[ \t]+)\[([ xX])\](?=[ \t])/;
// Inline code first (it suppresses other marks inside), then strong, then em.
const CODE_RE = /`([^`\n]+)`/g;
const STRONG_RE = /\*\*([^*\n]+)\*\*|__([^_\n]+)__/g;
// Emphasis content may not start or end with whitespace (so `2 * 3 * 4` in
// prose never turns into italics).
const EM_RE =
  /(?<![*\w])\*([^\s*][^*\n]*?[^\s*]|[^\s*])\*(?![*\w])|(?<![_\w])_([^\s_][^_\n]*?[^\s_]|[^\s_])_(?![_\w])/g;

function scanInline(
  text: string,
  lineFrom: number,
  out: InlineMark[],
  tagRanges: [number, number][],
) {
  const taken: [number, number][] = [];
  const overlapsTaken = (from: number, to: number) =>
    taken.some(([f, t]) => from < t && to > f) ||
    tagRanges.some(([f, t]) => lineFrom + from < t && lineFrom + to > f);

  const scan = (re: RegExp, kind: InlineMark["kind"], markLen: number) => {
    re.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = re.exec(text)) !== null) {
      const from = m.index;
      const to = m.index + m[0].length;
      if (overlapsTaken(from, to)) continue;
      taken.push([from, to]);
      out.push({
        kind,
        from: lineFrom + from + markLen,
        to: lineFrom + to - markLen,
        openFrom: lineFrom + from,
        openTo: lineFrom + from + markLen,
        closeFrom: lineFrom + to - markLen,
        closeTo: lineFrom + to,
      });
    }
  };
  scan(CODE_RE, "code", 1);
  scan(STRONG_RE, "strong", 2);
  scan(EM_RE, "em", 1);
}

/**
 * The markdown prose scan: headings + inline marks over every line that is
 * not inside a `verbatim` range. Shared between the .hick document parse
 * (verbatim = exec/file/fragment payloads, tagRanges = hick tags) and the
 * plain-markdown styling used by the output panes (see markdownStyling.ts),
 * which passes fenced-code ranges as verbatim and no tag ranges.
 */
export function scanMarkdownProse(
  text: string,
  verbatim: [number, number][],
  tagRanges: [number, number][],
): { headings: Heading[]; inline: InlineMark[]; quotes: QuoteLine[]; tasks: TaskItem[] } {
  const headings: Heading[] = [];
  const inline: InlineMark[] = [];
  const quotes: QuoteLine[] = [];
  const tasks: TaskItem[] = [];
  let lineFrom = 0;
  while (lineFrom <= text.length) {
    let lineTo = text.indexOf("\n", lineFrom);
    if (lineTo < 0) lineTo = text.length;
    if (lineTo > lineFrom && !inRanges(lineFrom, verbatim)) {
      const line = text.slice(lineFrom, lineTo);
      // A quote prefix is stripped BEFORE anything else looks at the line, so
      // `> ## Heading` and `> - [x] done` are a heading and a task inside a
      // quote rather than three unrelated features refusing to compose. What
      // is left is scanned at `base`, its offset in the document.
      let content = line;
      let base = lineFrom;
      const q = QUOTE_RE.exec(line);
      if (q) {
        quotes.push({
          from: lineFrom,
          to: lineTo,
          depth: (q[0].match(/>/g) ?? []).length,
          markFrom: lineFrom,
          markTo: lineFrom + q[0].length,
        });
        content = line.slice(q[0].length);
        base = lineFrom + q[0].length;
      }
      const h = HEADING_RE.exec(content);
      if (h) {
        const markEnd = content.length > h[1].length ? h[1].length + 1 : h[1].length;
        headings.push({
          level: h[1].length,
          from: lineFrom,
          to: lineTo,
          markFrom: base,
          markTo: base + markEnd,
        });
      } else {
        const t = TASK_RE.exec(content);
        if (t) {
          const boxFrom = base + t[1].length;
          tasks.push({
            from: lineFrom,
            to: lineTo,
            // Anything but a space is checked: `[x]`, `[X]`, and the `[-]`
            // some tools write all mean the box is not empty.
            checked: t[2] !== " ",
            boxFrom,
            boxTo: boxFrom + 3,
          });
        }
        scanInline(content, base, inline, tagRanges);
      }
    }
    lineFrom = lineTo + 1;
  }
  inline.sort((a, b) => a.openFrom - b.openFrom);
  return { headings, inline, quotes, tasks };
}

/**
 * Full structure parse. Linear in doc size (regex passes + one line walk);
 * cheap enough to run on every doc change for document-sized inputs, and the
 * view layer only materialises decorations for visible ranges.
 */
export function parseHickDoc(text: string): HickDocStructure {
  try {
    const tags = parseTags(text);
    const blocks = buildBlocks(text, tags);
    const verbatim = verbatimRanges(blocks);
    const tagRanges: [number, number][] = tags.map((t) => [t.from, t.to]);
    const { headings, inline, quotes, tasks } = scanMarkdownProse(text, verbatim, tagRanges);
    return { tags, blocks, headings, inline, quotes, tasks };
  } catch {
    // The document view must keep working on any input.
    return { tags: [], blocks: [], headings: [], inline: [], quotes: [], tasks: [] };
  }
}

/** Exec blocks in document order (the widget/cell list). */
export function execBlocksOf(structure: HickDocStructure): HickBlock[] {
  return structure.blocks.filter((b) => b.name === "exec");
}

/** A fenced code block sitting in the document's PROSE. */
export interface ProseFence {
  /** Whole fence, both fence lines included. */
  from: number;
  to: number;
  /** The info string after the opening fence: `python`, `bash`, or "". */
  info: string;
  /** The text between the fence lines, without the trailing newline. */
  body: string;
}

/**
 * Fenced code blocks in the document's prose — the ones a person could mean
 * to run.
 *
 * A fence inside a verbatim block is deliberately excluded: three backticks
 * in a `hick:file` body are content of a generated file, and in an exec they
 * are part of a command. Only prose fences are offered for conversion, and
 * only CLOSED ones — an unterminated fence has no end for the element to
 * take, and guessing where it stops would rewrite text nobody pointed at.
 */
export function proseFences(structure: HickDocStructure, text: string): ProseFence[] {
  const verbatim = verbatimRanges(structure.blocks);
  const fences: ProseFence[] = [];
  let openAt = -1;
  let marker = "";
  let info = "";
  let bodyFrom = 0;
  let lineFrom = 0;
  for (;;) {
    const nl = text.indexOf("\n", lineFrom);
    const lineTo = nl === -1 ? text.length : nl;
    const m = /^ {0,3}(```|~~~)(.*)$/.exec(text.slice(lineFrom, lineTo));
    if (m) {
      if (openAt === -1) {
        // A fence that OPENS inside verbatim payload is not prose, and
        // neither is its closer — skipping the open skips the pair.
        if (!inRanges(lineFrom, verbatim)) {
          openAt = lineFrom;
          marker = m[1];
          info = m[2].trim();
          bodyFrom = Math.min(lineTo + 1, text.length);
        }
      } else if (m[1] === marker && m[2].trim() === "") {
        fences.push({
          from: openAt,
          to: lineTo,
          info,
          body: text.slice(bodyFrom, Math.max(bodyFrom, lineFrom - 1)),
        });
        openAt = -1;
      }
    }
    if (nl === -1) break;
    lineFrom = nl + 1;
  }
  return fences;
}

/** File blocks in document order. */
export function fileBlocksOf(structure: HickDocStructure): HickBlock[] {
  return structure.blocks.filter((b) => b.name === "file");
}

/**
 * Every container name this document uses, in document order.
 *
 * Not just the `hick:container` declarations: a container also comes into
 * existence by being named on an exec (`image=` on the first one creates it),
 * or as the target of a fork. A chooser built only from declarations tells a
 * document with an implicit container that it has none, which is both wrong
 * and unhelpful — the name is right there in the source.
 */
export function containerNamesOf(structure: HickDocStructure): string[] {
  const names: string[] = [];
  const add = (name: string | undefined) => {
    if (name && !names.includes(name)) names.push(name);
  };
  for (const block of structure.blocks) {
    if (block.name === "container") add(block.attrs.name);
    else if (block.name === "exec") add(block.attrs.container);
    else if (block.name === "fork") add(block.attrs.to);
  }
  // Self-closing declarations are tags, not blocks.
  for (const tag of structure.tags) {
    if (tag.closing) continue;
    if (tag.name === "container") add(tag.attrs.name);
    else if (tag.name === "fork") add(tag.attrs.to);
  }
  return names;
}

/** Container declarations in document order (the environment cards). */
export function containerBlocksOf(structure: HickDocStructure): HickBlock[] {
  return structure.blocks.filter((b) => b.name === "container");
}

/**
 * Human-readable summary of a container's `hick:allow`/`hick:deny` children,
 * e.g. `["network: github.com:443", "deny network *"]`. Self-closing
 * containers (no children) yield an empty list.
 */
export function accessRulesOf(structure: HickDocStructure, container: HickBlock): string[] {
  const rules: string[] = [];
  for (const b of structure.blocks) {
    if (b.name !== "allow" && b.name !== "deny") continue;
    if (b.from < container.contentFrom || b.to > container.contentTo) continue;
    for (const [key, value] of Object.entries(b.attrs)) {
      rules.push(b.name === "deny" ? `deny ${key} ${value}` : `${key}: ${value}`);
    }
  }
  return rules;
}

/** Blocks with a given tag name, in document order. */
export function blocksNamed(structure: HickDocStructure, ...names: string[]): HickBlock[] {
  return structure.blocks.filter((b) => names.includes(b.name));
}

/** Copy/cut fragment blocks in document order. */
export function fragmentBlocksOf(structure: HickDocStructure): HickBlock[] {
  return blocksNamed(structure, "copy", "cut");
}

/** `hick:when` conditional blocks in document order. */
export function whenBlocksOf(structure: HickDocStructure): HickBlock[] {
  return blocksNamed(structure, "when");
}

/**
 * Does a paste selector (`#id`, `.class`, or a comma list of those) refer to
 * this copy/cut fragment?
 */
export function selectorMatches(selector: string | undefined, fragment: HickBlock): boolean {
  if (!selector) return false;
  const id = fragment.attrs.id;
  const classes = (fragment.attrs.class ?? "").split(/\s+/).filter(Boolean);
  for (const part of selector.split(",").map((s) => s.trim())) {
    if (id && part === `#${id}`) return true;
    if (part.startsWith(".") && classes.includes(part.slice(1))) return true;
  }
  return false;
}

/** The first copy/cut fragment a paste selector refers to, or null. */
export function resolvePasteTarget(
  structure: HickDocStructure,
  selector: string | undefined,
): HickBlock | null {
  for (const frag of fragmentBlocksOf(structure)) {
    if (selectorMatches(selector, frag)) return frag;
  }
  return null;
}

function within(inner: { from: number; to: number }, from: number, to: number): boolean {
  return inner.from >= from && inner.to <= to;
}

/**
 * The code sub-ranges of a verbatim block's content: the content span minus
 * every nested hick tag and nested block (an `expect` body is output, a
 * nested `exec` inside a `file` highlights under its own rules). Ranges are
 * sorted and non-overlapping; empty when the block has no highlightable code.
 */
export function codeRangesOf(structure: HickDocStructure, block: HickBlock): [number, number][] {
  const { contentFrom, contentTo } = block;
  if (contentTo <= contentFrom) return [];
  const cuts: [number, number][] = [];
  for (const t of structure.tags) {
    if (within(t, contentFrom, contentTo)) cuts.push([t.from, t.to]);
  }
  for (const b of structure.blocks) {
    if (b === block) continue;
    if (within(b, contentFrom, contentTo)) cuts.push([b.from, b.to]);
  }
  cuts.sort((a, b) => a[0] - b[0]);
  const out: [number, number][] = [];
  let pos = contentFrom;
  for (const [from, to] of cuts) {
    if (from > pos) out.push([pos, from]);
    pos = Math.max(pos, to);
  }
  if (pos < contentTo) out.push([pos, contentTo]);
  return out.filter(([a, b]) => b > a);
}

/**
 * The language a block's body should highlight as, or null for plain text:
 *  - `file`: the `language` attribute, else the path extension;
 *  - `exec`: shell;
 *  - `copy`/`cut`: an explicit `lang`/`language` attribute, else inferred
 *    from where the fragment is pasted — a `hick:paste` inside a `hick:file`
 *    whose selector matches this fragment lends the file's language.
 */
export function languageForBlock(
  structure: HickDocStructure,
  block: HickBlock,
): LanguageId | null {
  switch (block.name) {
    case "file":
      return normalizeLanguage(block.attrs.language) ?? languageFromPath(block.attrs.path);
    case "exec":
      return "shell";
    // A session's executed script: `<hick:action lang="bash">`. Highlighting
    // it like any other code block is the point — the agent's actions are
    // cells, not chat transcript.
    case "action":
      return normalizeLanguage(block.attrs.lang ?? block.attrs.language) ?? "shell";
    case "copy":
    case "cut": {
      const declared = normalizeLanguage(block.attrs.lang ?? block.attrs.language);
      if (declared) return declared;
      for (const file of fileBlocksOf(structure)) {
        const fileLang =
          normalizeLanguage(file.attrs.language) ?? languageFromPath(file.attrs.path);
        if (!fileLang) continue;
        for (const t of structure.tags) {
          if (t.name !== "paste" || !within(t, file.contentFrom, file.contentTo)) continue;
          if (selectorMatches(t.attrs.select, block)) return fileLang;
        }
      }
      return null;
    }
    default:
      return null;
  }
}

/**
 * Content range of the first `hick:expect` block nested in `exec`, or null.
 * Used to style the expect body in the source as the verified output.
 */
export function expectRangeOf(
  structure: HickDocStructure,
  exec: HickBlock,
): [number, number] | null {
  for (const b of structure.blocks) {
    if (b.name !== "expect") continue;
    if (b.from < exec.contentFrom || b.to > exec.contentTo) continue;
    if (b.contentTo <= b.contentFrom) return null;
    return [b.contentFrom, b.contentTo];
  }
  return null;
}
