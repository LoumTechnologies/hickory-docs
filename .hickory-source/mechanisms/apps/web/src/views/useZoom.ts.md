<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useZoom.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-49e957f1c44e"></a>
## `apps/web/src/views/useZoom.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Blame Column Is Optional And Honest](../../../../../guarantees/lineage/the-blame-column-is-optional-and-honest/contract.md).

<a id="f-49e957f1c44e-ZoomHandle"></a>
### `ZoomHandle`

The zoom commands, wired to keys and to the native menu.
Scope is chosen by a modifier, and the choice follows what people already
have in their fingers: ⌘+ / ⌘- / ⌘0 zoom the WHOLE WINDOW, exactly as a
browser and every editor do; adding Alt zooms only the focused TAB. The
common request gets the common chord.
The whole-UI level is applied to the document immediately and persisted to
localStorage, because it belongs to this screen rather than to this project
(lib/zoom.ts). The per-tab level goes into the workspace state, because it
belongs to the tab and should come back with the arrangement it was set in.

<storycode:copy id="f-49e957f1c44e-ZoomHandle">
// The zoom commands, wired to keys and to the native menu.
//
// Scope is chosen by a modifier, and the choice follows what people already
// have in their fingers: ⌘+ / ⌘- / ⌘0 zoom the WHOLE WINDOW, exactly as a
// browser and every editor do; adding Alt zooms only the focused TAB. The
// common request gets the common chord.
//
// The whole-UI level is applied to the document immediately and persisted to
// localStorage, because it belongs to this screen rather than to this project
// (lib/zoom.ts). The per-tab level goes into the workspace state, because it
// belongs to the tab and should come back with the arrangement it was set in.

import { useCallback, useEffect, useState } from "react";

import { onMenuAction, type MenuAction } from "../lib/menuBridge";
import {
  applyUiZoom,
  applyZoom,
  loadUiZoom,
  saveUiZoom,
  zoomCommandFor,
  type ZoomScope,
} from "../lib/zoom";
import { toggleBlameShown } from "../lib/blamePref";

export interface ZoomHandle {
  /** The whole-window level, for anything that wants to show it. */
  uiZoom: number;
  /** The focused tab's level, and a way to move it. */
  zoomFor: (target: string) => number;
}

</storycode:copy>
<a id="f-49e957f1c44e-MENU_ZOOM"></a>
### `MENU_ZOOM`

The menu items that mean a zoom, and what each one does.

<storycode:copy id="f-49e957f1c44e-MENU_ZOOM">
/** The menu items that mean a zoom, and what each one does. */
const MENU_ZOOM: Partial<Record<MenuAction, { scope: ZoomScope; command: "in" | "out" | "reset" }>> = {
  "zoom-in": { scope: "ui", command: "in" },
  "zoom-out": { scope: "ui", command: "out" },
  "zoom-reset": { scope: "ui", command: "reset" },
  "zoom-tab-in": { scope: "tab", command: "in" },
  "zoom-tab-out": { scope: "tab", command: "out" },
  "zoom-tab-reset": { scope: "tab", command: "reset" },
};

</storycode:copy>
<a id="f-49e957f1c44e-useZoom"></a>
### `useZoom`

<storycode:copy id="f-49e957f1c44e-useZoom">
export function useZoom({
  focusedTarget,
  zoomOfTab,
  setTabZoom,
}: {
  /** Which tab a tab-scoped command acts on; null when none is focused. */
  focusedTarget: () => string | null;
  zoomOfTab: (target: string) => number;
  setTabZoom: (target: string, level: number) => void;
}): { uiZoom: number } {
  // Read once and applied on mount: a window that opens at 100% and jumps to
  // 150% a frame later is worse than one that never remembered.
  const [uiZoom, setUiZoom] = useState<number>(loadUiZoom);
  useEffect(() => {
    applyUiZoom(uiZoom);
    saveUiZoom(uiZoom);
  }, [uiZoom]);

  const run = useCallback(
    (scope: ZoomScope, command: "in" | "out" | "reset") => {
      if (scope === "ui") {
        setUiZoom((current) => applyZoom(current, command));
        return;
      }
      const target = focusedTarget();
      // No focused tab: a tab-scoped zoom has nothing to act on, and doing
      // the UI instead would be a surprise rather than a kindness.
      if (!target) return;
      setTabZoom(target, applyZoom(zoomOfTab(target), command));
    },
    [focusedTarget, zoomOfTab, setTabZoom],
  );

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const command = zoomCommandFor(event);
      if (!command) return;
      // Taken before the editor sees it: ⌘- inside CodeMirror is otherwise a
      // plain keystroke, and ⌘0 is a browser reset that would undo the
      // window's own level.
      event.preventDefault();
      run(event.altKey ? "tab" : "ui", command);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [run]);

  useEffect(
    () =>
      onMenuAction((action) => {
        const zoom = MENU_ZOOM[action];
        if (zoom) run(zoom.scope, zoom.command);
        // The blame column rides this hook because it is the same kind of
        // thing: a View-menu switch that every open editor obeys at once.
        else if (action === "blame") toggleBlameShown();
      }),
    [run],
  );

  return { uiZoom };
}
</storycode:copy>

### Assemble `apps/web/src/views/useZoom.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useZoom.ts"><storycode:paste select="#f-49e957f1c44e-ZoomHandle"/><storycode:paste select="#f-49e957f1c44e-MENU_ZOOM"/><storycode:paste select="#f-49e957f1c44e-useZoom"/></storycode:file>

</storycode:doc>
