// The lenses on screen, as ribbon sources.
//
// A lens is a document view over something the workspace does not hold as a
// focused document — the agent pane's session, today. The ribbon overlay
// draws from lines of any editor it is handed, so a lens registers its
// editor, its text and the links its elements declared, and the workspace
// hands them on with everything else it draws. See
// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.

import type { EditorView } from "@codemirror/view";

import type { SessionLink } from "../api/types";
import type { RibbonLink } from "../shell/Ribbons";

export interface LensSource {
  /** The file the lens shows, folder-relative. */
  path: string;
  view: EditorView;
  source: string;
  links: RibbonLink[];
}

/** A session's declared links as the overlay draws them: from lines of the
 * session file to the path each element named. */
export function ribbonLinksOf(path: string, links: readonly SessionLink[]): RibbonLink[] {
  return links.map((link, i) => ({
    key: `lens:${path}:${i}`,
    family: link.family,
    from: { path, lines: link.lines },
    to: {
      path: link.to.path,
      lines: link.to.lines,
      kind: link.to.path.endsWith(".md") ? "document" : "file",
    },
    title: link.title,
  }));
}

const lenses = new Map<string, LensSource>();
const listeners = new Set<() => void>();
let version = 0;

function changed() {
  version += 1;
  for (const listener of listeners) listener();
}

export function registerLens(lens: LensSource): () => void {
  lenses.set(lens.path, lens);
  changed();
  return () => {
    if (lenses.get(lens.path) === lens) {
      lenses.delete(lens.path);
      changed();
    }
  };
}

export function lensSources(): LensSource[] {
  return [...lenses.values()];
}

export function lensVersion(): number {
  return version;
}

export function onLensChange(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
