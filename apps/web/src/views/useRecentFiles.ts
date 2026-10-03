import { useEffect } from "react";
import { paneById, type Layout } from "../shell/layout";

/** Report file navigation to the desktop's machine-wide recent list.
 * Browser hosts have no such route; remembering never interrupts opening. */
export function useRecentFiles(layout: Layout): string | null {
  const pane = paneById(layout, layout.focus);
  const tab = pane?.tabs[pane.active];
  const file = tab && ["document", "generated", "file"].includes(tab.kind)
    ? tab.target
    : null;
  useEffect(() => {
    if (!file) return;
    void fetch("/api/recent-files", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path: file }),
    }).catch(() => {});
  }, [file]);
  return tab && tab.kind !== "tree" && tab.kind !== "tool" ? tab.target : null;
}
