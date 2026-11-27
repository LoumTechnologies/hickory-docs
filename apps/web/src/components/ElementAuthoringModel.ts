import { markdownLanguage } from "@codemirror/lang-markdown";
import type { SyntaxNode } from "@lezer/common";
import { mathSpans } from "../lib/math";
import { parseStructure, verbatimRanges, yamlFrontmatter } from "../editor/hickDoc";
import {
  INSERT_ELEMENTS, elementForExisting, initialValues, defaultBody, buildInsertion, normalize, validate,
} from "../lib/insertCatalog";

type Values = Record<string, string>;
type Range = { from: number; to: number };
type Field = { name: string; label: string; choices?: readonly string[]; required?: boolean; hint?: string; placeholder?: string };
type Markdown = {
  id: string; title: string; sample: string; block?: boolean; fields?: Field[];
  build: (body: string, fields: Values) => { before: string; body: string; after: string };
};
const surround = (before: string, body: string, after = "") => ({ before, body, after });
const levels = ["1", "2", "3", "4", "5", "6"];
function code(body: string, minimum: number, marker = "`") {
  const runs = body.match(marker === "`" ? /`+/g : /~+/g) ?? [];
  return marker.repeat(runs.reduce((longest, run) => Math.max(longest, run.length + 1), minimum));
}

const markdown: Markdown[] = [
  { id: "heading", title: "Heading", sample: "Heading", block: true, fields: [{ name: "level", label: "Level", choices: levels }],
    build: (body, values) => surround("#".repeat(Number(values.level)) + " ", body) },
  { id: "bold", title: "Bold", sample: "bold text", build: body => surround("**", body, "**") },
  { id: "italic", title: "Italic", sample: "emphasized text", build: body => surround("*", body, "*") },
  { id: "strike", title: "Strikethrough", sample: "struck text", build: body => surround("~~", body, "~~") },
  { id: "inline-code", title: "Inline code", sample: "code", build: body => {
    const marker = code(body, 1);
    const padded = /^`|`$|^ .* $/.test(body) && !!body.trim();
    return surround(marker + (padded ? " " : ""), body, (padded ? " " : "") + marker);
  } },
  { id: "fence", title: "Code fence", sample: "code", block: true, fields: [{ name: "info", label: "Language or info string" }],
    build: (body, values) => { const marker = code(body, 3, values.info.includes("`") ? "~" : "`"); return surround(marker + values.info + "\n", body, "\n" + marker); } },
  ...([["link", "Link", "[", "link text"], ["image", "Image", "![", "alternative text"]] as const).map(([id, title, open, sample]) => ({
    id, title, sample, fields: [{ name: "destination", label: "Destination", required: true }],
    build: (body: string, values: Values) => surround(open, body, "](" + values.destination + ")"),
  })),
  { id: "inline-math", title: "Inline equation", sample: "x^2", build: body => surround("$", body.trim(), "$") },
  { id: "display-math", title: "Display equation", sample: "x^2 + y^2 = z^2", block: true, build: body => surround("$$\n", body, "\n$$") },
  { id: "quote", title: "Block quote", sample: "Quoted text", block: true, build: body => surround("> ", body.replace(/\n/g, "\n> ")) },
  { id: "bullet", title: "Bullet list item", sample: "List item", block: true, build: body => surround("- ", body) },
  { id: "ordered", title: "Numbered list item", sample: "List item", block: true, build: body => surround("1. ", body) },
  { id: "task", title: "Task list item", sample: "Task", block: true, fields: [{ name: "status", label: "Status", choices: ["Open", "Done"] }],
    build: (body, values) => surround(values.status === "Done" ? "- [x] " : "- [ ] ", body) },
  { id: "table", title: "Table", sample: "| Column | Value |\n| --- | --- |\n| Name | Value |", block: true, build: body => surround("", body) },
  { id: "rule", title: "Thematic break", sample: "---", block: true, build: body => surround("", body) },
];

export type ElementOption = {
  id: string; title: string; group: string; summary?: string; belongsIn?: string; fields: readonly Field[]; bodyLabel?: string;
};
export type ElementTarget = Range & { option: ElementOption; values: Values; body: string; render: (values: Values, body: string) => string };
export type ElementReading = ReturnType<typeof readElements>;
export type ElementPlan = Range & { insert: string; selectFrom: number; selectTo: number };
const hickOptions = new Map(INSERT_ELEMENTS.map(element => [element.id, element]));
const mdOptions = new Map(markdown.map(element => ["md:" + element.id, element]));
export const elementOptions: ElementOption[] = [
  ...markdown.map(element => ({ id: "md:" + element.id, title: element.title, group: "Markdown", fields: element.fields ?? [], bodyLabel: element.id === "table" ? "Table source" : element.id === "fence" || element.id === "inline-code" ? "Code" : "Text" })),
  ...INSERT_ELEMENTS.map(element => ({ ...element })),
];
const optionById = new Map(elementOptions.map(option => [option.id, option]));

function normalizeLines(text: string) { return text.replace(/\r\n?|\n/g, "\n"); }
function lineEnding(source: string, at: number) {
  return source.slice(at).match(/\r\n?|\n/)?.[0] ?? source.slice(0, at).match(/\r\n?|\n/g)?.at(-1) ?? "\n";
}

function hickTarget(source: string, block: ReturnType<typeof parseStructure>["blocks"][number]): ElementTarget | undefined {
  const element = elementForExisting(block.name, block.attrs);
  if (!element || !block.open || !block.open.selfClosing && !block.close) return;
  const { from, to, open, contentFrom, contentTo } = block;
  const values = Object.fromEntries(element.fields.map(field => [field.name, block.attrs[field.name] ?? ""]));
  let content = source.slice(contentFrom, contentTo);
  const left = element.body === "block" ? /^(?:\r\n|\r|\n)/.exec(content)?.[0] ?? "" : "";
  content = content.slice(left.length);
  const right = element.body === "block" ? /(?:\r\n|\r|\n)[ \t]*$/.exec(content)?.[0] ?? "" : "";
  const rawBody = content.slice(0, content.length - right.length);
  const body = normalizeLines(rawBody);
  const opening = source.slice(from, open.to);
  const close = source.slice(contentTo, to);
  return { from, to, option: optionById.get(element.id)!, values, body,
    render(next, editedBody) {
      const attrs = new Map(element.fields.filter(field => next[field.name] !== values[field.name]).map(field => [field.name, normalize(element, field.name, next[field.name] ?? "")]));
      const attr = (name: string, value: string) => {
        const quote = value.includes('"') ? "'" : '"';
        return `${name}=${quote}${value}${quote}`;
      };
      // The parser supplies the element boundary. Change only fields the form
      // owns, keeping untouched attributes and the opening tag's spelling.
      const tag = opening.replace(/(\s+)([^=/ >\t\r\n]+)(?:\s*=\s*(?:"[^"]*"|'[^']*'))?/g, (raw, space: string, name: string) => {
        const value = attrs.get(name);
        if (value === undefined) return raw;
        attrs.delete(name);
        return value ? space + attr(name, value) : "";
      }).replace(/\s*\/?>$/, suffix => [...attrs].map(([name, value]) => value ? " " + attr(name, value) : "").join("") + suffix);
      if (editedBody === body) return tag + source.slice(open.to, to);
      const replacement = editedBody.replace(/\n/g, lineEnding(source, contentFrom));
      if (open.selfClosing) {
        const wrap = element.body === "block" ? lineEnding(source, contentFrom) : "";
        return tag.replace(/\s*\/>$/, ">") + wrap + replacement + wrap + "</" + /^<([^\s/>]+)/.exec(tag)![1] + ">";
      }
      return tag + left + replacement + right + close;
    },
  };
}

function markdownTarget(source: string, node: SyntaxNode): ElementTarget | undefined {
  const { from } = node;
  const to = source[node.to - 1] === "\r" ? node.to - 1 : node.to;
  if (!/Heading[1-6]$/.test(node.name) &&
      !["FencedCode", "Link", "Image", "StrongEmphasis", "Emphasis", "Strikethrough", "InlineCode",
        "Blockquote", "ListItem", "Table", "HorizontalRule"].includes(node.name)) return;
  const text = source.slice(from, to);
  const children = (name: string) => node.getChildren(name);
  let id: string, start = from, end = to;
  const values: Values = {};
  let destination: Range | undefined;
  let level: Range | undefined;
  let fence: { open: Range; close: Range; info: Range; marker: string; empty: boolean } | undefined;
  const heading = /^(ATX|Setext)Heading([1-6])$/.exec(node.name);
  if (heading) {
    id = "heading";
    values.level = heading[2];
    const marks = children("HeaderMark");
    level = marks[0];
    if (heading[1] === "ATX") {
      start = marks[0].to;
      if (/\s/.test(source[start] ?? "")) start++;
      end = marks[1]?.from ?? to;
      while (end > start && /[ \t]/.test(source[end - 1])) end--;
    } else {
      end = marks[0].from;
      if (source[end - 1] === "\n") end--;
      if (source[end - 1] === "\r") end--;
    }
  } else if (node.name === "FencedCode") {
    id = "fence";
    const marks = children("CodeMark");
    if (marks.length !== 2) return;
    const open = marks[0], close = marks[1];
    const info = node.getChild("CodeInfo") ?? { from: open.to, to: open.to };
    start = source.indexOf("\n", info.to) + 1;
    if (!start || start > close.from) return;
    const lastBreak = source.lastIndexOf("\n", close.from - 1);
    end = Math.max(start, source[lastBreak - 1] === "\r" ? lastBreak - 1 : lastBreak);
    values.info = source.slice(info.from, info.to);
    fence = { open, close, info, marker: source[open.from], empty: lastBreak < start };
  } else if (node.name === "Link" || node.name === "Image") {
    id = node.name.toLowerCase();
    const marks = children("LinkMark");
    if (marks.length !== 4 || source[marks[2].from] !== "(") return;
    start = marks[0].to;
    end = marks[1].from;
    destination = node.getChild("URL") ?? { from: marks[2].to, to: marks[2].to };
    values.destination = source.slice(destination.from, destination.to);
  } else if (node.name === "StrongEmphasis" || node.name === "Emphasis" || node.name === "Strikethrough" || node.name === "InlineCode") {
    id = { StrongEmphasis: "bold", Emphasis: "italic", Strikethrough: "strike", InlineCode: "inline-code" }[node.name]!;
    const mark = { StrongEmphasis: "EmphasisMark", Emphasis: "EmphasisMark", Strikethrough: "StrikethroughMark", InlineCode: "CodeMark" }[node.name]!;
    const marks = children(mark);
    if (marks.length !== 2) return;
    start = marks[0].to;
    end = marks[1].from;
  } else if (node.name === "Blockquote" || node.name === "ListItem") {
    const task = node.getChild("Task")?.getChild("TaskMarker");
    const marker = task ?? node.getChild(node.name === "Blockquote" ? "QuoteMark" : "ListMark");
    if (!marker) return;
    id = task ? "task" : node.name === "Blockquote" ? "quote" : node.parent?.name === "OrderedList" ? "ordered" : "bullet";
    if (task) values.status = /x/i.test(source.slice(task.from, task.to)) ? "Done" : "Open";
    start = marker.to;
    while (start < end && /[ \t]/.test(source[start])) start++;
    if (task) level = task;
  } else if (node.name === "Table" || node.name === "HorizontalRule") {
    id = node.name === "Table" ? "table" : "rule";
  } else return;
  if (start > end || end > to) return;
  const rawBody = source.slice(start, end);
  const body = normalizeLines(rawBody);
  return { from, to, option: optionById.get("md:" + id)!, values, body,
    render(next, editedBody) {
      const edits: (Range & { text: string })[] = [];
      if (body !== editedBody) {
        const eol = lineEnding(source, start);
        const addEnd = fence?.empty ? eol : "";
        edits.push({ from: start, to: end, text: editedBody.replace(/\n/g, eol) + addEnd });
      }
      if (heading && level && next.level !== values.level) {
        if (heading[1] === "Setext") return "#".repeat(Number(next.level)) + " " + editedBody;
        edits.push({ from: level.from, to: level.to, text: "#".repeat(Number(next.level)) });
      }
      if (destination && next.destination !== values.destination) edits.push({ from: destination.from, to: destination.to, text: next.destination });
      if (id === "task" && level && next.status !== values.status) edits.push({ from: level.from, to: level.to, text: next.status === "Done" ? "[x]" : "[ ]" });
      if (fence) {
        if (next.info !== values.info) edits.push({ from: fence.info.from, to: fence.info.to, text: next.info });
        if (body !== editedBody || next.info !== values.info) {
          const marker = next.info.includes("`") ? "~" : fence.marker;
          const run = code(editedBody, fence.open.to - fence.open.from, marker);
          if (run !== source.slice(fence.open.from, fence.open.to)) edits.push({ from: fence.open.from, to: fence.open.to, text: run }, { from: fence.close.from, to: fence.close.to, text: run });
        }
      }
      if (id === "inline-code" && body !== editedBody) {
        const parts = mdOptions.get("md:inline-code")!.build(editedBody, next);
        return parts.before + parts.body + parts.after;
      }
      let result = text;
      for (const edit of edits.sort((a, b) => b.from - a.from || b.to - a.to)) {
        result = result.slice(0, edit.from - from) + edit.text + result.slice(edit.to - from);
      }
      return result;
    },
  };
}

export function readElements(source: string) {
  const structure = parseStructure(source);
  // The engine also projects executable Markdown fences as synthetic
  // Hickory tags. Their source spelling still belongs to the fence editor.
  const literalBlocks = structure.blocks.filter(block => source[block.from] === "<");
  const literalTags = structure.tags.filter(tag => source[tag.from] === "<");
  const ranges = verbatimRanges(literalBlocks).concat(literalTags.map(tag => [tag.from, tag.to]));
  const frontmatter = yamlFrontmatter(source);
  if (frontmatter) ranges.push([0, frontmatter.end]);
  // Mask Hickory tags and payloads in place, retaining line breaks and offsets.
  const masked: string[] = [];
  let end = 0;
  for (const [from, to] of ranges.sort((a, b) => a[0] - b[0])) {
    if (to <= end) continue;
    masked.push(source.slice(end, Math.max(end, from)), source.slice(Math.max(end, from), to).replace(/[^\r\n]/g, " "));
    end = to;
  }
  masked.push(source.slice(end));
  const markdownSource = masked.join("").replace(/\r(?!\n)/g, "\n");
  const prose = markdownLanguage.parser.parse(markdownSource);
  const codeRanges: Range[] = [];
  const targets: ElementTarget[] = [];
  const enclosedByCode = (offset: number) => {
    for (let node: SyntaxNode | null = prose.resolveInner(offset, 1); node; node = node.parent) {
      if (["FencedCode", "CodeBlock", "InlineCode"].includes(node.name)) return true;
    }
    return false;
  };
  for (const block of literalBlocks) {
    const target = hickTarget(source, block);
    if (target && !enclosedByCode(block.from)) targets.push(target);
  }
  prose.iterate({ enter: ref => {
    if (["FencedCode", "CodeBlock", "InlineCode"].includes(ref.name)) codeRanges.push({ from: ref.from, to: ref.to });
    const target = markdownTarget(source, ref.node);
    if (target) targets.push(target);
  } });
  for (const math of mathSpans(markdownSource, codeRanges.map(code => [code.from, code.to]))) {
    if (codeRanges.some(code => code.from < math.to && code.to > math.from)) continue;
    const { from, to, display } = math;
    const length = display ? 2 : 1;
    const raw = source.slice(from, to);
    const body = normalizeLines(raw.slice(length, -length));
    const option = optionById.get(display ? "md:display-math" : "md:inline-math")!;
    targets.push({ from, to, option, values: {}, body, render: (_, edited) => edited === body ? raw :
      raw.slice(0, length) + edited.replace(/\n/g, lineEnding(source, from)) + raw.slice(-length) });
  }
  return { source, structure, codeRanges, frontmatter, targets: targets.sort((a, b) => a.from - b.from || b.to - a.to) };
}

export function newElement(option: ElementOption, selected: string, configured?: Values) {
  const hick = hickOptions.get(option.id);
  const md = mdOptions.get(option.id);
  const values = configured ?? (hick ? initialValues(hick) : Object.fromEntries(option.fields.map(field => [field.name, field.choices?.[0] ?? ""])));
  return { values, body: normalizeLines(hick ? defaultBody(hick, selected, values) : selected.trim() ? selected : md?.sample ?? "") };
}

export function planElement(reading: ElementReading, range: Range, option: ElementOption, values: Values, body: string, target?: ElementTarget): ElementPlan {
  const { source, structure, codeRanges, frontmatter } = reading;
  const hick = hickOptions.get(option.id), md = mdOptions.get(option.id);
  const { from, to } = range;
  // Preserve existing values verbatim when validating edited attributes.
  const fields = hick && target ? { ...hick, fields: hick.fields.filter(field => values[field.name] !== target.values[field.name]) } : hick;
  const errors = fields ? Object.values(validate(fields, values)) : [];
  for (const field of option.fields) {
    if (!hick && field.required && !values[field.name]?.trim() && values[field.name] !== target?.values[field.name]) errors.push(`Enter a ${field.label.toLowerCase()}.`);
    if (!hick && /[\r\n]/.test(values[field.name] ?? "")) errors.push(`Keep ${field.label.toLowerCase()} on one line.`);
  }
  if (md && !["fence", "quote", "table", "display-math", ...(target ? ["bullet", "ordered", "task"] : [])].includes(md.id) && body.includes("\n") && body !== target?.body) errors.push("Use one line of text for this element.");
  if (errors.length) throw new Error(errors[0]);
  if (target) {
    const insert = target.render(values, body);
    return { from, to, insert, selectFrom: from + insert.length, selectTo: from + insert.length };
  }
  if (structure.tags.some(tag => from > tag.from && from < tag.to || to > tag.from && to < tag.to)) {
    throw new Error("Move the selection outside the tag before inserting.");
  }
  if (codeRanges.some(code => from < code.to && to > code.from && (from > code.from || to < code.to))
      || frontmatter && from < frontmatter.end) {
    throw new Error("Move the selection into document prose before inserting.");
  }
  if (md && verbatimRanges(structure.blocks).some(([start, end]) => from >= start && to <= end)) {
    throw new Error("Move the selection outside the Hickory payload before inserting Markdown.");
  }
  const before = normalizeLines(source.slice(0, from)), after = normalizeLines(source.slice(to));
  const useBreak = (text: string) => text.replace(/\n/g, lineEnding(source, from));
  let text, selectFrom, selectTo;
  if (hick) {
    const built = buildInsertion(hick, values, { before, after, selected: "" }, body, structure.prefix);
    ({ text, selectFrom, selectTo } = built);
  } else if (md) {
    const built = md.build(body, values);
    const lead = md.block && before ? "\n".repeat(Math.max(0, 2 - (before.match(/\n*$/)?.[0].length ?? 0))) : "";
    const trail = md.block && after ? "\n".repeat(Math.max(0, 2 - (after.match(/^\n*/)?.[0].length ?? 0))) : "";
    text = lead + built.before + built.body + built.after + trail;
    selectFrom = lead.length + built.before.length;
    selectTo = selectFrom + built.body.length;
  } else throw new Error("Choose an element to insert.");
  return { from, to, insert: useBreak(text), selectFrom: from + useBreak(text.slice(0, selectFrom)).length, selectTo: from + useBreak(text.slice(0, selectTo)).length };
}
