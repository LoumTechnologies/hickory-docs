<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/StartupNotice.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Editor Does Not Wait For Workspace Preparation](../../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md)

Captured SHA-256: `8a32b78e4ee54c56126364a2067d5b4994e140e1a5583cf2b4d832d22429a9ec`.

<book:entry path="apps/web/src/components/StartupNotice.tsx">
import { useSyncExternalStore } from "react";
import { startupNotice, subscribeStartup } from "../api/workspaceStartup";

export function StartupNotice() {
  const notice = useSyncExternalStore(subscribeStartup, startupNotice);
  return notice ? <div className="menu-notice" role="status">{notice}</div> : null;
}
</book:entry>

</book:doc>
