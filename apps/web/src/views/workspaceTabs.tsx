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
import { DebugStrip } from "../debug/DebugStrip";
import { RefactorBadge } from "../components/RefactorBadge";
import { GeneratedFileView } from "../shell/views";
import { untitledPath, wrapUntitled } from "../lib/newDoc";
import { useDocSession, type SessionRegistry } from "./documentSession";

/** The document itself: debugger chrome on top, the collaborative editor
 * under it. One per "document" tab; the session it draws survives the tab. */
export function DocTabBody({ registry, docId }: { registry: SessionRegistry; docId: string }) {
  const session = useDocSession(registry, docId);
  if (!session) return <p className="muted">Loading document…</p>;
  if (session.fatalError) return <p className="error">{session.fatalError}</p>;
  const { doc, blocks, debug } = session;
  if (!doc || !blocks) return <p className="muted">Loading document…</p>;
  const running = session.runningCells.size > 0;

  return (
    <div className="debug-block">
      {/* This document's own actions, at the top of its tab: run and verify
          act on THIS document, whichever tab has the focus. A slim strip,
          like the debug strip below it. */}
      <div className="doc-tab-toolbar" role="toolbar" aria-label={`Actions for ${doc.path}`}>
        <button
          className="btn"
          disabled={running}
          onClick={session.runAll}
          data-tip="Run every cell in this document"
        >
          {running ? "Running…" : "Run"}
        </button>
        <button
          className="btn btn-primary"
          disabled={running}
          onClick={session.verify}
          data-tip="Re-run this document and verify it against its expectations"
        >
          Verify
        </button>
        {/* The equivalence gate for restructuring: pin a baseline, edit
            freely, and this badge reports the moment a woven byte would
            move. See serve/refactor.rs. */}
        <RefactorBadge docId={docId} />
        {session.syncState !== "idle" && (
          <span
            className={`save-state save-state-${session.syncState}`}
            role="status"
          >
            {session.syncState === "editing" ? "Saving…" : "Saved"}
          </span>
        )}
      </div>
      {/* The debugger's chrome, on the block being debugged. Nothing while
          idle: the file chip's Debug button is the way in. */}
      <DebugStrip
        status={debug.status}
        program={debug.program}
        message={debug.message}
        capabilities={debug.capabilities}
        frames={debug.frames}
        selectedFrame={debug.selectedFrame}
        watches={debug.watches}
        exitCode={debug.exitCode}
        onSelectFrame={session.selectFrameAndReveal}
        onStep={debug.step}
        onJumpHere={() => {
          // "Move here" acts on the caret, which is how a person says
          // *here* without a second control to pick a line.
          const editor = session.docEditor;
          if (!editor) return;
          const line = editor.state.doc.lineAt(editor.state.selection.main.head).number - 1;
          debug.jumpTo(line);
        }}
        onStart={() => debug.start(debug.program ?? undefined)}
        onStop={debug.stop}
        onAddWatch={() => {
          void session.askText("Expression to watch:", "").then((expression) => {
            if (expression) debug.addWatch(expression);
          });
        }}
        onRemoveWatch={debug.removeWatch}
      />
      <DocumentEditor
        key={docId}
        docId={docId}
        initialSource={doc.source}
        realtime={session.realtime}
        onChange={session.setDirtySource}
        selectSpan={session.selectSpan}
        execBlocks={session.execBlocks}
        runningCells={session.runningCells}
        onRunCell={session.runCell}
        lspExtensions={session.lspExtensions}
        lspDiagnostics={session.lspDiagnostics}
        onDebugFile={(path) => debug.start(path)}
        onViewReady={session.onDocViewReady}
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
      onReady={(target) => session.registerOutputView(path, target ? target.view : null)}
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
      const created = await api.createDoc(project.id, path, wrapUntitled(typed));
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
          Check that the app's folder is writable, then keep typing to retry.
          ({error})
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
