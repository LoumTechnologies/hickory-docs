// Unsaved work recovered after an unexpected interruption.
// Explicit Discard removes recovery; Cancel leaves the buffer open.
//
// A draft is written to the user's own data directory (never the project —
// see api.md, "Workspace state and drafts") along with the BASE: the file's
// contents when that editing session began. The base is what makes coming
// back safe rather than merely possible, because the file may have moved on
// while the app was closed — a branch checked out, a `git pull`, an edit in
// another editor. With a base, that is a three-way merge that resolves every
// region only one side touched. Without one it would be a standoff.
//
// This module is the DECISION and the plumbing around it. The merge itself is
// lib/merge.ts, and the UI that asks about conflicts is
// components/MergeView.tsx.

import { useCallback, useEffect, useRef } from "react";

import { api } from "../api/client";
import type { WorkspaceDraft } from "../api/types";

type DiscardRequest = { path: string; waitUntil: (operation: Promise<unknown>) => void };
export async function discardRecoveryDraft(path: string) {
  const pending: Promise<unknown>[] = [];
  window.dispatchEvent(new CustomEvent<DiscardRequest>("hickory-draft-discarded", {
    detail: { path, waitUntil: operation => pending.push(operation) },
  }));
  if (pending.length) await Promise.all(pending);
  else await api.discardDraft(path);
}

/** What to do with a draft, given what the file says now. */
export type Disposition =
  /** The buffer and the file agree — there was nothing unsaved after all.
   * The draft is stale and should be discarded. */
  | { kind: "clean" }
  /** The file is exactly what the draft was taken from: nobody else touched
   * it. Put the draft in the buffer, still unsaved, and say nothing. */
  | { kind: "restore"; contents: string }
  /** The file moved on while we were away. Both sides have changes, and the
   * reader has to see them. */
  | { kind: "merge"; base: string; ours: string; theirs: string };

/**
 * What should happen to `draft` now that the file says `onDisk`.
 *
 * The order of these tests matters. "Clean" is checked first because a draft
 * whose contents match the file is not a conflict however far the file has
 * moved — somebody saved the same text from somewhere else, and there is
 * nothing to merge or to warn about.
 */
export function draftDisposition(
  draft: Pick<WorkspaceDraft, "contents" | "base">,
  onDisk: string,
): Disposition {
  if (draft.contents === onDisk) return { kind: "clean" };
  if (draft.base === onDisk) return { kind: "restore", contents: draft.contents };
  return { kind: "merge", base: draft.base, ours: draft.contents, theirs: onDisk };
}

/** How often a changed buffer is written down. */
export const DRAFT_POLL_MS = 2000;

/**
 * Keep a draft of one buffer.
 *
 * The buffer is READ through a callback rather than passed in as a prop, and
 * that is the whole design of this hook. An editor pane deliberately does not
 * re-render on every keystroke — the CodeMirror view owns the text — so there
 * is no prop that changes when the buffer does, and forcing one would put a
 * React render on the typing path to power a background save.
 *
 * So it polls, on a slow timer, and writes only when the text has actually
 * moved since the last write. Two things fall out of that which a
 * debounced-on-change version would not give:
 *
 *  - It covers a CRASH, not just a graceful close. A draft written only at
 *    shutdown is a draft that is not there after the one event most likely to
 *    lose work.
 *  - It costs nothing while nobody is typing: an unchanged buffer is a string
 *    comparison, not a request.
 *
 * `pagehide` flushes as well, because it is the last moment the page reliably
 * gets, and a poll due in a second's time would never fire.
 *
 * `contents === base` means nothing is unsaved, and the draft is DISCARDED
 * rather than written — otherwise every file ever opened would accumulate a
 * draft identical to itself, and every start would have to filter them out.
 */
export function useDraftKeeper({
  path,
  read,
  enabled = true,
}: {
  /** Project-relative path. Nothing is written without one. */
  path: string | null;
  /** The buffer as it stands, and what the file held when this session began. */
  read: () => { contents: string; base: string };
  enabled?: boolean;
}): () => void {
  const readRef = useRef(read);
  readRef.current = read;
  const activePath = useRef(path); activePath.current = path;
  // A buffer can become a real file immediately before its component
  // unmounts. Remember that transition outside React state, so the cleanup
  // below cannot race it by writing the just-saved buffer back as a draft.
  const discardedRef = useRef(false);
  const discardedContents = useRef<string | null>(null);

  useEffect(() => {
    if (!enabled || !path) return;
    discardedRef.current = false;
    // What we last put in the store, so an untouched buffer costs a string
    // comparison rather than a request.
    let written: string | null = null;
    let pending: Promise<unknown> = Promise.resolve();
    const onDiscarded = (event: Event) => {
      const request = (event as CustomEvent<DiscardRequest>).detail;
      if (request.path !== path) return;
      discardedRef.current = true; discardedContents.current = readRef.current().contents;
      pending = pending.catch(() => {}).then(() => api.discardDraft(path));
      request.waitUntil(pending);
    };

    const flush = () => {
      if (activePath.current !== path) return;
      const { contents, base } = readRef.current();
      if (discardedRef.current) {
        if (contents === discardedContents.current) return;
        discardedRef.current = false; written = null;
      }
      if (contents === base) {
        if (written !== null) {
          written = null;
          pending = pending.catch(() => {}).then(() => api.discardDraft(path)).catch(() => { written = contents; });
        }
        return;
      }
      if (contents === written) return;
      written = contents;
      pending = pending.catch(() => {}).then(() => api.saveDraft({ path, contents, base, saved_at: Date.now() })).catch(() => {
        // A draft that cannot be written must not interrupt typing. The route
        // already degrades loudly; here the honest response is to let the
        // editor keep working, and to try again on the next tick.
        written = null;
      });
    };

    const timer = window.setInterval(flush, DRAFT_POLL_MS);
    // `pagehide` rather than `beforeunload`: it fires for a tab being frozen
    // or discarded as well as closed, and it is the event WebKit delivers.
    window.addEventListener("pagehide", flush);
    window.addEventListener("hickory-draft-discarded", onDiscarded);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("pagehide", flush);
      window.removeEventListener("hickory-draft-discarded", onDiscarded);
      // Keep recovery on unmount unless an explicit Discard suppressed it.
      flush();
    };
  }, [path, enabled]);

  return useCallback(() => {
    if (!path) return;
    if (activePath.current === path) { discardedRef.current = true; discardedContents.current = readRef.current().contents; }
    void discardRecoveryDraft(path).catch(() => {});
  }, [path]);
}
