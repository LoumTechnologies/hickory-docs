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

import { useEffect, useMemo, useRef, useState } from "react";
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
        className="btn doc-tab-toolbar__more"
        aria-label="More actions for this document"
        aria-haspopup="menu"
        data-tip="More actions for this document"
        onClick={(e) => {
          const box = (e.currentTarget as HTMLElement).getBoundingClientRect();
          setMenuAt({ x: box.left, y: box.bottom + 2 });
        }}
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
            if (id === "run" && !running) onRun();
            if (id === "test" && !running) onTest();
            if (id === "review") void openLiterate({ kind: "document", doc_id: docId }, "HEAD").catch(e => setError(e instanceof Error ? e.message : String(e)));
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

// ---------------------------------------------------------------------------
// The untitled buffer, as a tab
// ---------------------------------------------------------------------------

const PLACEHOLDER =
  "Type markdown — # heading, **bold**, ```code```… Save when you name it.";

const NO_RUNNING_CELLS = new Set<string>();

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
