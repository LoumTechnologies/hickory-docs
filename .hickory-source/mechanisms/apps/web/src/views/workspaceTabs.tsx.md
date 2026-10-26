<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/workspaceTabs.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b5a48fd337ae"></a>
## `apps/web/src/views/workspaceTabs.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Read the retained branch of an agent conversation](../../../../../guarantees/agent/a-session-is-the-conversation/contract.md), [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [A Refactor Baseline Reports The Moment An Output Would Move](../../../../../guarantees/authoring/a-refactor-baseline-reports-drift/contract.md), [The App Starts As A Lightweight Editor](../../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md), [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [The Measure Says What It Measures](../../../../../guarantees/authoring/the-measure-says-what-it-measures/contract.md), [Install a supported missing debugger and resume the requested launch](../../../../../guarantees/debugging/a-missing-debugger-is-a-button/contract.md), [Choose a launch without replacing the editor being read](../../../../../guarantees/debugging/debug-controls-preserve-editor-space/contract.md), [Keep editor actions attached to the view they affect](../../../../../guarantees/editor-intelligence/actions-belong-to-their-subject/contract.md), [Run and test from the pane menu without adding a toolbar row](../../../../../guarantees/editor-intelligence/the-toolbar-uses-words-an-ide-user-knows/contract.md), [A failed action keeps its editor](../../../../../guarantees/execution/a-failed-action-keeps-its-editor/contract.md), [A view declares where edits land](../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md).

<a id="f-b5a48fd337ae-DocToolbar"></a>
### `DocToolbar`

<storycode:copy id="f-b5a48fd337ae-DocToolbar">
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { observationSourceChanged } from "../debug/observations";
import { PaneHeaderContent } from "../shell/PaneHeader";
import { SurfaceInfo } from "../components/SurfaceInfo";
import { ActionNotice } from "../components/ActionNotice";
// What the workspace's tabs render.
//
// Each body is a thin subscriber: the machinery lives in the document's
// session (views/documentSession.tsx), published through the registry, and
// these components draw whatever slice of it their tab shows. Kept apart
// from WorkspaceView so the workspace file stays about OWNING the layout
// rather than about what is inside any pane.

import { useId, useEffect, useMemo, useRef, useState } from "react";
import { openLiterate } from "../lib/openRepresentation";
import type { EditorView } from "@codemirror/view";

import { api } from "../api/client";
import { LocalRealtime } from "../api/realtime";
import { DocumentEditor } from "../editor/DocumentEditor";
import type { TableLayout } from "../components/TablePanel";
import { DebugStrip } from "../debug/DebugStrip";
import { ContextMenu } from "../components/ContextMenu";
import type { ContextMenuItem } from "../components/ContextMenu";
import { RefactorBadge, useBaseline } from "../components/RefactorBadge";
import type { Baseline } from "../components/RefactorBadge";
import { GeneratedFileView } from "../shell/views";
import { registerBufferEditor } from "../lib/editorBufferTargets";
import { markUntitledEditor } from "../editor/activeEditor";
import { focusWhenShown } from "../lib/focusWhenShown";
import { useDocSession, type SessionRegistry } from "./documentSession";

/** Document-wide actions share the pane header; cell actions stay on their rail. */
function DocToolbar({
  path,
  running,
  onRun,
  onTest,
  baseline,
  syncState,
  docId,
}: {
  path: string;
  running: boolean;
  onRun: () => void;
  onTest: () => void;
  baseline: Baseline;
  syncState: string;
  docId: string;
}) {
  const [menuAt, setMenuAt] = useState<{ x: number; y: number } | null>(null);
  const [error, setError] = useState("");
  const launcher = useRef<HTMLButtonElement>(null);
  const parent = `document-tab-menu:${useId()}`, key = `${parent}:open`;
  const revision = useUiRevision(() => [path, docId, running, menuAt, baseline.active, baseline.busy]);
  const invoke = useUiCommands([{ key, command: {
    parent, subject: parent, label: "More actions for this document", parameters: {}, prerequisites: ["Current mounted document tab header"],
    read: () => ({ revision: revision(), enabled: !!launcher.current?.isConnected, value: { open: !!menuAt } }),
    execute: async () => {
      if (!launcher.current?.isConnected) return { status: "unavailable", reason: "Document tab header retired" };
      const box = launcher.current.getBoundingClientRect();
      setMenuAt({ x: box.left, y: box.bottom + 2 });
      return { status: "completed", value: { open_requested: true, source_changed: false } };
    },
  } }]);
  useUiReading("document tab header status", { path, doc_id: docId, sync_state: syncState, error, menu_open: !!menuAt });

  const items: ContextMenuItem[] = [
    { id: "run", label: running ? "Running…" : "Run", disabled: running, tip: "Run every cell and rewrite its generated files." },
    { id: "test", label: "Test", disabled: running, tip: "Re-run and check expectations and output drift, like hick test." },
    { id: "review", label: "Review changes", tip: "Compare this document with HEAD in an editable literate view." },
    baseline.active
      ? {
          id: "baseline-off",
          disabled: baseline.busy,
          label: "Stop comparing to the baseline",
          tip: "The outputs as they stand become the new truth.",
        }
      : {
          id: "baseline-on",
          disabled: baseline.busy,
          label: "Pin outputs as a baseline",
          tip:
            "Restructure freely; a badge reports the moment a generated " +
            "byte would change. The same check as `hick equiv`.",
        },
  ];

  return (
    <PaneHeaderContent part="actions"><div className="doc-tab-toolbar" role="toolbar" aria-label={`Actions for ${path}`}>
      {error && <span role="alert">{error}</span>}
      <button
        ref={launcher}
        className="btn doc-tab-toolbar__more"
        aria-label="More actions for this document"
        aria-haspopup="menu"
        data-tip="More actions for this document"
        onClick={() => { void invoke(key).catch(cause => setError(String(cause))); }}
      >
        ⋯
      </button>
      {menuAt && (
        <ContextMenu
          x={menuAt.x}
          y={menuAt.y}
          items={items}
          subject={path}
          onPick={(id) => {
            setMenuAt(null);
            if (id === "run" && !running) return onRun();
            if (id === "test" && !running) return onTest();
            if (id === "review") return openLiterate({ kind: "document", doc_id: docId }, "HEAD").catch(e => { setError(e instanceof Error ? e.message : String(e)); throw e; });
            if (id === "baseline-on") return baseline.begin();
            if (id === "baseline-off") return baseline.end();
          }}
          onClose={() => setMenuAt(null)}
        />
      )}
      {/* Only once there is a verdict: an idle mode advertising itself in a
          toolbar is a slot spent on a constant. */}
      <RefactorBadge baseline={baseline} />
      {syncState !== "idle" && (
        <span className={`save-state save-state-${syncState}`} role="status">
          {syncState === "editing" ? "Saving…" : syncState === "failed" ? "Save not confirmed" : "Saved to disk"}
        </span>
      )}
    </div></PaneHeaderContent>
  );
}

</storycode:copy>
<a id="f-b5a48fd337ae-DocTabBody"></a>
### `DocTabBody`

The document itself: debugger chrome on top, the collaborative editor
under it. One per "document" tab; the session it draws survives the tab.

<storycode:copy id="f-b5a48fd337ae-DocTabBody">
/** The document itself: debugger chrome on top, the collaborative editor
 * under it. One per "document" tab; the session it draws survives the tab. */
export function DocTabBody({
  registry,
  docId,
  wrapColumn,
  onWrapColumn,
  tableLayouts,
  onTableLayout,
}: {
  registry: SessionRegistry;
  docId: string;
  /** Where prose wraps in this tab, restored with the arrangement. */
  wrapColumn?: number;
  onWrapColumn?: (column: number) => void;
  /** How big each table in this document was left, and where to report a
   * resize. Presentation, held by the workspace and never by the document. */
  tableLayouts?: Record<string, TableLayout>;
  onTableLayout?: (key: string, size: TableLayout) => void;
}) {
  const session = useDocSession(registry, docId);
  // Before the early returns: hooks may not be conditional, and the baseline
  // outlives whatever the session is doing.
  const baseline = useBaseline(docId);
  const loadMessage = session?.fatalError || (!session?.doc || !session.blocks ? "Loading document…" : null);
  useUiReading("document tab load state", { doc_id: docId, phase: session?.fatalError ? "failed" : loadMessage ? "loading" : "loaded", message: loadMessage });
  if (!session) return <p className="muted">Loading document…</p>;
  if (session.fatalError) return <p className="error">{session.fatalError}</p>;
  const { doc, blocks, debug } = session;
  if (!doc || !blocks) return <p className="muted">Loading document…</p>;
  const running = session.runningCells.size > 0;

  return (
    <div className="debug-block">
      <SurfaceInfo name={`Document · ${doc.path}`} editable destination="Edits save automatically to this document on disk." />
      <DocToolbar
        docId={docId}
        path={doc.path}
        running={running}
        onRun={session.runAll}
        onTest={session.verify}
        baseline={baseline}
        syncState={session.syncState}
      />
      {session.banner && <ActionNotice kind={session.banner.kind} retry={session.banner.retry}>{session.banner.text}</ActionNotice>}
      {session.renderError && <ActionNotice kind="fail" retry={{ label: "Retry rendering", run: session.refresh }}>Could not render {doc.path}. Editing and saving remain available. {session.renderError}</ActionNotice>}
      {session.syncState === "failed" && session.banner?.retry?.label !== "Retry Save" && <ActionNotice kind="fail" retry={{ label: "Retry Save", run: () => { void session.menuSave(); } }}>Saving could not be confirmed. Your text is still in this editor.</ActionNotice>}
      {/* The debugger's chrome, on the block being debugged. Nothing while
          idle: the file chip's Debug button is the way in. */}
      <DebugStrip reading={debug.reading} readingChanged={observationSourceChanged(debug.reading, session.liveSource)}
        editorView={session.docEditor}
        pausedLine={debug.pausedLine}
        status={debug.status}
        program={debug.program}
        launchTarget={debug.launchTarget}
        message={debug.message}
        offerInstall={debug.offerInstall}
        onInstall={offer => api.installTool(offer.kind, offer.language).then(() => undefined)}
        capabilities={debug.capabilities}
        frames={debug.frames}
        selectedFrame={debug.selectedFrame}
        watches={debug.watches}
        exitCode={debug.exitCode}
        buildOutput={debug.buildOutput}
        onSelectFrame={session.selectFrameAndReveal}
        onStep={debug.step}
        onJumpHere={() => {
          // "Move here" acts on the caret, which is how a person says
          // *here* without a second control to pick a line.
          const editor = session.docEditor;
          if (!editor) return;
          const line =
            editor.state.doc.lineAt(editor.state.selection.main.head).number -
            1;
          debug.jumpTo(line);
        }}
        onStart={() => {if(debug.program)session.startDebugFile(debug.program);}}
        onStop={debug.stop}
        onAddWatch={() => {
          void session
            .askText("Expression to watch:", "")
            .then((expression) => {
              if (expression) debug.addWatch(expression);
            });
        }}
        exceptionFilters={debug.exceptionFilters}
        onToggleExceptionFilter={debug.toggleExceptionFilter}
        onRemoveWatch={debug.removeWatch}
      />
      {/* A session file opens in the same editor as every other document —
          drawn as the conversation it is (bubbles, the agent's work folded;
          see editor/wysiwyg.ts and editor/folding.ts) with every byte on
          screen, because the file IS the conversation and a second surface
          would be a second story. */}
      <DocumentEditor
        key={docId}
        docId={docId}
        initialSource={doc.source}
        realtime={session.realtime}
        onChange={session.setDirtySource}
        selectSpan={session.selectSpan}
        execBlocks={session.execBlocks}
        diagramBlocks={session.diagramBlocks}
        runningCells={session.runningCells}
        onRunCell={session.runCell}
        lspExtensions={session.lspExtensions}
        lspDiagnostics={session.lspDiagnostics}
        lspCompletion={session.lspCompletion}
        onDebugFile={session.startDebugFile}
        onViewReady={session.onDocViewReady}
        wrapColumn={wrapColumn}
        onWrapColumn={onWrapColumn}
        tableLayouts={tableLayouts}
        onTableLayout={onTableLayout}
        path={doc.path}
      />
    </div>
  );
}

</storycode:copy>
<a id="f-b5a48fd337ae-GeneratedTabBody"></a>
### `GeneratedTabBody`

One generated file, owned by `docId` — the tab carries the owner, so the
fetch, the debounced save, and the provenance all reach the right
document when several are open at once.

<storycode:copy id="f-b5a48fd337ae-GeneratedTabBody">
/** One generated file, owned by `docId` — the tab carries the owner, so the
 * fetch, the debounced save, and the provenance all reach the right
 * document when several are open at once. */
export function GeneratedTabBody({
  registry,
  docId,
  path,
  bufferId,
  onSource,
}: {
  registry: SessionRegistry;
  docId: string;
  path: string;
  bufferId?: string;
  onSource?: (path: string, source: string) => void;
}) {
  const session = useDocSession(registry, docId);
  useUiReading("generated tab session state", { doc_id: docId, path, session_loaded: !!session, message: session ? null : `Loading ${path}…` });
  // The session appears one tick after its host mounts; the file fetch is
  // cheap enough to wait for the LSP wiring rather than mount without it.
  if (!session) return <p className="muted">Loading {path}…</p>;
  return (
    <GeneratedFileView
      docId={docId}
      path={path}
      bufferId={bufferId}
      onSource={onSource}
      liveFile={session.outputs.get(path) ?? null}
      debugger={session.doc ? {debug:session.debug,source:session.liveSource,docPath:session.doc.path,askText:session.askText} : undefined}
      makeOutputLsp={session.makeOutputLsp}
      onSourceEdits={session.flashSourceEdits}
      onReady={(target) =>
        session.registerOutputView(path, target ? target.view : null)
      }
    />
  );
}

</storycode:copy>
<a id="f-b5a48fd337ae-PLACEHOLDER"></a>
### `PLACEHOLDER`

<storycode:copy id="f-b5a48fd337ae-PLACEHOLDER">
// ---------------------------------------------------------------------------
// The untitled buffer, as a tab
// ---------------------------------------------------------------------------

const PLACEHOLDER =
  "Type markdown — # heading, **bold**, ```code```… Save when you name it.";

</storycode:copy>
<a id="f-b5a48fd337ae-NO_RUNNING_CELLS"></a>
### `NO_RUNNING_CELLS`

<storycode:copy id="f-b5a48fd337ae-NO_RUNNING_CELLS">
const NO_RUNNING_CELLS = new Set<string>();

</storycode:copy>
<a id="f-b5a48fd337ae-UntitledTab"></a>
### `UntitledTab`

A document that does not exist yet, in a tab of the workspace it will
join.
The buffer holds only prose — no wrapper tags, no file on disk — so the
first thing a new person faces is a place to type markdown, not XML. The
real document is created only when the person saves and names it. Until
then it is a draft in the user's workspace state — outside the folder and
therefore outside git — so closing the app cannot lose a thought or turn
one into an unexpected `untitled.md`.

<storycode:copy id="f-b5a48fd337ae-UntitledTab">
/**
 * A document that does not exist yet, in a tab of the workspace it will
 * join.
 *
 * The buffer holds only prose — no wrapper tags, no file on disk — so the
 * first thing a new person faces is a place to type markdown, not XML. The
 * real document is created only when the person saves and names it. Until
 * then it is a draft in the user's workspace state — outside the folder and
 * therefore outside git — so closing the app cannot lose a thought or turn
 * one into an unexpected `untitled.md`.
 */
export function UntitledTab({
  tabId,
  initialSource = "",
  onSource,
  wrapColumn,
  onWrapColumn,
}: {
  tabId: string;
  initialSource?: string;
  onSource?: (tabId: string, source: string) => void;
  wrapColumn?: number;
  onWrapColumn?: (column: number) => void;
}) {
  // A local, client-seeded realtime: there is no server room to join until
  // the document exists, and the CRDT is happy with one writer.
  const realtime = useMemo(() => new LocalRealtime(), []);
  useEffect(() => () => realtime.close(), [realtime]);

  const viewRef = useRef<EditorView | null>(null);
  const unregisterBuffer = useRef(() => {});
  useEffect(() => () => unregisterBuffer.current(), []);
  const cancelFocus = useRef(() => {});
  useEffect(() => () => cancelFocus.current(), []);
  return (
    <div className="untitled-tab">
      <SurfaceInfo name="Untitled document" editable destination="Save names a .md file. Recovery drafts are kept outside the project." />
      <DocumentEditor
        docId="untitled"
        initialSource={initialSource}
        wrapColumn={wrapColumn}
        onWrapColumn={onWrapColumn}
        realtime={realtime}
        placeholderText={PLACEHOLDER}
        execBlocks={[]}
        runningCells={NO_RUNNING_CELLS}
        onRunCell={() => {}}
        onViewReady={(view) => {
          viewRef.current = view;
          markUntitledEditor(view);
          unregisterBuffer.current();
          unregisterBuffer.current = view ? registerBufferEditor(tabId, view) : () => {};
          // Land ready to type — an empty editor you still have to click
          // into is a chooser with extra steps.
          cancelFocus.current();
          cancelFocus.current = view ? focusWhenShown(view) : () => {};
        }}
        onChange={(source) => onSource?.(tabId, source)}
      />
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/views/workspaceTabs.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceTabs.tsx"><storycode:paste select="#f-b5a48fd337ae-DocToolbar"/><storycode:paste select="#f-b5a48fd337ae-DocTabBody"/><storycode:paste select="#f-b5a48fd337ae-GeneratedTabBody"/><storycode:paste select="#f-b5a48fd337ae-PLACEHOLDER"/><storycode:paste select="#f-b5a48fd337ae-NO_RUNNING_CELLS"/><storycode:paste select="#f-b5a48fd337ae-UntitledTab"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-5370130a675d"></a>
## `apps/web/src/views/workspaceTabs.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [A write exposes its proposed effects](../../../../../guarantees/authoring/a-write-exposes-its-proposed-effects/contract.md), [Conflict actions name the write](../../../../../guarantees/authoring/conflict-actions-name-the-write/contract.md), [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [The Measure Says What It Measures](../../../../../guarantees/authoring/the-measure-says-what-it-measures/contract.md), [Keep editor actions attached to the view they affect](../../../../../guarantees/editor-intelligence/actions-belong-to-their-subject/contract.md), [Run and test from the pane menu without adding a toolbar row](../../../../../guarantees/editor-intelligence/the-toolbar-uses-words-an-ide-user-knows/contract.md), [A failed action keeps its editor](../../../../../guarantees/execution/a-failed-action-keeps-its-editor/contract.md), [A run result is distinct from what was checked](../../../../../guarantees/execution/a-run-result-is-distinct-from-what-was-checked/contract.md), [A view declares where edits land](../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md).

<a id="f-5370130a675d-readyEditor"></a>
### `readyEditor`

@vitest-environment jsdom
Document-wide actions live in the pane header's menu and target its session.

<storycode:copy id="f-5370130a675d-readyEditor">
// @vitest-environment jsdom
//
// Document-wide actions live in the pane header's menu and target its session.
// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md

import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render } from "@testing-library/react";

import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import type { Doc } from "../api/types";
import { api } from "../api/client";
import { SessionRegistry, type DocSession } from "./documentSession";
import { DocTabBody, GeneratedTabBody, UntitledTab } from "./workspaceTabs";

const readyEditor = vi.hoisted(() => ({ view: null as import("@codemirror/view").EditorView | null }));

</storycode:copy>
<a id="f-5370130a675d-expression_statement"></a>
### `expression_statement`

The editor and debugger chrome are heavyweight and irrelevant here; the
toolbar sits beside them in DocTabBody, not inside them.

<storycode:copy id="f-5370130a675d-expression_statement">
// The editor and debugger chrome are heavyweight and irrelevant here; the
// toolbar sits beside them in DocTabBody, not inside them.
vi.mock("../editor/DocumentEditor", () => ({
  DocumentEditor: ({ onChange, wrapColumn, onWrapColumn, onViewReady }: {
    onViewReady?: (view: EditorView | null) => void;
    onChange?: (source: string) => void;
    wrapColumn?: number;
    onWrapColumn?: (column: number) => void;
  }) => (
    <>
      <button data-testid="ready" onClick={() => onViewReady?.(readyEditor.view)} />
      <button data-testid="editor" onClick={() => onChange?.("first thought")} />
      <button data-testid="wrap" onClick={() => onWrapColumn?.(60)}>{wrapColumn}</button>
    </>
  ),
}));
</storycode:copy>
<a id="f-5370130a675d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-5370130a675d-expression_statement-2">
vi.mock("../debug/DebugStrip", () => ({
  DebugStrip: () => <div data-testid="debug-strip" />,
}));
</storycode:copy>
<a id="f-5370130a675d-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-5370130a675d-expression_statement-3">
vi.mock("../shell/views", () => ({
  GeneratedFileView: () => <div data-testid="generated" />,
}));

</storycode:copy>
<a id="f-5370130a675d-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-5370130a675d-expression_statement-4">
afterEach(cleanup);

</storycode:copy>
<a id="f-5370130a675d-doc"></a>
### `doc`

<storycode:copy id="f-5370130a675d-doc">
const doc: Doc = {
  id: "d1",
  path: "notes/demo.md",
  source: "<hick:doc></hick:doc>",
} as Doc;

</storycode:copy>
<a id="f-5370130a675d-makeSession"></a>
### `makeSession`

A published session with just enough shape for the tab bodies.

<storycode:copy id="f-5370130a675d-makeSession">
/** A published session with just enough shape for the tab bodies. */
function makeSession(overrides: Partial<DocSession> = {}): DocSession {
  return {
    docId: "d1",
    doc,
    liveSource: doc.source,
    startDebugFile: vi.fn(),
    blocks: [],
    fatalError: null,
    renderError: null,
    banner: null,
    syncState: "idle",
    runningCells: new Set<string>(),
    execBlocks: [],
    outputs: new Map(),
    openOutputs: new Map(),
    docEditor: null,
    debug: {},
    lspExtensions: [],
    runAll: vi.fn(),
    verify: vi.fn(),
    runCell: vi.fn(),
    ...overrides,
  } as unknown as DocSession;
}

</storycode:copy>
<a id="f-5370130a675d-mount"></a>
### `mount`

<storycode:copy id="f-5370130a675d-mount">
function mount(session: DocSession) {
  const registry = new SessionRegistry();
  registry.publish(session);
  return render(<DocTabBody registry={registry} docId={session.docId} />);
}

</storycode:copy>
<a id="f-5370130a675d-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-5370130a675d-expression_statement-5">
describe("the document tab's toolbar", () => {
  // Protects docs/guarantees/editor-intelligence/the-toolbar-uses-words-an-ide-user-knows.md

  it("offers Run and Test, and nothing a traditional IDE would misread", () => {
    const { getByRole, queryByRole } = mount(makeSession());
    const toolbar = getByRole("toolbar", { name: "Actions for notes/demo.md" });
    expect(toolbar.className).toBe("doc-tab-toolbar");
    expect(queryByRole("button", { name: "Run" })).toBeNull();
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    expect(getByRole("menuitem", { name: "Run" })).toBeTruthy();
    // `hick test` is the command; the button says the same word, so each one
    // is findable from the other.
    expect(getByRole("menuitem", { name: "Test" })).toBeTruthy();
    expect(queryByRole("button", { name: "Verify" })).toBeNull();
    // "Refactor" means rename/extract in every other editor. It named a mode
    // here, and a mode belongs in a menu.
    expect(queryByRole("button", { name: "Refactor" })).toBeNull();
  });

  it("keeps the rare document-wide action in a menu", () => {
    const { getByRole, getByText } = mount(makeSession());
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    // Named by what it produces, not by a mode nobody can search for.
    expect(getByText("Pin outputs as a baseline")).toBeTruthy();
  });

  it("calls THIS session's runAll and verify", () => {
    const session = makeSession();
    const { getByRole } = mount(session);
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    fireEvent.click(getByRole("menuitem", { name: "Run" }));
    expect(session.runAll).toHaveBeenCalledTimes(1);
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    fireEvent.click(getByRole("menuitem", { name: "Test" }));
    expect(session.verify).toHaveBeenCalledTimes(1);
  });

  it("disables both buttons while the document's run is in flight", () => {
    const session = makeSession({ runningCells: new Set(["cell-1"]) });
    const { getByRole } = mount(session);
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    const run = getByRole("menuitem", { name: "Running…" }) as HTMLButtonElement;
    const test = getByRole("menuitem", { name: "Test" }) as HTMLButtonElement;
    expect(run.disabled).toBe(true);
    expect(test.disabled).toBe(true);
    fireEvent.click(run);
    fireEvent.click(test);
    expect(session.runAll).not.toHaveBeenCalled();
    expect(session.verify).not.toHaveBeenCalled();
  });

  it("gives a generated-file tab no toolbar — the actions are the document's", () => {
    const registry = new SessionRegistry();
    registry.publish(makeSession());
    const { queryByRole } = render(
      <GeneratedTabBody registry={registry} docId="d1" path="src/hello.py" />,
    );
    expect(queryByRole("toolbar")).toBeNull();
  });
});

</storycode:copy>
<a id="f-5370130a675d-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-5370130a675d-expression_statement-6">
describe("an untitled document", () => {
  // Guarantee: docs/guarantees/authoring/the-measure-says-what-it-measures.md
  it("passes the stored prose measure and ruler changes through to its editor", () => {
    const onWrapColumn = vi.fn();
    const { getByTestId, rerender } = render(
      <UntitledTab tabId="new-note" wrapColumn={72} onWrapColumn={onWrapColumn} />,
    );
    expect(getByTestId("wrap").textContent).toBe("72");
    fireEvent.click(getByTestId("wrap"));
    expect(onWrapColumn).toHaveBeenCalledWith(60);
    rerender(<UntitledTab tabId="new-note" wrapColumn={60} onWrapColumn={onWrapColumn} />);
    expect(getByTestId("wrap").textContent).toBe("60");
  });

  // Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
  it("does not create a project file on its first keystroke", () => {
    const create = vi.spyOn(api, "createDoc");
    const { getByTestId } = render(<UntitledTab tabId="new-note" />);
    fireEvent.click(getByTestId("editor"));
    expect(create).not.toHaveBeenCalled();
    create.mockRestore();
  });

  it("never restores a prior unnamed draft into a new document", () => {
    const drafts = vi.spyOn(api, "drafts").mockResolvedValue({
      drafts: [
        {
          path: "untitled:closed-tab",
          contents: "a thought that still matters",
          base: "",
          saved_at: 42,
        },
      ],
    });
    render(<UntitledTab tabId="reopened-tab" />);

    expect(drafts).not.toHaveBeenCalled();

    drafts.mockRestore();
  });
});

</storycode:copy>
<a id="f-5370130a675d-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-5370130a675d-expression_statement-7">
// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("does not focus an unnamed editor that finishes loading in a hidden pane", () => {
  const ui = render(<div hidden><UntitledTab tabId="background-note" /></div>);
  const parent = document.createElement("div");
  ui.container.querySelector(".untitled-tab")!.appendChild(parent);
  const view = new EditorView({ parent });
  const focus = vi.spyOn(view, "focus");
  readyEditor.view = view;
  fireEvent.click(ui.getByTestId("ready"));
  expect(focus).not.toHaveBeenCalled();
  ui.rerender(<div><UntitledTab tabId="background-note" /></div>);
  fireEvent.click(ui.getByTestId("ready"));
  expect(focus).toHaveBeenCalledOnce();
  readyEditor.view = null;
  view.destroy();
});

</storycode:copy>
<a id="f-5370130a675d-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-5370130a675d-expression_statement-8">
it("shares the document header launcher and disclosed menu callback with agent requests", async () => {
  const session = makeSession(), sessions = new SessionRegistry(); sessions.publish(session);
  const registry = new UiActions({ workspace: "p", window: "doc-menu" }, () => true, async () => {});
  const mounted = render(<UiActionContext.Provider value={registry}><DocTabBody registry={sessions} docId="d1" /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:tabs", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: {}, request_id: String(++id) }); }); return reply; };
  expect(actions().some(action => action.label === "Run")).toBe(false);
  await run("More actions for this document");
  expect(mounted.getByRole("menuitem", { name: "Run" })).toBeTruthy();
  const old = actions().find(action => action.label === "Run")!;
  expect(await run("Run")).toMatchObject({ outcome: { status: "completed" } });
  expect(session.runAll).toHaveBeenCalledTimes(1);
  expect(await registry.act(registry.scope, { id: "acp:tabs", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).not.toMatchObject({ outcome: { status: "completed" } });
});

</storycode:copy>
<a id="f-5370130a675d-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-5370130a675d-expression_statement-9">
it("reads actual loading/fatal states before the document editor exists without including source", async () => {
  const sessions = new SessionRegistry(), registry = new UiActions({ workspace: "p", window: "loading" }, () => true, async () => {});
  const mounted = render(<UiActionContext.Provider value={registry}><DocTabBody registry={sessions} docId="d1" /></UiActionContext.Provider>);
  let id = 0;
  const read = async () => { const action = registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === "Read document tab load state bytes")!; let reply: any; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:status", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: String(++id) }); }); return new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes)); };
  expect(JSON.parse(await read())).toMatchObject({ phase: "loading", message: "Loading document…" });
  await act(async () => sessions.publish(makeSession({ fatalError: "Document cannot be opened", liveSource: "NEVER INCLUDE SOURCE" })));
  expect(mounted.getByText("Document cannot be opened")).toBeTruthy();
  const failed = await read(); expect(JSON.parse(failed)).toMatchObject({ phase: "failed", message: "Document cannot be opened" });
  expect(failed).not.toContain("NEVER INCLUDE SOURCE");
});
</storycode:copy>

### Assemble `apps/web/src/views/workspaceTabs.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceTabs.test.tsx"><storycode:paste select="#f-5370130a675d-readyEditor"/><storycode:paste select="#f-5370130a675d-expression_statement"/><storycode:paste select="#f-5370130a675d-expression_statement-2"/><storycode:paste select="#f-5370130a675d-expression_statement-3"/><storycode:paste select="#f-5370130a675d-expression_statement-4"/><storycode:paste select="#f-5370130a675d-doc"/><storycode:paste select="#f-5370130a675d-makeSession"/><storycode:paste select="#f-5370130a675d-mount"/><storycode:paste select="#f-5370130a675d-expression_statement-5"/><storycode:paste select="#f-5370130a675d-expression_statement-6"/><storycode:paste select="#f-5370130a675d-expression_statement-7"/><storycode:paste select="#f-5370130a675d-expression_statement-8"/><storycode:paste select="#f-5370130a675d-expression_statement-9"/></storycode:file>


</storycode:doc>
