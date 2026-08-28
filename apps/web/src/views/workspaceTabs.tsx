// What the workspace's tabs render.
//
// Each body is a thin subscriber: the machinery lives in the document's
// session (views/documentSession.tsx), published through the registry, and
// these components draw whatever slice of it their tab shows. Kept apart
// from WorkspaceView so the workspace file stays about OWNING the layout
// rather than about what is inside any pane.

import { useEffect, useMemo, useRef, useState } from "react";
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
import { untitledPath, wrapUntitled } from "../lib/newDoc";
import { useDocSession, type SessionRegistry } from "./documentSession";

/**
 * A document's own actions, at the top of its tab.
 *
 * Two buttons and a menu, and the split is deliberate. The toolbar used to
 * read `Run · Verify · Refactor`, with **Verify** as the primary — and to
 * anyone arriving from a traditional IDE, two of those three were wrong.
 *
 * - `Run` is the everyday verb and is now the primary one, because it is what
 *   a person presses fifty times a day.
 * - `Verify` is what `hick test` does, so it says **Test**. Matching the CLI
 *   verb is worth more than a word of our own: somebody who reads the toolbar
 *   can type the command, and somebody who reads the command can find the
 *   button.
 * - `Refactor` named a *mode*, and in every other editor that word opens
 *   rename / extract / inline. It is now an item in the overflow menu that
 *   says what it produces — a baseline — and the toolbar shows the verdict
 *   only once there is one.
 *
 * The menu is where a document-wide action goes when it is real but rare. A
 * toolbar is read every time the tab is opened; anything on it is being
 * charged to every reader forever.
 */
function DocToolbar({
  path,
  running,
  onRun,
  onTest,
  baseline,
  syncState,
}: {
  path: string;
  running: boolean;
  onRun: () => void;
  onTest: () => void;
  baseline: Baseline;
  syncState: string;
}) {
  const [menuAt, setMenuAt] = useState<{ x: number; y: number } | null>(null);

  const items: ContextMenuItem[] = [
    baseline.active
      ? {
          id: "baseline-off",
          label: "Stop comparing to the baseline",
          tip: "The outputs as they stand become the new truth.",
        }
      : {
          id: "baseline-on",
          label: "Pin outputs as a baseline",
          tip:
            "Restructure freely; a badge reports the moment a generated " +
            "byte would change. The same check as `hick equiv`.",
        },
  ];

  return (
    <div className="doc-tab-toolbar" role="toolbar" aria-label={`Actions for ${path}`}>
      <button
        className="btn btn-primary"
        disabled={running}
        onClick={onRun}
        data-tip="Run every cell in this document and rewrite the files it generates"
      >
        {running ? "Running…" : "Run"}
      </button>
      <button
        className="btn"
        disabled={running}
        onClick={onTest}
        data-tip="Re-run this document and check it against its own expectations, the way `hick test` does. Fails if a claim stopped being true or a committed output drifted."
      >
        Test
      </button>
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
            if (id === "baseline-on") baseline.begin();
            if (id === "baseline-off") baseline.end();
          }}
          onClose={() => setMenuAt(null)}
        />
      )}
      {/* Only once there is a verdict: an idle mode advertising itself in a
          toolbar is a slot spent on a constant. */}
      <RefactorBadge baseline={baseline} />
      {syncState !== "idle" && (
        <span className={`save-state save-state-${syncState}`} role="status">
          {syncState === "editing" ? "Saving…" : "Saved"}
        </span>
      )}
    </div>
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
      <DocToolbar
        path={doc.path}
        running={running}
        onRun={session.runAll}
        onTest={session.verify}
        baseline={baseline}
        syncState={session.syncState}
      />
      {/* The debugger's chrome, on the block being debugged. Nothing while
          idle: the file chip's Debug button is the way in. */}
      <DebugStrip
        status={debug.status}
        program={debug.program}
        message={debug.message}
        offerInstall={debug.offerInstall}
        onInstall={async (offer) => {
          await api.installTool(offer.kind, offer.language);
          // Straight back into the session that failed: installing and then
          // asking somebody to press Debug again is the same missing step
          // this button exists to remove.
          debug.start(debug.program ?? undefined);
        }}
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
        onStart={() => debug.start(debug.program ?? undefined)}
        onStop={debug.stop}
        onAddWatch={() => {
          void session
            .askText("Expression to watch:", "")
            .then((expression) => {
              if (expression) debug.addWatch(expression);
            });
        }}
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
        onDebugFile={(path) => debug.start(path)}
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
}: {
  registry: SessionRegistry;
  docId: string;
  path: string;
}) {
  const session = useDocSession(registry, docId);
  // The session appears one tick after its host mounts; the file fetch is
  // cheap enough to wait for the LSP wiring rather than mount without it.
  if (!session) return <p className="muted">Loading {path}…</p>;
  return (
    <GeneratedFileView
      docId={docId}
      path={path}
      liveFile={session.outputs.get(path) ?? null}
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
  "Type markdown — # heading, **bold**, ```code```… It's saved as you type.";

const NO_RUNNING_CELLS = new Set<string>();

/**
 * A document that does not exist yet, in a tab of the workspace it will
 * join.
 *
 * The buffer holds only prose — no wrapper tags, no file on disk — so the
 * first thing a new person faces is a place to type markdown, not XML. The
 * real document is created on the FIRST edit: named `untitled.hick`
 * (counting past whatever the folder already holds), wrapped in the standard
 * `<hick:doc>` envelope with the typed prose inside — then `onCreated` lets
 * the workspace adopt this very tab as the created document's, in place.
 * Until that first keystroke, closing the app leaves nothing behind.
 */
export function UntitledTab({
  tabId,
  onCreated,
}: {
  tabId: string;
  /** The buffer became a real document: adopt this tab. */
  onCreated: (tabId: string, docId: string, path: string) => void;
}) {
  // A local, client-seeded realtime: there is no server room to join until
  // the document exists, and the CRDT is happy with one writer.
  const realtime = useMemo(() => new LocalRealtime(), []);
  useEffect(() => () => realtime.close(), [realtime]);

  const viewRef = useRef<EditorView | null>(null);
  // The create fires exactly once; a failure re-arms it so the next
  // keystroke retries without ever touching the typed text.
  const creatingRef = useRef(false);
  const [error, setError] = useState<string | null>(null);
  const onCreatedRef = useRef(onCreated);
  onCreatedRef.current = onCreated;

  const createFromFirstEdit = async () => {
    try {
      const projects = await api.projects();
      const project = projects[0];
      if (!project) throw new Error("no folder is open");
      const docs = await api.projectDocs(project.id).catch(() => []);
      const path = untitledPath(docs.map((d) => d.path));
      const typed = viewRef.current?.state.doc.toString() ?? "";
      const created = await api.createDoc(
        project.id,
        path,
        wrapUntitled(typed),
      );
      // Keystrokes that landed while the create was in flight are replayed
      // with a save; the room the document tab opens seeds from the server's
      // copy, so it must hold everything typed before the handoff.
      const latest = viewRef.current?.state.doc.toString() ?? typed;
      if (latest !== typed) await api.saveDoc(created.id, wrapUntitled(latest));
      onCreatedRef.current(tabId, created.id, created.path);
    } catch (e) {
      creatingRef.current = false;
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="untitled-tab">
      {error && (
        <div className="banner banner-fail" role="status">
          Could not create the document — your text is still in this buffer.
          Check that the app's folder is writable, then keep typing to retry. (
          {error})
        </div>
      )}
      <DocumentEditor
        docId="untitled"
        initialSource=""
        realtime={realtime}
        placeholderText={PLACEHOLDER}
        execBlocks={[]}
        runningCells={NO_RUNNING_CELLS}
        onRunCell={() => {}}
        onViewReady={(view) => {
          viewRef.current = view;
          // Land ready to type — an empty editor you still have to click
          // into is a chooser with extra steps.
          view?.focus();
        }}
        onChange={(source) => {
          if (creatingRef.current || source.length === 0) return;
          creatingRef.current = true;
          void createFromFirstEdit();
        }}
      />
    </div>
  );
}
