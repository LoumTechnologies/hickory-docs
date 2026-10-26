<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/App.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-25cc6f821b46"></a>
## `apps/web/src/App.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A New Project Is A Recipe Commit, Made As One Act](../../../../guarantees/authoring/a-new-project-is-a-recipe-commit/contract.md), [Every Hover Hint Is Drawn In The App Theme](../../../../guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme/contract.md), [Inserting An Element Writes Hick You Could Have Typed Yourself](../../../../guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md), [Settings preserves the open workspace](../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md), [The App Starts As A Lightweight Editor](../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md), [The Editor Does Not Wait For Workspace Preparation](../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md), [Close or recover a document without losing the wrong draft](../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [A Command The App Runs On Your Behalf Is Watched In A Terminal](../../../../guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal/contract.md).

<a id="f-25cc6f821b46-LineageView"></a>
### `LineageView`

<storycode:copy id="f-25cc6f821b46-LineageView">
import { lazy, Suspense, useEffect, useRef, useState } from "react";
import { navigate, useRoute, newDocument } from "./router";
import { WorkspaceView, type WorkspaceRoute } from "./views/WorkspaceView";
import { requestApplicationClose, requestWorkspaceClose } from "./lib/windowClose";
import { api } from "./api/client";
import { insertTarget, onMenuAction, type MenuAction } from "./lib/menuBridge";
import { UiActionsVisibility, useUiCommands } from "./lib/uiActionBridge";
import { TooltipLayer } from "./components/TooltipLayer";
import { showTerminalRequest } from "./lib/revealLine";
import type { ScaffoldStarted } from "./api/types";
import { FILES_CHANGED_EVENT } from "./shell/FolderTreePane";
import { BlankWelcome } from "./views/BlankWelcome";
import { SETTINGS_CLOSED } from "./lib/workspaceVisibility";
import { StartupNotice } from "./components/StartupNotice";

const LineageView = lazy(() => import("./views/LineageView").then((m) => ({ default: m.LineageView })));
</storycode:copy>
<a id="f-25cc6f821b46-SettingsView"></a>
### `SettingsView`

<storycode:copy id="f-25cc6f821b46-SettingsView">
const SettingsView = lazy(() => import("./views/SettingsView").then((m) => ({ default: m.SettingsView })));
</storycode:copy>
<a id="f-25cc6f821b46-NewProjectDialog"></a>
### `NewProjectDialog`

<storycode:copy id="f-25cc6f821b46-NewProjectDialog">
const NewProjectDialog = lazy(() => import("./components/NewProjectDialog").then((m) => ({ default: m.NewProjectDialog })));

</storycode:copy>
<a id="f-25cc6f821b46-SCAFFOLD_POLL_MS"></a>
### `SCAFFOLD_POLL_MS`

How often the app asks whether a scaffold has landed, and for how long.
The person is watching the terminal; this is only so the tree and the
history pane catch up on their own. Ten minutes is longer than any
`dotnet new` and shorter than forever.

<storycode:copy id="f-25cc6f821b46-SCAFFOLD_POLL_MS">
/** How often the app asks whether a scaffold has landed, and for how long.
 * The person is watching the terminal; this is only so the tree and the
 * history pane catch up on their own. Ten minutes is longer than any
 * `dotnet new` and shorter than forever. */
const SCAFFOLD_POLL_MS = 400;
</storycode:copy>
<a id="f-25cc6f821b46-SCAFFOLD_POLL_TRIES"></a>
### `SCAFFOLD_POLL_TRIES`

<storycode:copy id="f-25cc6f821b46-SCAFFOLD_POLL_TRIES">
const SCAFFOLD_POLL_TRIES = 1500;

</storycode:copy>
<a id="f-25cc6f821b46-App"></a>
### `App`

The desktop app's shell.
There is no authentication, no landing page, and no account menu. This UI
only ever talks to the local server running in the same process, which
answers one person: the one whose machine it is. The marketing site is a
separate build — see `docs/specs/freeform/local-only.md`.

<storycode:copy id="f-25cc6f821b46-App">
/// The desktop app's shell.
///
/// There is no authentication, no landing page, and no account menu. This UI
/// only ever talks to the local server running in the same process, which
/// answers one person: the one whose machine it is. The marketing site is a
/// separate build — see `docs/specs/freeform/local-only.md`.
export function App() {
  const requestedRoute = useRoute();
  const route = requestedRoute.name === "landing"
    ? { name: "new" as const, introduction: true }
    : requestedRoute;
  const inWorkspace = route.name === "doc" || route.name === "new" || route.name === "scratchpad";
  const workspace = useRef<{ route: WorkspaceRoute; hash: string } | null>(null);
  if (inWorkspace) workspace.current = { route: route as WorkspaceRoute, hash: location.hash || "#/" };
  if (route.name === "blank") workspace.current = null;
  const pendingWorkspaceEvents = useRef<string[]>([]);
  const returnToWorkspace = () => {
    navigate(workspace.current?.hash.replace(/^#/, "") ?? "/");
    if (route.name === "settings") window.dispatchEvent(new Event(SETTINGS_CLOSED));
  };
  const revealWorkspaceAndDispatch = (event: string) => {
    if (inWorkspace) window.dispatchEvent(new CustomEvent(event));
    else {
      pendingWorkspaceEvents.current.push(event);
      returnToWorkspace();
    }
  };
  useEffect(() => {
    if (!inWorkspace) return;
    // Wait until the retained workspace is visible before a command moves
    // its focus or opens a pane. Its editors and layout stay mounted.
    for (const event of pendingWorkspaceEvents.current.splice(0)) {
      window.dispatchEvent(new CustomEvent(event));
    }
  }, [inWorkspace]);

  // The desktop shell's native menu, arriving as DOM events (menuBridge).
  // Navigation is answered here; Save / Save As need the focused document,
  // which only the workspace knows — they are re-dispatched as a window
  // event it listens for. The subscription is mounted once, so the current route is
  // read through a ref rather than re-subscribing (which would reset the
  // bridge's duplicate-keypress guard).
  const [notice, setNotice] = useState<string | null>(null);
  // New Project lives here rather than in the workspace: it needs no buffer,
  // no focused document and no layout — it writes a file into the folder —
  // so hanging it off the workspace would have made it unreachable from
  // Settings and from the landing redirect for no reason.
  const [newProject, setNewProject] = useState(false);

  // What happens after New Project's button: the scaffolder is running in a
  // terminal, and that terminal is what the person watches.
  //
  // The app has its own reason to know how it ended — the tree has a new
  // folder in it and the history a new commit — so it polls for the verdict
  // while the terminal shows the person the same act in `dotnet`'s own words.
  // Two readers of one thing, neither pretending to be the other; this one
  // says one sentence and stops, because everything worth reading is in the
  // tab. See docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md.
  const watchScaffold = (started: ScaffoldStarted) => {
    showTerminalRequest(started.session.id, started.session.title);
    let tries = 0;
    const poll = () => {
      void api
        .scaffoldResult(started.session.id)
        .then((result) => {
          if (result.state === "running") {
            // Bounded, because a `dotnet new` that never exits is a terminal
            // the person is already looking at — not a poll to keep forever.
            if (tries++ < SCAFFOLD_POLL_TRIES) setTimeout(poll, SCAFFOLD_POLL_MS);
            return;
          }
          window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
          if (result.state === "committed") {
            setNotice(
              `${result.output}/ scaffolded and committed as ${result.short} — ` +
                `${result.files.length} file${result.files.length === 1 ? "" : "s"}. ` +
                "Read it as a story in the History pane.",
            );
          } else {
            // The terminal has the whole of it; this only says where to look.
            setNotice(
              `Nothing was committed — the ${started.session.title} terminal says why.`,
            );
          }
        })
        .catch(() => {
          /* The terminal is the record. A poll that cannot reach the server
             has nothing useful to add to it. */
        });
    };
    setTimeout(poll, SCAFFOLD_POLL_MS);
  };

  const routeRef = useRef(route);
  routeRef.current = route;
  const handleMenuAction = (action: MenuAction) => {
      // Insert — bare, or naming one element — wants a buffer to write into,
      // which only the workspace has. Handled before the switch because the
      // element-carrying form is a whole family of actions, not one case.
      if (action === "insert" || insertTarget(action) !== null || action === "edit-element") {
        if (routeRef.current.name === "doc" || routeRef.current.name === "new") {
          window.dispatchEvent(new CustomEvent("hickory-doc-command", { detail: action }));
        } else {
          setNotice("Open a document first.");
        }
        return;
      }
      switch (action) {
        case "new":
          if (routeRef.current.name === "blank") {
            setNotice("Open a folder before creating a document.");
            return;
          }
          newDocument();
          return;
        case "new-project":
          if (routeRef.current.name === "blank") {
            setNotice("Open a folder before creating a project.");
            return;
          }
          setNewProject(true);
          return;
        case "files":
          if (routeRef.current.name === "blank") {
            setNotice("Open a folder to show Files.");
            return;
          }
          revealWorkspaceAndDispatch("hickory-show-files");
          break;
        case "show-agent":
          if (routeRef.current.name === "blank") {
            setNotice("Open a file or folder to use the agent.");
            return;
          }
          revealWorkspaceAndDispatch("hickory-show-agent");
          break;
        case "terminal":
        case "attention":
          // Terminals belong to the workspace, which owns the layout they
          // open into and the queue cursor ⌘J walks.
          if (routeRef.current.name === "doc" || routeRef.current.name === "new") {
            window.dispatchEvent(
              new CustomEvent("hickory-terminal-command", { detail: action }),
            );
          } else if (routeRef.current.name === "blank") {
            setNotice("Open a file or folder to use a terminal.");
          }
          return;
        case "settings":
          if (routeRef.current.name === "blank") {
            setNotice("Open a file or folder to use Settings.");
            return;
          }
          navigate("/settings");
          return;
        case "save":
        case "save-as":
        case "save-all":
        case "print":
          // The workspace routes the command to whichever document is
          // focused; it is mounted for both the doc and untitled routes.
          if (routeRef.current.name === "doc" || routeRef.current.name === "new") {
            window.dispatchEvent(new CustomEvent("hickory-doc-command", { detail: action }));
          } else {
            setNotice("Open a document first.");
          }
          return;
      }
  };
  const menuHandler = useRef(handleMenuAction); menuHandler.current = handleMenuAction;
  useEffect(() => onMenuAction(action => menuHandler.current(action)), []);
  const pendingApplicationClose = useRef<ReturnType<typeof requestApplicationClose> | null>(null);
  const closeApplication = (signal?: AbortSignal) => {
    if (pendingApplicationClose.current) return pendingApplicationClose.current;
    const current = routeRef.current;
    const pending = requestApplicationClose({ workspace: workspace.current?.hash.replace(/^#/, "") ?? null,
      inWorkspace: current.name === "doc" || current.name === "new" || current.name === "scratchpad", signal,
      reveal: navigate, workspaceClose: requestWorkspaceClose, hostClose: api.closeWindow });
    pendingApplicationClose.current = pending;
    void pending.then(outcome => {
      if (pendingApplicationClose.current === pending) pendingApplicationClose.current = null;
      if (outcome.status === "unavailable") setNotice(outcome.reason);
    });
    return pending;
  };
  const closeApplicationRef = useRef(closeApplication); closeApplicationRef.current = closeApplication;
  useUiCommands([
    ...(route.name === "lineage" ? [{ key: "application-navigation/back-to-workspace", command: {
      label: "Back to workspace", parent: "application-navigation", subject: "application-navigation", parameters: {}, prerequisites: [],
      read: () => ({ revision: JSON.stringify([route, workspace.current?.hash]), enabled: true }),
      execute: async () => { returnToWorkspace(); return { status: "completed" as const, value: { navigation: "requested", result: "Observe the retained workspace" } }; },
    } }] : []),
    { key: "application-navigation/close-window", command: { label: "Close current window", parent: "application-navigation", subject: "application-window:close", parameters: {},
      prerequisites: ["Reveals the retained workspace for source-bound close decisions. Host acknowledgement is not a witness of window destruction; shutdown can prevent reply delivery."],
      read: () => ({ revision: JSON.stringify([route, workspace.current?.hash, newProject, !!pendingApplicationClose.current]), enabled: !pendingApplicationClose.current, reason: pendingApplicationClose.current ? "A window close flow is pending; inspect its prompt or workspace close status" : undefined }),
      execute: async (_args, _actor, signal) => closeApplicationRef.current(signal),
    } },
    ...([['new', 'Create untitled document'], ['new-project', 'Open new project dialog'], ['files', 'Show workspace Files'], ['show-agent', 'Show workspace agent'], ['settings', 'Open settings']] as const).map(([action, label]) => ({
      key: `application-navigation/${action}`, command: {
        label, parent: 'application-navigation', subject: 'application-navigation', parameters: {}, prerequisites: ['An open workspace is required'],
        read: () => ({ revision: JSON.stringify([route, newProject]), enabled: route.name !== 'blank', reason: route.name === 'blank' ? 'Open a workspace first' : undefined }),
        execute: async () => { handleMenuAction(action); return { status: 'completed' as const, value: { dispatch: 'requested', result: 'Observe the actual route, pane or dialog' } }; },
      },
    })),
    { key: 'application-navigation/state', command: {
      label: 'Read application route and notice', parent: 'application-navigation', subject: 'application-navigation', parameters: {}, prerequisites: [],
      read: () => ({ revision: JSON.stringify([route, notice, newProject]), enabled: true }),
      execute: async () => ({ status: 'completed', value: JSON.parse(JSON.stringify({ route, notice, new_project_dialog: newProject, retained_workspace: workspace.current?.route ?? null })) }),
    } },
  ]);
  useEffect(() => {
    const onOpenPath = (event: Event) => {
      const path = (event as CustomEvent<string>).detail;
      if (typeof path !== "string" || !path) return;
      if (routeRef.current.name === "new" || routeRef.current.name === "doc") return;
      navigate(`/file/${encodeURIComponent(path)}`);
    };
    window.addEventListener("hickory-open-path", onOpenPath);
    return () => window.removeEventListener("hickory-open-path", onOpenPath);
  }, []);
  useEffect(() => {
    const onClose = () => { void closeApplicationRef.current(); };
    window.addEventListener("hickory-window-close-request", onClose);
    return () => window.removeEventListener("hickory-window-close-request", onClose);
  }, []);
  useEffect(() => {
    const onQuit = () => {
      void closeApplicationRef.current().finally(() => { void api.cancelQuit().catch(() => {}); });
    };
    window.addEventListener("hickory-application-quit-request", onQuit);
    return () => window.removeEventListener("hickory-application-quit-request", onQuit);
  }, []);

  useEffect(() => {
    if (notice === null) return;
    const timer = setTimeout(() => setNotice(null), 2500);
    return () => clearTimeout(timer);
  }, [notice]);

  return (
    <div className="app">
      <main className="content">
        {workspace.current && <div data-workspace hidden={!inWorkspace} style={{ display: inWorkspace ? "contents" : "none" }}>
          <UiActionsVisibility visible={inWorkspace}><WorkspaceView route={workspace.current.route} /></UiActionsVisibility>
        </div>}
        <Suspense fallback={<div role="status">Opening…</div>}>
          {inWorkspace ? null : route.name === "lineage" ? (
            <section aria-label="Project lineage" style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
              <nav style={{ padding: ".75rem .75rem 0" }} aria-label="Lineage navigation">
                <button type="button" className="btn" onClick={returnToWorkspace}>← Back to workspace</button>
              </nav>
              <LineageView projectId={route.id} />
            </section>
          ) : route.name === "settings" ? (
            <SettingsView onBack={returnToWorkspace} />
          ) : route.name === "blank" ? (
            <BlankWelcome />
          ) : (
            <BlankWelcome />
          )}
        </Suspense>
      </main>
      {notice && (
        <div className="menu-notice" role="status">
          {notice}
        </div>
      )}
      {newProject && (
        <Suspense fallback={<div role="status">Opening New Project…</div>}>
          <NewProjectDialog onStarted={watchScaffold} onClose={() => setNewProject(false)} />
        </Suspense>
      )}
      {/* One tooltip for the whole app; every `data-tip` in it lands here. */}
      <TooltipLayer />
      <StartupNotice />
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/App.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/App.tsx"><storycode:paste select="#f-25cc6f821b46-LineageView"/><storycode:paste select="#f-25cc6f821b46-SettingsView"/><storycode:paste select="#f-25cc6f821b46-NewProjectDialog"/><storycode:paste select="#f-25cc6f821b46-SCAFFOLD_POLL_MS"/><storycode:paste select="#f-25cc6f821b46-SCAFFOLD_POLL_TRIES"/><storycode:paste select="#f-25cc6f821b46-App"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-65ac24c24624"></a>
## `apps/web/src/App.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Context Provenance Is Derived From The Session, Never From The Model](../../../../guarantees/agent/context-provenance-is-derived-from-the-session/contract.md), [A Folder Remembers Its Window](../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [The File Menu Preserves The Originating Window](../../../../guarantees/authoring/file-menu-preserves-the-originating-window/contract.md), [Settings preserves the open workspace](../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md), [The App Starts As A Lightweight Editor](../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md), [The Measure Says What It Measures](../../../../guarantees/authoring/the-measure-says-what-it-measures/contract.md), [Close or recover a document without losing the wrong draft](../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md), [A Local Install Is a Launchable macOS App](../../../../guarantees/release/a-local-install-is-a-launchable-macos-app/contract.md).

<a id="f-65ac24c24624-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement">
import { act, cleanup, fireEvent, render, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { App } from "./App";
import { UiActions } from "./lib/uiActions";
import { UiActionContext } from "./lib/uiActionBridge";
import { newDocument } from "./router";
import { api } from "./api/client";
import { installMockApi } from "./mock/mockApi";
import { STARTUP_INTRODUCTION } from "./lib/newDoc";
import { WELCOME_KEY } from "./lib/welcomePref";
import { openChatTab, openDocTab, initialWorkspace } from "./views/workspaceState";
import { tab, withTree } from "./shell/layout";
import { wrapColumnOf } from "./editor/wrapColumn";

// Keep this fixture wrapped so opening it exercises the clean-introduction
// guarantee independently of edits to the bundled startup copy.
vi.mock("./content/startup.md?raw", () => ({ default: "# Hickory Docs\n\nA wrapped introduction\ncontinues on another line.\n" }));

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-2">
beforeEach(() => {
  window.history.replaceState(null, "", "/");
  localStorage.setItem(WELCOME_KEY, "1");
  installMockApi();
  vi.spyOn(api, "workspaceUi").mockResolvedValue({ state: { version: 1, layout: null } });
});
</storycode:copy>
<a id="f-65ac24c24624-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-3">
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  localStorage.clear();
});

</storycode:copy>
<a id="f-65ac24c24624-editor"></a>
### `editor`

<storycode:copy id="f-65ac24c24624-editor">
function editor(container: HTMLElement): EditorView {
  const element = container.querySelector("[data-spatial-center=true] .untitled-tab .cm-editor") ?? container.querySelector(".untitled-tab .cm-editor");
  if (!element) throw new Error("no untitled editor");
  return EditorView.findFromDOM(element as HTMLElement)!;
}

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-4">
it("keeps the editor, selection and unsaved bytes through Settings without replaying the startup route", async () => {
  const app = render(<App />);
  await waitFor(() => expect(editor(app.container).hasFocus).toBe(true));
  const view = editor(app.container);
  act(() => view.dispatch({ changes: { from: 0, insert: "My draft " }, selection: { anchor: 2, head: 6 } }));
  const text = view.state.doc.toString(), selection = view.state.selection.toJSON();
  const count = app.getAllByRole("tab", { name: /Untitled/ }).length;
  fireEvent(window, new CustomEvent("hickory-menu", { detail: "settings" }));
  await app.findByRole("heading", { name: /^Settings$/ });
  fireEvent.keyDown(window, { key: "w", ctrlKey: true });
  expect(view.state.doc.toString()).toBe(text);
  fireEvent.click(app.getByRole("button", { name: "← Back" }));
  await waitFor(() => expect(app.queryByRole("heading", { name: /^Settings$/ })).toBeNull());
  expect(editor(app.container)).toBe(view);
  expect(view.state.doc.toString()).toBe(text);
  expect(view.state.selection.toJSON()).toEqual(selection);
  expect(app.getAllByRole("tab", { name: /Untitled/ })).toHaveLength(count);
});

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-5">
it("checks unsaved work visibly when closing the app from Settings", async () => {
  const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  const app = render(<App />);
  await waitFor(() => expect(editor(app.container).hasFocus).toBe(true));
  const view = editor(app.container);
  act(() => view.dispatch({ changes: { from: 0, insert: "Unsaved work " } }));
  fireEvent(window, new CustomEvent("hickory-menu", { detail: "settings" }));
  await app.findByRole("heading", { name: /^Settings$/ });
  fireEvent(window, new Event("hickory-window-close-request"));
  const dialog = await app.findByRole("dialog", { name: /^Save changes before closing Hickory Docs\?/ });
  expect(app.queryByRole("heading", { name: /^Settings$/ })).toBeNull();
  expect(close).not.toHaveBeenCalled();
  fireEvent.click(within(dialog).getAllByRole("button", { name: "Cancel" })[0]);
  expect(editor(app.container)).toBe(view);
  expect(view.state.doc.toString()).toContain("Unsaved work");
});

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-6">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("lets an agent request the same visible window-close decision from Settings", async () => {
  const registry = new UiActions({ workspace: "fixture", window: "fixture-window" }, () => true, async () => {});
  const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  const app = render(<UiActionContext.Provider value={registry}><App /></UiActionContext.Provider>);
  await waitFor(() => expect(editor(app.container).hasFocus).toBe(true));
  const view = editor(app.container);
  act(() => view.dispatch({ changes: { from: 0, insert: "Retained agent-close draft " } }));
  fireEvent(window, new CustomEvent("hickory-menu", { detail: "settings" }));
  await app.findByRole("heading", { name: /^Settings$/ });
  const action = registry.observe("application-navigation", 64).actions.find(action => action.label === "Close current window")!;
  expect(action).toBeDefined();
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:test", kind: "agent" }, { handle: action.handle, arguments: {}, request_id: "close-from-settings" }); });
  const dialog = await app.findByRole("dialog", { name: /^Save changes before closing Hickory Docs\?/ });
  expect(app.queryByRole("heading", { name: /^Settings$/ })).toBeNull();
  fireEvent.click(within(dialog).getAllByRole("button", { name: "Cancel" })[0]);
  await act(async () => { expect(await pending).toMatchObject({ status: "result", outcome: { status: "completed", value: { cancelled: true, host_close_submitted: false } } }); });
  expect(close).not.toHaveBeenCalled(); expect(editor(app.container)).toBe(view);
  expect(view.state.doc.toString()).toContain("Retained agent-close draft");
});

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-7">
describe("the startup workspace", () => {
  // Guarantee: docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md
  it("does not check merge-driver setup when no folder is open", async () => {
    const files = await api.files();
    vi.spyOn(api, "files").mockResolvedValue({ ...files, folder_open: false });
    const check = vi.spyOn(api, "mergeDriver");
    const { container, getByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "show-agent" }));
    await waitFor(() => expect(getByRole("tab", { name: /Agent/ })).toBeTruthy());
    expect(check).not.toHaveBeenCalled();
    expect(container.textContent).not.toContain("Run hick init");
  });

  // Guarantee: docs/guarantees/authoring/the-measure-says-what-it-measures.md
  it("keeps a dragged prose margin in the initial untitled document and stores it", async () => {
    vi.mocked(api.workspaceUi).mockResolvedValue({ state: { version: 1, wrap: { untitled: 72 } } });
    const save = vi.spyOn(api, "saveWorkspaceUi");
    const { container, getByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    // The Files pane rearranges the layout during hydration and remounts the
    // initial editor. Wait for that pane before retaining a live view.
    const live = await waitFor(() => {
      getByRole("tab", { name: /Files/, hidden: true });
      const view = editor(container);
      expect(view.hasFocus).toBe(true);
      return view;
    });
    const marker = getByRole("slider", { name: "Where prose wraps" });
    await waitFor(() => expect(marker.getAttribute("aria-valuenow")).toBe("72"));
    // jsdom has no layout: supply the geometry read by the real ruler.
    Object.defineProperty(live, "defaultCharacterWidth", { value: 8, configurable: true });
    const requestMeasure = live.requestMeasure;
    live.requestMeasure = (<T,>(request: {
      read: (view: EditorView) => T;
      write?: (value: T, view: EditorView) => void;
    }) => request.write?.(request.read(live), live)) as typeof live.requestMeasure;
    fireEvent(window, new Event("resize"));
    live.requestMeasure = requestMeasure;
    fireEvent(marker, new MouseEvent("pointerdown", { bubbles: true, clientX: 576 }));
    fireEvent(marker, new MouseEvent("pointermove", { bubbles: true, clientX: 480 }));
    expect(marker.getAttribute("aria-valuenow")).toBe("60");
    fireEvent(marker, new MouseEvent("pointerup", { bubbles: true, clientX: 480 }));
    expect(marker.getAttribute("aria-valuenow")).toBe("60");
    expect(wrapColumnOf(live.state)).toBe(60);
    expect(live.state.doc.toString()).toBe(STARTUP_INTRODUCTION);
    await waitFor(() => expect(save).toHaveBeenLastCalledWith(
      expect.objectContaining({ wrap: { untitled: 60 } }), false,
    ));
  });

  it("opens a focused unsaved introduction in a folder without stored panes", async () => {
    const create = vi.spyOn(api, "createDoc");
    const saveDialog = vi.spyOn(api, "saveFileDialog").mockResolvedValue({ path: null });
    const { container, getAllByRole, getByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    await waitFor(() => expect(editor(container).hasFocus).toBe(true));
    expect(getAllByRole("tab")).toHaveLength(1);
    expect(getByRole("tab", { name: /Files/, hidden: true })).toBeTruthy();
    expect(getByRole("tab", { name: /Untitled/ }).textContent).toContain("Untitled");
    expect(getByRole("tab", { name: /Untitled/ }).textContent).not.toContain("*");
    expect(container.querySelector(".filesystem-editor")).not.toBeNull();
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "save" }));
    await waitFor(() => expect(saveDialog).toHaveBeenCalledWith("Hickory Docs.md"));
    act(() => editor(container).dispatch({ changes: { from: editor(container).state.doc.length, insert: "My note" } }));
    expect(create).not.toHaveBeenCalled();
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "files" }));
    await waitFor(() => expect(getAllByRole("tab", { hidden: true }).some((el) => el.textContent?.includes("Files"))).toBe(true));
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "show-agent" }));
    await waitFor(() => expect(getAllByRole("tab").some((el) => el.textContent?.includes("Agent"))).toBe(true));
    expect(editor(container).state.doc.toString()).toContain("My note");
    expect(create).not.toHaveBeenCalled();
  });

  it("creates independent unnamed tabs on every New request and recovers them together", async () => {
    const drafts = new Map<string, { path: string; contents: string; base: string; saved_at: number }>();
    vi.spyOn(api, "drafts").mockImplementation(async () => ({ drafts: [...drafts.values()] }));
    vi.spyOn(api, "saveDraft").mockImplementation(async draft => { drafts.set(draft.path, draft); return { ok: true }; });
    vi.spyOn(api, "discardDraft").mockImplementation(async path => { drafts.delete(path); return { ok: true }; });
    const first = render(<App />);
    await waitFor(() => expect(first.getByRole("tab", { name: /Files/, hidden: true })).toBeTruthy());
    await waitFor(() => expect(editor(first.container).hasFocus).toBe(true));
    act(() => editor(first.container).dispatch({ changes: { from: 0, to: editor(first.container).state.doc.length, insert: "First note" } }));
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "new" }));
    await waitFor(() => expect(within(first.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ })).toHaveLength(2));
    await waitFor(() => expect(editor(first.container).state.doc.toString()).toBe(""));
    act(() => editor(first.container).dispatch({ changes: { from: 0, insert: "Second note" } }));
    act(() => newDocument());
    await waitFor(() => expect(within(first.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ })).toHaveLength(3));
    fireEvent(window, new Event("pagehide"));
    await waitFor(() => expect([...drafts.values()].map(draft => draft.contents).sort()).toEqual(["First note", "Second note"]));
    first.unmount();
    window.history.replaceState(null, "", "/");
    const second = render(<App />);
    await waitFor(() => expect(within(second.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ })).toHaveLength(2));
    const restored = new Set<string>();
    for (const id of within(second.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ }).map(tab => tab.id)) {
      fireEvent.click(second.container.querySelector(`#${id}`)!);
      await waitFor(() => expect(editor(second.container)).toBeTruthy());
      restored.add(editor(second.container).state.doc.toString());
    }
    expect(restored.has("First note")).toBe(true);
    expect(restored.has("Second note")).toBe(true);
  });

  it("restores unnamed documents without adding the startup introduction on each launch", async () => {
    vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "parent", tree: [] });
    const drafts = new Map([
      ["untitled:first", { path: "untitled:first", contents: "A note begun blank", base: "", saved_at: 1 }],
      ["untitled:second", { path: "untitled:second", contents: "# Older introduction\nMy edits", base: "# Older introduction\n", saved_at: 2 }],
    ]);
    vi.spyOn(api, "drafts").mockImplementation(async () => ({ drafts: [...drafts.values()] }));
    vi.spyOn(api, "saveDraft").mockImplementation(async draft => { drafts.set(draft.path, draft); return { ok: true }; });
    vi.spyOn(api, "discardDraft").mockImplementation(async path => { drafts.delete(path); return { ok: true }; });
    for (let launch = 0; launch < 3; launch++) {
      window.history.replaceState(null, "", "/");
      const app = render(<App />);
      await waitFor(() => expect(within(app.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ })).toHaveLength(2));
      const restored = new Set<string>();
      for (const id of within(app.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ }).map(tab => tab.id)) {
        fireEvent.click(app.container.querySelector(`#${id}`)!);
        const expected = drafts.get(`untitled:${id.slice("spatial-tab-".length)}`)!.contents;
        await waitFor(() => expect(editor(app.container).state.doc.toString()).toBe(expected));
        restored.add(editor(app.container).state.doc.toString());
      }
      expect(restored).toEqual(new Set(["A note begun blank", "# Older introduction\nMy edits"]));
      fireEvent(window, new Event("pagehide"));
      app.unmount();
      expect(drafts.size).toBe(2);
    }
  });

  it("opens a blank document through New after the startup document is discarded", async () => {
    const { container, getByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    fireEvent.click(getByRole("button", { name: "Close Untitled" }));
    await waitFor(() => expect(container.querySelector(".untitled-tab")).toBeNull());
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "new" }));
    await waitFor(() => expect(editor(container).state.doc.toString()).toBe(""));
  });
  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("closes the untouched introduction without prompting and clears dirty state on undo", async () => {
    const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
    const saveDraft = vi.spyOn(api, "saveDraft");
    const { container, getByRole, queryByRole } = render(<App />);
    // Folder hydration remounts the editor; exercise the settled layout.
    await waitFor(() => expect(getByRole("tab", { name: /Files/, hidden: true })).toBeTruthy());
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    act(() => editor(container).dispatch({ changes: { from: 0, insert: "edit" } }));
    await waitFor(() => expect(getByRole("tab", { name: /Untitled/ }).textContent).toContain("*"));
    act(() => editor(container).dispatch({ changes: { from: 0, to: 4 } }));
    await waitFor(() => expect(getByRole("tab", { name: /Untitled/ }).textContent).not.toContain("*"));
    fireEvent(window, new Event("hickory-workspace-close-request"));
    await waitFor(() => expect(close).toHaveBeenCalled());
    expect(queryByRole("button", { name: "Save all" })).toBeNull();
    expect(saveDraft).not.toHaveBeenCalled();
  });

  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("writes a recovery draft on pagehide and deletes it when edits are undone", async () => {
    const save = vi.spyOn(api, "saveDraft");
    const discard = vi.spyOn(api, "discardDraft");
    const { container, getByRole } = render(<App />);
    // Folder hydration remounts the editor; exercise the settled layout.
    await waitFor(() => expect(getByRole("tab", { name: /Files/, hidden: true })).toBeTruthy());
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    act(() => editor(container).dispatch({ changes: { from: 0, insert: "edit" } }));
    fireEvent(window, new Event("pagehide"));
    await waitFor(() => expect(save).toHaveBeenCalledWith(expect.objectContaining({
      path: expect.stringMatching(/^untitled:/), contents: "edit" + STARTUP_INTRODUCTION,
    })));
    act(() => editor(container).dispatch({ changes: { from: 0, to: 4 } }));
    await waitFor(() => expect(discard).toHaveBeenCalledWith(expect.stringMatching(/^untitled:/)));
  });

  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("discards Untitled recovery before approving window close", async () => {
    let finishWrite!: () => void;
    const writing = new Promise<void>(resolve => { finishWrite = resolve; });
    const save = vi.spyOn(api, "saveDraft").mockImplementation(async () => { await writing; return { ok: true }; });
    const discard = vi.spyOn(api, "discardDraft");
    const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
    const { container, getByRole } = render(<App />);
    // Folder hydration adds a split and remounts the introduction editor.
    // Wait for that arrangement before testing the close gesture.
    await waitFor(() => expect(getByRole("tab", { name: /Files/, hidden: true })).toBeTruthy());
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    act(() => editor(container).dispatch({ changes: { from: 0, insert: "edit" } }));
    fireEvent(window, new Event("pagehide"));
    await waitFor(() => expect(save).toHaveBeenCalled());
    discard.mockClear();
    fireEvent(window, new Event("hickory-workspace-close-request"));
    await waitFor(() => expect(getByRole("button", { name: "Close without saving" })).toBeTruthy());
    fireEvent.click(getByRole("button", { name: "Close without saving" }));
    expect(close).not.toHaveBeenCalled();
    expect(discard).not.toHaveBeenCalled();
    await act(async () => { finishWrite(); });
    await waitFor(() => expect(close).toHaveBeenCalled());
    fireEvent(window, new Event("pagehide"));
    expect(save).toHaveBeenCalledTimes(1);
    expect(discard).toHaveBeenCalledWith(expect.stringMatching(/^untitled:/));
    expect(discard.mock.invocationCallOrder.at(-1)).toBeLessThan(close.mock.invocationCallOrder[0]);
  });

  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it.each([true, false])("discards closed Untitled work when folder_open is %s", async (folderOpen) => {
    vi.spyOn(api, "files").mockResolvedValue({ folder_open: folderOpen, root: "parent", tree: [] });
    let draft: { path: string; contents: string; base: string; saved_at: number } | undefined;
    vi.spyOn(api, "drafts").mockImplementation(async () => ({ drafts: draft ? [draft] : [] }));
    vi.spyOn(api, "saveDraft").mockImplementation(async (value) => { draft = value; return { ok: true }; });
    vi.spyOn(api, "discardDraft").mockImplementation(async () => { draft = undefined; return { ok: true }; });
    const first = render(<App />);
    if (folderOpen) await waitFor(() => expect(first.getByRole("tab", { name: /Files/, hidden: true })).toBeTruthy());
    await waitFor(() => expect(editor(first.container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    act(() => editor(first.container).dispatch({ changes: { from: 0, to: editor(first.container).state.doc.length, insert: "My unsaved note" } }));
    fireEvent.click(first.getByRole("button", { name: "Close Untitled" }));
    await waitFor(() => expect(first.getByRole("button", { name: "Discard" })).toBeTruthy());
    expect(first.getByText(/Save changes to Untitled/).textContent).toContain("discards these changes");
    fireEvent.click(first.getByRole("button", { name: "Discard" }));
    await waitFor(() => expect(first.container.querySelector(".untitled-tab")).toBeNull());
    fireEvent(window, new Event("pagehide"));
    expect(draft).toBeUndefined();
    first.unmount();
    const second = render(<App />);
    await waitFor(() => expect(editor(second.container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    expect(second.getByRole("tab", { name: /Untitled/ }).textContent).not.toContain("*");
  });

});


</storycode:copy>
<a id="f-65ac24c24624-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-8">
describe("windows without an open folder", () => {
  // Guarantee: docs/guarantees/authoring/file-menu-preserves-the-originating-window.md
  it("keeps each unsaved editor when the first folder is attached", async () => {
    const files = vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "parent", tree: [] });
    const app = render(<App />);
    await waitFor(() => expect(editor(app.container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    act(() => editor(app.container).dispatch({ changes: { from: 0, to: editor(app.container).state.doc.length, insert: "First unsaved note" } }));
    act(() => newDocument());
    await waitFor(() => expect(editor(app.container).state.doc.toString()).toBe(""));
    act(() => editor(app.container).dispatch({ changes: { from: 0, insert: "Second unsaved note" } }));
    const ids = within(app.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ }).map(tab => tab.id);
    files.mockResolvedValue({ folder_open: true, root: "selected", tree: [] });
    fireEvent(window, new Event("hickory:files-changed"));
    await waitFor(() => expect(app.getByRole("tab", { name: /Files/, hidden: true })).toBeTruthy());
    expect(within(app.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ }).map(tab => tab.id)).toEqual(ids);
    const contents = new Set<string>();
    for (const id of within(app.getByRole("tablist", { name: "Center views" })).getAllByRole("tab", { name: /Untitled/ }).map(tab => tab.id)) {
      fireEvent.click(app.container.querySelector(`#${id}`)!);
      await waitFor(() => expect(editor(app.container).state.doc.toString()).not.toBe(""));
      contents.add(editor(app.container).state.doc.toString());
    }
    expect(contents).toEqual(new Set(["First unsaved note", "Second unsaved note"]));
  });

  it("keeps Files closed while editing and saving still work", async () => {
    vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "parent", tree: [] });
    const save = vi.spyOn(api, "saveFileDialog").mockResolvedValue({ path: null });
    const { container, queryByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "files" }));
    await waitFor(() => expect(queryByRole("status")?.textContent).toContain("Open a folder"));
    expect(container.querySelector(".folder-tree")).toBeNull();
    expect(container.querySelectorAll('[role="tab"]').length).toBe(1);
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "save" }));
    await waitFor(() => expect(save).toHaveBeenCalled());
    expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
  });

  it("explains unavailable tools in a blank window without calling their APIs", async () => {
    window.history.replaceState(null, "", "/#/blank");
    const settings = vi.spyOn(api, "settingsKeys");
    const { getByRole, queryByRole } = render(<App />);
    for (const action of ["files", "show-agent", "terminal", "settings", "new", "new-project"]) {
      fireEvent(window, new CustomEvent("hickory-menu", { detail: action }));
      await waitFor(() => expect(getByRole("status").textContent).toContain("Open a"));
    }
    expect(queryByRole("tab")).toBeNull();
    expect(settings).not.toHaveBeenCalled();
    expect(window.location.hash).toBe("#/blank");
  });
});

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-9">
// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("restores a folder's open document without adding the startup introduction", async () => {
  const previous = openDocTab(openChatTab(withTree(initialWorkspace(), tab("tree", "folder", "Files"))), "d1", "quickstart.md");
  vi.mocked(api.workspaceUi).mockResolvedValue({ state: { version: 1, layout: previous } });
  const app = render(<App />);
  await app.findByRole("tab", { name: /quickstart.md/ });
  expect(app.queryByRole("tab", { name: /Untitled/ })).toBeNull();
  expect(app.getByRole("tab", { name: /Files/, hidden: true })).toBeTruthy();
});

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-10">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("joins repeated native close requests in a blank window while the host reply is pending", async () => {
  window.history.replaceState(null, "", "/#/blank");
  let finish!: (value: { ok: true }) => void;
  const close = vi.spyOn(api, "closeWindow").mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const app = render(<App />);
  fireEvent(window, new Event("hickory-window-close-request"));
  fireEvent(window, new Event("hickory-window-close-request"));
  expect(close).toHaveBeenCalledTimes(1);
  await act(async () => { finish({ ok: true }); });
  expect(app.queryByRole("dialog", { name: /^Save changes/ })).toBeNull();
});

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-11">
it.each(["button", "shared action"])("returns from full lineage through its %s without losing retained draft/layout", async method => {
  vi.spyOn(api, "projectDocs").mockResolvedValue([]);
  const registry = new UiActions({ workspace: "fixture", window: "lineage-window" }, () => true, async () => {});
  const app = render(<UiActionContext.Provider value={registry}><App /></UiActionContext.Provider>);
  await waitFor(() => expect(editor(app.container).hasFocus).toBe(true));
  const view = editor(app.container);
  act(() => view.dispatch({ changes: { from: 0, insert: "Retained lineage draft " }, selection: { anchor: 2, head: 7 } }));
  const bytes = view.state.doc.toString(), selection = view.state.selection.toJSON();
  const tabs = app.getAllByRole("tab").map(tab => tab.textContent);
  act(() => { location.hash = "/projects/qa/lineage"; window.dispatchEvent(new HashChangeEvent("hashchange")); });
  await app.findByRole("button", { name: "← Back to workspace" });
  expect((app.container.querySelector("[data-workspace]") as HTMLElement).hidden).toBe(true);
  if (method === "button") fireEvent.click(app.getByRole("button", { name: "← Back to workspace" }));
  else {
    const back = registry.observe("application-navigation", 64).actions.find(action => action.label === "Back to workspace")!;
    await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: back.handle, arguments: {}, request_id: "lineage-back" })).toMatchObject({ status: "result", outcome: { status: "completed" } }); });
  }
  await waitFor(() => expect(app.queryByRole("button", { name: "← Back to workspace" })).toBeNull());
  expect((app.container.querySelector("[data-workspace]") as HTMLElement).hidden).toBe(false);
  expect(editor(app.container)).toBe(view);
  expect(view.state.doc.toString()).toBe(bytes);
  expect(view.state.selection.toJSON()).toEqual(selection);
  expect(app.getAllByRole("tab").map(tab => tab.textContent)).toEqual(tabs);
});

</storycode:copy>
<a id="f-65ac24c24624-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-65ac24c24624-expression_statement-12">
it.each([["files", "hickory-show-files"], ["show-agent", "hickory-show-agent"]])("reveals the retained workspace before dispatching native %s from lineage", async (menu, event) => {
  vi.spyOn(api, "projectDocs").mockResolvedValue([]);
  const app = render(<App />);
  await waitFor(() => expect(editor(app.container).hasFocus).toBe(true));
  const view = editor(app.container);
  act(() => view.dispatch({ changes: { from: 0, insert: "Menu keeps this draft " } }));
  const bytes = view.state.doc.toString();
  act(() => { location.hash = "/projects/qa/lineage"; window.dispatchEvent(new HashChangeEvent("hashchange")); });
  await app.findByRole("button", { name: "← Back to workspace" });
  const observedVisibility: boolean[] = [];
  const observe = () => observedVisibility.push(!(app.container.querySelector("[data-workspace]") as HTMLElement).hidden);
  window.addEventListener(event, observe);
  try {
    fireEvent(window, new CustomEvent("hickory-menu", { detail: menu }));
    await waitFor(() => expect(observedVisibility).toEqual([true]));
    expect(app.queryByRole("button", { name: "← Back to workspace" })).toBeNull();
    expect(view.state.doc.toString()).toBe(bytes);
    if (menu === "show-agent") await app.findByRole("tab", { name: /Agent/ });
  } finally { window.removeEventListener(event, observe); }
});
</storycode:copy>

### Assemble `apps/web/src/App.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/App.test.tsx"><storycode:paste select="#f-65ac24c24624-expression_statement"/><storycode:paste select="#f-65ac24c24624-expression_statement-2"/><storycode:paste select="#f-65ac24c24624-expression_statement-3"/><storycode:paste select="#f-65ac24c24624-editor"/><storycode:paste select="#f-65ac24c24624-expression_statement-4"/><storycode:paste select="#f-65ac24c24624-expression_statement-5"/><storycode:paste select="#f-65ac24c24624-expression_statement-6"/><storycode:paste select="#f-65ac24c24624-expression_statement-7"/><storycode:paste select="#f-65ac24c24624-expression_statement-8"/><storycode:paste select="#f-65ac24c24624-expression_statement-9"/><storycode:paste select="#f-65ac24c24624-expression_statement-10"/><storycode:paste select="#f-65ac24c24624-expression_statement-11"/><storycode:paste select="#f-65ac24c24624-expression_statement-12"/></storycode:file>


</storycode:doc>
