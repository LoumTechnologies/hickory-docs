<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/welcomeActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d5db7c66e0ea"></a>
## `apps/web/src/views/welcomeActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d5db7c66e0ea-WelcomeVerbs"></a>
### `WelcomeVerbs`

What the welcome page offers. Every one of them does something — a row
here is a verb, never a link to a tour. Pure over the workspace's verbs,
so the list is data and WorkspaceView stays about arranging the window.

<storycode:copy id="f-d5db7c66e0ea-WelcomeVerbs">
// What the welcome page offers. Every one of them does something — a row
// here is a verb, never a link to a tour. Pure over the workspace's verbs,
// so the list is data and WorkspaceView stays about arranging the window.

import { navigate, newDocument } from "../router";
import { requestMenuAction } from "../lib/menuBridge";
import type { Layout } from "../shell/layout";
import type { WelcomeAction } from "./WelcomePane";
import {
  openFleetTab,
  openGitTab,
  openMergedTab,
  openStoryTab,
} from "./workspaceState";

export interface WelcomeVerbs {
  setLayout: (update: (current: Layout) => Layout) => void;
  openTerminal: () => Promise<unknown> | unknown;
  /** The focused editor's path, if any — the merged view is OF a path. */
  focusedPath: () => string | null;
  focusTree: () => void;
}

</storycode:copy>
<a id="f-d5db7c66e0ea-welcomeActionsFor"></a>
### `welcomeActionsFor`

<storycode:copy id="f-d5db7c66e0ea-welcomeActionsFor">
export function welcomeActionsFor({
  setLayout,
  openTerminal,
  focusedPath,
  focusTree,
}: WelcomeVerbs): WelcomeAction[] {
  return [
    {
      id: "new",
      label: "New document…",
      hint: "An untitled buffer, adopted into a document on its first save",
      run: newDocument,
    },
    {
      id: "new-project",
      label: "New project…",
      hint: "Create a .NET project and record its scaffold recipe in git",
      run: () => requestMenuAction("new-project"),
    },
    {
      id: "scratchpad",
      label: "Scratchpad",
      hint: "Text on its way to becoming a note",
      run: () => navigate("/scratchpad"),
    },
    {
      id: "agent",
      label: "Agent — show the conversation",
      hint: "Work with your open editors; /tree explores the conversation, /rewind branches",
      run: () => window.dispatchEvent(new CustomEvent("hickory-show-agent")),
    },
    {
      id: "terminal",
      label: "Open a terminal",
      hint: "In this folder; it appears on the folder's row in the tree",
      run: () => {
        void openTerminal();
      },
    },
    {
      id: "history",
      label: "History",
      hint: "The commit graph, with every commit's files",
      run: () => setLayout(openGitTab),
    },
    {
      id: "story",
      label: "History as a story",
      hint: "The same commits, oldest first, drawn as cards — a lens, read-only",
      run: () => setLayout(openStoryTab),
    },
    {
      id: "fleet",
      label: "Machines",
      hint: "Pair another of your machines, and choose what each may do here",
      run: () => setLayout(openFleetTab),
    },
    {
      id: "merged",
      label: "Compare across worktrees",
      hint: "One file as it exists in several worktrees at once — open a file first",
      run: () => {
        // The view is OF a path, so it needs one. With nothing focused this
        // opens the fleet's sibling question instead of an empty pane.
        const path = focusedPath();
        if (path) setLayout((current) => openMergedTab(current, path));
      },
    },
    {
      id: "find",
      label: "Find in folder…",
      hint: "Exhaustive find and replace across every file",
      run: () => focusTree(),
    },
  ];
}
</storycode:copy>

### Assemble `apps/web/src/views/welcomeActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/welcomeActions.ts"><storycode:paste select="#f-d5db7c66e0ea-WelcomeVerbs"/><storycode:paste select="#f-d5db7c66e0ea-welcomeActionsFor"/></storycode:file>

</storycode:doc>
