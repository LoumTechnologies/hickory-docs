<authoring:doc xmlns:authoring="http://www.hickorydocs.com/1.0">
# Insert and edit document elements

Follow the authoring flow from the current source through the form to the editor transaction. Browser, KDE/Linux and Windows share the browser editor. macOS, iOS and Android mount the same form beside their native editor.

## Read exact element ranges and plan source edits

Read Hickory ranges with the same parser used by the engine. Mask those tags and verbatim payloads before walking Markdown so the forms address prose rather than embedded program text. A form keeps a concrete target and computes one UTF-16 edit for the host to validate and apply.

<authoring:copy id="element-authoring-model">
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

</authoring:copy>

### Keep the original Hickory spelling

Refill fields from the parsed tag. An attribute edit changes the selected field while preserving unknown attributes, bare flags and an untouched body.

<authoring:copy id="element-authoring-hick-reading">
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

</authoring:copy>

### Keep delimiters around Markdown edits

Use the Markdown tree to identify the body and any editable fields. Retain the authored spelling of the surrounding markers and titles. Widen a code fence when its edited body needs a longer delimiter.

<authoring:copy id="element-authoring-markdown-reading">
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

</authoring:copy>

### Distinguish prose from embedded source

Mask literal tags and their verbatim payloads without moving offsets. Map the remaining prose and the tags themselves back to the original source. Executable Markdown fences retain their fence spelling.

<authoring:copy id="element-authoring-gutter-reading">
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

</authoring:copy>

### Preview the exact transaction

Fill from the selection or a starter body. Reuse Hickory placement rules and the document namespace. Return the replacement and the starter body selection together in authored UTF-16 coordinates.

<authoring:copy id="element-authoring-plan">
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
</authoring:copy>

<authoring:file path="apps/web/src/components/ElementAuthoringModel.ts"><authoring:paste select="#element-authoring-model"/><authoring:paste select="#element-authoring-hick-reading"/><authoring:paste select="#element-authoring-markdown-reading"/><authoring:paste select="#element-authoring-gutter-reading"/><authoring:paste select="#element-authoring-plan"/></authoring:file>

## Share the searchable menu and form

Use the same fields for insertion and revision. Show the exact replacement text before applying it. The native bridge exposes line numbers for gutter placement and returns a proposed edit, leaving final authority with the mounted editor.

<authoring:copy id="element-authoring-form">
import { useLayoutEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { loadHickLang } from "../editor/hickLang";
import {
  elementOptions, newElement, planElement, readElements,
  type ElementOption, type ElementPlan, type ElementReading, type ElementTarget,
} from "./ElementAuthoringModel";
import "./ElementAuthoring.css";

type FormProps = {
  reading: ElementReading;
  range: { from: number; to: number };
  targets?: ElementTarget[];
  close: () => void;
  apply: (plan: ElementPlan) => string | undefined;
};
const unavailable = "The document parser could not load. Reopen the document to retry.";

export function ElementAuthoring({ reading, range, targets, close, apply }: FormProps) {
  const [target, setTarget] = useState(targets?.length === 1 ? targets[0] : undefined);
  const [option, setOption] = useState<ElementOption | undefined>(target?.option);
  const [query, setQuery] = useState("");
  const [failure, setFailure] = useState("");
  const editing = !!targets;
  const title = editing ? "Edit element" : "Insert element";
  const results = elementOptions.filter(item => query.trim().toLowerCase().split(/\s+/)
    .every(word => [item.title, item.id, item.group, item.summary].join(" ").toLowerCase().includes(word)));
  const matches = targets?.map(item => item.option) ?? results;
  const origin = target ?? range;
  const selected = reading.source.slice(origin.from, origin.to);

  return <section className="element-authoring" aria-label={title}>
    <header><strong>{title}</strong><button type="button" aria-label="Close element form" onClick={close}>×</button></header>
    {option ? <>
      {(targets?.length !== 1) && <button className="element-back" type="button" onClick={() => { setTarget(undefined); setOption(undefined); setFailure(""); }}>← Choose an element</button>}
      <ElementFields key={option.id + ":" + origin.from} option={option} target={target} selected={selected}
        plan={(values, body) => planElement(reading, origin, option, values, body, target)}
        apply={plan => setFailure(apply(plan) ?? "")} />
    </> : <>
      {!editing && <input onKeyDown={event => {
        if (event.key === "Enter" && !event.nativeEvent.isComposing && results.length) { event.preventDefault(); setOption(results[0]); }
      }} className="element-search" aria-label="Find an element" placeholder="Find an element" value={query} onChange={e => setQuery(e.target.value)} autoFocus />}
      <div className="element-options">
        {matches.map((item, index) => <button type="button" key={item.id + ":" + index} onClick={() => {
          setTarget(targets?.[index]); setOption(item);
        }}>
          <span>{item.title}</span>
          <small>{editing ? reading.source.slice(targets![index].from, targets![index].to).split(/\r?\n/)[0] : item.group}</small>
        </button>)}
        {!matches.length && <p>No matching elements.</p>}
      </div>
    </>}
    {failure && <p role="alert">{failure}</p>}
  </section>;
}

</authoring:copy>

### Preview each form before confirmation

Use the same controls for both actions. Recompute the source preview locally as the author changes values. A failed plan shows a field or placement error while the draft stays available.

<authoring:copy id="element-authoring-fields">
function ElementFields({ option, target, selected, plan, apply }: {
  option: ElementOption; target?: ElementTarget; selected: string;
  plan: (values: Record<string, string>, body: string) => ElementPlan;
  apply: (plan: ElementPlan) => void;
}) {
  const [initial] = useState(() => target ?? newElement(option, selected));
  const [values, setValues] = useState(initial.values);
  const [body, setBody] = useState(initial.body);
  const [bodyEdited, setBodyEdited] = useState(false);
  const first = useRef<HTMLElement | null>(null);
  useLayoutEffect(() => first.current?.focus(), []);
  let preview: ElementPlan | undefined;
  let error = "";
  try { preview = plan(values, body); }
  catch (failure) { error = failure instanceof Error ? failure.message : String(failure); }
  const changed = preview?.insert !== selected;
  const update = (name: string, value: string) => {
    const next = { ...values, [name]: value };
    setValues(next);
    if (!target && !bodyEdited) setBody(newElement(option, selected, next).body);
  };
  return <form onSubmit={event => { event.preventDefault(); if (preview && changed) apply(preview); }}>
    <h2>{option.title}</h2>
    {option.summary && <p>{option.summary}</p>}
    {option.belongsIn && <p>Place inside <code>{option.belongsIn}</code>.</p>}
    {option.fields.map((field, index) => {
      const value = values[field.name] ?? "";
      const multiline = option.group !== "Markdown" &&
        (/[\r\n]/.test(initial.values[field.name] ?? "") || ["sources", "exclude"].includes(field.name));
      return <label key={field.name}>
        <span>{field.label}{option.group !== "Markdown" && <> <code>({field.name})</code></>}</span>
        {field.choices ? <select value={value} onChange={event => update(field.name, event.target.value)}
          ref={node => { if (!index) first.current = node; }}>
          {!field.choices.includes(value) && <option value={value}>{value || "Default"}</option>}
          {field.choices.map(choice => <option key={choice} value={choice}>{choice || "Default"}</option>)}
        </select> : multiline ? <textarea value={value} placeholder={field.placeholder} ref={node => { if (!index) first.current = node; }}
          onChange={event => update(field.name, event.target.value)} /> :
          <input value={value} placeholder={field.placeholder} ref={node => { if (!index) first.current = node; }}
            onChange={event => update(field.name, event.target.value)} />}
        {field.hint && <small>{field.hint}</small>}
      </label>;
    })}
    {option.bodyLabel && <label>{option.bodyLabel}
      <textarea value={body} spellCheck={false} ref={node => { if (!option.fields.length) first.current = node; }}
        onChange={event => { setBodyEdited(true); setBody(event.target.value); }} />
    </label>}
    {error && <p role="status">{error}</p>}
    {preview && <div><strong>Source preview</strong><pre aria-label="Source preview">{preview.insert}</pre></div>}
    <button className="element-apply" type="submit" disabled={!preview || !changed}>{target ? "Apply edit" : "Insert"}</button>
  </form>;
}

export function ElementDialog({ anchor, ...props }: FormProps & { anchor?: HTMLElement }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const position = anchor?.getBoundingClientRect();
  useLayoutEffect(() => {
    const node = dialog.current!;
    node.showModal();
    return () => node.close();
  }, []);
  return <dialog className="element-dialog" ref={dialog} aria-label={props.targets ? "Edit element" : "Insert element"}
    style={position ? { margin: 0, right: Math.max(8, Math.min(window.innerWidth - Math.min(360, window.innerWidth - 24) - 8, window.innerWidth - position.left + 6)), left: "auto", top: Math.max(8, Math.min(position.top, window.innerHeight - 490)) } : undefined}
    onCancel={event => { event.preventDefault(); props.close(); }}
    onClick={event => { if (event.target === event.currentTarget) props.close(); }}>
    <ElementAuthoring {...props} />
  </dialog>;
}

</authoring:copy>

### Bind native messages to their source and request

Send the gutter only the parsed line numbers and their revision. Apply and Cancel carry the individual request id so a previous form cannot act on a later one.

<authoring:copy id="element-authoring-native-protocol">
type NativeOpen = { id: string; revision: string; from: number; to: number; line?: number };
type NativeMessage =
  | { kind: "ready" }
  | { kind: "cancel"; id: string }
  | { kind: "lines"; revision: string; lines: number[] }
  | { kind: "apply"; id: string; plan: ElementPlan }
  | { kind: "error"; message: string };
export function mountNativeElements(container: HTMLElement, send: (message: NativeMessage) => void) {
  const root = createRoot(container);
  let reading: ElementReading | undefined, revision = "";
  const ready = loadHickLang();
  void ready.then(() => send({ kind: "ready" }), () => send({ kind: "error", message: unavailable }));
  return {
    async read(next: string, source: string) {
      revision = next;
      reading = undefined;
      await ready.catch(() => undefined);
      if (next !== revision) return;
      try {
        reading = readElements(source);
        send({ kind: "lines", revision, lines: [...elementsByLine(reading).keys()] });
      } catch {
        reading = undefined;
        send({ kind: "error", message: unavailable });
      }
    },
    open(request: NativeOpen) {
      if (!reading || revision !== request.revision) {
        send({ kind: "error", message: "The element reading changed. Open the form again." });
        return;
      }
      const targets = request.line === undefined ? undefined : elementsByLine(reading).get(request.line);
      if (request.line !== undefined && !targets?.length) {
        send({ kind: "error", message: "That line changed. Choose its current edit icon." });
        return;
      }
      root.render(<ElementAuthoring key={request.id} reading={reading} range={request} targets={targets} close={() => { root.render(null); send({ kind: "cancel", id: request.id }); }}
        apply={plan => { send({ kind: "apply", id: request.id, plan }); return undefined; }} />);
    },
    close() { root.render(null); },
  };
}

export function elementsByLine(reading: ElementReading): Map<number, ElementTarget[]> {
  const lines = new Map<number, ElementTarget[]>();
  const breaks = reading.source.matchAll(/\r\n?|\n/g);
  let line = 0, next = breaks.next().value;
  for (const target of reading.targets) {
    while (next && next.index + next[0].length <= target.from) { line++; next = breaks.next().value; }
    const items = lines.get(line) ?? [];
    items.push(target);
    lines.set(line, items);
  }
  return lines;
}
</authoring:copy>

<authoring:file path="apps/web/src/components/ElementAuthoring.tsx"><authoring:paste select="#element-authoring-form"/><authoring:paste select="#element-authoring-fields"/><authoring:paste select="#element-authoring-native-protocol"/></authoring:file>

## Check exact source and form behavior

Use mixed line endings, non-default namespaces and opaque attributes to test what the source edit preserves. Exercise the same form and revision-bound bridge used by the native hosts. The [browser witness](scripts/test-simple-browser.py.md) drives the menu and gutter on a live workspace with `python3 scripts/test-simple-browser.py --element-authoring-only`.

The focused browser run confirmed insertion and gutter edits, two isolated
Undo steps, unchanged editor bounds and unchanged on-disk CRLF bytes. The
local form bundle loaded and returned an edit in Chromium and WebKit. The
macOS, iOS simulator and Android products built with the shared assets.
Native device interaction and the KDE and Windows host launches remain
unverified in this run.

<authoring:copy id="element-authoring-witnesses">
// Contract: .hickory-source/frontends/contract.md
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ElementAuthoring, elementsByLine, mountNativeElements } from "./ElementAuthoring";
import { elementOptions, newElement, planElement, readElements } from "./ElementAuthoringModel";

const option = (name: string) => elementOptions.find(item => item.id === name)!;
function target(source: string, id: string) {
  const reading = readElements(source);
  const item = reading.targets.find(item => item.option.id === id)!;
  expect(item, id).toBeDefined();
  return { reading, item };
}

describe("element authoring plans", () => {
  it("writes into the current namespace and selects the starter body through CRLF", () => {
    const source = '<x:doc xmlns:x="http://www.hickorydocs.com/1.0">\r\n\r\n😀 selected\r\n\r\n</x:doc>';
    const from = source.indexOf("😀"), to = from + "😀 selected".length;
    const reading = readElements(source), copy = option("copy");
    const start = newElement(copy, source.slice(from, to));
    const plan = planElement(reading, { from, to }, copy, { id: "keep" }, start.body);
    expect(plan.insert).toBe('<x:copy id="keep">\r\n😀 selected\r\n</x:copy>');
    expect(plan.insert.slice(plan.selectFrom - from, plan.selectTo - from)).toBe("😀 selected");
  });

  it("preserves unknown Hickory attributes, quote style and mixed body bytes on attribute edits", () => {
    const source = "😀\r\n<hick:copy  id = 'before' future='a id=\"b\"' intact>\r\n α\n β\r\n</hick:copy>";
    const { reading, item } = target(source, "copy");
    expect(item.from).toBe(source.indexOf("<"));
    const plan = planElement(reading, item, item.option, { id: "after" }, item.body, item);
    expect(plan.insert).toBe(source.slice(item.from).replace("id = 'before'", 'id="after"'));
  });

  it("keeps Markdown syntax inside Hickory file payloads out of the gutter", () => {
    const source = '<hick:file path="README.md">\n# Generated\n**example**\n</hick:file>\n\n# Actual **prose**\n\n```md\n# Sample\n```';
    const reading = readElements(source);
    expect([...elementsByLine(reading)].map(([line, items]) => [line, items.map(item => item.option.id)])).toEqual([
      [0, ["file"]], [5, ["md:heading", "md:bold"]], [7, ["md:fence"]],
    ]);
    expect(() => planElement(reading, { from: 12, to: 12 }, option("md:bold"), {}, "Example")).toThrow("outside the tag");
  });

  it("keeps executable fence spelling when editing its body", () => {
    const source = '```bash exec="true"\nprintf hi\n```';
    const { reading, item } = target(source, "md:fence");
    expect(reading.structure.blocks.some(block => block.name === "exec")).toBe(true);
    expect(item.render(item.values, "printf hello")).toBe(source.replace("printf hi", "printf hello"));
  });

  it("edits the level and content of a heading without replacing its closing markers", () => {
    const source = "# Before ##\r\nNext";
    const { reading, item } = target(source, "md:heading");
    const plan = planElement(reading, item, item.option, { level: "3" }, "After", item);
    expect(plan.insert).toBe("### After ##");
    expect(source.slice(plan.to)).toBe("\r\nNext");
  });

  it("keeps link titles and widens a fence when the edited body contains its delimiter", () => {
    const link = '[Title](<a path> "tip")';
    const { item } = target(link, "md:link");
    expect(item.render({ destination: "https://example.com" }, "Next")).toBe('[Next](https://example.com "tip")');
    const fence = target("```ts\n```", "md:fence").item;
    expect(fence.render({ info: "ts" }, "x\n```")).toBe("````ts\nx\n```\n````");
  });

  it("distinguishes a project entry route and makes new construction forms available", () => {
    const { item } = target('<hick:project entry="../project.md" />', "project-route");
    expect(item.render({ entry: "project.md" }, "")).toBe('<hick:project entry="project.md" />');
    for (const id of ["project", "project-route", "entry", "patch", "scope"]) expect(option(id)).toBeDefined();
  });

  it("uses Markdown alternatives with inline selections and validates required fields", () => {
    const source = "Before text after";
    const reading = readElements(source);
    const range = { from: 7, to: 11 };
    const bold = planElement(reading, range, option("md:bold"), {}, "text");
    expect(bold).toEqual({ ...range, insert: "**text**", selectFrom: 9, selectTo: 13 });
    expect(() => planElement(reading, range, option("md:link"), { destination: "" }, "text")).toThrow("destination");
    expect(planElement(reading, range, option("allow-network"), { network: "*" }, "").insert).toContain('<hick:allow network="*" />');
  });
});

it("shows the exact preview and applies only after submitting the form", () => {
  const apply = vi.fn();
  render(<ElementAuthoring reading={readElements("Title")} range={{ from: 0, to: 5 }} close={() => {}} apply={apply} />);
  fireEvent.change(screen.getByRole("textbox", { name: "Find an element" }), { target: { value: "heading" } });
  fireEvent.click(screen.getByRole("button", { name: "Heading Markdown" }));
  fireEvent.change(screen.getByRole("combobox", { name: "Level" }), { target: { value: "2" } });
  expect(screen.getByLabelText("Source preview").textContent).toBe("## Title");
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Insert" }));
  expect(apply).toHaveBeenCalledWith({ from: 0, to: 5, insert: "## Title", selectFrom: 3, selectTo: 8 });
});

it("shows multiline project attributes without folding their source into a single input line", () => {
  const source = '<hick:project exclude="target/\r\nnode_modules/" />';
  const { reading, item } = target(source, "project");
  const result = render(<ElementAuthoring reading={reading} range={item} targets={[item]} close={() => {}} apply={() => undefined} />);
  const form = within(result.container);
  expect((form.getByRole("textbox", { name: /^Exclude/ }) as HTMLTextAreaElement).value).toBe("target/\nnode_modules/");
  expect(form.getByLabelText("Source preview").textContent).toBe(source);
  result.unmount();
});

it("binds native gutter readings and submissions to the document revision and request", async () => {
  const node = document.createElement("div");
  document.body.append(node);
  const send = vi.fn();
  const form = mountNativeElements(node, send);
  try {
    await form.read("one", "## Original");
    expect(send).toHaveBeenCalledWith({ kind: "lines", revision: "one", lines: [0] });
    await form.read("two", "## Current");
    form.open({ id: "old", revision: "one", from: 0, to: 0, line: 0 });
    expect(send).toHaveBeenCalledWith({ kind: "error", message: expect.stringContaining("changed") });
    act(() => form.open({ id: "opened", revision: "two", from: 0, to: 0, line: 0 }));
    const formView = within(node);
    const field = formView.getByRole("textbox", { name: "Text" });
    fireEvent.change(field, { target: { value: "Revised" } });
    expect(formView.getByLabelText("Source preview").textContent).toBe("## Revised");
    fireEvent.click(formView.getByRole("button", { name: "Apply edit" }));
    expect(send).toHaveBeenCalledWith({ kind: "apply", id: "opened", plan: { from: 0, to: 10, insert: "## Revised", selectFrom: 10, selectTo: 10 } });
  } finally {
    act(() => form.close());
    node.remove();
  }
});
</authoring:copy>

<authoring:file path="apps/web/src/components/ElementAuthoring.test.tsx"><authoring:paste select="#element-authoring-witnesses"/></authoring:file>

## Keep the controls floating

Keep the palette scrollable within a small floating surface. The editor retains its size and scroll position while the palette is open.

<authoring:copy id="element-authoring-style">
.element-dialog, .element-authoring {
  color: CanvasText;
  background: Canvas;
  font: 14px/1.4 system-ui, sans-serif;
}
.element-dialog {
  border: 1px solid GrayText;
  border-radius: 8px;
  padding: 0;
  width: min(360px, calc(100vw - 24px));
  max-height: calc(100dvh - 24px);
}
.element-dialog::backdrop { background: transparent; }
.element-authoring { box-sizing: border-box; padding: 12px; }
.element-authoring header { display: flex; align-items: center; justify-content: space-between; }
.element-authoring button, .element-authoring input, .element-authoring textarea, .element-authoring select {
  font: inherit;
  color: inherit;
  background: Canvas;
  border: 1px solid GrayText;
  border-radius: 4px;
  padding: 4px 6px;
}
.element-authoring button:not(:disabled) { cursor: pointer; }
.element-authoring button:disabled { opacity: .55; }
.element-authoring form, .element-authoring label, .element-options { display: grid; gap: 5px; }
.element-authoring form { gap: 10px; margin-top: 8px; }
.element-authoring h2, .element-authoring p { margin: 0; }
.element-authoring h2 { font-size: 16px; }
.element-authoring label small { color: GrayText; }
.element-authoring .element-back { margin-top: 8px; }
.element-authoring textarea { resize: vertical; min-height: 4em; font-family: ui-monospace, monospace; }
.element-authoring pre {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font: 12px/1.4 ui-monospace, monospace;
  background: color-mix(in srgb, CanvasText 6%, Canvas);
  padding: 8px;
  max-height: 12em;
  overflow: auto;
  margin: 4px 0 0;
}
.element-authoring .element-search { width: 100%; box-sizing: border-box; margin: 10px 0 6px; }
.element-options { max-height: min(330px, 60dvh); overflow: auto; }
.element-options button { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; text-align: left; }
.element-options small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.element-options span { flex-shrink: 0; }
.element-authoring .element-apply { justify-self: start; background: ButtonFace; }
</authoring:copy>

<authoring:file path="apps/web/src/components/ElementAuthoring.css"><authoring:paste select="#element-authoring-style"/></authoring:file>

## Apply through the browser source editor

Capture the originating buffer, source revision and editor intent before the palette moves focus. Apply one isolated undo transaction through the authored-text adapter, which converts UTF-16 positions without changing existing line separators.

<authoring:copy id="element-authoring-browser">
import { useEffect, useMemo, useState } from "react";
import { EditorView } from "@codemirror/view";
import { isolateHistory } from "@codemirror/commands";
import { ElementDialog, elementsByLine } from "../components/ElementAuthoring";
import { readElements, type ElementReading, type ElementTarget, type ElementPlan } from "../components/ElementAuthoringModel";
import { authoredChanges, authoredOffset, authoredSource, editorOffsetIn } from "./SourceText";
import { readSourceEditorIntent } from "./SourceSnippets";
import type { Buffer } from "./buffers";

const insertCommands = new WeakMap<Buffer, () => boolean>();
export function canAuthorElements(buffer: Buffer | undefined): buffer is Buffer {
  return !!buffer && !buffer.readonly && !buffer.saving && !buffer.resourceDeleted && !buffer.transient
    && (!!buffer.file.doc_id || /\.(md|markdown)$/i.test(buffer.file.path));
}
export function insertDocumentElement(buffer: Buffer) { return insertCommands.get(buffer)?.() ?? false; }

type Intent = {
  reading: ElementReading; range: { from: number; to: number }; targets?: ElementTarget[];
  anchor?: HTMLElement; current: () => boolean;
};

export function useElementAuthoring(buffer: Buffer, view: EditorView | null, parserReady: boolean, current?: () => boolean) {
  const [reading, setReading] = useState<ElementReading>();
  const [intent, setIntent] = useState<Intent>();
  const eligible = canAuthorElements(buffer), source = buffer.text;
  useEffect(() => {
    if (!eligible || !parserReady) return;
    const handle = window.setTimeout(() => {
      try { setReading(readElements(source)); }
      catch { setReading(undefined); }
    }, 70);
    return () => window.clearTimeout(handle);
  }, [buffer, source, eligible, parserReady]);
  const fresh = eligible && parserReady && reading?.source === source ? reading : undefined;
  const lines = useMemo(() => fresh ? elementsByLine(fresh) : new Map<number, ElementTarget[]>(), [fresh]);
  const [viewport, setViewport] = useState({ first: 0, last: 0 });
  useEffect(() => {
    if (!view || !eligible) return;
    let frame = 0;
    const measure = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const { from, to } = view.viewport;
        const doc = view.state.doc;
        const first = Math.max(0, doc.lineAt(from).number - 4);
        const last = Math.min(doc.lines - 1, doc.lineAt(to).number + 2);
        setViewport(previous => previous.first === first && previous.last === last ? previous : { first, last });
      });
    };
    measure();
    view.scrollDOM.addEventListener("scroll", measure, { passive: true });
    const resize = typeof ResizeObserver === "undefined" ? undefined : new ResizeObserver(measure);
    resize?.observe(view.scrollDOM);
    resize?.observe(view.contentDOM);
    return () => { cancelAnimationFrame(frame); resize?.disconnect(); view.scrollDOM.removeEventListener("scroll", measure); };
  }, [view, eligible, source]);
  const close = () => { setIntent(undefined); view?.focus(); };
  function open(captured: ElementReading | undefined, targets?: ElementTarget[], anchor?: HTMLElement) {
    if (!view || !view.dom.isConnected || view.composing || view.state.readOnly || !view.state.facet(EditorView.editable)
      || !canAuthorElements(buffer) || !parserReady || current?.() === false || authoredSource(view.state) !== buffer.text) return false;
    const selected = view.state.selection;
    if (!targets && selected.ranges.length !== 1) return false;
    const { from, to } = selected.main;
    const revision = buffer.sourceRevision, path = buffer.file.path, editorIntent = readSourceEditorIntent(view.state);
    let next;
    try { next = captured ?? readElements(buffer.text); }
    catch { return false; }
    if (next.source !== buffer.text) return false;
    const row = anchor?.closest<HTMLElement>(".cm-debug-rail__group") ?? anchor;
    setIntent({ reading: next, range: { from: authoredOffset(view.state, from), to: authoredOffset(view.state, to) }, targets, anchor: row,
      current: () => view.dom.isConnected && !view.composing && !view.state.readOnly && view.state.facet(EditorView.editable)
        && canAuthorElements(buffer) && current?.() !== false && buffer.file.path === path && buffer.sourceRevision === revision
        && buffer.text === next.source && authoredSource(view.state) === next.source && readSourceEditorIntent(view.state) === editorIntent,
    });
    return true;
  }
  useEffect(() => {
    if (!view) return;
    const command = () => open(fresh);
    insertCommands.set(buffer, command);
    const shortcut = (event: KeyboardEvent) => {
      if (!event.defaultPrevented && (event.metaKey || event.ctrlKey) && !event.altKey && !event.shiftKey
        && event.key.toLowerCase() === "i" && command()) event.preventDefault();
    };
    view.dom.addEventListener("keydown", shortcut);
    return () => { insertCommands.delete(buffer); view.dom.removeEventListener("keydown", shortcut); };
  });
  function apply(plan: ElementPlan) {
    if (!view || !intent?.current()) return "The editor changed. Close this form and open it again.";
    const source = intent.reading.source;
    const next = source.slice(0, plan.from) + plan.insert + source.slice(plan.to);
    view.dispatch({
      ...authoredChanges(view.state, [{ from: plan.from, to: plan.to, insert: plan.insert }]),
      selection: { anchor: editorOffsetIn(next, plan.selectFrom), head: editorOffsetIn(next, plan.selectTo) },
      annotations: isolateHistory.of("full"),
      userEvent: "input.element",
    });
    close();
  }
  return {
    actions: view && fresh ? Array.from({ length: viewport.last - viewport.first + 1 }, (_, index) => viewport.first + index).flatMap(line => {
      const targets = lines.get(line);
      if (!targets || line >= view.state.doc.lines) return [];
      return [{ at: view.state.doc.line(line + 1).from, label: `Edit ${targets.length === 1 ? targets[0].option.title : "elements"} on line ${line + 1}`,
        glyph: "✎", run: (anchor?: HTMLElement) => open(fresh, targets, anchor) }];
    }) : [],
    dialog: intent && <ElementDialog reading={intent.reading} range={intent.range} targets={intent.targets}
      anchor={intent.anchor} close={close} apply={apply} />,
  };
}
</authoring:copy>

<authoring:file path="apps/web/src/simple/ElementAuthoring.tsx"><authoring:paste select="#element-authoring-browser"/></authoring:file>

## Load the form from the app bundle

Both native hosts load this local page with no remote resources. The bridge treats document text as data. The compiled parser is embedded in the script so loading it requires no network or filesystem fetch.

<authoring:copy id="element-authoring-local-page">
<!doctype html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self' 'wasm-unsafe-eval' file:; style-src 'self' file:">
<title>Document elements</title>
<link rel="stylesheet" href="node_modules/element-authoring.css">
<script defer src="node_modules/element-authoring.js"></script>
</head>
<body><main id="elements"></main></body>
</html>
</authoring:copy>

<authoring:file path="apps/apple-simple/Sources/Resources/element-authoring.html"><authoring:paste select="#element-authoring-local-page"/></authoring:file>

## Bind the shared form to the Apple editor

Show the controls in an overlay while the native text editor retains its geometry. Accept a plan only from the local page and only while the original TextKit view, source revision, selection and permission remain current. Apply it through the same undo-aware edit path as other native actions.

<authoring:copy id="element-authoring-apple">
import SwiftUI
import WebKit

@MainActor extension DocumentBuffer {
    var canAuthorElements: Bool {
        !readOnly && !resourceDeleted && !saving &&
            (file.doc_id != nil || ["md", "markdown"].contains((file.path as NSString).pathExtension.lowercased()))
    }

    func closeElementForm() {
        elementForm = nil
        focusSource?()
    }

    func openElementForm(line: Int? = nil) {
        guard canAuthorElements, let editor = sourceEditorState?(), editor.attached, editor.editable,
              !editor.composing, line != nil || editor.selections == 1 else {
            error = "Finish typing and choose a single selection in an editable Markdown document to insert an element."
            return
        }
        error = ""
        if elementLines != nil { elementFormError = "" }
        elementForm = NativeElementIntent(buffer: self, editor: editor.identity, range: editor.selection, line: line)
    }
}

struct NativeElementIntent {
    let id = UUID()
    let revision: UUID
    let editor: UUID
    let selection: UUID
    let path: String
    let source: String
    let range: NSRange
    let line: Int?

    @MainActor init(buffer: DocumentBuffer, editor: UUID, range: NSRange, line: Int?) {
        revision = buffer.sourceEditIdentity
        selection = buffer.selectionIdentity
        path = buffer.file.path
        source = buffer.text
        self.editor = editor
        self.range = range
        self.line = line
    }

    @MainActor func current(_ buffer: DocumentBuffer) -> Bool {
        guard buffer.canAuthorElements, buffer.sourceEditIdentity == revision, buffer.selectionIdentity == selection,
              exactUTF8(buffer.file.path, path), exactUTF8(buffer.text, source),
              let state = buffer.sourceEditorState?(), state.identity == editor, state.attached,
              state.editable, !state.composing, exactUTF8(state.source, source) else { return false }
        return line != nil || (state.selections == 1 && state.selection == range)
    }
}

@MainActor final class NativeElementCoordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
    let buffer: DocumentBuffer
    weak var view: WKWebView?
    var asset: URL?
    private var ready = false
    private var sentRevision: UUID?
    private var loadedRevision: UUID?
    private var sentIntent: UUID?
    private var readTask: Task<Void, Never>?

    init(_ buffer: DocumentBuffer) { self.buffer = buffer }

    func update() {
        guard ready, let view else { return }
        let revision = buffer.sourceEditIdentity
        if sentRevision != revision {
            sentRevision = revision
            readTask?.cancel()
            readTask = Task { [weak view, weak buffer] in
                try? await Task.sleep(for: .milliseconds(70))
                guard !Task.isCancelled, let view, let buffer, buffer.sourceEditIdentity == revision else { return }
                view.callAsyncJavaScript("await globalThis.hickoryElements.read(revision, source)",
                    arguments: ["revision": revision.uuidString, "source": buffer.text], in: nil, in: .page, completionHandler: nil)
            }
        }
        guard let intent = buffer.elementForm else {
            if sentIntent != nil { view.evaluateJavaScript("globalThis.hickoryElements.close()") }
            sentIntent = nil
            return
        }
        if !intent.current(buffer) {
            if sentIntent != nil { view.evaluateJavaScript("globalThis.hickoryElements.close()") }
            sentIntent = nil
            let message = "The editor changed. Close this form and open it again."
            if buffer.elementFormError != message { buffer.elementFormError = message }
            return
        }
        guard intent.revision == loadedRevision, sentIntent != intent.id else { return }
        sentIntent = intent.id
        var request: [String: Any] = ["id": intent.id.uuidString, "revision": revision.uuidString,
                                      "from": intent.range.location, "to": NSMaxRange(intent.range)]
        if let line = intent.line { request["line"] = line }
        view.callAsyncJavaScript("globalThis.hickoryElements.open(request)", arguments: ["request": request],
                                 in: nil, in: .page, completionHandler: nil)
    }

    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        guard message.frameInfo.isMainFrame, view != nil, let value = message.body as? [String: Any],
              let kind = value["kind"] as? String else { return }
        switch kind {
        case "ready":
            ready = true
            buffer.elementFormError = ""
            update()
        case "lines":
            let revision = buffer.sourceEditIdentity
            guard value["revision"] as? String == revision.uuidString, let lines = value["lines"] as? [Int] else { return }
            loadedRevision = revision
            buffer.elementLines = (revision, Set(lines))
            update()
        case "error":
            buffer.elementFormError = value["message"] as? String ?? "Close the form and reopen the document to retry."
        case "cancel":
            guard value["id"] as? String == buffer.elementForm?.id.uuidString else { return }
            buffer.closeElementForm()
        case "apply":
            guard let intent = buffer.elementForm, value["id"] as? String == intent.id.uuidString,
                  let plan = value["plan"] as? [String: Any] else { return }
            if apply(plan, intent: intent) { buffer.closeElementForm() }
            else { buffer.elementFormError = "The edit could not be applied. Close the form and try again in the current source." }
        default: break
        }
    }

    private func apply(_ plan: [String: Any], intent: NativeElementIntent) -> Bool {
        guard intent.current(buffer), let from = plan["from"] as? Int, let to = plan["to"] as? Int,
              let text = plan["insert"] as? String, from >= 0, to >= from, to <= intent.source.utf16.count,
              let start = plan["selectFrom"] as? Int, let end = plan["selectTo"] as? Int,
              start >= from, end >= start, end <= from + text.utf16.count else { return false }
        let range = NSRange(location: from, length: to - from)
        guard intent.line != nil || range == intent.range else { return false }
        return buffer.applyElementDraft?(NativeSourceEdit(range: range, text: text), intent.source,
                                         NSRange(location: start, length: end - start)) == true
    }

    func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction,
                 decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        decisionHandler(action.targetFrame?.isMainFrame == true && action.request.url == asset ? .allow : .cancel)
    }

    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) { unavailable() }
    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) { unavailable() }
    func unavailable() {
        readTask?.cancel()
        ready = false
        buffer.elementLines = nil
        buffer.elementFormError = "Element forms are unavailable. Reopen the document to retry."
    }

    func makeView() -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.websiteDataStore = .nonPersistent()
        configuration.userContentController.add(self, name: "elements")
        let web = WKWebView(frame: .zero, configuration: configuration)
        web.navigationDelegate = self
        view = web
        let owner = Bundle(for: MathCoordinator.self)
        let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL, owner.resourceURL, owner.bundleURL,
                     URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
        if let folder = nativeMathResource(roots: roots)?.deletingLastPathComponent() {
            let file = folder.appendingPathComponent("element-authoring.html").standardizedFileURL.resolvingSymlinksInPath()
            if FileManager.default.isReadableFile(atPath: folder.appendingPathComponent("node_modules/element-authoring.js").path) {
                asset = file
                web.loadFileURL(file, allowingReadAccessTo: folder)
            }
        }
        if asset == nil { Task { unavailable() } }
        return web
    }

    func dismantle(_ view: WKWebView) {
        readTask?.cancel()
        self.view = nil
        view.stopLoading()
        view.navigationDelegate = nil
        view.configuration.userContentController.removeScriptMessageHandler(forName: "elements")
        buffer.elementLines = nil
        buffer.elementForm = nil
    }
}

struct NativeElementForm: View {
    var buffer: DocumentBuffer
    @State private var activated = false

    var body: some View {
        GeometryReader { geometry in
            let open = buffer.elementForm != nil
            let width = min(360, max(1, geometry.size.width - 20))
            let height = min(490, max(1, geometry.size.height - 16))
            let row = buffer.elementForm?.line.flatMap { line in buffer.visibleDebugLines.first { $0.line == line }?.y } ?? 0
            Group {
                if activated {
                    NativeElementWebView(buffer: buffer, revision: buffer.sourceEditIdentity, intent: buffer.elementForm?.id)
                } else { Color.clear }
            }
                .frame(width: open ? width : 1, height: open ? height : 1)
                .background(.background)
                .overlay {
                    if open, !buffer.elementFormError.isEmpty || buffer.elementLines?.revision != buffer.sourceEditIdentity {
                        VStack(alignment: .leading, spacing: 8) {
                            Text(buffer.elementFormError.isEmpty ? "Reading document elements…" : buffer.elementFormError).textSelection(.enabled)
                            Button("Close") { buffer.closeElementForm() }
                        }.padding().frame(maxWidth: .infinity, maxHeight: .infinity).background(.background)
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .overlay(RoundedRectangle(cornerRadius: 8).strokeBorder(.secondary.opacity(0.35)).allowsHitTesting(false))
                .opacity(open ? 1 : 0).allowsHitTesting(open).accessibilityHidden(!open)
                .position(x: geometry.size.width - width / 2 - 10,
                          y: min(max(8, row), max(8, geometry.size.height - height - 8)) + height / 2)
        }.task {
            try? await Task.sleep(for: .milliseconds(70))
            if !Task.isCancelled { activated = true }
        }
    }
}

#if os(macOS)
private struct NativeElementWebView: NSViewRepresentable {
    var buffer: DocumentBuffer
    var revision: UUID
    var intent: UUID?
    func makeCoordinator() -> NativeElementCoordinator { NativeElementCoordinator(buffer) }
    func makeNSView(context: Context) -> WKWebView { context.coordinator.makeView() }
    func updateNSView(_ view: WKWebView, context: Context) { Task { context.coordinator.update() } }
    static func dismantleNSView(_ view: WKWebView, coordinator: NativeElementCoordinator) { coordinator.dismantle(view) }
}
#else
private struct NativeElementWebView: UIViewRepresentable {
    var buffer: DocumentBuffer
    var revision: UUID
    var intent: UUID?
    func makeCoordinator() -> NativeElementCoordinator { NativeElementCoordinator(buffer) }
    func makeUIView(context: Context) -> WKWebView { context.coordinator.makeView() }
    func updateUIView(_ view: WKWebView, context: Context) { Task { context.coordinator.update() } }
    static func dismantleUIView(_ view: WKWebView, coordinator: NativeElementCoordinator) { coordinator.dismantle(view) }
}
#endif
</authoring:copy>

<authoring:file path="apps/apple-simple/Sources/ElementAuthoring.swift"><authoring:paste select="#element-authoring-apple"/></authoring:file>

## Bind the shared form to Compose

Read element lines from a local WebView while the Compose editor remains usable. Use the existing undo-aware source transaction for Apply, after checking the captured revision and writable buffer. Put edit icons through the same gutter overflow as other line actions.

<authoring:copy id="element-authoring-android">
package com.hickorydocs.simple

import android.annotation.SuppressLint
import android.webkit.JavascriptInterface
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.hideFromAccessibility
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import kotlinx.coroutines.delay
import org.json.JSONArray
import org.json.JSONObject
import java.util.UUID

internal val DocumentBuffer.canAuthorElements: Boolean
    get() = !readOnly && !saving && !resourceDeleted &&
        (file.docID != null || file.path.substringAfterLast('.').lowercase() in listOf("md", "markdown"))

internal fun DocumentBuffer.openElementForm(line: Int? = null, top: Float = 0f): Boolean {
    if (!canAuthorElements || state.composition != null || (line == null && multiRanges.size > 1)) return false
    elementForm = NativeElementIntent(text, file.path, sourceRevision, state.selection, line, top)
    return true
}

internal data class NativeElementIntent(
    val source: String, val path: String, val revision: Long, val selection: TextRange,
    val line: Int?, val top: Float, val id: String = UUID.randomUUID().toString(),
) {
    fun current(buffer: DocumentBuffer): Boolean = buffer.canAuthorElements && buffer.text == source &&
        buffer.file.path == path && buffer.sourceRevision == revision && buffer.state.composition == null &&
        (line != null || (buffer.state.selection == selection && buffer.multiRanges.size < 2))
}

private const val localPage = "file:///android_asset/element-authoring.html"
private class ElementFormController(val buffer: DocumentBuffer) {
    var view: WebView? = null
    var isReady by mutableStateOf(false)
    var usable by mutableStateOf(true)
    var error by mutableStateOf("")
    private var loadedRevision by mutableStateOf<Long?>(null)
    private var sentIntent: String? = null

    fun read(revision: Long, source: String) {
        val args = JSONArray().put(revision.toString()).put(source)
        view?.evaluateJavascript("globalThis.hickoryElements.read(...$args)", null)
    }

    fun present(intent: NativeElementIntent?) {
        val web = view ?: return
        if (!isReady) return
        if (intent == null || !intent.current(buffer)) {
            if (sentIntent != null) web.evaluateJavascript("globalThis.hickoryElements.close()", null)
            sentIntent = null
            if (intent != null) error = "The editor changed. Close this form and open it again."
            return
        }
        if (intent.revision != loadedRevision || sentIntent == intent.id) return
        sentIntent = intent.id
        val request = JSONObject().put("id", intent.id).put("revision", intent.revision.toString())
            .put("from", intent.selection.min).put("to", intent.selection.max)
        intent.line?.let { request.put("line", it) }
        web.evaluateJavascript("globalThis.hickoryElements.open($request)", null)
    }

    fun unavailable() {
        isReady = false
        buffer.elementLines = null
        error = "Element forms are unavailable. Reopen the document to retry."
    }

    fun close() {
        buffer.elementForm = null
        if (isReady) error = ""
        buffer.focusSource?.invoke()
    }

    @JavascriptInterface fun postMessage(raw: String) {
        val web = view ?: return
        web.post {
            if (view !== web) return@post
            val message = runCatching { JSONObject(raw) }.getOrNull() ?: return@post
            when (message.optString("kind")) {
                "ready" -> isReady = true
                "lines" -> {
                    val revision = message.optString("revision").toLongOrNull() ?: return@post
                    if (revision != buffer.sourceRevision) return@post
                    val rows = message.optJSONArray("lines") ?: return@post
                    buffer.elementLines = revision to (0 until rows.length()).map { rows.optInt(it, -1) }.filter { it >= 0 }.toSet()
                    loadedRevision = revision
                    present(buffer.elementForm)
                }
                "error" -> error = message.optString("message", "Close the form and reopen the document to retry.")
                "cancel" -> if (message.optString("id") == buffer.elementForm?.id) close()
                "apply" -> {
                    val intent = buffer.elementForm ?: return@post
                    if (message.optString("id") != intent.id) return@post
                    val plan = message.optJSONObject("plan") ?: return@post
                    if (apply(plan, intent)) close()
                    else error = "The edit could not be applied. Close the form and try again in the current source."
                }
            }
        }
    }

    private fun apply(plan: JSONObject, intent: NativeElementIntent): Boolean {
        if (!intent.current(buffer)) return false
        val from = plan.optInt("from", -1)
        val to = plan.optInt("to", -1)
        val text = plan.opt("insert") as? String ?: return false
        val start = plan.optInt("selectFrom", -1)
        val end = plan.optInt("selectTo", -1)
        if (from < 0 || to < from || to > intent.source.length || start < from || end < start || end > from + text.length) return false
        if (intent.line == null && (from != intent.selection.min || to != intent.selection.max)) return false
        return buffer.applySourceEdits(listOf(NativeSourceEdit(from, to, text)), intent.source, TextRange(start, end))
    }
}

@SuppressLint("SetJavaScriptEnabled")
@Composable internal fun NativeElementForm(buffer: DocumentBuffer) {
    val controller = remember(buffer) { ElementFormController(buffer) }
    var activated by remember(buffer) { mutableStateOf(false) }
    LaunchedEffect(controller) { delay(70); activated = true }
    DisposableEffect(controller) {
        onDispose { buffer.elementLines = null; buffer.elementForm = null }
    }
    val intent = buffer.elementForm
    val open = intent != null
    val revision = buffer.sourceRevision
    LaunchedEffect(revision, controller.isReady) {
        if (controller.isReady) {
            delay(70)
            controller.read(revision, buffer.text)
        }
    }
    LaunchedEffect(intent?.id, revision, controller.isReady) { controller.present(intent) }
    BackHandler(open) { controller.close() }
    BoxWithConstraints(Modifier.fillMaxSize(), contentAlignment = Alignment.TopEnd) {
        val width = (maxWidth - 20.dp).coerceIn(1.dp, 360.dp)
        val height = (maxHeight - 16.dp).coerceIn(1.dp, 490.dp)
        val top = (intent?.top ?: 0f).dp.coerceIn(8.dp, (maxHeight - height - 8.dp).coerceAtLeast(8.dp))
        Box(Modifier.offset(x = (-10).dp, y = top).size(if (open) width else 1.dp, if (open) height else 1.dp)
            .alpha(if (open) 1f else 0f)
            .then(if (open) Modifier.border(1.dp, MaterialTheme.colorScheme.outline) else Modifier.clearAndSetSemantics { hideFromAccessibility() })) {
            if (activated && controller.usable) AndroidView(modifier = Modifier.fillMaxSize(), factory = { context ->
                WebView(context).apply {
                    controller.view = this
                    settings.javaScriptEnabled = true
                    settings.allowFileAccess = false
                    settings.allowContentAccess = false
                    settings.blockNetworkLoads = true
                    addJavascriptInterface(controller, "HickoryElements")
                    webViewClient = object : WebViewClient() {
                        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest) = request.url.toString() != localPage
                        override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) { controller.unavailable() }
                        override fun onRenderProcessGone(view: WebView, detail: android.webkit.RenderProcessGoneDetail): Boolean {
                            controller.unavailable()
                            controller.usable = false
                            return true
                        }
                    }
                    loadUrl(localPage)
                }
            }, update = { web ->
                web.visibility = if (open) android.view.View.VISIBLE else android.view.View.INVISIBLE
            }, onRelease = { web ->
                controller.view = null
                web.removeJavascriptInterface("HickoryElements")
                web.stopLoading()
                web.destroy()
            })
            if (open && controller.error.isEmpty() && buffer.elementLines?.first != revision) {
                Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).padding(12.dp)) {
                    Text("Reading document elements…")
                    TextButton(onClick = controller::close) { Text("Cancel") }
                }
            }
            if (open && controller.error.isNotEmpty()) {
                Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).padding(12.dp)) {
                    Text(controller.error)
                    TextButton(onClick = controller::close) { Text("Close") }
                }
            }
        }
    }
}
</authoring:copy>

<authoring:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ElementAuthoring.kt"><authoring:paste select="#element-authoring-android"/></authoring:file>

</authoring:doc>
