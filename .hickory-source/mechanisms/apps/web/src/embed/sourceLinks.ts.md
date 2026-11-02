<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Follow the tutorial through its own prose links

We have removed the separate related-links panel. Make the links in the rendered
document do that job instead. Use the existing Markdown parser to locate prose
links, excluding Hickory literal blocks. Decorations hide link syntax and attach
a browser anchor to the label without altering source bytes. Published source
targets stay in the reader; other repository paths retain the exact commit on
GitHub. Refuse non-HTTP schemes.

<storycode:copy id="public-source-document-links">
import { StateField, type Extension, type Range } from "@codemirror/state";
import { Decoration, EditorView, type DecorationSet } from "@codemirror/view";
import { markdownLanguage } from "@codemirror/lang-markdown";
import { verbatimRanges } from "../editor/hickDoc";
import { structureOf } from "../editor/wysiwyg";

interface LinkIndex { revision: string; source_url?: string; documents: readonly { path: string }[] }

/** Internal prose links stay in the published tutorial; other files retain their commit. */
export function sourceHref(target: string, path: string, index: LinkIndex): string | null {
  try {
    const url = new URL(target, new URL(path, "https://source.invalid/"));
    if (!["http:", "https:"].includes(url.protocol)) return null;
    if (url.origin !== "https://source.invalid") return url.href;
    const destination = decodeURIComponent(url.pathname.slice(1));
    if (index.documents.some(doc => doc.path === destination)) return `?path=${encodeURIComponent(destination)}${url.hash}`;
    return index.source_url ? new URL(`${url.pathname.slice(1)}${url.hash}`, index.source_url).href : null;
  } catch { return null; }
}

/** Decorate prose only. The source string and its byte provenance remain unchanged. */
export function sourceLinks(path: string, index: LinkIndex): Extension {
  const field = StateField.define<DecorationSet>({
    create(state) {
      const source = state.doc.toString();
      const literal = verbatimRanges(structureOf(state).blocks);
      const ranges: Range<Decoration>[] = [];
      const tree = markdownLanguage.parser.parse(source);
      const codeRanges = [...literal];
      tree.iterate({ enter(node) {
        if (["FencedCode", "CodeBlock", "InlineCode"].includes(node.name)) codeRanges.push([node.from, node.to]);
      } });
      for (const anchor of source.matchAll(/^<a id="[^"\r\n]+"><\/a>\r?\n?/gm)) {
        const from = anchor.index, to = from + anchor[0].length;
        if (!codeRanges.some(([start, end]) => from < end && to > start)) ranges.push(Decoration.replace({}).range(from, to));
      }
      tree.iterate({ enter(node) {
        if (node.name !== "Link" || literal.some(([from, to]) => node.from < to && node.to > from)) return;
        const url = node.node.getChild("URL"), marks = node.node.getChildren("LinkMark");
        const open = marks.find(mark => source.slice(mark.from, mark.to) === "[");
        const close = marks.find(mark => source.slice(mark.from, mark.to) === "]");
        if (!url || !open || !close || close.from <= open.to) return;
        const href = sourceHref(source.slice(url.from, url.to), path, index);
        if (!href) return;
        ranges.push(Decoration.replace({}).range(node.from, open.to));
        ranges.push(Decoration.mark({ tagName: "a", class: "source-prose-link", attributes: { href } }).range(open.to, close.from));
        ranges.push(Decoration.replace({}).range(close.from, node.to));
      } });
      return Decoration.set(ranges, true);
    },
    update(value, transaction) { return transaction.docChanged ? this.create(transaction.state) : value; },
    provide: value => EditorView.decorations.from(value),
  });
  return field;
}
</storycode:copy>

<storycode:file path="apps/web/src/embed/sourceLinks.ts"><storycode:paste select="#public-source-document-links"/></storycode:file>
</storycode:doc>
