import { useDocumentRender } from "./useDocumentRender";
import { runCellFresh } from "./runCellFresh";
import { completedRunBlocks, failedRunBanner, settleDocumentRun } from "./documentRunResults";
export { completedRunBlocks, failedRunBanner, settleDocumentRun } from "./documentRunResults";
import { inlineEvaluationOwner } from "../debug/inlineEval";
import { useDocumentRunActions } from "./uiDocumentRunActions";
import { rememberReadingState, restoreReadingState } from "../lib/contextProjection";
import { launchWithProfiles } from "../debug/launchProfiles";
import { documentDebugContext } from "../debug/documentDebugContext";
import { bindProjectDebugView } from "../debug/projectDebugSurface";
import { completionSource, completionSources } from "../lsp/completion";
import { usePublishDebugSession } from "../debug/plainDebugHosts";
import { useDocumentPersistence, type SyncState } from "./useDocumentPersistence";
import type { RecordedPatchSource } from "../simple/AgentProposalAdapter";
import { historySourceHash } from "../simple/SourceHistory";
import { documentDraftBinding } from "../editor/DocumentEditor";
import { admittedManagedPreview, type ManagedDocumentPreview, type StartupManagedDocument } from "./folderPane";
import type { ReferenceReading } from "../components/ReferencesPanel";
import { useHickLangStatus } from "../editor/hickLang";
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
import type { Extension } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";

import { api, MOCK, request } from "../api/client";
import { WsRealtime, getSharedRealtime, type Realtime } from "../api/realtime";
import type { Block, DiagramBlock, Doc, ExecBlock, OutputFile, RunWsMessage, SourceEdit } from "../api/types";
import { flashSpans } from "../editor/changeFlash";
import { usePrompt } from "../components/PromptPanel";
import { byteToChar } from "../lib/offsets";
import { useDraftKeeper } from "../lib/drafts";
import { useLsp } from "../lsp/useLsp";
import {
  lspSupport,
  offsetToPosition,
  positionToOffset,
  type LspNavigationTarget,
  type RelatedLocationReading,
  type LocalSourceNavigation,
} from "../lsp/cmLsp";
import { validProblemPosition } from "../shell/nextWorkspaceProblem";
import { lspFeatures, type ProviderPreviewRequest } from "../lsp/cmLspFeatures";
import type { UiOutcome } from "../lib/uiActions";
import { useDebugger } from "../debug/useDebugger";
import {documentUri} from "../lsp/documentUri";
import {DebugSourceSignal,type DebugSourceCapability} from "../debug/SourceGuards";
import { debugEditor, debugStateEffects, revealLine } from "../debug/cmDebug";
import { positionToUtf16 } from "../lsp/positions";
import { sourcePositionAt, type OutputProvenance } from "../lsp/outputMapping";
import type { LspLocation } from "../lsp/client";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { claimRevealPosition, onRevealLine, openLocation, openProviderLocation, pathOfDocUri } from "../lib/revealLine";
import { authoredSource } from "../simple/SourceText";
import { publishPausedElsewhere } from "../lib/pausedElsewhere";
import { navigate } from "../router";

export type Banner = { kind: "pending" | "pass" | "fail"; text: string; retry?: { label: string; run: () => void } } | null;

/** The up-loop's "this document's generated files changed on disk" event
 * (watch.rs::notify_files_changed). It rides the run channel but is neither
 * a transcript event nor a terminal status — without this guard it would
 * fall into the terminal-status branch and clear running cells for a run
 * that never was. */
export function isFilesChanged(msg: RunWsMessage): msg is { files_changed: true; doc: string } {
  return "files_changed" in msg && msg.files_changed === true;
}

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
  detachedDraft: boolean;
  diskReviewBase: string | null;
  needsDraftReconciliation: boolean;
  recordedPatchSource: () => RecordedPatchSource | null;
  startupRoomReady: boolean;
  admittedPreview?: ManagedDocumentPreview;
  onStartupRoomReady: () => void;
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
  references: ReferenceReading | null;
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
  adoptDiskSource: (disk: Doc, text: string, expected: string) => Promise<void>;
}

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

/** Shallow equality over a session's fields — the registry's "did anything
 * a consumer reads change" test. */
function sameSession(a: DocSession, b: DocSession): boolean {
  const keys = Object.keys(b) as (keyof DocSession)[];
  if (keys.length !== Object.keys(a).length) return false;
  return keys.every((key) => Object.is(a[key], b[key]));
}

/** Re-render whenever ANY session publishes; read sessions off the registry
 * during render. For the workspace, which draws chrome for several. */
export function useSessionVersion(registry: SessionRegistry): number {
  return useSyncExternalStore(registry.subscribe, () => registry.version);
}

/** Read one document's live session, re-rendering as it changes. */
export function useDocSession(
  registry: SessionRegistry,
  docId: string | null | undefined,
): DocSession | undefined {
  return useSyncExternalStore(registry.subscribe, () => registry.get(docId));
}

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
  startup,
  initialDocument,
  captureNavigationContext,
  onRelatedLocations,
  onLocalNavigation,
  onProviderPreview,
}: {
  registry: SessionRegistry;
  docId: string;
  startup?: StartupManagedDocument;
  initialDocument?: {doc: Doc; workspaceCurrent: () => boolean};
  captureNavigationContext?: () => () => boolean;
  onRelatedLocations?: (reading: RelatedLocationReading) => void;
  onLocalNavigation?: LocalSourceNavigation;
  onProviderPreview?: (request: ProviderPreviewRequest) => Promise<UiOutcome>;
  /** Open one of THIS document's generated files in the layout. */
  openGenerated: (docId: string, path: string) => void;
  retainUnsaved?: boolean;
}) {
  const startupAtOpen = useRef(startup).current;
  const admitted = useRef(startup ? undefined : initialDocument).current;
  // A link chooses whether to open the tab. Once mounted, the document's
  // editing confirmation belongs to the session and its original workspace.
  const [preview] = useState(() => admitted && admittedManagedPreview(admitted.doc, admitted.workspaceCurrent));
  const session = useDocumentSession(docId, openGenerated, retainUnsaved, startupAtOpen ?? preview, admitted?.doc,
    captureNavigationContext, onRelatedLocations, onLocalNavigation, onProviderPreview);
  useEffect(() => {
    registry.publish(preview && !session.startupRoomReady ? {...session, admittedPreview: preview} : session);
  });
  useEffect(() => () => registry.drop(docId), [registry, docId]);
  return null;
});

// ---------------------------------------------------------------------------
// The session itself (moved, near-verbatim, from views/DocumentView.tsx)
// ---------------------------------------------------------------------------

type ManagedDraft = {
  view: EditorView;
  disk: string;
  recoveryBase: string;
  authority: { room: string; guard: string; scope: RecordedPatchSource["scope"] } | null;
  attempt?: { id: string; source: string };
};

type ManagedDraftReceipt = {
  version: number;
  history_protocol: number;
  id: string;
  path: string;
  source: string;
  source_hash: string;
  request_id: string;
  save_guard: string;
  history: { available: boolean; outcome: string; id: string };
};

function useDocumentSession(
  docId: string,
  openGeneratedFor: (docId: string, path: string) => void,
  retainUnsaved: boolean,
  startup?: ManagedDocumentPreview,
  initialDocument?: Doc,
  captureNavigationContext?: () => () => boolean,
  onRelatedLocations?: (reading: RelatedLocationReading) => void,
  onLocalNavigation?: LocalSourceNavigation,
  onProviderPreview?: (request: ProviderPreviewRequest) => Promise<UiOutcome>,
): DocSession {
  const openGenerated = useCallback(
    (path: string) => openGeneratedFor(docId, path),
    [openGeneratedFor, docId],
  );
  const sourceSignal=useMemo(()=>new DebugSourceSignal(),[docId]),debugDoc=useRef<Doc|null>(initialDocument ?? null),debugNamespace=useRef<string|null>(null),proposalWorkspace=useRef<string|null>(null),debugDirty=useRef<string|null>(null);
  const [doc,setDocState]=useState<Doc|null>(initialDocument ?? null);
  const [startupRoomReady, setStartupRoomReady] = useState(false);
  const previewConfirmed = useRef(false);
  const onStartupRoomReady = useCallback(() => {
    previewConfirmed.current = true;
    setStartupRoomReady(true);
  }, []);
  const startRoom = !startup || !!doc;
  const parserStatus = useHickLangStatus();
  const setDoc=useCallback((next:Doc|null)=>{debugDoc.current=next;sourceSignal.changed();setDocState(next);},[sourceSignal]);
  useEffect(()=>{let current=true;debugNamespace.current=null;void api.files().then(files=>{if(current){debugNamespace.current=files.root_path??null;proposalWorkspace.current=files.workspace_id??null;sourceSignal.changed();}},()=>{});return()=>{current=false;debugNamespace.current=null;};},[docId,sourceSignal]);
  const [blocks, setBlocks] = useState<Block[] | null>(null);
  const [fatalError, setFatalError] = useState<string | null>(null);
  const [renderError, setRenderError] = useState<string | null>(null);
  const [runningCells, setRunningCells] = useState<Set<string>>(new Set());
  const [banner, setBanner] = useState<Banner>(null);
  const [selectSpan, setSelectSpan] = useState<[number, number] | null>(null);
  const [dirtySource,setDirtySourceState]=useState<string|null>(null);
  const setDirtySource=useCallback((next:string|null)=>{debugDirty.current=next;sourceSignal.changed();setDirtySourceState(next);},[sourceSignal]);
  const managedDraft = useRef<ManagedDraft | null>(null);
  const publishing = useRef(0);
  const [detached, setDetached] = useState(false);
  const [detachedSync, setDetachedSync] = useState<SyncState>("idle");
  const { savedSource: ordinarySavedSource, setSavedSource, syncState, setSyncState } = useDocumentPersistence(
    docId, doc, detached ? null : dirtySource, stored => {
      if (!managedDraft.current) { setDoc(stored); void refreshRender(stored.source); }
    });
  const savedSource = managedDraft.current?.disk ?? ordinarySavedSource;
  const [references, setReferences] = useState<ReferenceReading | null>(
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
    return new WsRealtime(`doc:${docId}`, undefined, !!startup);
  }, [docId, startup]);

  useEffect(() => {
    // `reopen` before `close`, and both on the same object, because the memo
    // above is not re-run when this effect mounts a second time. Under
    // StrictMode — a dev build only — React mounts, cleans up, and mounts
    // again; without the reopen the second mount would hold the socket the
    // first cleanup shut, and the document would never sync. A production
    // bundle never double-mounts, which is why running this app from Vite for
    // the first time is what surfaced it.
    if (!startRoom) return;
    realtime.reopen();
    return () => {
      if (realtime !== getSharedRealtime()) realtime.close();
    };
  }, [realtime, startRoom]);

  // The debugger. It is a READER: nothing it does can change the document,
  // its generated files, or a recorded transcript — the channel carries no
  // edit operation and the debuggee runs in a scratch copy.
  const debugSources=useMemo<DebugSourceCapability>(()=>({read:uri=>{
    const source=debugDoc.current,workspace=debugNamespace.current,revision=sourceSignal.revision;if(!source||!workspace||documentUri(source.path)!==uri)throw Error('The document source namespace is still loading or changed.');
    const text=debugDirty.current??source.source;return {uri,workspace,namespace:workspace,identity:source.id,text,saved:text===source.source,current:()=>sourceSignal.revision===revision&&debugNamespace.current===workspace&&debugDoc.current?.id===source.id&&debugDoc.current.path===source.path&&(debugDirty.current??debugDoc.current.source)===text};
  },subscribe:sourceSignal.subscribe}),[sourceSignal]);
  const debug = useDebugger(realtime, doc?.path ?? "","new",debugSources);
  usePublishDebugSession(doc?.path ?? "", debug);
  const debugRef = useRef(debug);
  debugRef.current = debug;
  const editorRef = useRef<EditorView | null>(null);
  const proposalRoom = useRef<Realtime | null>(null);
  const recoveryObserved = useRef(false);
  useEffect(() => {
    let active = true;
    void realtime.whenSynced().then(() => { if (active) proposalRoom.current = realtime; }, () => {});
    return () => { active = false; proposalRoom.current = null; };
  }, [realtime]);
  const recordedPatchSource = useCallback((): RecordedPatchSource | null => {
    const source = debugDoc.current, view = editorRef.current, revision = sourceSignal.revision;
    const namespace = debugNamespace.current, workspace = proposalWorkspace.current;
    const draft = managedDraft.current;
    if (!source?.save_guard || !view?.dom.isConnected || view.state.readOnly || !namespace || !workspace
      || proposalRoom.current !== realtime || publishing.current || (draft && (!draft.authority || draft.attempt))) return null;
    const document = view.state.doc;
    const text = document.toString(), scope = {namespace, workspace};
    const current = () => editorRef.current === view && view.dom.isConnected && view.state.doc === document
      && sourceSignal.revision === revision && debugDoc.current === source && proposalRoom.current === realtime
      && debugNamespace.current === namespace && proposalWorkspace.current === workspace
      && managedDraft.current === draft && !publishing.current;
    return { path: source.path, text, base: source.source, guard: source.save_guard,
      revision, scope, current, prepareDraft(after) {
        const binding = documentDraftBinding(view), room = binding?.roomSource();
        if (!current() || (retainUnsaved && !recoveryObserved.current) ||
            room === undefined || room !== (draft?.authority?.room ?? text)) return null;
        const editor = binding?.prepare(text, after);
        if (!editor) return null;
        const stageCurrent = () => current() && editor.current() && binding?.roomSource() === room;
        const previousDirty = debugDirty.current;
        const next: ManagedDraft = draft ?? {view, disk: source.source, recoveryBase: source.source,
          authority: {room, guard: source.save_guard!, scope}};
        let staged = false;
        return {
          current: stageCurrent,
          commit() {
            if (!stageCurrent()) throw Error("The managed source changed before Apply.");
            staged = true;
            managedDraft.current = next;
            setDetached(true);
            setDetachedSync(after === next.disk ? "saved" : "editing");
            editor.commit();
          },
          rollback() {
            if (!staged) return;
            editor.rollback();
            managedDraft.current = draft;
            setDetached(!!draft);
            setDetachedSync(draft && text !== draft.disk ? "editing" : "idle");
            setDirtySource(previousDirty);
            staged = false;
          },
        };
      } };
  }, [realtime, sourceSignal, setDirtySource, retainUnsaved]);
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
    api.doc(docId).then(d => { if (!managedDraft.current) { setDoc(d); setFatalError(null); } }, e => {
      if (managedDraft.current) return;
      if (loadedDoc.current) setBanner({ kind: "fail", text: `Could not reload ${loadedDoc.current.path}: ${String(e.message ?? e)}`, retry: { label: "Retry reload", run: refresh } });
      else setFatalError(String(e.message ?? e));
    });
    void refreshRender();
  }, [docId, refreshRender]);

  useEffect(() => {
    if (initialDocument) {
      // The workspace already fetched this document to verify the source
      // location. Its text can mount while the independent rendering arrives.
      void refreshRender(initialDocument.source);
      return;
    }
    if (!startup) return refresh();
    let current = true;
    void startup.authority.then(confirmed => {
      if (!current || !startup.current() || managedDraft.current) return;
      setDoc(confirmed);
      setFatalError(null);
      void refreshRender(confirmed.source);
    }, error => { if (current) setFatalError(String(error)); });
    return () => { current = false; };
  }, [refresh, refreshRender, setDoc, startup, initialDocument]);

  // Recovery drafts are separate from acknowledged file persistence.
  // Previously named files opt into recovery; Untitled has its own policy.
  useDraftKeeper({
    path: doc?.path ?? null,
    enabled: retainUnsaved && doc !== null && savedSource !== null && (!startup || startupRoomReady),
    read: () => {
      const draft = managedDraft.current;
      return draft ? {contents: draft.view.state.doc.toString(), base: draft.recoveryBase, detached: true as const} : {
        contents: dirtySource ?? doc?.source ?? "", base: savedSource ?? doc?.source ?? "",
      };
    },
  });
  const restoredDraft = useRef(false);
  useEffect(() => {
    if (!retainUnsaved || !doc || !docEditor || parserStatus !== "ready" || (startup && !startupRoomReady) || restoredDraft.current) return;
    restoredDraft.current = true;
    void api.drafts().then(async ({ drafts }) => {
      const draft = drafts.find((candidate) => candidate.path === doc.path);
      if (!draft || draft.contents === draft.base || managedDraft.current) return;
      if (draft.detached === true) {
        await realtime.whenSynced();
        if (managedDraft.current || editorRef.current !== docEditor || !docEditor.dom.isConnected || publishing.current) return;
        const disk = debugDoc.current;
        if (!disk || disk.path !== draft.path) return;
        const editor = documentDraftBinding(docEditor)?.prepare(docEditor.state.doc.toString(), draft.contents);
        if (!editor?.current()) return;
        // The private archive holds exact draft bytes. A newly opened room
        // or disk observation does not grant this recovered draft authority.
        managedDraft.current = {view: docEditor, disk: disk.source, recoveryBase: draft.base, authority: null};
        setDetached(true);
        setDetachedSync("editing");
        editor.commit();
        setBanner({kind: "fail", text: "Recovered a private draft. Reconcile it against the current saved source before publishing."});
        return;
      }
      const current = docEditor.state.doc.toString();
      if (current !== draft.contents) {
        docEditor.dispatch({
          changes: { from: 0, to: docEditor.state.doc.length, insert: draft.contents },
        });
      }
      setSavedSource(draft.base);
      setDirtySource(draft.contents);
    }).then(() => { recoveryObserved.current = true; }, () => { /* Draft Apply must wait for a known recovery state. */ });
  }, [retainUnsaved, doc, docEditor, parserStatus, realtime, startup, startupRoomReady]);

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
      const loaded = new Map<string, OutputFile>();
      // Each provenance request weaves its owner. Queue one background read at
      // a time so opening a large project cannot flood the server with weaves.
      // Publish each result immediately; a slow later output hides no earlier one.
      for (const meta of listed.files) {
        if (!live) return;
        const file = await api.outputFile(docId, meta.path).catch(() => null);
        if (!live) return;
        if (file) {
          loaded.set(file.path, file);
          setOutputs(current => new Map(current).set(file.path, file));
        }
      }
      if (!live) return;
      setOutputs(loaded);
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
  // A detached draft retains its captured write authority independently of
  // room observations. A retry keeps the submitted bytes and request ID.
  const menuSave = useCallback(async () => {
    const { doc, dirtySource } = menuStateRef.current;
    if (!doc || (startup && (!previewConfirmed.current || !editorRef.current)) || editorRef.current?.state.readOnly) return false;
    const draft = managedDraft.current;
    if (draft) {
      if (publishing.current) return false;
      const authority = draft.authority, view = draft.view;
      const binding = documentDraftBinding(view);
      if (!authority || !binding) {
        setBanner({kind: "fail", text: "This private draft requires reconciliation with the saved source before publication."});
        return false;
      }
      const source = view.state.doc.toString();
      const prior = draft.attempt;
      const current = () => managedDraft.current === draft && draft.authority === authority
        && editorRef.current === view && view.dom.isConnected && debugDoc.current === doc
        && debugNamespace.current === authority.scope.namespace && proposalWorkspace.current === authority.scope.workspace
        && proposalRoom.current === realtime && doc.source === draft.disk && doc.save_guard === authority.guard;
      if (!prior && source === draft.disk) return current() && binding.roomSource() === authority.room;
      const attempt = prior ?? {id: crypto.randomUUID().replaceAll("-", ""), source};
      publishing.current++;
      try {
        const hash = await historySourceHash(attempt.source);
        const room = binding.roomSource();
        if (!current() || (room !== authority.room && (!prior || room !== attempt.source))) {
          throw Error("Saved source authority or the document room changed. Your local draft is retained.");
        }
        draft.attempt = attempt;
        const receipt = await request<ManagedDraftReceipt>("PUT", `/api/source-history/docs/${encodeURIComponent(doc.id)}`, {
          version: 1, request_id: attempt.id, save_guard: authority.guard, source: attempt.source,
          expected: {source: authority.room, disk: draft.disk},
        });
        if (!current() || receipt.version !== 1 || receipt.history_protocol !== 1 || receipt.id !== doc.id
          || receipt.path !== doc.path || receipt.source !== attempt.source || receipt.source_hash !== hash
          || receipt.request_id !== attempt.id || typeof receipt.save_guard !== "string" || !receipt.save_guard
          || receipt.history?.available !== true || receipt.history.outcome !== "published" || receipt.history.id !== attempt.id) {
          throw Error("Guarded save could not confirm this exact source. Your local draft is retained.");
        }
        draft.disk = attempt.source;
        draft.recoveryBase = attempt.source;
        draft.authority = { ...authority, room: attempt.source, guard: receipt.save_guard };
        draft.attempt = undefined;
        setDoc({...doc, source: attempt.source, save_guard: receipt.save_guard});
        void refreshRender(attempt.source);
        const unchanged = view.state.doc.toString() === attempt.source;
        setDetachedSync(unchanged ? "saved" : "editing");
        setBanner(previous => previous?.retry?.label === "Retry Save" ? null : previous);
        return unchanged;
      } catch (error) {
        if (managedDraft.current === draft) {
          setDetachedSync("failed");
          setBanner({kind: "fail", text: String(error), retry: {label: "Retry Save", run: () => { void menuSave(); }}});
        }
        return false;
      } finally { publishing.current--; }
    }
    const source = dirtySource ?? doc.source;
    publishing.current++;
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
    } finally { publishing.current--; }
  }, [realtime, refreshRender, setDoc, startup]);
  const discardUnsaved = useCallback(async () => {
    const current = menuStateRef.current.doc;
    if (!current || savedSource === null || (startup && (!previewConfirmed.current || !editorRef.current)) || editorRef.current?.state.readOnly) return;
    const draft = managedDraft.current;
    if (draft) {
      if (publishing.current) return;
      const edit = documentDraftBinding(draft.view)?.prepare(draft.view.state.doc.toString(), savedSource);
      if (!edit?.current()) return;
      edit.commit();
      setDetachedSync("saved");
      void api.discardDraft(current.path).catch(() => {});
      return;
    }
    publishing.current++;
    try {
      const restored = await api.saveDoc(current.id, savedSource);
      setDoc(restored);
      setDirtySource(savedSource);
      setSyncState("saved");
      void api.discardDraft(current.path).catch(() => {});
    } finally { publishing.current--; }
  }, [savedSource, setDirtySource, setDoc, startup]);
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
    if (managedDraft.current) {
      setBanner({kind: "fail", text: "Open the published source to debug a detached draft after saving it explicitly."});
      return;
    }
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
  const localNavigationRef = useRef(onLocalNavigation);
  localNavigationRef.current = onLocalNavigation;
  const providerPreviewRef = useRef(onProviderPreview);
  providerPreviewRef.current = onProviderPreview;

  const openTarget = useCallback(
    (target: LspNavigationTarget | LspLocation, current?: () => boolean, view?: EditorView) => {
      if (current && !current()) return;
      const {line, character} = target.range.start;
      setReferences(null);
      if (target.uri.startsWith("hick-output:///")) {
        // The bridge could not map this position back to prose — open the
        // generated file itself, positioned on the hit.
        const path = pathOfDocUri(target.uri.replace(/^hick-output:/,"hick:"));
        if (!path) return;
        if (current) openProviderLocation(path, line + 1, character, current);
        else openGenerated(path);
        return;
      }
      // Another file altogether — a definition in the repository's own
      // `src/lib.rs`, or in another document. This session cannot open a
      // tab; the workspace can, and its pane takes the line from there.
      const targetPath = pathOfDocUri(target.uri);
      if (targetPath === null) return;
      const ownPath = (docPathRef.current ?? "").replace(/^\/+/, "");
      if (targetPath.replace(/^\/+/, "") !== ownPath || (current && view !== editorRef.current)) {
        if (current) openProviderLocation(targetPath, line + 1, character, current);
        else openLocation(targetPath, line + 1, character);
        return;
      }
      const text = liveSourceRef.current;
      if (current && view) {
        const lines = text.split("\n");
        if (authoredSource(view.state) !== text || !validProblemPosition(lines, target.range.start) ||
            !validProblemPosition(lines, target.range.end)) return;
        const {doc, selection} = view.state;
        const from = positionToOffset(doc, target.range.start);
        const to = positionToOffset(doc, target.range.end);
        if (to < from) return;
        revealRange(view, [from, to]);
        if (view.state.doc === doc && view.state.selection.main.head === to) {
          localNavigationRef.current?.(view, selection.main.head);
        }
        return;
      }
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
        navigationIntent: true,
        captureNavigationContext,
        onRelatedLocations,
        onLocalNavigation: (view, departure) => localNavigationRef.current?.(view, departure),
        chooseOutline: (options, current) =>
          askChoice("Structure", options, {agent: true, filterable: true, onlyWhenIdle: true, currentRevision: current}),
        onNavigate: openTarget,
        onReferences: (locations, from, current) =>
          setReferences({
            locations,
            query: wordAt(positionToUtf16(liveSourceRef.current, from.range.start)),
            originUri: from.uri,
            current,
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
        captureChangeContext: captureNavigationContext,
        onProviderPreview: providerPreviewRef.current ? request => providerPreviewRef.current!(request) : undefined,
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
        navigationIntent: true,
        captureNavigationContext,
        onRelatedLocations,
        onNavigate: openTarget,
        onReferences: (locations, from, current) => setReferences({ locations, query: "", originUri: from.uri, current }),
      }),
      completionSources.of(completionSource({lsp:lsp.client?{client:lsp.client,uri:lsp.uri,positionAt:offset=>sourcePositionAt(offset,provenance,docPathRef.current ?? "",liveSourceRef.current)}:undefined})),
    ],
    [lsp.client, lsp.uri, openTarget, captureNavigationContext, onRelatedLocations],
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
  useEffect(() => {
    if (!docEditor || !doc?.path) return;
    const path = doc.path;
    // A room-backed editor can mount with an empty Y.Doc. Wait for the
    // initial room contents before asking it to claim an exact source jump.
    // An explicit startup preview already carries its readable source.
    let live = true;
    let readable = (!!startup && !previewConfirmed.current) || !realtime.serverAuthoritative;
    const jump = (target: string) => {
      if (!readable || target !== path) return;
      const position = claimRevealPosition(path, authoredSource(docEditor.state), docEditor.dom);
      if (!position) return;
      const row = docEditor.state.doc.line(Math.min(position.line, docEditor.state.doc.lines));
      const offset = Math.min(row.to, row.from + (position.character ?? 0));
      revealRange(docEditor, [offset, offset]);
      position.onRevealed?.();
    };
    const stop = onRevealLine(jump);
    if (readable) jump(path);
    else void realtime.whenSynced().then(() => {
      if (!live) return;
      readable = true;
      jump(path);
    }, () => { /* The room connection owns the loading error. */ });
    return () => { live = false; stop(); };
  }, [docEditor, doc?.path, realtime, startup]);

  const adoptDiskSource = useCallback(async (disk: Doc, text: string, expected: string) => {
    const before = editorRef.current?.state.doc.toString() ?? liveSourceRef.current;
    if (before !== expected) throw new Error("The buffer changed; its newer edits were kept. Choose again.");
    const draft = managedDraft.current;
    if (draft) {
      const view = editorRef.current, diskOwner = debugDoc.current;
      const namespace = debugNamespace.current, workspace = proposalWorkspace.current;
      if (!view || view !== draft.view || diskOwner?.id !== disk.id || diskOwner.path !== disk.path
        || publishing.current || draft.attempt || !namespace || !workspace || !disk.save_guard
        || proposalRoom.current !== realtime) throw Error("The saved document's current guarded source is unavailable.");
      const binding = documentDraftBinding(view);
      if (binding?.roomSource() !== disk.source) throw Error("The observed document room differs from the reviewed saved source. Review the source again when they agree.");
      const edit = binding.prepare(expected, text);
      if (!edit?.current()) throw Error("The editor could not keep the exact reconciled text. Your private draft is retained.");
      try { edit.commit(); } catch (error) { edit.rollback(); throw error; }
      // Reconciliation only admits a replacement baseline. Save remains the
      // sole route that can publish this text and advance the guard again.
      draft.disk = disk.source;
      draft.recoveryBase = disk.source;
      draft.authority = {room: disk.source, guard: disk.save_guard, scope: {namespace, workspace}};
      setDoc(disk);
      setDetachedSync(text === disk.source ? "saved" : "editing");
      setBanner(null);
      return;
    }
    publishing.current++;
    try {
      const restored = await api.saveDoc(docId, text, { source: before, disk: disk.source });
      setDoc(restored); setSavedSource(text);
      const view = editorRef.current;
      const latest = view?.state.doc.toString() ?? liveSourceRef.current;
      // The guarded room replacement may already have reached this view. Any
      // other newer text belongs to typing after the request and stays dirty.
      const next = latest === before || latest === text ? text : latest;
      setDirtySource(next);
      if (view && view.state.doc.toString() !== next) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: next } });
      void refreshRender(next);
    } finally { publishing.current--; }
  }, [docId, realtime, refreshRender, setDirtySource, setDoc]);

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
    syncState: detached ? detachedSync === "failed" ? "failed" : dirtySource !== savedSource ? "editing" : "saved" : syncState,
    dirty:
      dirtySource !== null &&
      savedSource !== null &&
      dirtySource !== savedSource,
    liveSource: dirtySource ?? doc?.source ?? "",
    savedSource,
    detachedDraft: detached,
    diskReviewBase: managedDraft.current?.recoveryBase ?? savedSource,
    needsDraftReconciliation: detached && managedDraft.current?.authority === null,
    recordedPatchSource,
    startupRoomReady,
    onStartupRoomReady,
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
    adoptDiskSource,
  };
}
