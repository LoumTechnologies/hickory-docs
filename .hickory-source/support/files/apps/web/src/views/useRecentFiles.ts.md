<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useRecentFiles.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ef55bc4c3ac520782d5f5e7f1a1711b96180fd584bf70af195b94d263b45e8c8`.

<book:entry path="apps/web/src/views/useRecentFiles.ts">
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
  return file;
}
</book:entry>

</book:doc>
