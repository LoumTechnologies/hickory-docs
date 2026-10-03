import { markdownLanguage } from "@codemirror/lang-markdown";
import { isolateHistory } from "@codemirror/commands";
import { Transaction, type ChangeSpec, type Extension } from "@codemirror/state";
import { ViewPlugin, type EditorView, type ViewUpdate } from "@codemirror/view";
import { loadUnwrapParagraphs } from "../lib/unwrapParagraphs";
import { parseHickDoc, verbatimRanges } from "./hickDoc";

/** Join Markdown soft breaks only in ordinary paragraphs. Hickory's own
 * parser supplies the byte-preserving payload and tag boundaries. */
export function paragraphUnwrapChanges(source: string): ChangeSpec[] {
  const structure = parseHickDoc(source);
  const excluded = [
    ...verbatimRanges(structure.blocks),
    ...structure.tags.map((tag) => [tag.from, tag.to] as [number, number]),
  ];
  // YAML is data, including folded/literal scalars that look like prose.
  const frontmatter = /^(?:\uFEFF)?---[ \t]*\n/.exec(source);
  if (frontmatter) {
    const close = /^(?:---|\.\.\.)[ \t]*(?:\n|$)/m.exec(source.slice(frontmatter[0].length));
    excluded.push([0, close ? frontmatter[0].length + close.index + close[0].length : source.length]);
  }
  // Blank masked regions keep offsets stable and make container tag lines
  // paragraph boundaries, allowing prose inside <hick:doc> to be parsed.
  const chars = source.split("");
  for (const [from, to] of excluded) {
    for (let i = from; i < to; i++) if (chars[i] !== "\n") chars[i] = " ";
  }
  const changes: ChangeSpec[] = [];
  markdownLanguage.parser.parse(chars.join("")).iterate({
    enter(node) {
      // Lists and quotes have structural line prefixes; leave those intact.
      if (node.name !== "Paragraph" || node.node.parent?.name !== "Document") return;
      if (excluded.some(([from, to]) => from < node.to && to > node.from)) return false;
      const paragraph = source.slice(node.from, node.to);
      // Multi-line inline code, links and HTML can depend on their bytes.
      let sensitive = false;
      node.node.toTree().iterate({
        enter(child) {
          if (["InlineCode", "Link", "Image", "HTMLTag"].includes(child.name) &&
              paragraph.slice(child.from, child.to).includes("\n")) sensitive = true;
        },
      });
      if (sensitive || paragraph.includes("$$")) return false;
      for (const match of paragraph.matchAll(/\n[ \t]*/g)) {
        const before = paragraph.slice(0, match.index);
        // Two trailing spaces or an odd run of backslashes is an explicit
        // Markdown hard break, and must keep its newline.
        const slashes = /\\+$/.exec(before)?.[0].length ?? 0;
        if (/ {2,}$/.test(before) || slashes % 2 === 1) continue;
        changes.push({ from: node.from + match.index, to: node.from + match.index + match[0].length, insert: " " });
      }
      return false;
    },
  });
  return changes;
}

/** Open/sync and paste reflow prose; typing Enter and Undo remain literal.
 * Dispatch after the update so CRDT writes and selection mapping use the
 * normal editor path, and the formatting is a separate undoable act. */
export function unwrapParagraphs(): Extension {
  return ViewPlugin.fromClass(class {
    private pending = false;
    private destroyed = false;
    private shouldRun = true;
    constructor(private view: EditorView) { this.schedule(); }
    update(update: ViewUpdate) {
      if (!update.docChanged) return;
      this.shouldRun = update.transactions.every((tr) =>
        tr.annotation(Transaction.userEvent) === undefined ||
        tr.isUserEvent("input.paste") || tr.isUserEvent("input.drop"),
      );
      if (this.shouldRun) this.schedule();
    }
    private schedule() {
      if (this.pending) return;
      this.pending = true;
      queueMicrotask(() => {
        this.pending = false;
        if (this.destroyed || !this.shouldRun || this.view.state.readOnly || !loadUnwrapParagraphs()) return;
        const changes = paragraphUnwrapChanges(this.view.state.doc.toString());
        if (changes.length) this.view.dispatch({
          changes,
          annotations: [Transaction.userEvent.of("input.unwrap"), isolateHistory.of("full")],
        });
      });
    }
    destroy() { this.destroyed = true; }
  });
}
