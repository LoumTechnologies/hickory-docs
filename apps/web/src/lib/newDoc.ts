// Where the app lands when it opens, and what a fresh document is made of.
//
// The app never lands on a chooser: an empty folder lands in a new untitled
// buffer, a folder with documents lands in the one touched last. The untitled
// buffer holds only prose — the real file is created on the first edit, named
// past whatever the folder already holds, and holding exactly what was typed.

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
 * The path the untitled document is created at: `untitled.md`, then
 * `untitled-2.md` and so on past whatever the folder already holds.
 *
 * Taken-ness is judged on the final path segment so a nested
 * `notes/untitled.md` still pushes the name along — two files a person can
 * only tell apart by directory is a worse outcome than skipping a number.
 */
export function untitledPath(existing: string[]): string {
  const taken = new Set(
    existing.map((p) => p.replace(/\\/g, "/").split("/").pop() ?? p),
  );
  for (let n = 1; ; n++) {
    const candidate = n === 1 ? "untitled.md" : `untitled-${n}.md`;
    if (!taken.has(candidate)) return candidate;
  }
}

/**
 * The created document: the typed prose, and nothing else.
 *
 * A new note is **bare** (`docs/specs/freeform/bare-documents.md`): the root
 * element is optional, the prefix defaults to `hick`, and `weave=` defaults to
 * the document's own name — so the envelope this used to add bought nothing
 * and cost the thing the spec exists for. Its motivating case *is* this one:
 * "the file is a thing you open on a phone in a meeting, and the first
 * impression of the format is the first line of the file." Typing `# Standup`
 * and getting three lines of XML around it is the impression that spec was
 * adopted to remove.
 *
 * A document that needs the wrapper still opts into it by typing it — that is
 * rule 1 working in the only direction that matters. Prose about hick itself,
 * which rebinds the prefix to `h:`, is the one kind that must.
 */
export function wrapUntitled(prose: string): string {
  return prose.endsWith("\n") ? prose : `${prose}\n`;
}
