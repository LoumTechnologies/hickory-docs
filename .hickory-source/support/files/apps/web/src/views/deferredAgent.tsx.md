<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/deferredAgent.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Editor Does Not Wait For Workspace Preparation](../../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md)

Captured SHA-256: `35593db4434075155ffd5ce9e7611a72e136347d28d0a6107d102bbdcdb82aa5`.

<book:entry path="apps/web/src/views/deferredAgent.tsx">
// Agent panes are optional; opening a note need not parse the conversation UI.
import { lazy, Suspense, type ComponentProps } from "react";
import type { WorkspaceChat as WorkspaceChatComponent } from "./WorkspaceChat";
import type { ChatDockProps } from "../components/ChatDock";

const Agent = lazy(() => import("./WorkspaceChat").then((m) => ({ default: m.WorkspaceChat })));
const Dock = lazy(() => import("../components/ChatDock").then((m) => ({ default: m.ChatDock })));

export function WorkspaceChat(props: ComponentProps<typeof WorkspaceChatComponent>) {
  return <Suspense fallback={<div role="status">Opening Agent…</div>}><Agent {...props} /></Suspense>;
}

export function ChatDock(props: ChatDockProps) {
  return <Suspense fallback={<div role="status">Opening Agent…</div>}><Dock {...props} /></Suspense>;
}
</book:entry>

</book:doc>
