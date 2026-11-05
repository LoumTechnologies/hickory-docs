<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/mdLinks.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1645f056db0c"></a>
## `apps/web/src/editor/mdLinks.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Image Dropped In Becomes A File And A Reference](../../../../../guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference/contract.md), [The Gutters Never Skip A Number](../../../../../guarantees/authoring/the-gutters-never-skip-a-number/contract.md).

<a id="f-1645f056db0c-FOLLOW_KEY"></a>
### `FOLLOW_KEY`

Links show their labels while reading. Entering a link exposes its original
Markdown until the selection leaves it; decorations never rewrite the buffer.
Double-click and Ctrl/Cmd-click follow the destination. Parsed include tags use
the same navigation. Local context menus delegate to the Files menu owner.
Images retain their source and preview below the line.

<storycode:copy id="f-1645f056db0c-FOLLOW_KEY">
// Read links as labels, edit their Markdown in place, and follow on double-click.

import { request } from "../api/client";
import { editorLanguageActions } from "./languageActions";
import type { UiOutcome } from "../lib/uiActions";
import { RangeSetBuilder, StateField } from "@codemirror/state";
import type { EditorState, Extension, Range } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import { scanMarkdownProse } from "./hickDoc";
import { fencedCodeRanges } from "./markdownStyling";
import { findLinks, isLocalTarget, resolveTarget } from "../lib/mdLinks";
import type { MdLink } from "../lib/mdLinks";

/** The modifier that follows a link, named the way the platform names it. */
export const FOLLOW_KEY =
  typeof navigator !== "undefined" && /mac/i.test(navigator.platform ?? "")
    ? "\u2318"
    : "Ctrl";

</storycode:copy>
<a id="f-1645f056db0c-linkLabel"></a>
### `linkLabel`

<storycode:copy id="f-1645f056db0c-linkLabel">
const linkLabel = Decoration.mark({ class: "cm-md-link" });
</storycode:copy>
<a id="f-1645f056db0c-linkMark"></a>
### `linkMark`

<storycode:copy id="f-1645f056db0c-linkMark">
const linkMark = Decoration.mark({ class: "cm-md-mark cm-md-link-mark" });

</storycode:copy>
<a id="f-1645f056db0c-assetUrl"></a>
### `assetUrl`

The bytes route an image is read through. Same origin, so no base URL.

<storycode:copy id="f-1645f056db0c-assetUrl">
/** The bytes route an image is read through. Same origin, so no base URL. */
export function assetUrl(rootRelative: string): string {
  return `/api/asset?path=${encodeURIComponent(rootRelative)}`;
}

</storycode:copy>
<a id="f-1645f056db0c-ImageWidget"></a>
### `ImageWidget`

The picture under the line that references it.

<storycode:copy id="f-1645f056db0c-ImageWidget">
/** The picture under the line that references it. */
class ImageWidget extends WidgetType {
  constructor(
    readonly src: string,
    readonly alt: string,
    /** Root-relative path of the pictured file, for click-through. */
    readonly target: string | null,
    readonly onOpenSource?: (rootRelative: string) => boolean,
  ) {
    super();
  }

  // Two widgets for the same picture are the same widget: without this,
  // every keystroke elsewhere on the line rebuilds the <img> and the image
  // visibly reloads while you type.
  eq(other: ImageWidget): boolean {
    return other.src === this.src && other.alt === this.alt && other.target === this.target && other.onOpenSource === this.onOpenSource;
  }

  toDOM(): HTMLElement {
    const wrap = document.createElement("div");
    wrap.className = "cm-md-image";
    const img = document.createElement("img");
    img.src = this.src;
    img.alt = this.alt;
    // A missing picture says so in words. An <img> that fails silently leaves
    // a broken-icon glyph and no clue which file is not there.
    img.addEventListener("error", () => {
      wrap.classList.add("cm-md-image--missing");
      wrap.textContent = `${this.alt || "image"} — not found at ${decodeURIComponent(
        this.src.replace(/^.*path=/, ""),
      )}`;
    });
    // A picture this document GENERATES can answer "where did you come
    // from?": clicking it jumps to the block that writes it. The handler
    // says whether it knew the file; an outside image keeps a plain cursor.
    if (this.target && this.onOpenSource) {
      const target = this.target;
      const open = this.onOpenSource;
      img.addEventListener("click", () => {
        if (open(target)) return;
      });
      wrap.classList.add("cm-md-image--sourced");
      wrap.dataset.tip = "Click: go to the block that generates this image";
    }
    wrap.appendChild(img);
    return wrap;
  }

  // The picture is decoration over text the editor still owns; clicks in it
  // are not the editor's business.
  ignoreEvent(): boolean {
    return true;
  }
}

</storycode:copy>
<a id="f-1645f056db0c-MdLinksConfig"></a>
### `MdLinksConfig`

<storycode:copy id="f-1645f056db0c-MdLinksConfig">
export interface MdLinksConfig {
  /** The document being edited, root-relative. Decides what a relative
   * destination resolves to; null (an untitled buffer) resolves against the
   * folder root. */
  docPath: () => string | null;
  /** Parsed include tags share navigation, without guessing namespace syntax. */
  includes?: (state: EditorState) => readonly { from: number; to: number; target: string }[];
  /** Ranges to leave alone — a fenced block, a generated file's body. A
   * `[…](…)` inside a shell script is shell, not a link. */
  skip?: (state: EditorState) => readonly (readonly [number, number])[];
  /** Draw the pictures. Off for a pane where a wide image would fight the
   * layout (a split output, a diff). */
  images?: boolean;
  /** A click on a pictured file this document generates: jump to the block
   * that writes it. Return true when handled (the path was one of ours). */
  onOpenImageSource?: (rootRelative: string) => boolean;
}

</storycode:copy>
<a id="f-1645f056db0c-skipped"></a>
### `skipped`

Whether `pos` falls inside one of the sorted ranges.

<storycode:copy id="f-1645f056db0c-skipped">
/** Whether `pos` falls inside one of the sorted ranges. */
function skipped(pos: number, ranges: readonly (readonly [number, number])[]): boolean {
  for (const [from, to] of ranges) {
    if (from > pos) return false;
    if (pos < to) return true;
  }
  return false;
}

</storycode:copy>
<a id="f-1645f056db0c-linksIn"></a>
### `linksIn`

Every link in the buffer that is not inside a range this extension skips.

Scan `@path` references alongside Markdown links so double-click, modifier-click
and file context menus share one destination lookup. Keep the entire reference
visible: it has no separate label to collapse. Exclude email addresses, bare
mentions, escaped references, code fences, inline code and Markdown destinations;
these are literal examples or other syntax, not a second navigation target.

<storycode:copy id="f-1645f056db0c-linksIn">
/** Every link in the buffer that is not inside a range this extension skips. */
export function linksIn(
  state: EditorState,
  config: MdLinksConfig,
): { link: MdLink; from: number }[] {
  const text = state.doc.toString();
  const skips = config.skip?.(state) ?? [];
  const links = findLinks(text).filter(link => !skipped(link.from, skips));
  // Agent instruction files use @path references, optionally quoted for spaces.
  // Require a filename/path rather than treating @mentions as filesystem links.
  if (!text.includes("@")) return links.map(link => ({ link, from: link.from }));
  const excluded = [...skips, ...fencedCodeRanges(text)];
  const code = scanMarkdownProse(text, [], []).inline.filter(mark => mark.kind === "code");
  const reference = /(?:^|[\s(])@(?:include[ \t]+)?(?:"([^"\n]+)"|'([^'\n]+)'|([^\s<>()[\]`"',;!?]+))/gm;
  for (const match of text.matchAll(reference)) {
    const target = match[1] ?? match[2] ?? match[3];
    const from = match.index! + match[0].indexOf("@");
    const to = match.index! + match[0].length;
    if (text[from - 1] === "\\" || !/[/.\\]/.test(target) || !isLocalTarget(target) || target.includes(":") ||
        excluded.some(([start, end]) => from < end && to > start) ||
        code.some(mark => from >= mark.openFrom && from < mark.closeTo) ||
        links.some(link => from >= link.from && from < link.to)) continue;
    const targetFrom = match.index! + match[0].indexOf(target, match[0].indexOf("@") + 1);
    links.push({ from, to, textFrom: from, textTo: to, targetFrom,
      targetTo: targetFrom + target.length, text: text.slice(from, to), target, image: false });
  }
  return links.sort((a, b) => a.from - b.from).map(link => ({ link, from: link.from }));
}

</storycode:copy>
<a id="f-1645f056db0c-decorationsFor"></a>
### `decorationsFor`

Replace only punctuation and destination ranges while reading. Selection changes
rebuild decorations so leaving one link hides its syntax even on the same line.

<storycode:copy id="f-1645f056db0c-decorationsFor">
function decorationsFor(state: EditorState, config: MdLinksConfig): DecorationSet {
  const ranges: Range<Decoration>[] = [];
  const docPath = config.docPath();
  const seenLines = new Set<number>();
  for (const { link } of linksIn(state, config)) {
    // Keep a destination editable after entering its label, until the caret
    // leaves the whole link. A caret elsewhere on this line reveals nothing.
    const editing = link.image || state.selection.ranges.some(r => r.from <= link.to && r.to >= link.from);
    const punctuation = editing ? linkMark : Decoration.replace({});
    if (link.from < link.textFrom) ranges.push(punctuation.range(link.from, link.textFrom));
    if (link.textTo > link.textFrom) ranges.push(linkLabel.range(link.textFrom, link.textTo));
    if (link.textTo < link.to) ranges.push(punctuation.range(link.textTo, link.to));
    if (!config.images || !link.image) continue;
    const resolved = resolveTarget(docPath, link.target);
    const src = resolved
      ? assetUrl(resolved)
      : isLocalTarget(link.target)
        ? null
        : link.target;
    if (!src) continue;
    // One picture per line, drawn after it: two images on one line would
    // otherwise stack two block widgets at the same offset in an order
    // nobody chose.
    const line = state.doc.lineAt(link.from);
    if (seenLines.has(line.number)) continue;
    seenLines.add(line.number);
    ranges.push(
      Decoration.widget({
        widget: new ImageWidget(src, link.text, resolved, config.onOpenImageSource),
        side: 1,
        block: true,
      }).range(line.to),
    );
  }
  ranges.sort((a, b) => a.from - b.from || (a.value.startSide ?? 0) - (b.value.startSide ?? 0));
  const builder = new RangeSetBuilder<Decoration>();
  for (const range of ranges) builder.add(range.from, range.to, range.value);
  return builder.finish();
}

</storycode:copy>
<a id="f-1645f056db0c-followLink"></a>
### `followLink`

Follow `target` from the document at `docPath`.
A folder-relative destination goes through the window event the shell
already uses for File > Open File… (`hickory-open-path`, matched by
path suffix), so this module needs no route table and no props threaded
down from the workspace. A URL goes to the browser, which is where a link
to the web has always gone.

<storycode:copy id="f-1645f056db0c-followLink">
/**
 * Follow `target` from the document at `docPath`.
 *
 * A folder-relative destination goes through the window event the shell
 * already uses for File > Open File… (`hickory-open-path`, matched by
 * path suffix), so this module needs no route table and no props threaded
 * down from the workspace. A URL goes to the browser, which is where a link
 * to the web has always gone.
 */
export function followLink(docPath: string | null, target: string): UiOutcome {
  if (!isLocalTarget(target)) {
    if (/^(https?|mailto|tel):/i.test(target)) {
      void request("POST", "/api/open-external", { url: target }).catch(error => {
        window.dispatchEvent(new CustomEvent("hickory-link-error", { detail: String(error) }));
      });
      return { status: "completed", value: { navigation_requested: true, tab_handle_returned: false, target, scope: "Default browser request only; no page-load witness" } };
    }
    return { status: "unavailable", reason: "This link has no supported navigation target" };
  }
  const resolved = resolveTarget(docPath, target);
  if (!resolved) return { status: "unavailable", reason: "This link cannot resolve against the current document" };
  window.dispatchEvent(
    new CustomEvent<string>("hickory-open-path", { detail: resolved }),
  );
  return { status: "completed", value: { navigation_requested: true, path: resolved, scope: "Shell open request only; no loaded-document witness" } };
}

</storycode:copy>
<a id="f-1645f056db0c-linkAt"></a>
### `linkAt`

<storycode:copy id="f-1645f056db0c-linkAt">
/** The link whose text covers `pos`, or null. */
export function linkAt(
  state: EditorState,
  config: MdLinksConfig,
  pos: number,
): MdLink | null {
  for (const { link } of linksIn(state, config)) {
    if (pos >= link.from && pos <= link.to) return link;
  }
  for (const include of config.includes?.(state) ?? []) {
    if (pos >= include.from && pos <= include.to) return {
      ...include, image: false, text: include.target,
      textFrom: include.from, textTo: include.to, targetFrom: include.from, targetTo: include.to,
    };
  }
  return null;
}

</storycode:copy>
<a id="f-1645f056db0c-mdLinks"></a>
### `mdLinks`

The first mousedown remembers the destination before selection exposes syntax.
Double-click reuses it only for the same nearby gesture; ordinary clicks remain
editor selection. Includes arrive from the language parser rather than a second
tag regex. Context-menu events carry destinations to the shared Files menu owner.

Links styled, followed on Mod-click, and images drawn.

<storycode:copy id="f-1645f056db0c-mdLinks">
/** Labels for reading, Markdown for editing, double-click navigation and images. */
export function mdLinks(config: MdLinksConfig): Extension {
  const position = { type: "number" as const, integer: true, minimum: 0, maximum: 100_000_000 };
  // First click expands the source and can move text under the second click.
  // Retain its destination for the duration of this double-click gesture.
  let pressed: { target: string; x: number; y: number; time: number } | null = null;
  return [
    editorLanguageActions.of({
      id: "markdown-links", label: "Read markdown link page",
      parameters: { first: position, count: { ...position, minimum: 1, maximum: 64 } },
      enabled: () => true, revision: () => config.docPath(),
      execute: async (view, args) => {
        const links = linksIn(view.state, config), first = args.first as number, count = args.count as number;
        if (first > links.length) return { status: "unavailable", reason: "Link page is outside current source" };
        const page = links.slice(first, first + count).map(({ link }) => ({ ...link }));
        if (new TextEncoder().encode(JSON.stringify(page)).length > 8192) return { status: "unavailable", reason: "Link page exceeds 8192 UTF-8 bytes; read the bounded editor source instead" };
        return { status: "completed", value: { links: page, total: links.length, next: first + count < links.length ? first + count : null, coordinate_unit: "UTF-16 code units" } };
      },
    }),
    editorLanguageActions.of({
      id: "follow-markdown-link", label: "Follow markdown link",
      parameters: { position }, enabled: () => true, revision: () => config.docPath(),
      execute: async (view, args) => {
        const link = linkAt(view.state, config, args.position as number);
        return link ? followLink(config.docPath(), link.target) : { status: "unavailable", reason: "There is no navigable markdown link at this current source position" };
      },
    }),
    ...(config.images && config.onOpenImageSource ? [editorLanguageActions.of({
      id: "markdown-image-source", label: "Open generated markdown image source",
      parameters: { position }, enabled: () => true, revision: () => config.docPath(),
      execute: async (view, args) => {
        const link = linkAt(view.state, config, args.position as number);
        const target = link?.image ? resolveTarget(config.docPath(), link.target) : null;
        if (!target || !config.onOpenImageSource?.(target)) return { status: "unavailable", reason: "This image has no generating source block in the current document" };
        return { status: "completed", value: { source_selection_requested: true, target, source_written: false } };
      },
    })] : []),
    StateField.define<{ path: string | null; decorations: DecorationSet }>({
      create: (state) => ({ path: config.docPath(), decorations: decorationsFor(state, config) }),
      // Renaming the owning document can change asset resolution without edits.
      update: (value, tr) => tr.docChanged || tr.selection || value.path !== config.docPath() ? { path: config.docPath(), decorations: decorationsFor(tr.state, config) } : value,
      provide: (f) => EditorView.decorations.from(f, value => value.decorations),
    }),
    EditorView.domEventHandlers({
      dblclick(event, view) {
        if (event.button !== 0) return false;
        const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
        const target = pressed && event.timeStamp - pressed.time < 1000 && Math.abs(event.clientX - pressed.x) < 5 && Math.abs(event.clientY - pressed.y) < 5
          ? pressed.target : pos === null ? null : linkAt(view.state, config, pos)?.target;
        pressed = null;
        if (!target) return false;
        event.preventDefault();
        followLink(config.docPath(), target);
        return true;
      },
      contextmenu(event, view) {
        const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
        const link = pos === null ? null : linkAt(view.state, config, pos);
        if (!link) return false;
        event.preventDefault();
        window.dispatchEvent(new CustomEvent("hickory-link-menu", { detail: {
          path: resolveTarget(config.docPath(), link.target), target: link.target,
          x: event.clientX, y: event.clientY,
        } }));
        return true;
      },
      mousedown(event, view) {
        if (event.button !== 0) return false;
        if (event.detail < 2) {
          const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
          const link = pos === null ? null : linkAt(view.state, config, pos);
          pressed = link ? { target: link.target, x: event.clientX, y: event.clientY, time: event.timeStamp } : null;
        }
        if (!(event.metaKey || event.ctrlKey)) return false;
        const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
        if (pos === null) return false;
        const link = linkAt(view.state, config, pos);
        if (!link) return false;
        event.preventDefault();
        followLink(config.docPath(), link.target);
        return true;
      },
    }),
  ];
}
</storycode:copy>

### Assemble `apps/web/src/editor/mdLinks.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/mdLinks.ts"><storycode:paste select="#f-1645f056db0c-FOLLOW_KEY"/><storycode:paste select="#f-1645f056db0c-linkLabel"/><storycode:paste select="#f-1645f056db0c-linkMark"/><storycode:paste select="#f-1645f056db0c-assetUrl"/><storycode:paste select="#f-1645f056db0c-ImageWidget"/><storycode:paste select="#f-1645f056db0c-MdLinksConfig"/><storycode:paste select="#f-1645f056db0c-skipped"/><storycode:paste select="#f-1645f056db0c-linksIn"/><storycode:paste select="#f-1645f056db0c-decorationsFor"/><storycode:paste select="#f-1645f056db0c-followLink"/><storycode:paste select="#f-1645f056db0c-linkAt"/><storycode:paste select="#f-1645f056db0c-mdLinks"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-419d07f37b80"></a>
## `apps/web/src/editor/mdLinks.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [An Image Dropped In Becomes A File And A Reference](../../../../../guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference/contract.md).

<a id="f-419d07f37b80-editorWith"></a>
### `editorWith`

<storycode:copy id="f-419d07f37b80-editorWith">
import { afterEach, describe, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { assetUrl, followLink, linkAt, linksIn, mdLinks } from "./mdLinks";
import { installMockHandler } from "../api/client";

function editorWith(doc: string, options: Parameters<typeof mdLinks>[0]) {
  return new EditorView({
    state: EditorState.create({ doc, extensions: [mdLinks(options)] }),
    parent: document.body,
  });
}

</storycode:copy>
<a id="f-419d07f37b80-inNotes"></a>
### `inNotes`

<storycode:copy id="f-419d07f37b80-inNotes">
const inNotes = { docPath: () => "notes/a.md" };

</storycode:copy>
<a id="f-419d07f37b80-expression_statement"></a>
### `expression_statement`

The display witness moves the caret into and out of the same link, checking both
the rendered label and unchanged source. Navigation assertions inspect the shell
or browser request boundary; only the Python browser witness demonstrates a
loaded destination document.

<storycode:copy id="f-419d07f37b80-expression_statement">
afterEach(() => {
  document.body.innerHTML = "";
});

</storycode:copy>
<a id="f-419d07f37b80-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-419d07f37b80-expression_statement-2">
describe("what counts as a link", () => {
  // Contract: .hickory-source/guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md
  it("recognizes agent @file includes without making code or mentions navigable", () => {
    const text = '@AGENTS.md\n@.instructions/just.md\nRead @"notes/my plan.md"\n@include notes/other.md\nmail@example.com @someone `@literal.md`\n```md\n@fenced.md\n```\n[an @label.md](target.md)';
    const state = EditorState.create({ doc: text });
    expect(linksIn(state, inNotes).map(({ link }) => link.target)).toEqual([
      'AGENTS.md', '.instructions/just.md', 'notes/my plan.md', 'notes/other.md', 'target.md',
    ]);
    expect(linkAt(state, inNotes, text.indexOf('@.instructions') + 3)?.target).toBe('.instructions/just.md');
    expect(linksIn(state, { ...inNotes, skip: () => [[0, text.length]] })).toEqual([]);
    const view = editorWith('@AGENTS.md', inNotes);
    expect(view.contentDOM.querySelector('.cm-md-link')?.textContent).toBe('@AGENTS.md');
    view.destroy();
  });
  it("skips the ranges the caller says are not prose", () => {
    const doc = "prose [a](a.md)\ncode [b](b.md)";
    const state = EditorState.create({ doc });
    const all = linksIn(state, inNotes);
    expect(all.map((l) => l.link.target)).toEqual(["a.md", "b.md"]);
    const skipped = linksIn(state, { ...inNotes, skip: () => [[18, doc.length]] });
    expect(skipped.map((l) => l.link.target)).toEqual(["a.md"]);
  });

  it("finds the link under a position, brackets included", () => {
    const state = EditorState.create({ doc: "see [the plan](plan.md) now" });
    expect(linkAt(state, inNotes, 6)?.target).toBe("plan.md");
    expect(linkAt(state, inNotes, 27)).toBeNull();
  });
});

</storycode:copy>
<a id="f-419d07f37b80-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-419d07f37b80-expression_statement-3">
describe("how a link looks", () => {
  it("shows only the label until the caret enters that link", () => {
    const view = editorWith("see [the plan](plan.md)", inNotes);
    expect(view.state.doc.toString()).toBe("see [the plan](plan.md)");
    expect(view.contentDOM.querySelector(".cm-md-link")?.textContent).toBe("the plan");
    expect(view.contentDOM.textContent).toBe("see the plan");
    view.dispatch({ selection: { anchor: 8 } });
    expect(view.contentDOM.textContent).toBe("see [the plan](plan.md)");
    expect(view.contentDOM.querySelectorAll(".cm-md-link-mark").length).toBe(2);
    view.dispatch({ selection: { anchor: 0 } });
    expect(view.contentDOM.textContent).toBe("see the plan");
    view.destroy();
  });

  it("draws an image under its line, through the asset route", () => {
    const view = editorWith("![a chart](assets/c.png)", { ...inNotes, images: true });
    const img = view.contentDOM.querySelector<HTMLImageElement>(".cm-md-image img");
    expect(img?.getAttribute("src")).toBe(assetUrl("notes/assets/c.png"));
    expect(img?.alt).toBe("a chart");
    // Never instead of: the markdown is still in the buffer and on screen.
    expect(view.state.doc.toString()).toBe("![a chart](assets/c.png)");
    view.destroy();
  });

  it("draws nothing for an image when pictures are off", () => {
    const view = editorWith("![a](assets/c.png)", inNotes);
    expect(view.contentDOM.querySelector(".cm-md-image")).toBeNull();
    view.destroy();
  });
});

</storycode:copy>
<a id="f-419d07f37b80-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-419d07f37b80-expression_statement-4">
describe("following a link", () => {
  it("opens parsed includes through the same destination lookup", () => {
    const state = EditorState.create({ doc: '<book:include file="other.md" />' });
    expect(linkAt(state, { ...inNotes, includes: () => [{ from: 0, to: state.doc.length, target: "other.md" }] }, 12)?.target).toBe("other.md");
    expect(linkAt(state, inNotes, 12)).toBeNull();
  });
  it("asks the shell to open a document in the folder", () => {
    const seen: string[] = [];
    const listener = (e: Event) => seen.push((e as CustomEvent<string>).detail);
    window.addEventListener("hickory-open-path", listener);
    followLink("notes/weekly/mon.md", "../plan.md");
    window.removeEventListener("hickory-open-path", listener);
    expect(seen).toEqual(["notes/plan.md"]);
  });

  it("sends a web address to the default OS browser", async () => {
    const calls: unknown[] = [];
    installMockHandler(async (...args) => { calls.push(args); return { ok: true }; });
    followLink("a.md", "https://example.com");
    expect(calls).toEqual([["POST", "/api/open-external", { url: "https://example.com" }]]);
  });

  it("does nothing for a destination with nowhere to go", () => {
    const open = vi.spyOn(window, "open").mockReturnValue(null);
    const seen: string[] = [];
    const listener = (e: Event) => seen.push((e as CustomEvent<string>).detail);
    window.addEventListener("hickory-open-path", listener);
    followLink("a.md", "#section");
    window.removeEventListener("hickory-open-path", listener);
    expect(open).not.toHaveBeenCalled();
    expect(seen).toEqual([]);
    open.mockRestore();
  });
});

</storycode:copy>
<a id="f-419d07f37b80-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-419d07f37b80-expression_statement-5">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("refreshes rendered image ownership when the document path changes without a source edit", () => {
  let path = "notes/a.md";
  const clicked: string[] = [];
  const view = editorWith("![chart](chart.png)", { docPath: () => path, images: true, onOpenImageSource: target => { clicked.push(target); return true; } });
  try {
    expect(view.contentDOM.querySelector("img")?.getAttribute("src")).toBe(assetUrl("notes/chart.png"));
    path = "renamed/a.md";
    view.dispatch({});
    expect(view.contentDOM.querySelector("img")?.getAttribute("src")).toBe(assetUrl("renamed/chart.png"));
    view.contentDOM.querySelector("img")?.click();
    expect(clicked).toEqual(["renamed/chart.png"]);
  } finally { view.destroy(); }
});
</storycode:copy>

### Assemble `apps/web/src/editor/mdLinks.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/mdLinks.test.ts"><storycode:paste select="#f-419d07f37b80-editorWith"/><storycode:paste select="#f-419d07f37b80-inNotes"/><storycode:paste select="#f-419d07f37b80-expression_statement"/><storycode:paste select="#f-419d07f37b80-expression_statement-2"/><storycode:paste select="#f-419d07f37b80-expression_statement-3"/><storycode:paste select="#f-419d07f37b80-expression_statement-4"/><storycode:paste select="#f-419d07f37b80-expression_statement-5"/></storycode:file>


</storycode:doc>
