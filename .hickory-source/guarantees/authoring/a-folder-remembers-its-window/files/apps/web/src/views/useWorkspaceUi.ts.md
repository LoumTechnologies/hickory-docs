<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useWorkspaceUi.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Folder Remembers Its Window](../../../../../contract.md)

Captured SHA-256: `2a3878108d11dd46171b627dda6bccac8cf42fa40093a0da00d357eebba46a31`.

<book:entry path="apps/web/src/views/useWorkspaceUi.ts">
// Hydrating the window from what it looked like last time, and writing that
// down again as it changes.
//
// The ordering here is the whole difficulty, and it is worth stating before
// the code. Two things want to arrange the workspace at launch:
//
//  1. The **stored layout**, which arrives over a fetch — so, late.
//  2. The **route** (`#/docs/<id>`), which is a request against the workspace
//     and runs on the first render — so, early.
//
// If they race, the route wins by arriving first, the workspace is no longer
// empty when the layout lands, and the restore is silently dropped. So the
// route's opener WAITS: `hydrated` gates it, and it is set whether the fetch
// succeeded, failed, or found nothing. A window that will not open because
// its layout store is unreachable would be a far worse bug than a window that
// opens with default tabs, so there is no path here that leaves `hydrated`
// false.
//
// Folder identity gates both loading and saving. A late response from an old
// folder is ignored; the caller preserves any editors already in use.

import { useCallback, useEffect, useRef, useState } from "react";

import { api } from "../api/client";
import {
  emptyUi,
  normalizeUi,
  withTableLayout,
  withWrap,
  withZoom,
  wrapFor,
  zoomFor,
  type WorkspaceUi,
} from "../lib/uiState";
import type { SpatialState } from "../shell/spatialLayout";
import type { Layout } from "../shell/layout";
import type { TableLayout } from "../components/TablePanel";

/** How long the arrangement must sit still before it is written down. */
export const UI_SAVE_DEBOUNCE_MS = 600;

export interface WorkspaceUiHandle {
  /** False until the stored state has been asked for and answered. The
   * route's opener must not touch the layout before this. */
  hydrated: boolean;
  restored: boolean;
  spatial: SpatialState | undefined;
  setSpatial: (state: SpatialState) => void;
  flush: () => Promise<void>;
  /** The prose measure for a tab, by its target path. */
  wrapFor: (target: string) => number;
  /** Record a measure the reader dragged on a ruler. */
  setWrap: (target: string, column: number) => void;
  /** How far one tab is zoomed, and a way to move it. The whole-window level
   * is not here — it belongs to the screen, not the project. */
  zoomFor: (target: string) => number;
  setZoom: (target: string, level: number) => void;
  /** How big every table was left, keyed by `tableKey` — the whole record,
   * because the editor names its own tables and a per-key getter would make
   * every table re-render whenever any of them moved. */
  tables: Record<string, TableLayout>;
  setTableLayout: (key: string, size: TableLayout) => void;
}

/**
 * Load each folder's stored window state, then keep it up to date.
 *
 * The caller decides which existing editors to carry into the arrangement.
 */
export function useWorkspaceUi(
  layout: Layout,
  applyLayout: (restored: Layout) => void,
  scope: string | null = "window",
): WorkspaceUiHandle {
  const [ui, setUi] = useState<WorkspaceUi>(emptyUi);
  const [loadedScope, setLoadedScope] = useState<string | null>(null);
  const [restored, setRestored] = useState(false);
  const hydrated = scope !== null && loadedScope === scope;
  const applyRef = useRef(applyLayout);
  applyRef.current = applyLayout;
  const uiRef = useRef(ui);
  uiRef.current = ui;
  const pending = useRef(Promise.resolve());
  const scopeRef = useRef(scope); scopeRef.current = scope;
  // The layout is read through a ref by the save timer, so a change to the
  // arrangement does not restart the timer on every keystroke that happens to
  // re-render this view.
  const layoutRef = useRef(layout);
  layoutRef.current = layout;

  useEffect(() => {
    if (scope === null) return;
    let cancelled = false;
    setRestored(false);
    void api.workspaceUi().then(
      (response) => {
        if (cancelled) return;
        const stored = normalizeUi(response.state);
        setUi(stored);
        if (stored.layout) applyRef.current(stored.layout);
        setRestored(stored.layout !== null);
        setLoadedScope(scope);
      },
      () => {
        // No store, no home directory, a read-only disk: the window opens
        // with default tabs and says nothing. There is nothing here a person
        // could act on, and the route must not stay blocked.
        if (!cancelled) { setUi(emptyUi()); setLoadedScope(scope); }
      },
    );
    return () => {
      cancelled = true;
    };
  }, [scope]);

  const flush = useCallback(async (keepalive = false) => {
    if (!hydrated) return;
    const state: WorkspaceUi = { ...uiRef.current, layout: layoutRef.current };
    pending.current = pending.current.then(() => {
      if (scopeRef.current === scope) return api.saveWorkspaceUi(state, keepalive);
    }).then(() => {}, () => {});
    await pending.current;
  }, [hydrated, scope]);

  useEffect(() => {
    const hide = () => { void flush(true); };
    window.addEventListener("pagehide", hide);
    return () => window.removeEventListener("pagehide", hide);
  }, [flush]);

  // Write it down, debounced, once hydration has happened — saving before the
  // load returns would overwrite the stored layout with the default one.
  useEffect(() => {
    if (!hydrated) return;
    const timer = window.setTimeout(() => { void flush(); }, UI_SAVE_DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [hydrated, ui, layout, flush]);

  const setWrap = useCallback((target: string, column: number) => {
    setUi((current) => withWrap(current, target, column));
  }, []);

  const setZoom = useCallback((target: string, level: number) => {
    setUi((current) => withZoom(current, target, level));
  }, []);

  const setTableLayout = useCallback((key: string, size: TableLayout) => {
    setUi((current) => withTableLayout(current, key, size));
  }, []);

  return {
    hydrated,
    restored,
    spatial: ui.spatial,
    setSpatial: useCallback((spatial: SpatialState) => {
      uiRef.current = { ...uiRef.current, spatial };
      setUi(uiRef.current);
    }, []),
    flush,
    wrapFor: useCallback((target: string) => wrapFor(ui, target), [ui]),
    setWrap,
    zoomFor: useCallback((target: string) => zoomFor(ui, target), [ui]),
    setZoom,
    tables: ui.tables,
    setTableLayout,
  };
}
</book:entry>

</book:doc>
