// A box in an architecture diagram is a way INTO the code it stands for.
//
// A scene node may carry two topology fields: `path` — the file or directory
// the node stands for — and `doc` — the `.hick` document ABOUT it. Both are
// spelled relative to the document that draws the diagram, exactly as a
// markdown link in the same document would be, so one resolver serves both
// (`lib/mdLinks.ts`'s `resolveTarget`) and a diagram and a link can never come
// to disagree about where a relative destination points.
//
// Opening goes through the window event the shell already answers for
// File > Open File… (`hickory-open-path`, matched by path suffix in
// WorkspaceView) rather than through props threaded down into a lazily loaded
// canvas. Whether the target turns out to be a document, a plain file, or a
// DIRECTORY is the workspace's decision, because the workspace is what holds
// the folder tree; this module only says where to go.

import { resolveTarget } from "../../lib/mdLinks";
import type { SceneNode } from "./scene";

/** The event the shell answers with "open that". */
export const OPEN_PATH_EVENT = "hickory-open-path";

/**
 * Where a gesture on this node leads, or undefined when it leads nowhere.
 *
 * The modifier is the second button on one gesture: a plain double-click goes
 * to the code (`path`), and a modifier-double-click goes to the document
 * about it (`doc`). A node with only one of the two answers both ways —
 * refusing to navigate because a key was held would be a puzzle, not a rule.
 */
export function nodeTarget(node: SceneNode, modifier: boolean): string | undefined {
  const path = node.path?.trim() || undefined;
  const doc = node.doc?.trim() || undefined;
  return (modifier ? (doc ?? path) : (path ?? doc)) ?? undefined;
}

/**
 * Ask the shell to open `target`, spelled relative to the document at
 * `docPath`. Returns whether anything was asked for — false when the target
 * resolves nowhere this app opens (an empty path, a URL, `..` off the root).
 */
export function openTarget(docPath: string | null, target: string): boolean {
  const resolved = resolveTarget(docPath, target);
  if (!resolved) return false;
  window.dispatchEvent(
    new CustomEvent<string>(OPEN_PATH_EVENT, { detail: `/${resolved}` }),
  );
  return true;
}

/** The whole gesture: what this node leads to, opened. */
export function openNode(
  node: SceneNode,
  docPath: string | null,
  modifier: boolean,
): boolean {
  const target = nodeTarget(node, modifier);
  return target === undefined ? false : openTarget(docPath, target);
}

/** Whether a mouse event is asking for the node's document rather than its
 * code. The same modifier that follows a markdown link. */
export function wantsDoc(event: { metaKey?: boolean; ctrlKey?: boolean } | undefined): boolean {
  return Boolean(event?.metaKey || event?.ctrlKey);
}
