<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/documentSession.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bfb59577f48b"></a>
## `apps/web/src/views/documentSession.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](../../../../../guarantees/authoring/a-keystroke-redraws-only-the-editor/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [Debug the latest atomic project that still contains the displayed bytes](../../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md), [A failed action keeps its editor](../../../../../guarantees/execution/a-failed-action-keeps-its-editor/contract.md), [An Element Is Drawn By Its View](../../../../../guarantees/language/an-element-is-drawn-by-its-view/contract.md).

<a id="f-bfb59577f48b-export_statement"></a>
### `export_statement`

<storycode:copy id="f-bfb59577f48b-export_statement">
import { useDocumentRender } from "./useDocumentRender";
import { runCellFresh } from "./runCellFresh";
import { completedRunBlocks, failedRunBanner, settleDocumentRun } from "./documentRunResults";
export { completedRunBlocks, failedRunBanner, settleDocumentRun } from "./documentRunResults";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement">
import { inlineEvaluationOwner } from "../debug/inlineEval";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-2">
import { useDocumentRunActions } from "./uiDocumentRunActions";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-3">
import { rememberReadingState, restoreReadingState } from "../lib/contextProjection";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-4"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-4">
import { launchWithProfiles } from "../debug/launchProfiles";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-5"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-5">
import { documentDebugContext } from "../debug/documentDebugContext";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-6"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-6">
import { bindProjectDebugView } from "../debug/projectDebugSurface";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-7"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-7">
import { completionSource, completionSources } from "../lsp/completion";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-8"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-8">
import { usePublishDebugSession } from "../debug/plainDebugHosts";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-9"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-9">
import { useDocumentPersistence, type SyncState } from "./useDocumentPersistence";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-10"></a>
### `import_statement`

One document's machinery, alive for as long as the workspace holds any of
its tabs.
This is the per-document half of what views/DocumentView.tsx used to be.
The workspace (views/WorkspaceView.tsx) owns the layout and outlives
navigation; each open document gets one of these sessions — its CRDT room,
runs, outputs, LSP, and debugger — keyed by doc id. Rooms are keyed by doc
id server-side too, so N live rooms is the natural shape, not a special
case.
Sessions are published through a small external store (SessionRegistry)
because their consumers live in different panes of one layout: the
document's own tab, each generated-file tab it owns, the toolbar, and the
ribbon overlay all read the same session. A React context would work for
the tabs but not for the workspace chrome above them.

<storycode:copy id="f-bfb59577f48b-import_statement-10">
// One document's machinery, alive for as long as the workspace holds any of
// its tabs.
//
// This is the per-document half of what views/DocumentView.tsx used to be.
// The workspace (views/WorkspaceView.tsx) owns the layout and outlives
// navigation; each open document gets one of these sessions — its CRDT room,
// runs, outputs, LSP, and debugger — keyed by doc id. Rooms are keyed by doc
// id server-side too, so N live rooms is the natural shape, not a special
// case.
//
// Sessions are published through a small external store (SessionRegistry)
// because their consumers live in different panes of one layout: the
// document's own tab, each generated-file tab it owns, the toolbar, and the
// ribbon overlay all read the same session. A React context would work for
// the tabs but not for the workspace chrome above them.

import { memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-11"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-11">
import type { Extension } from "@codemirror/state";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-12"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-12">
import type { EditorView } from "@codemirror/view";

</storycode:copy>
<a id="f-bfb59577f48b-import_statement-13"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-13">
import { api, MOCK } from "../api/client";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-14"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-14">
import { WsRealtime, getSharedRealtime, type Realtime } from "../api/realtime";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-15"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-15">
import type { Block, DiagramBlock, Doc, ExecBlock, OutputFile, RunWsMessage, SourceEdit } from "../api/types";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-16"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-16">
import { flashSpans } from "../editor/changeFlash";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-17"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-17">
import { usePrompt } from "../components/PromptPanel";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-18"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-18">
import { byteToChar } from "../lib/offsets";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-19"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-19">
import { useDraftKeeper } from "../lib/drafts";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-20"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-20">
import { useLsp } from "../lsp/useLsp";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-21"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-21">
import {
  lspSupport,
  offsetToPosition,
  positionToOffset,
  type LspNavigationTarget,
} from "../lsp/cmLsp";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-22"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-22">
import { lspFeatures } from "../lsp/cmLspFeatures";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-23"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-23">
import { useDebugger } from "../debug/useDebugger";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-24"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-24">
import { debugEditor, debugStateEffects, revealLine } from "../debug/cmDebug";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-25"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-25">
import { positionToUtf16 } from "../lsp/positions";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-26"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-26">
import { sourcePositionAt, type OutputProvenance } from "../lsp/outputMapping";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-27"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-27">
import type { LspLocation } from "../lsp/client";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-28"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-28">
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-29"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-29">
import { openLocation, pathOfDocUri } from "../lib/revealLine";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-30"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-30">
import { publishPausedElsewhere } from "../lib/pausedElsewhere";
</storycode:copy>
<a id="f-bfb59577f48b-import_statement-31"></a>
### `import_statement`

<storycode:copy id="f-bfb59577f48b-import_statement-31">
import { navigate } from "../router";

</storycode:copy>
<a id="f-bfb59577f48b-Banner"></a>
### `Banner`

<storycode:copy id="f-bfb59577f48b-Banner">
export type Banner = { kind: "pending" | "pass" | "fail"; text: string; retry?: { label: string; run: () => void } } | null;

</storycode:copy>
<a id="f-bfb59577f48b-isFilesChanged"></a>
### `isFilesChanged`

The up-loop's "this document's generated files changed on disk" event
(watch.rs::notify_files_changed). It rides the run channel but is neither
a transcript event nor a terminal status — without this guard it would
fall into the terminal-status branch and clear running cells for a run
that never was.

<storycode:copy id="f-bfb59577f48b-isFilesChanged">
/** The up-loop's "this document's generated files changed on disk" event
 * (watch.rs::notify_files_changed). It rides the run channel but is neither
 * a transcript event nor a terminal status — without this guard it would
 * fall into the terminal-status branch and clear running cells for a run
 * that never was. */
export function isFilesChanged(msg: RunWsMessage): msg is { files_changed: true; doc: string } {
  return "files_changed" in msg && msg.files_changed === true;
}

</storycode:copy>
<a id="f-bfb59577f48b-revealRange"></a>
### `revealRange`

Scroll a generated pane to a range and flash it.
The same idea as the debugger's frame reveal: moving somewhere silently
leaves a person hunting for what changed.

<storycode:copy id="f-bfb59577f48b-revealRange">
/**
 * Scroll a generated pane to a range and flash it.
 *
 * The same idea as the debugger's frame reveal: moving somewhere silently
 * leaves a person hunting for what changed.
 */
export function revealRange(view: EditorView, range: [number, number]) {
  const to = Math.min(range[1], view.state.doc.length);
  const from = Math.min(range[0], to);
  view.dispatch({
    selection: { anchor: from, head: to },
    scrollIntoView: true,
  });
  view.focus();
}

</storycode:copy>
<a id="f-bfb59577f48b-DocSession"></a>
### `DocSession`

Everything one open document is: state for its tabs, chrome for the
workspace, actions for both. Published anew on every change.

<storycode:copy id="f-bfb59577f48b-DocSession">
/** Everything one open document is: state for its tabs, chrome for the
 * workspace, actions for both. Published anew on every change. */
export interface DocSession {
  docId: string;
  doc: Doc | null;
  blocks: Block[] | null;
  /** A failure to LOAD the document (missing, forbidden) — fatal to its tab,
   * never to the workspace. */
  fatalError: string | null;
  /** A failed weave, reported inline; the editor stays reachable. */
  renderError: string | null;
  banner: Banner;
  syncState: SyncState;
  /** Live text differs from the last confirmed disk bytes. */
  dirty: boolean;
  liveSource: string;
  startDebugFile: (path: string, at?: number, choose?: boolean) => void;
  savedSource: string | null;
  runningCells: Set<string>;
  execBlocks: ExecBlock[];
  /** Diagram bodies with pastes resolved server-side — a derived diagram is
   * unreadable from its raw source alone. */
  diagramBlocks: DiagramBlock[];
  /** Every file this document generates, with provenance — open or not. */
  outputs: Map<string, OutputFile>;
  /** The editors currently showing one, keyed by path. */
  openOutputs: Map<string, EditorView>;
  docEditor: EditorView | null;
  realtime: Realtime;
  debug: ReturnType<typeof useDebugger>;
  selectSpan: [number, number] | null;
  references: { locations: LspLocation[]; query: string } | null;
  prompt: ReturnType<typeof usePrompt>["prompt"];
  settle: ReturnType<typeof usePrompt>["settle"];
  askText: ReturnType<typeof usePrompt>["askText"];
  lspDiagnostics: ReturnType<typeof useLsp>["diagnostics"];
  lspExtensions: Extension[];
  /** The session's client and URI, for the editor's completion source. */
  lspCompletion: { client: import("../lsp/client").LspClient; uri: string } | null;

  refresh: () => void;
  runCell: (execId: string) => void;
  runAll: () => void;
  verify: () => void;
  setDirtySource: (source: string) => void;
  /** A byte span from provenance or LSP: select it in the document editor. */
  onSelectSpan: (span: [number, number]) => void;
  /** An output edit resolved into the document: flash where it landed. */
  flashSourceEdits: (edits: SourceEdit[]) => void;
  makeOutputLsp: (provenance: OutputProvenance[]) => Extension[];
  registerOutputView: (path: string, view: EditorView | null) => void;
  onDocViewReady: (view: EditorView | null) => void;
  selectFrameAndReveal: (id: number) => void;
  openTarget: (target: LspNavigationTarget | LspLocation) => void;
  clearReferences: () => void;
  revealDocLine: (line: number) => void;
  /** Reveal a range in a generated file's editor — now, or as soon as the
   * pane the caller just opened has one. */
  revealOutput: (path: string, range: [number, number]) => void;
  menuSave: () => Promise<boolean>;
  menuSaveAs: () => void;
  discardUnsaved: () => Promise<void>;
}

</storycode:copy>
<a id="f-bfb59577f48b-SessionRegistry"></a>
### `SessionRegistry`

The registry: sessions as an external store

<storycode:copy id="f-bfb59577f48b-SessionRegistry">
// ---------------------------------------------------------------------------
// The registry: sessions as an external store
// ---------------------------------------------------------------------------

export class SessionRegistry {
  private sessions = new Map<string, DocSession>();
  private listeners = new Set<() => void>();
  /** Bumped on every publish/drop — a cheap snapshot for subscribers that
   * read several sessions at once (the workspace chrome). */
  version = 0;

  publish(session: DocSession): void {
    // A session hook re-runs on every keystroke (the dirty buffer is its
    // state), and each run hands over a fresh object. Re-publishing one whose
    // every field is the same would re-render the workspace chrome — every
    // pane, rail and ruler in the window — for a change nothing can see.
    // Shallow equality is exactly the test, because the hook keeps every
    // field referentially stable unless it changed.
    const current = this.sessions.get(session.docId);
    if (current && sameSession(current, session)) return;
    this.sessions.set(session.docId, session);
    this.emit();
  }

  drop(docId: string): void {
    if (this.sessions.delete(docId)) this.emit();
  }

  get(docId: string | null | undefined): DocSession | undefined {
    return docId ? this.sessions.get(docId) : undefined;
  }

  all(): DocSession[] {
    return [...this.sessions.values()];
  }

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  private emit(): void {
    this.version += 1;
    for (const listener of [...this.listeners]) listener();
  }
}

</storycode:copy>
<a id="f-bfb59577f48b-sameSession"></a>
### `sameSession`

Shallow equality over a session's fields — the registry's "did anything
a consumer reads change" test.

<storycode:copy id="f-bfb59577f48b-sameSession">
/** Shallow equality over a session's fields — the registry's "did anything
 * a consumer reads change" test. */
function sameSession(a: DocSession, b: DocSession): boolean {
  const keys = Object.keys(b) as (keyof DocSession)[];
  if (keys.length !== Object.keys(a).length) return false;
  return keys.every((key) => Object.is(a[key], b[key]));
}

</storycode:copy>
<a id="f-bfb59577f48b-useSessionVersion"></a>
### `useSessionVersion`

Re-render whenever ANY session publishes; read sessions off the registry
during render. For the workspace, which draws chrome for several.

<storycode:copy id="f-bfb59577f48b-useSessionVersion">
/** Re-render whenever ANY session publishes; read sessions off the registry
 * during render. For the workspace, which draws chrome for several. */
export function useSessionVersion(registry: SessionRegistry): number {
  return useSyncExternalStore(registry.subscribe, () => registry.version);
}

</storycode:copy>
<a id="f-bfb59577f48b-useDocSession"></a>
### `useDocSession`

Read one document's live session, re-rendering as it changes.

<storycode:copy id="f-bfb59577f48b-useDocSession">
/** Read one document's live session, re-rendering as it changes. */
export function useDocSession(
  registry: SessionRegistry,
  docId: string | null | undefined,
): DocSession | undefined {
  return useSyncExternalStore(registry.subscribe, () => registry.get(docId));
}

</storycode:copy>
<a id="f-bfb59577f48b-DocSessionHost"></a>
### `DocSessionHost`

The host: one mounted component per open document
Runs a document's session and publishes it. Renders nothing — the tabs
that SHOW the document subscribe through the registry, which is what lets
the session outlive any particular tab (a generated file can stay open
after its document's tab closes, and the ribbons still know the way back).
Memoized so the workspace re-rendering on a publish does not re-render
every host, which would publish again, forever.

<storycode:copy id="f-bfb59577f48b-DocSessionHost">
// ---------------------------------------------------------------------------
// The host: one mounted component per open document
// ---------------------------------------------------------------------------

/**
 * Runs a document's session and publishes it. Renders nothing — the tabs
 * that SHOW the document subscribe through the registry, which is what lets
 * the session outlive any particular tab (a generated file can stay open
 * after its document's tab closes, and the ribbons still know the way back).
 *
 * Memoized so the workspace re-rendering on a publish does not re-render
 * every host, which would publish again, forever.
 */
export const DocSessionHost = memo(function DocSessionHost({
  registry,
  docId,
  openGenerated,
  retainUnsaved = false,
}: {
  registry: SessionRegistry;
  docId: string;
  /** Open one of THIS document's generated files in the layout. */
  openGenerated: (docId: string, path: string) => void;
  retainUnsaved?: boolean;
}) {
  const session = useDocumentSession(docId, openGenerated, retainUnsaved);
  useEffect(() => {
    registry.publish(session);
  });
  useEffect(() => () => registry.drop(docId), [registry, docId]);
  return null;
});

</storycode:copy>
<a id="f-bfb59577f48b-useDocumentSession"></a>
### `useDocumentSession`

The session itself (moved, near-verbatim, from views/DocumentView.tsx)

<storycode:copy id="f-bfb59577f48b-useDocumentSession">
// ---------------------------------------------------------------------------
// The session itself (moved, near-verbatim, from views/DocumentView.tsx)
// ---------------------------------------------------------------------------

function useDocumentSession(
  docId: string,
  openGeneratedFor: (docId: string, path: string) => void,
  retainUnsaved: boolean,
): DocSession {
  const openGenerated = useCallback(
    (path: string) => openGeneratedFor(docId, path),
    [openGeneratedFor, docId],
  );
  const [doc, setDoc] = useState<Doc | null>(null);
  const [blocks, setBlocks] = useState<Block[] | null>(null);
  const [fatalError, setFatalError] = useState<string | null>(null);
  const [renderError, setRenderError] = useState<string | null>(null);
  const [runningCells, setRunningCells] = useState<Set<string>>(new Set());
  const [banner, setBanner] = useState<Banner>(null);
  const [selectSpan, setSelectSpan] = useState<[number, number] | null>(null);
  const [dirtySource, setDirtySource] = useState<string | null>(null);
  const { savedSource, setSavedSource, syncState, setSyncState } = useDocumentPersistence(docId, doc, dirtySource, stored => { setDoc(stored); void refreshRender(stored.source); });
  const [references, setReferences] = useState<{ locations: LspLocation[]; query: string } | null>(
    null,
  );

  // Rename and code-action need an answer from the user before the edit can
  // be applied; this is that answer, in-app rather than in a modal.
  const { prompt, askText, askChoice, settle } = usePrompt();

  const checkRunRef = useRef<string | null>(null);
  const [lastRunId, setLastRunId] = useState<string | null>(null);
  // Fast local runs can finish (and emit their terminal WS message) before
  // the POST /check response delivers the run_id — remember terminals so
  // verify() can resolve the banner after the fact.
  const terminalStatusesRef = useRef<Map<string, string>>(new Map());
  const startedCellsRef = useRef<Set<string>>(new Set());
  const requestedCellsRef = useRef<Set<string>>(new Set());
  const knownDocumentRunsRef = useRef<Set<string>>(new Set());

  const realtime: Realtime = useMemo(() => {
    // In mock mode a shared local bus is registered at startup so fake run
    // events reach this session without a network socket.
    const shared = getSharedRealtime();
    if (MOCK && shared) return shared;
    return new WsRealtime(`doc:${docId}`);
  }, [docId]);

  useEffect(() => {
    // `reopen` before `close`, and both on the same object, because the memo
    // above is not re-run when this effect mounts a second time. Under
    // StrictMode — a dev build only — React mounts, cleans up, and mounts
    // again; without the reopen the second mount would hold the socket the
    // first cleanup shut, and the document would never sync. A production
    // bundle never double-mounts, which is why running this app from Vite for
    // the first time is what surfaced it.
    realtime.reopen();
    return () => {
      if (realtime !== getSharedRealtime()) realtime.close();
    };
  }, [realtime]);

  // The debugger. It is a READER: nothing it does can change the document,
  // its generated files, or a recorded transcript — the channel carries no
  // edit operation and the debuggee runs in a scratch copy.
  const debug = useDebugger(realtime, doc?.path ?? "");
  usePublishDebugSession(doc?.path ?? "", debug);
  const debugRef = useRef(debug);
  debugRef.current = debug;
  const editorRef = useRef<EditorView | null>(null);
  // Selecting a frame — from the strip's combo box or a gutter stack mark —
  // is asking to go there, and a jump with no sign of having moved leaves
  // you hunting for what changed.
  const selectFrameAndReveal = useCallback((id: number) => {
    const session = debugRef.current;
    session.selectFrame(id);
    const frame = session.frames.find((candidate) => candidate.id === id);
    const view = editorRef.current;
    if (view && frame?.line !== null && frame?.line !== undefined) {
      revealLine(view, frame.line);
    }
  }, []);
  // Every file this document generates, with its provenance — not only the
  // ones on screen. A relationship you cannot see is one you will not look
  // for, so a file that is closed still gets a stub reaching off the edge of
  // the document's pane with its name on it.
  const [outputs, setOutputs] = useState<Map<string, OutputFile>>(new Map());
  // The editors currently showing one, keyed by path.
  const [openOutputs, setOpenOutputs] = useState<Map<string, EditorView>>(new Map());
  const [docEditor, setDocEditor] = useState<EditorView | null>(null);
  useEffect(() => { if(docEditor && doc?.path) bindProjectDebugView(doc.path,docEditor); }, [docEditor,doc?.path]);
  // Bumped by the up-loop's files_changed event: outputs changed on disk
  // without a run or a render in this window.
  const [outputsRefresh, setOutputsRefresh] = useState(0);

  // Push the debugger's state into the editor: the gutter dots, the paused
  // line, and the values shown at the end of each line.
  useEffect(() => {
    const view = editorRef.current;
    if (!view) return;
    view.dispatch({ effects: debugStateEffects(debug) });
  }, [debug.breakpoints, debug.pausedLine, debug.variables, debug.frames, debug.watches, docEditor]);

  // A document's debuggee stopped in a plain file of the folder: that file's
  // pane draws the paused line, and the workspace opens it. Same store the
  // plain-file pane publishes to (lib/pausedElsewhere.ts).
  useEffect(() => {
    const owner = docPathRef.current ?? "";
    const top = debug.frames[0];
    const source = top?.source ?? null;
    const inFolder = !!source && !/^([A-Za-z]:)?[\\/]/.test(source);
    if (debug.status === "paused" && top && !top.in_document && inFolder && typeof top.source_line === "number") {
      publishPausedElsewhere(owner, { path: source, line: top.source_line });
      openLocation(source, top.source_line + 1);
    } else {
      publishPausedElsewhere(owner, null);
    }
  }, [debug.status, debug.frames]);

  const { refreshRender, scheduleRender } = useDocumentRender(docId,
    () => dirtySource ?? doc?.source ?? null, setBlocks,
    error => { setRenderError(error); if (error) setBlocks(previous => previous ?? []); });

  const loadedDoc = useRef(doc); loadedDoc.current = doc;
  const refresh = useCallback(() => {
    api.doc(docId).then(d => { setDoc(d); setFatalError(null); }, e => {
      if (loadedDoc.current) setBanner({ kind: "fail", text: `Could not reload ${loadedDoc.current.path}: ${String(e.message ?? e)}`, retry: { label: "Retry reload", run: refresh } });
      else setFatalError(String(e.message ?? e));
    });
    void refreshRender();
  }, [docId, refreshRender]);

  useEffect(refresh, [refresh]);

  // Recovery drafts are separate from acknowledged file persistence.
  // Previously named files opt into recovery; Untitled has its own policy.
  useDraftKeeper({
    path: doc?.path ?? null,
    enabled: retainUnsaved && doc !== null && savedSource !== null,
    read: () => ({
      contents: dirtySource ?? doc?.source ?? "",
      base: savedSource ?? doc?.source ?? "",
    }),
  });
  const restoredDraft = useRef(false);
  useEffect(() => {
    if (!retainUnsaved || !doc || !docEditor || restoredDraft.current) return;
    restoredDraft.current = true;
    void api.drafts().then(({ drafts }) => {
      const draft = drafts.find((candidate) => candidate.path === doc.path);
      if (!draft || draft.contents === draft.base) return;
      const current = docEditor.state.doc.toString();
      if (current !== draft.contents) {
        docEditor.dispatch({
          changes: { from: 0, to: docEditor.state.doc.length, insert: draft.contents },
        });
      }
      setSavedSource(draft.base);
      setDirtySource(draft.contents);
    });
  }, [retainUnsaved, doc, docEditor]);

  // Live run events: append to the matching cell's transcript.
  useEffect(() => {
    return realtime.onRunEvent((msg) => {
      if (isFilesChanged(msg)) {
        // The server's up-loop re-wove this document and something on disk
        // actually changed: refetch the outputs (the effect below), which
        // flows fresh content into every open generated pane, where the
        // changed text gets its flash.
        setOutputsRefresh((n) => n + 1);
        return;
      }
      if (!("event" in msg)) {
        // Terminal run status.
        const started = new Set(startedCellsRef.current);
        const requested = new Set(requestedCellsRef.current);
        terminalStatusesRef.current.set(msg.run_id, msg.status);
        if (msg.run_id === checkRunRef.current) {
          checkRunRef.current = null;
          setBanner(
            msg.status === "ok"
              ? { kind: "pass", text: "Test passed — expectations match and outputs have no drift." }
              : { kind: "fail", text: "Test failed — inspect expectation differences and output drift." },
          );
        }
        const clearProgress = () => {
          setRunningCells(new Set());
          startedCellsRef.current = new Set();
          requestedCellsRef.current = new Set();
        };
        // The RUN's blocks, not a fresh render.
        //
        // `/render` re-weaves the document without executing, and a document
        // with no recorded transcript reports every cell as "never run" —
        // which is what it says a second after a successful run, because a
        // run does not record unless it was asked to. The run's own block
        // model has the transcripts in it, so that is what the cells show.
        void settleDocumentRun(msg.run_id, knownDocumentRunsRef.current,
          (run) => {
            clearProgress();
            setBlocks(prev => completedRunBlocks(run, prev, started, requested));
            const failure = failedRunBanner(run);
            if (failure) setBanner(failure);
            if (!run.blocks?.length && run.status !== "failed") scheduleRender();
          },
          (error) => {
            clearProgress();
            setBanner({ kind: "fail", text: `Could not load run results: ${error instanceof Error ? error.message : String(error)}` });
          },
        );
        return;
      }
      const { exec_id, event } = msg;
      if (exec_id === "agent") return; // agent panel handles its own stream
      // Agent-shaped events carry no `t`; only transcript entries belong in
      // a cell's transcript (the exec_id guard above should already have
      // filtered them, but the type does not promise that correlation).
      if (!("t" in event)) return;
      setRunningCells((prev) => (prev.has(exec_id) ? prev : new Set(prev).add(exec_id)));
      setBlocks((prev) => {
        if (!prev) return prev;
        return prev.map((b) => {
          if (b.kind !== "exec" || b.id !== exec_id) return b;
          const fresh = !startedCellsRef.current.has(exec_id);
          if (fresh) startedCellsRef.current.add(exec_id);
          const transcript = fresh ? [event] : [...(b.transcript ?? []), event];
          const status =
            event.kind === "exit" ? (event.code === 0 ? ("ok" as const) : ("failed" as const)) : b.status;
          return { ...b, transcript, status, runError: undefined };
        });
      });
      if (event.kind === "exit") {
        setRunningCells((prev) => {
          const next = new Set(prev);
          next.delete(exec_id);
          return next;
        });
      }
    });
  }, [realtime, docId, scheduleRender]);

  const runCell = useCallback(
    (execId: string) => {
      void (async () => {
        setBanner(null);
        startedCellsRef.current.delete(execId);
        requestedCellsRef.current.add(execId);
        setBlocks(prev => prev?.map(block => block.kind === "exec" && block.id === execId ? { ...block, runError: undefined } : block) ?? prev);
        setRunningCells((prev) => new Set(prev).add(execId));
        try {
          const outcome = await runCellFresh(docId, execId, () => liveSourceRef.current);
          if (outcome.outcome === "run") knownDocumentRunsRef.current.add(outcome.run_id);
        } catch (e) {
          setRunningCells((prev) => {
            const next = new Set(prev);
            next.delete(execId);
            return next;
          });
          setBanner({ kind: "fail", text: `Could not run this cell: ${e instanceof Error ? e.message : String(e)}`, retry: { label: "Retry cell", run: () => runCell(execId) } });
        }
      })();
    },
    [docId],
  );

  const blocksRef = useRef(blocks);
  blocksRef.current = blocks;

  const runAll = useCallback((expectedSourceHash?: string): Promise<string | null> => {
    return (async () => {
      setBanner(null);
      startedCellsRef.current = new Set();
      const ids = (blocksRef.current ?? []).filter((b) => b.kind === "exec").map((b) => b.id);
      requestedCellsRef.current = new Set(ids);
      setBlocks(prev => prev?.map(block => block.kind === "exec" ? { ...block, runError: undefined } : block) ?? prev);
      // A document with no cells has nothing to run, and a button that
      // appears to do nothing is worse than one that says so: the run
      // happens either way (it re-weaves), but the only visible sign of it
      // is in cells that do not exist here.
      if (ids.length === 0) {
        setBanner({ kind: "pending", text: "Nothing to run — this document has no exec cells. Re-weaving it." });
      }
      setRunningCells(new Set(ids));
      try {
        const { run_id } = await api.run(docId, undefined, expectedSourceHash);
        knownDocumentRunsRef.current.add(run_id);
        setLastRunId(run_id);
        if (ids.length === 0) {
          setBanner({ kind: "pass", text: "Re-woven. This document has no exec cells to run." });
          scheduleRender();
        }
        return run_id;
      } catch (e) {
        setRunningCells(new Set());
        setBanner({ kind: "fail", text: `Could not run this document: ${e instanceof Error ? e.message : String(e)}`, retry: { label: "Retry Run", run: runAll } });
        return null;
      }
    })();
  }, [docId, scheduleRender]);

  const verify = useCallback((expectedSourceHash?: string): Promise<string | null> => {
    return (async () => {
      setBanner({ kind: "pending", text: "Testing — re-running cells and checking expectations and output drift…" });
      startedCellsRef.current = new Set();
      const requested = new Set((blocksRef.current ?? []).filter(block => block.kind === "exec").map(block => block.id));
      requestedCellsRef.current = requested;
      setBlocks(prev => prev?.map(block => block.kind === "exec" ? { ...block, runError: undefined } : block) ?? prev);
      try {
        const { run_id } = await api.check(docId, expectedSourceHash);
        knownDocumentRunsRef.current.add(run_id);
        setLastRunId(run_id);
        const already = terminalStatusesRef.current.get(run_id);
        if (already !== undefined) {
          setBanner(
            already === "ok"
              ? { kind: "pass", text: "Test passed — expectations match and outputs have no drift." }
              : { kind: "fail", text: "Test failed — inspect expectation differences and output drift." },
          );
          // A short run may finish before the check request returns. Read
          // its receipt here too: a fresh render would erase its failure
          // transcript and the generic banner would hide its diagnostic.
          void api.runStatus(run_id).then(run => {
            setBlocks(prev => completedRunBlocks(run, prev, new Set(), requested));
            const failure = failedRunBanner(run);
            if (failure) setBanner(failure);
          }, () => undefined);
        } else {
          checkRunRef.current = run_id;
        }
        return run_id;
      } catch (e) {
        setBanner({ kind: "fail", text: `Could not test this document: ${e instanceof Error ? e.message : String(e)}`, retry: { label: "Retry Test", run: verify } });
        return null;
      }
    })();
  }, [docId, scheduleRender]);

  useDocumentRunActions(docId, () => editorRef.current?.state.doc.toString() ?? dirtySource ?? savedSource ?? doc?.source ?? null, lastRunId, runAll, verify);

  // Load every output's provenance, so a closed file still gets its stub.
  // Refreshed after a run, because a run is when they change.
  useEffect(() => {
    let live = true;
    void (async () => {
      const listed = await api.outputs(docId).catch(() => ({ files: [] }));
      const loaded = await Promise.all(
        listed.files.map((meta) => api.outputFile(docId, meta.path).catch(() => null)),
      );
      if (!live) return;
      setOutputs(new Map(loaded.filter(Boolean).map((file) => [file!.path, file!])));
      // A run is when files on disk change; every mounted folder tree
      // listens for this and refetches.
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    })();
    return () => {
      live = false;
    };
  }, [docId, blocks, outputsRefresh]);

  // A click on a stub opens the file; the reveal has to wait for its editor.
  const pendingReveal = useRef<{ path: string; range: [number, number] } | null>(null);
  useEffect(() => {
    const waiting = pendingReveal.current;
    if (!waiting) return;
    const view = openOutputs.get(waiting.path);
    if (!view) return;
    pendingReveal.current = null;
    revealRange(view, waiting.range);
  }, [openOutputs]);

  const openOutputsRef = useRef(openOutputs);
  openOutputsRef.current = openOutputs;
  const revealOutput = useCallback((path: string, range: [number, number]) => {
    const view = openOutputsRef.current.get(path);
    if (view) revealRange(view, range);
    else pendingReveal.current = { path, range };
  }, []);

  const revealDocLine = useCallback((line: number) => {
    const view = editorRef.current;
    if (view) revealLine(view, line);
  }, []);

  // ---- native menu: Save / Save As ---------------------------------------
  //
  // The desktop menu's Save and Save As arrive from App as one window event
  // (the accelerators live in the native menu, so nothing here binds Cmd+S);
  // the workspace routes them to the FOCUSED document's session. Save is
  // explicit persistence of the live buffer; Save As creates a copy at a new
  // relative path via the same createDoc the untitled buffer uses.
  const menuStateRef = useRef({ doc, dirtySource });
  menuStateRef.current = { doc, dirtySource };
  const menuSave = useCallback(async () => {
    const { doc, dirtySource } = menuStateRef.current;
    if (!doc) return false;
    const source = dirtySource ?? doc.source;
    try {
      const saved = await api.saveDoc(doc.id, source);
      setDoc(saved);
      setSavedSource(source);
      void refreshRender(source);
      const latest = menuStateRef.current;
      const unchanged = (latest.dirtySource ?? latest.doc?.source) === source;
      setSyncState(unchanged ? "saved" : "editing");
      setBanner(current => current?.retry?.label === "Retry Save" ? null : current);
      if (unchanged) void api.discardDraft(doc.path).catch(() => {});
      return unchanged;
    } catch (e) {
      setSyncState("failed");
      setBanner({ kind: "fail", text: e instanceof Error ? e.message : String(e), retry: { label: "Retry Save", run: () => { void menuSave(); } } });
      return false;
    }
  }, [refreshRender]);
  const discardUnsaved = useCallback(async () => {
    const current = menuStateRef.current.doc;
    if (!current || savedSource === null) return;
    const restored = await api.saveDoc(current.id, savedSource);
    setDoc(restored);
    setDirtySource(savedSource);
    setSyncState("saved");
    void api.discardDraft(current.path).catch(() => {});
  }, [savedSource]);
  const menuSaveAs = useCallback(() => {
    const { doc, dirtySource } = menuStateRef.current;
    if (!doc) return;
    const source = dirtySource ?? doc.source;
    void (async () => {
      const path = await askText("Save a copy as (relative path):", doc.path, { agent: true, currentRevision: () => `${menuStateRef.current.doc?.id}:${menuStateRef.current.dirtySource ?? menuStateRef.current.doc?.source}` });
      if (!path) return;
      try {
        const project = (await api.projects())[0];
        if (!project) throw new Error("no folder is open");
        const created = await api.createDoc(project.id, path.trim(), source);
        navigate(`/docs/${created.id}`);
      } catch (e) {
        // createDoc refuses paths outside the folder; its message says so.
        setBanner({ kind: "fail", text: e instanceof Error ? e.message : String(e) });
      }
    })();
  }, [askText]);

  // A span arriving from provenance or an LSP hit indexes the document's
  // BYTES; the editor selects by character.
  const onSelectSpan = useCallback(
    (span: [number, number]) => {
      setSelectSpan(doc ? [byteToChar(doc.source, span[0]), byteToChar(doc.source, span[1])] : span);
    },
    [doc],
  );

  // An output edit resolved into the document (POST /outputs/edit answered
  // with the doc byte spans it rewrote): flash them in the document editor,
  // so the resolution is something you SEE rather than trust. Spans index
  // the pre-edit document; the flash covers at least the new text's width,
  // and the marks map through the room reconcile when it lands a beat later.
  // No open document pane means no editor, and silently no flash.
  const docPathRef = useRef(doc?.path);
  docPathRef.current = doc?.path;
  const flashSourceEdits = useCallback((edits: SourceEdit[]) => {
    const view = editorRef.current;
    if (!view || edits.length === 0) return;
    const docPath = docPathRef.current;
    const mine = docPath
      ? edits.filter((e) => e.doc_path === docPath || e.doc_path.endsWith(`/${docPath}`))
      : edits;
    const text = view.state.doc.toString();
    flashSpans(
      view,
      mine.map((e) => {
        const from = byteToChar(text, e.span[0]);
        return { from, to: Math.max(byteToChar(text, e.span[1]), from + e.text.length) };
      }),
    );
  }, []);

  // ---- editor intelligence ------------------------------------------------
  //
  // One LSP session per document, shared by both editors. It is fed the LIVE
  // text (unsaved edits included) so positions always match the screen.
  const liveSource = dirtySource ?? doc?.source ?? "";
  const startDebugFile = useCallback((path: string, at?: number, choose = false) => {
    if (!doc) return;
    const source = editorRef.current?.state.doc.toString() ?? liveSource;
    let context: string;
    try { context = documentDebugContext(doc.path, source, at); }
    catch (e) { setBanner({kind: "fail", text: e instanceof Error ? e.message : String(e)}); return; }
    // Save the exact reading first. The native resolver also checks those bytes
    // so an external write cannot silently change the launch under this view.
    void menuSave().then(saved => {
      if (saved && (!editorRef.current || editorRef.current.state.doc.toString() === source))
        void launchWithProfiles({uri:`hick:///${doc.path.replace(/^\/+/, "")}`,program:path,project:context,expected_document:source},
          profile=>debug.start(path, {}, context,profile), {choose,isCurrent:()=>!editorRef.current || editorRef.current.state.doc.toString()===source});
    });
  }, [doc, liveSource, menuSave, debug.start]);
  const lsp = useLsp(realtime, doc?.path ?? "", liveSource);
  // The callbacks below read the live text at the moment they are CALLED,
  // through a ref, rather than closing over it: closing over it would give
  // them a new identity on every keystroke, and the session they belong to
  // is published by identity (see SessionRegistry.publish).
  const liveSourceRef = useRef(liveSource);
  liveSourceRef.current = liveSource;

  const openTarget = useCallback(
    (target: LspNavigationTarget | LspLocation) => {
      setReferences(null);
      if (target.uri.startsWith("hick-output:///")) {
        // The bridge could not map this position back to prose — open the
        // generated file itself, positioned on the hit.
        const path = target.uri.slice("hick-output:///".length);
        openGenerated(path);
        return;
      }
      // Another file altogether — a definition in the repository's own
      // `src/lib.rs`, or in another document. This session cannot open a
      // tab; the workspace can, and its pane takes the line from there.
      const targetPath = pathOfDocUri(target.uri);
      const ownPath = (docPathRef.current ?? "").replace(/^\/+/, "");
      if (targetPath !== null && targetPath.replace(/^\/+/, "") !== ownPath) {
        openLocation(targetPath, target.range.start.line + 1);
        return;
      }
      const text = liveSourceRef.current;
      const from = positionToUtf16(text, target.range.start);
      const to = positionToUtf16(text, target.range.end);
      setSelectSpan([from, Math.max(to, from)]);
    },
    [openGenerated],
  );

  const wordAt = (offset: number) => {
    const text = liveSourceRef.current;
    const m = /[A-Za-z_][A-Za-z0-9_]*/y;
    let start = offset;
    while (start > 0 && /[A-Za-z0-9_]/.test(text[start - 1] ?? "")) start--;
    m.lastIndex = start;
    return m.exec(text)?.[0] ?? "";
  };

  const lspExtensions = useMemo(
    () => [
      ...lspSupport({
        client: lsp.client,
        uri: lsp.uri,
        positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
        onNavigate: openTarget,
        onReferences: (locations, from) =>
          setReferences({
            locations,
            query: wordAt(positionToUtf16(liveSourceRef.current, from.range.start)),
          }),
        // While paused, a hover answers two questions at once: what this
        // symbol IS, and what it currently HOLDS. The type is what it should
        // be and the value is what it is, and a debugger exists for the
        // moments those disagree.
        runtimeValue: (word) => debugRef.current.valueAt(word),
      }),
      // The gutter, the paused line and the inline values. Its own layer
      // rather than part of the LSP one: they answer different questions and
      // a document with no debugger running still has diagnostics.
      ...debugEditor({
        onToggleBreakpoint: (line) => debugRef.current.toggleBreakpoint(line),
        onSetBreakpointCondition: (line, patch) =>
          debugRef.current.setBreakpointCondition(line, patch),
        // A gutter stack mark is a caller; clicking it shows that frame.
        onSelectFrame: selectFrameAndReveal,
        // The inline eval at the paused line, answered in the selected frame.
        evaluationOwner: () => inlineEvaluationOwner(debugRef.current),
        onEvaluate: (expression) => debugRef.current.query(expression),
        onAddWatch: (expression) => debugRef.current.addWatch(expression),
      }),
      ...lspFeatures({
        client: lsp.client,
        uri: lsp.uri,
        positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
        offsetAt: (position, view) => positionToOffset(view.state.doc, position),
        inlayHints: true,
        onRename: (current) => askText(`Rename "${current}" to:`, current, { agent: true, currentRevision: () => editorRef.current?.state.doc }),
        onCodeActions: (actions) =>
          askChoice(
            "Code actions",
            actions.map((action) => ({ label: action.title, value: action })),
            { agent: true, currentRevision: () => editorRef.current?.state.doc },
          ),
        onMessage: (message) => setBanner({ kind: "pending", text: message }),
      }),
    ],
    // Rebuilding these would recreate the editor, so they intentionally track
    // only the session identity; the callbacks read live state through refs.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [lsp.client, lsp.uri],
  );

  // The output panes build their own bindings: the same session, but their
  // positions must travel back through provenance first.
  const makeOutputLsp = useCallback(
    (provenance: OutputProvenance[]) => [
      ...lspSupport({
        client: lsp.client,
        uri: lsp.uri,
        positionAt: (offset) =>
          sourcePositionAt(offset, provenance, docPathRef.current ?? "", liveSourceRef.current),
        onNavigate: openTarget,
        onReferences: (locations) => setReferences({ locations, query: "" }),
      }),
      completionSources.of(completionSource({lsp:lsp.client?{client:lsp.client,uri:lsp.uri,positionAt:offset=>sourcePositionAt(offset,provenance,docPathRef.current ?? "",liveSourceRef.current)}:undefined})),
    ],
    [lsp.client, lsp.uri, openTarget],
  );

  const readingOutputViews = useRef(new Map<string, EditorView>());
  const registerOutputView = useCallback((path: string, view: EditorView | null) => {
    rememberReadingState(`output:${docId}:${path}`, readingOutputViews.current.get(path));
    if (view) restoreReadingState(`output:${docId}:${path}`, view);
    if (view) readingOutputViews.current.set(path, view); else readingOutputViews.current.delete(path);
    setOpenOutputs((current) => {
      const next = new Map(current);
      if (view) next.set(path, view);
      else next.delete(path);
      return next;
    });
  }, [docId]);

  const onDocViewReady = useCallback((view: EditorView | null) => {
    rememberReadingState(`doc:${docId}`, editorRef.current);
    if (view) restoreReadingState(`doc:${docId}`, view);
    editorRef.current = view;
    setDocEditor(view);
  }, [docId]);

  const clearReferences = useCallback(() => setReferences(null), []);

  const execBlocks = useMemo(
    () => (blocks ?? []).filter((b): b is ExecBlock => b.kind === "exec"),
    [blocks],
  );
  const diagramBlocks = useMemo(
    () => (blocks ?? []).filter((b): b is DiagramBlock => b.kind === "diagram"),
    [blocks],
  );

  return {
    docId,
    doc,
    blocks,
    fatalError,
    renderError,
    banner,
    syncState,
    dirty:
      dirtySource !== null &&
      savedSource !== null &&
      dirtySource !== savedSource,
    liveSource: dirtySource ?? doc?.source ?? "",
    savedSource,
    runningCells,
    execBlocks,
    diagramBlocks,
    outputs,
    openOutputs,
    docEditor,
    startDebugFile,
    realtime,
    debug,
    selectSpan,
    references,
    prompt,
    settle,
    askText,
    lspDiagnostics: lsp.diagnostics,
    lspExtensions,
    lspCompletion: lsp.client ? { client: lsp.client, uri: lsp.uri } : null,
    refresh,
    runCell,
    runAll,
    verify,
    setDirtySource,
    onSelectSpan,
    flashSourceEdits,
    makeOutputLsp,
    registerOutputView,
    onDocViewReady,
    selectFrameAndReveal,
    openTarget,
    clearReferences,
    revealDocLine,
    revealOutput,
    menuSave,
    menuSaveAs,
    discardUnsaved,
  };
}
</storycode:copy>

### Assemble `apps/web/src/views/documentSession.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/documentSession.tsx"><storycode:paste select="#f-bfb59577f48b-export_statement"/><storycode:paste select="#f-bfb59577f48b-import_statement"/><storycode:paste select="#f-bfb59577f48b-import_statement-2"/><storycode:paste select="#f-bfb59577f48b-import_statement-3"/><storycode:paste select="#f-bfb59577f48b-import_statement-4"/><storycode:paste select="#f-bfb59577f48b-import_statement-5"/><storycode:paste select="#f-bfb59577f48b-import_statement-6"/><storycode:paste select="#f-bfb59577f48b-import_statement-7"/><storycode:paste select="#f-bfb59577f48b-import_statement-8"/><storycode:paste select="#f-bfb59577f48b-import_statement-9"/><storycode:paste select="#f-bfb59577f48b-import_statement-10"/><storycode:paste select="#f-bfb59577f48b-import_statement-11"/><storycode:paste select="#f-bfb59577f48b-import_statement-12"/><storycode:paste select="#f-bfb59577f48b-import_statement-13"/><storycode:paste select="#f-bfb59577f48b-import_statement-14"/><storycode:paste select="#f-bfb59577f48b-import_statement-15"/><storycode:paste select="#f-bfb59577f48b-import_statement-16"/><storycode:paste select="#f-bfb59577f48b-import_statement-17"/><storycode:paste select="#f-bfb59577f48b-import_statement-18"/><storycode:paste select="#f-bfb59577f48b-import_statement-19"/><storycode:paste select="#f-bfb59577f48b-import_statement-20"/><storycode:paste select="#f-bfb59577f48b-import_statement-21"/><storycode:paste select="#f-bfb59577f48b-import_statement-22"/><storycode:paste select="#f-bfb59577f48b-import_statement-23"/><storycode:paste select="#f-bfb59577f48b-import_statement-24"/><storycode:paste select="#f-bfb59577f48b-import_statement-25"/><storycode:paste select="#f-bfb59577f48b-import_statement-26"/><storycode:paste select="#f-bfb59577f48b-import_statement-27"/><storycode:paste select="#f-bfb59577f48b-import_statement-28"/><storycode:paste select="#f-bfb59577f48b-import_statement-29"/><storycode:paste select="#f-bfb59577f48b-import_statement-30"/><storycode:paste select="#f-bfb59577f48b-import_statement-31"/><storycode:paste select="#f-bfb59577f48b-Banner"/><storycode:paste select="#f-bfb59577f48b-isFilesChanged"/><storycode:paste select="#f-bfb59577f48b-revealRange"/><storycode:paste select="#f-bfb59577f48b-DocSession"/><storycode:paste select="#f-bfb59577f48b-SessionRegistry"/><storycode:paste select="#f-bfb59577f48b-sameSession"/><storycode:paste select="#f-bfb59577f48b-useSessionVersion"/><storycode:paste select="#f-bfb59577f48b-useDocSession"/><storycode:paste select="#f-bfb59577f48b-DocSessionHost"/><storycode:paste select="#f-bfb59577f48b-useDocumentSession"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-38c35dc40355"></a>
## `apps/web/src/views/documentSession.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-38c35dc40355-expression_statement"></a>
### `expression_statement`

The run channel carries three shapes: transcript events, terminal run
statuses, and the up-loop's files_changed notice (watch.rs::
notify_files_changed). The session's handler must route the third to the
outputs refetch — before this guard existed it fell into the terminal-
status branch, cleared "running" state for a run that never was, and
fetched /api/runs/undefined.

<storycode:copy id="f-38c35dc40355-expression_statement">
// The run channel carries three shapes: transcript events, terminal run
// statuses, and the up-loop's files_changed notice (watch.rs::
// notify_files_changed). The session's handler must route the third to the
// outputs refetch — before this guard existed it fell into the terminal-
// status branch, cleared "running" state for a run that never was, and
// fetched /api/runs/undefined.

import { describe, expect, it } from "vitest";
import type { RunWsMessage } from "../api/types";
import { isFilesChanged } from "./documentSession";

describe("isFilesChanged", () => {
  it("recognises the up-loop's exact payload", () => {
    const msg: RunWsMessage = { files_changed: true, doc: "doc-1" };
    expect(isFilesChanged(msg)).toBe(true);
  });

  it("rejects transcript events and terminal statuses", () => {
    const transcript: RunWsMessage = {
      run_id: "r1",
      exec_id: "e1",
      event: { t: 0, kind: "out", data: "hi" },
    };
    const terminal: RunWsMessage = { run_id: "r1", status: "ok" };
    expect(isFilesChanged(transcript)).toBe(false);
    expect(isFilesChanged(terminal)).toBe(false);
  });
});

</storycode:copy>
<a id="f-38c35dc40355-import_statement"></a>
### `import_statement`

<storycode:copy id="f-38c35dc40355-import_statement">
import { completedRunBlocks, failedRunBanner } from "./documentSession";
</storycode:copy>
<a id="f-38c35dc40355-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-38c35dc40355-import_statement-2">
import type { Block, Run } from "../api/types";

</storycode:copy>
<a id="f-38c35dc40355-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-38c35dc40355-expression_statement-2">
describe("terminal execution failures", () => {
  const cell = (id: string, transcript: Extract<Block, {kind: "exec"}>["transcript"] = []): Block => ({
    kind: "exec", id, command: "printf before; exit 9", container: "shell", span: [0, 1], status: "unrecorded", transcript,
  });
  const failed = (error: string): Run => ({ id: "r1", started_at: "now", status: "failed", blocks: [], error });

  it.each(["cell failed with exit code 9", "cell exceeded its timeout of 1s"])("keeps partial output and displays the execution diagnostic: %s", error => {
    // The server sends an empty receipt when execution itself aborts. A
    // re-render here would discard 'before' and return the cell to never run.
    const output = [{ t: 0, kind: "out" as const, data: "before" }];
    const blocks = completedRunBlocks(failed(error), [cell("started", output), cell("not-reached")], new Set(["started"]));
    expect(blocks?.[0]).toMatchObject({ status: "failed", transcript: output });
    expect(blocks?.[1]).toMatchObject({ status: "unrecorded", transcript: [] });
    expect(failedRunBanner(failed(error))).toEqual({ kind: "fail", text: `Run failed: ${error}` });
  });

  it("preserves an earlier successful cell when a later cell aborts", () => {
    const blocks = completedRunBlocks(failed("timeout"), [{ ...cell("done", [{ t: 0, kind: "exit", code: 0 }]), status: "ok" } as Block, cell("aborted")], new Set(["done", "aborted"]));
    expect(blocks?.[0]).toMatchObject({ status: "ok" });
    expect(blocks?.[1]).toMatchObject({ status: "failed" });
  });

  it("uses the authoritative receipt for expectation failures and successful reruns", () => {
    const receipt = [cell("new")];
    expect(completedRunBlocks({ ...failed(""), blocks: receipt }, [cell("old")], new Set())).toBe(receipt);
    expect(failedRunBanner({ ...failed(""), status: "ok" })).toBeNull();
    expect(failedRunBanner(failed(""))?.text).toContain("inspect cell output");
  });
});

</storycode:copy>
<a id="f-38c35dc40355-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-38c35dc40355-import_statement-3">
import { api } from "../api/client";
</storycode:copy>
<a id="f-38c35dc40355-import_statement-4"></a>
### `import_statement`

<storycode:copy id="f-38c35dc40355-import_statement-4">
import { settleDocumentRun } from "./documentSession";
</storycode:copy>
<a id="f-38c35dc40355-import_statement-5"></a>
### `import_statement`

<storycode:copy id="f-38c35dc40355-import_statement-5">
import { vi } from "vitest";

</storycode:copy>
<a id="f-38c35dc40355-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-38c35dc40355-expression_statement-3">
it("ignores an agent terminal without a run receipt while a document remains pending, then settles the real document run", async () => {
  let documentPending = true;
  const known = new Set(["document-run"]);
  const settled = vi.fn(() => { documentPending = false; });
  const failed = vi.fn();
  const status = vi.spyOn(api, "runStatus");
  try {
    // Agent status rides this same channel but /api/runs has no such record.
    status.mockRejectedValueOnce(new Error("no such run in this session"));
    await settleDocumentRun("agent-turn", known, settled, failed);
    expect(documentPending).toBe(true);
    expect(settled).not.toHaveBeenCalled();
    expect(failed).not.toHaveBeenCalled();
    const receipt: Run = { id: "document-run", status: "failed", started_at: "now", blocks: [], error: "timeout" };
    status.mockResolvedValueOnce(receipt);
    await settleDocumentRun("document-run", known, settled, failed);
    expect(documentPending).toBe(false);
    expect(settled).toHaveBeenCalledWith(receipt);
    // A known document run genuinely losing its receipt is still actionable.
    status.mockRejectedValueOnce(new Error("document receipt unavailable"));
    await settleDocumentRun("document-run", known, settled, failed);
    expect(failed).toHaveBeenCalledWith(expect.objectContaining({ message: "document receipt unavailable" }));
  } finally { status.mockRestore(); }
});
</storycode:copy>

### Assemble `apps/web/src/views/documentSession.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/documentSession.test.ts"><storycode:paste select="#f-38c35dc40355-expression_statement"/><storycode:paste select="#f-38c35dc40355-import_statement"/><storycode:paste select="#f-38c35dc40355-import_statement-2"/><storycode:paste select="#f-38c35dc40355-expression_statement-2"/><storycode:paste select="#f-38c35dc40355-import_statement-3"/><storycode:paste select="#f-38c35dc40355-import_statement-4"/><storycode:paste select="#f-38c35dc40355-import_statement-5"/><storycode:paste select="#f-38c35dc40355-expression_statement-3"/></storycode:file>


</storycode:doc>
