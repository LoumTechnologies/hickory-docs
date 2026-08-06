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

/** Ranges whose content is verbatim (exec commands, file bodies, copy slots) —
 * markdown styling must not apply inside them. */
function verbatimRanges(blocks: HickBlock[]): [number, number][] {
  const ranges: [number, number][] = [];
  for (const b of blocks) {
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
 * Full structure parse. Linear in doc size (regex passes + one line walk);
 * cheap enough to run on every doc change for document-sized inputs, and the
 * view layer only materialises decorations for visible ranges.
 */
export function parseHickDoc(text: string): HickDocStructure {
  try {
    const tags = parseTags(text);
    const blocks = buildBlocks(text, tags);
    const verbatim = verbatimRanges(blocks.filter((b) => b.name !== "session"));
    const tagRanges: [number, number][] = tags.map((t) => [t.from, t.to]);

    const headings: Heading[] = [];
    const inline: InlineMark[] = [];
    let lineFrom = 0;
    while (lineFrom <= text.length) {
      let lineTo = text.indexOf("\n", lineFrom);
      if (lineTo < 0) lineTo = text.length;
      if (lineTo > lineFrom && !inRanges(lineFrom, verbatim)) {
        const line = text.slice(lineFrom, lineTo);
        const h = HEADING_RE.exec(line);
        if (h) {
          const markEnd = line.length > h[1].length ? h[1].length + 1 : h[1].length;
          headings.push({
            level: h[1].length,
            from: lineFrom,
            to: lineTo,
            markFrom: lineFrom,
            markTo: lineFrom + markEnd,
          });
        } else {
          scanInline(line, lineFrom, inline, tagRanges);
        }
      }
      lineFrom = lineTo + 1;
    }
    inline.sort((a, b) => a.openFrom - b.openFrom);
    return { tags, blocks, headings, inline };
  } catch {
    // The document view must keep working on any input.
    return { tags: [], blocks: [], headings: [], inline: [] };
  }
}

/** Exec blocks in document order (the widget/cell list). */
export function execBlocksOf(structure: HickDocStructure): HickBlock[] {
  return structure.blocks.filter((b) => b.name === "exec");
}

/** File blocks in document order. */
export function fileBlocksOf(structure: HickDocStructure): HickBlock[] {
  return structure.blocks.filter((b) => b.name === "file");
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
