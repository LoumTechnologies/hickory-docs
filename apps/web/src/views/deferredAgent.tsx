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
