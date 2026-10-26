<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/deferredAgent.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-47b9f1579b62"></a>
## `apps/web/src/views/deferredAgent.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Editor Does Not Wait For Workspace Preparation](../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md).

<a id="f-47b9f1579b62-Agent"></a>
### `Agent`

Agent panes are optional; opening a note need not parse the conversation UI.

<storycode:copy id="f-47b9f1579b62-Agent">
// Agent panes are optional; opening a note need not parse the conversation UI.
import { lazy, Suspense, type ComponentProps } from "react";
import type { WorkspaceChat as WorkspaceChatComponent } from "./WorkspaceChat";
import type { ChatDockProps } from "../components/ChatDock";

const Agent = lazy(() => import("./WorkspaceChat").then((m) => ({ default: m.WorkspaceChat })));
</storycode:copy>
<a id="f-47b9f1579b62-Dock"></a>
### `Dock`

<storycode:copy id="f-47b9f1579b62-Dock">
const Dock = lazy(() => import("../components/ChatDock").then((m) => ({ default: m.ChatDock })));

</storycode:copy>
<a id="f-47b9f1579b62-WorkspaceChat"></a>
### `WorkspaceChat`

<storycode:copy id="f-47b9f1579b62-WorkspaceChat">
export function WorkspaceChat(props: ComponentProps<typeof WorkspaceChatComponent>) {
  return <Suspense fallback={<div role="status">Opening Agent…</div>}><Agent {...props} /></Suspense>;
}

</storycode:copy>
<a id="f-47b9f1579b62-ChatDock"></a>
### `ChatDock`

<storycode:copy id="f-47b9f1579b62-ChatDock">
export function ChatDock(props: ChatDockProps) {
  return <Suspense fallback={<div role="status">Opening Agent…</div>}><Dock {...props} /></Suspense>;
}
</storycode:copy>

### Assemble `apps/web/src/views/deferredAgent.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/deferredAgent.tsx"><storycode:paste select="#f-47b9f1579b62-Agent"/><storycode:paste select="#f-47b9f1579b62-Dock"/><storycode:paste select="#f-47b9f1579b62-WorkspaceChat"/><storycode:paste select="#f-47b9f1579b62-ChatDock"/></storycode:file>

</storycode:doc>
