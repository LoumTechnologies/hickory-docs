// The tree as dired: marks, and the keys that act on them.
//
// Emacs's dired is a list of files you MARK and then act on — `m` marks,
// `u` unmarks, `U` unmarks all, and `D`, `C` delete and copy
// the marks (or the row under point when nothing is marked). `+` makes a
// folder. Those are the keys here, verbatim, because a person who wants
// dired wants those keys and nobody else is hurt by them: they fire only while
// a tree row has focus. Rename is F2, the familiar editor gesture, because
// this tree is meant to feel like an editor.
//
// Pure: a key and the state in, an intent out. The pane owns the marks and
// runs the intents (docs/guarantees/authoring/the-tree-is-a-dired.md).

import { isAction } from "../lib/keymap";

export type DiredIntent =
  | { kind: "mark"; path: string }
  | { kind: "unmark"; path: string }
  | { kind: "unmark-all" }
  | { kind: "delete"; paths: string[] }
  | { kind: "rename"; paths: string[] }
  | { kind: "copy"; paths: string[] }
  | { kind: "move"; paths: string[] }
  | { kind: "mkdir"; dir: string }
  | { kind: "create"; dir: string };

export type DiredKey = {
  key: string;
  ctrlKey?: boolean;
  metaKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
};

/** The rows a verb acts on: the marks when the row is one of them, else
 * the row alone. */
export function targets(path: string, marked: ReadonlySet<string>): string[] {
  return marked.has(path) ? [...marked] : [path];
}

/** The directory a new entry goes in: the row when it is a directory, else
 * the row's parent (`""` is the root). */
export function containing(path: string, dir: boolean): string {
  if (dir) return path;
  const i = path.lastIndexOf("/");
  return i < 0 ? "" : path.slice(0, i);
}

/**
 * What a key on a focused row means, or null when it is not a dired key.
 * `path` is the focused row (`""` for the folder header), `dir` whether it
 * is a directory. The keys are the keymap's `tree.*` actions
 * (lib/keymap.ts), so a person can move them; Delete always deletes.
 */
export function diredIntent(
  event: DiredKey,
  path: string,
  dir: boolean,
  marked: ReadonlySet<string>,
): DiredIntent | null {
  const full = {
    key: event.key,
    ctrlKey: !!event.ctrlKey,
    metaKey: !!event.metaKey,
    altKey: !!event.altKey,
    shiftKey: !!event.shiftKey,
  };
  const is = (id: string) => isAction(full, id);
  const onRow = path !== "";
  const something = onRow || marked.size > 0;
  if (is("tree.mark")) return onRow ? { kind: "mark", path } : null;
  if (is("tree.unmark")) return onRow ? { kind: "unmark", path } : null;
  if (is("tree.unmarkAll")) return { kind: "unmark-all" };
  if (is("tree.delete") || (full.key === "Delete" && !full.ctrlKey && !full.metaKey && !full.altKey))
    return something ? { kind: "delete", paths: targets(path, marked) } : null;
  if (is("tree.rename")) return onRow ? { kind: "rename", paths: targets(path, marked) } : null;
  if (is("tree.copy")) return something ? { kind: "copy", paths: targets(path, marked) } : null;
  if (is("tree.move")) return something ? { kind: "move", paths: targets(path, marked) } : null;
  if (is("tree.newFolder")) return { kind: "mkdir", dir: containing(path, dir) };
  if (is("tree.newFile")) return { kind: "create", dir: containing(path, dir) };
  return null;
}

/** The marks after a mark/unmark intent; other intents leave them alone. */
export function nextMarks(marked: ReadonlySet<string>, intent: DiredIntent): ReadonlySet<string> {
  const next = new Set(marked);
  if (intent.kind === "mark") next.add(intent.path);
  else if (intent.kind === "unmark") next.delete(intent.path);
  else if (intent.kind === "unmark-all") next.clear();
  return next;
}

/** A mark toggled by Ctrl+click (Cmd on a Mac). */
export function toggleMark(marked: ReadonlySet<string>, path: string): ReadonlySet<string> {
  const next = new Set(marked);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  return next;
}
