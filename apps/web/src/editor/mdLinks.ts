// Markdown links and images, in the editor: styled, followed, and shown.
//
// Three jobs that are one extension because they share one scan:
//
//  - **A link looks like a link.** The label is coloured; the brackets and
//    the destination are dimmed the way every other markdown mark in this app
//    is (`cm-md-mark`, markdownStyling.ts). Nothing is hidden — the source is
//    still the source, which is the rule the rest of the markdown display
//    follows and the reason a `.hick` file stays a file you can edit in vim.
//  - **A link goes somewhere.** Mod-click follows it: a document opens as a
//    tab, any other file in the folder opens as a pane, and a URL goes to the
//    browser. Mod-click rather than click because this is an editor first —
//    a plain click has to be able to put the caret inside the label.
//  - **An image is the picture.** A `![…](…)` gets the picture drawn under
//    its line, read through `GET /api/asset`. Under, not instead of: the
//    markdown stays visible and editable, so there is never a state where the
//    text is hidden behind a widget you have to guess how to get back out of.
//
// ## Where a link points
//
// A relative destination resolves against the LINKING document's directory,
// which is what it means in every other markdown tool and what keeps a folder
// of notes movable. Resolution and the `.hick` → `.md` rewrite for the weave
// are both in lib/mdLinks.ts, shared with the weaver's own rule so the editor
// and the woven file cannot come to disagree about where a link goes.

import { RangeSetBuilder, StateField } from "@codemirror/state";
import type { EditorState, Extension, Range } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import { findLinks, isLocalTarget, resolveTarget } from "../lib/mdLinks";
import type { MdLink } from "../lib/mdLinks";

/** The modifier that follows a link, named the way the platform names it. */
export const FOLLOW_KEY =
  typeof navigator !== "undefined" && /mac/i.test(navigator.platform ?? "")
    ? "\u2318"
    : "Ctrl";

const linkLabel = Decoration.mark({ class: "cm-md-link" });
const linkMark = Decoration.mark({ class: "cm-md-mark cm-md-link-mark" });

/** The bytes route an image is read through. Same origin, so no base URL. */
export function assetUrl(rootRelative: string): string {
  return `/api/asset?path=${encodeURIComponent(rootRelative)}`;
}

/** The picture under the line that references it. */
class ImageWidget extends WidgetType {
  constructor(
    readonly src: string,
    readonly alt: string,
  ) {
    super();
  }

  // Two widgets for the same picture are the same widget: without this,
  // every keystroke elsewhere on the line rebuilds the <img> and the image
  // visibly reloads while you type.
  eq(other: ImageWidget): boolean {
    return other.src === this.src && other.alt === this.alt;
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
    wrap.appendChild(img);
    return wrap;
  }

  // The picture is decoration over text the editor still owns; clicks in it
  // are not the editor's business.
  ignoreEvent(): boolean {
    return true;
  }
}

export interface MdLinksConfig {
  /** The document being edited, root-relative. Decides what a relative
   * destination resolves to; null (an untitled buffer) resolves against the
   * folder root. */
  docPath: () => string | null;
  /** Ranges to leave alone — a fenced block, a generated file's body. A
   * `[…](…)` inside a shell script is shell, not a link. */
  skip?: (state: EditorState) => readonly (readonly [number, number])[];
  /** Draw the pictures. Off for a pane where a wide image would fight the
   * layout (a split output, a diff). */
  images?: boolean;
}

/** Whether `pos` falls inside one of the sorted ranges. */
function skipped(pos: number, ranges: readonly (readonly [number, number])[]): boolean {
  for (const [from, to] of ranges) {
    if (from > pos) return false;
    if (pos < to) return true;
  }
  return false;
}

/** Every link in the buffer that is not inside a range this extension skips. */
export function linksIn(
  state: EditorState,
  config: MdLinksConfig,
): { link: MdLink; from: number }[] {
  const text = state.doc.toString();
  const skips = config.skip?.(state) ?? [];
  return findLinks(text)
    .filter((link) => !skipped(link.from, skips))
    .map((link) => ({ link, from: link.from }));
}

function decorationsFor(state: EditorState, config: MdLinksConfig): DecorationSet {
  const ranges: Range<Decoration>[] = [];
  const docPath = config.docPath();
  const seenLines = new Set<number>();
  for (const { link } of linksIn(state, config)) {
    // `[` … `](` … `)` dimmed, the label lit. An image's `!` goes with the
    // punctuation it belongs to.
    ranges.push(linkMark.range(link.from, link.textFrom));
    if (link.textTo > link.textFrom) ranges.push(linkLabel.range(link.textFrom, link.textTo));
    ranges.push(linkMark.range(link.textTo, link.to));
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
        widget: new ImageWidget(src, link.text),
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

/**
 * Follow `target` from the document at `docPath`.
 *
 * A folder-relative destination goes through the window event the shell
 * already uses for File > Open File… (`hickory-open-path`, matched by
 * path suffix), so this module needs no route table and no props threaded
 * down from the workspace. A URL goes to the browser, which is where a link
 * to the web has always gone.
 */
export function followLink(docPath: string | null, target: string): void {
  if (!isLocalTarget(target)) {
    if (/^(https?|mailto|tel):/i.test(target)) {
      window.open(target, "_blank", "noopener,noreferrer");
    }
    return;
  }
  const resolved = resolveTarget(docPath, target);
  if (!resolved) return;
  window.dispatchEvent(
    new CustomEvent<string>("hickory-open-path", { detail: `/${resolved}` }),
  );
}

/** The link whose text covers `pos`, or null. */
export function linkAt(
  state: EditorState,
  config: MdLinksConfig,
  pos: number,
): MdLink | null {
  for (const { link } of linksIn(state, config)) {
    if (pos >= link.from && pos <= link.to) return link;
  }
  return null;
}

/** Links styled, followed on Mod-click, and images drawn. */
export function mdLinks(config: MdLinksConfig): Extension {
  return [
    StateField.define<DecorationSet>({
      create: (state) => decorationsFor(state, config),
      // The scan is over the whole buffer, so it runs on a document change
      // and nothing else: a selection move cannot change where a link is.
      update: (value, tr) => (tr.docChanged ? decorationsFor(tr.state, config) : value),
      provide: (f) => EditorView.decorations.from(f),
    }),
    EditorView.domEventHandlers({
      mousedown(event, view) {
        if (!(event.metaKey || event.ctrlKey) || event.button !== 0) return false;
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
