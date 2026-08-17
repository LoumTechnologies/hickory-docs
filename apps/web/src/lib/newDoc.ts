// Where the app lands when it opens, and what a fresh document is made of.
//
// The app never lands on a chooser: an empty folder lands in a new untitled
// buffer, a folder with documents lands in the one touched last. The untitled
// buffer holds only prose — the real file is created on the first edit, named
// past whatever the folder already holds, wrapped in the standard envelope.

import type { DocSummary } from "../api/types";

export type LandingTarget = { kind: "new" } | { kind: "doc"; id: string };

/** Which document to land in: the most recently updated one, or a fresh
 * untitled buffer when the folder has none. */
export function landingTarget(docs: DocSummary[]): LandingTarget {
  if (docs.length === 0) return { kind: "new" };
  let best = docs[0];
  for (const doc of docs) {
    if (stamp(doc.updated_at) > stamp(best.updated_at)) best = doc;
  }
  return { kind: "doc", id: best.id };
}

// An unparseable timestamp must not decide the landing by throwing; it simply
// never wins against a real one.
function stamp(iso: string): number {
  const t = Date.parse(iso);
  return Number.isNaN(t) ? Number.NEGATIVE_INFINITY : t;
}

/**
 * The path the untitled document is created at: `untitled.hick`, then
 * `untitled-2.hick` and so on past whatever the folder already holds.
 *
 * Taken-ness is judged on the final path segment so a nested
 * `notes/untitled.hick` still pushes the name along — two files a person can
 * only tell apart by directory is a worse outcome than skipping a number.
 */
export function untitledPath(existing: string[]): string {
  const taken = new Set(
    existing.map((p) => p.replace(/\\/g, "/").split("/").pop() ?? p),
  );
  for (let n = 1; ; n++) {
    const candidate = n === 1 ? "untitled.hick" : `untitled-${n}.hick`;
    if (!taken.has(candidate)) return candidate;
  }
}

/** The created document: the standard wrapper with the typed prose inside. */
export function wrapUntitled(prose: string): string {
  return `<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n${prose}\n</hick:doc>\n`;
}
