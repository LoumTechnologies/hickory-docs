// Where a plain file's debug session lives: above its pane.
//
// Only the active tab of a pane is rendered, so a plain file's pane
// unmounts the moment another tab is fronted — and a session held in that
// pane's own state would die with it. Found in a real browser: stepping
// from `app.py` into `helpers.py` opened the callee's tab, which unmounted
// the caller's pane, which stopped the session it had just stepped. A
// document does not have this problem because its session lives in the
// workspace's registry; this is the same move for plain files.
//
// A HOST is a component with no DOM that owns one `useWorkspaceDebugger`
// per path and publishes the session it holds into a store the pane reads.
// Hosts are mounted by the workspace (`<PlainDebugHosts />`) for every path
// a pane has asked for, and are never unmounted while the workspace lives:
// each is one hook, and a session that outlives its tab is the point.
//
// The host also publishes "paused elsewhere" (lib/pausedElsewhere.ts) —
// the fact belongs to the session, not to whichever pane happens to be on
// screen.

import { useEffect, useSyncExternalStore } from "react";

import { openLocation } from "../lib/revealLine";
import { publishPausedElsewhere } from "../lib/pausedElsewhere";
import type { DebugSession } from "./useDebugger";
import { useWorkspaceDebugger } from "./useDebugger";

/** Whether a frame's source is a file in the folder the app opened. */
export function isWorkspaceSource(source: string | null | undefined): source is string {
  return !!source && !/^([A-Za-z]:)?[\\/]/.test(source);
}

// The paths that want a host, and the session each host currently holds.
let wanted: readonly string[] = [];
const sessions = new Map<string, DebugSession>();
const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) listener();
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Ask for a host for `path`. Idempotent; hosts are never removed. */
export function ensurePlainDebugHost(path: string): void {
  if (wanted.includes(path)) return;
  wanted = [...wanted, path];
  emit();
}

function wantedPaths(): readonly string[] {
  return wanted;
}

/** The session for `path`, or null until its host has mounted. */
export function usePlainDebugSession(path: string): DebugSession | null {
  useEffect(() => ensurePlainDebugHost(path), [path]);
  return useSyncExternalStore(
    subscribe,
    () => sessions.get(path) ?? null,
    () => null,
  );
}

/** Test seam. */
export function resetPlainDebugHosts(): void {
  wanted = [];
  sessions.clear();
  emit();
}

function Host({ path }: { path: string }) {
  const debug = useWorkspaceDebugger(path);
  useEffect(() => {
    sessions.set(path, debug);
    emit();
  }, [path, debug]);

  // Where this session is paused when that is another file of the folder:
  // say so, and open that file, the way stepping into a callee in another
  // file opens it in any IDE. Cleared the moment the program moves on or
  // the session ends; only this owner can clear what it published.
  useEffect(() => {
    const top = debug.frames[0];
    if (
      debug.status === "paused" &&
      top &&
      !top.in_document &&
      isWorkspaceSource(top.source) &&
      typeof top.source_line === "number"
    ) {
      publishPausedElsewhere(path, { path: top.source, line: top.source_line });
      openLocation(top.source, top.source_line + 1);
    } else {
      publishPausedElsewhere(path, null);
    }
  }, [debug.status, debug.frames, path]);
  return null;
}

/** Mount once, in the workspace: one host per path a pane has asked for. */
export function PlainDebugHosts() {
  const paths = useSyncExternalStore(subscribe, wantedPaths, wantedPaths);
  return (
    <>
      {paths.map((path) => (
        <Host key={path} path={path} />
      ))}
    </>
  );
}
