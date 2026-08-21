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
// Restoring is still "add, never arrange": the stored layout is applied only
// into an untouched workspace, the same rule a document's own declared layout
// follows. Once it is in, the route's `ensureDocOpen` runs on top and either
// activates the tab the restore already brought back or adds one.

import { useCallback, useEffect, useRef, useState } from "react";

import { api } from "../api/client";
import { isWorkspaceEmpty } from "./workspaceState";
import {
  emptyUi,
  normalizeUi,
  withTableLayout,
  withWrap,
  withZoom,
  worthStoring,
  wrapFor,
  zoomFor,
  type WorkspaceUi,
} from "../lib/uiState";
import type { Layout } from "../shell/layout";
import type { TableLayout } from "../components/TablePanel";

/** How long the arrangement must sit still before it is written down. */
export const UI_SAVE_DEBOUNCE_MS = 600;

export interface WorkspaceUiHandle {
  /** False until the stored state has been asked for and answered. The
   * route's opener must not touch the layout before this. */
  hydrated: boolean;
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
 * Load the stored window state once, then keep it up to date.
 *
 * `applyLayout` is handed the restored arrangement exactly once, and only
 * when there is one worth restoring; it is expected to install it only into
 * an untouched workspace.
 */
export function useWorkspaceUi(
  layout: Layout,
  applyLayout: (restored: Layout) => void,
): WorkspaceUiHandle {
  const [ui, setUi] = useState<WorkspaceUi>(emptyUi);
  const [hydrated, setHydrated] = useState(false);
  // The layout is read through a ref by the save timer, so a change to the
  // arrangement does not restart the timer on every keystroke that happens to
  // re-render this view.
  const layoutRef = useRef(layout);
  layoutRef.current = layout;

  useEffect(() => {
    let cancelled = false;
    void api.workspaceUi().then(
      (response) => {
        if (cancelled) return;
        const stored = normalizeUi(response.state);
        setUi(stored);
        if (stored.layout && isWorkspaceEmpty(layoutRef.current)) applyLayout(stored.layout);
        setHydrated(true);
      },
      () => {
        // No store, no home directory, a read-only disk: the window opens
        // with default tabs and says nothing. There is nothing here a person
        // could act on, and the route must not stay blocked.
        if (!cancelled) setHydrated(true);
      },
    );
    return () => {
      cancelled = true;
    };
    // Once per mount, deliberately: this is hydration, not synchronisation.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Write it down, debounced, once hydration has happened — saving before the
  // load returns would overwrite the stored layout with the default one.
  useEffect(() => {
    if (!hydrated) return;
    const timer = window.setTimeout(() => {
      const current = layoutRef.current;
      const state: WorkspaceUi = {
        ...ui,
        layout: worthStoring(current) ? current : null,
      };
      void api.saveWorkspaceUi(state).catch(() => {
        // Forgetting the arrangement is an annoyance, not something to
        // interrupt anyone over.
      });
    }, UI_SAVE_DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [hydrated, ui, layout]);

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
    wrapFor: useCallback((target: string) => wrapFor(ui, target), [ui]),
    setWrap,
    zoomFor: useCallback((target: string) => zoomFor(ui, target), [ui]),
    setZoom,
    tables: ui.tables,
    setTableLayout,
  };
}
