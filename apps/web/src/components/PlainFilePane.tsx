import { performanceInline } from "../performance/inline";
import { coverageGutter } from "../coverage/gutter";
import { inlineEvaluationOwner } from "../debug/inlineEval";
import { useEditorEditing } from "../editor/useEditorEditing";
import { PlainFileActions } from "./PlainFileActions";
import { useUiReading } from "../lib/uiReading";
import type { usePrompt } from "./PromptPanel";
import { useEditorActions } from "../editor/uiEditorActions";
import { observationSourceChanged } from "../debug/observations";
import { launchWithProfiles } from "../debug/launchProfiles";
import { DebugLaunchRail } from "../debug/DebugRail";
import { RightRail } from "../editor/RightRail";
import { PaneHeaderContent } from "../shell/PaneHeader";
import { ActionNotice } from "./ActionNotice";
import { MoreActions } from "./MoreActions";
import { SurfaceInfo } from "./SurfaceInfo";
import { registerSearchEditor } from "../lib/workspaceSearch";
import type { RecordedPatchSource, WorkspaceDraftStage } from "../simple/AgentProposalAdapter";
import type { StartupPlainFile } from "../views/folderPane";
import { authoredChanges, authoredSource, normalizedSource, sourceText } from "../simple/SourceText";
import { historySourceHash } from "../simple/SourceHistory";
import { computeEdits } from "../lib/diff";
// A plain file, in a pane: any text file in the folder that is neither a
// document nor a woven output. CodeMirror over `GET /api/file`, saved whole
// as you type through lib/plainFileSave.ts — debounced, serialized, and
// guarded by the content hash the load carried, so a file rewritten on disk
// underneath the buffer becomes a visible conflict instead of a silent
// overwrite.
//
// External edits (git, a formatter, an agent) are picked up when the window
// regains focus or the tree announces a change — the same cheap signals the
// folder pane refreshes on — and reconciled into the live buffer with a
// flash, never a rebuild, unless this pane's own edits are still unsent.

import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Compartment, EditorState, Transaction } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab, isolateHistory } from "@codemirror/commands";
import { search, searchKeymap } from "@codemirror/search";
import { wordMotionBindings } from "../editor/wordMotion";

import { api, request } from "../api/client";
import type { AdoptResponse, PlainFile } from "../api/types";
import { changeFlashField, flashSpans } from "../editor/changeFlash";
import { languageExtensions, languageFromPath } from "../editor/languages";
import { forgetFocusedEditor, markFocusedEditor } from "../editor/activeEditor";
import { fencedCodeRanges, isMarkdownPath, markdownStyling } from "../editor/markdownStyling";
import { taskCheckboxes } from "../editor/taskList";
import { renderedMath } from "../editor/mathRender";
import { proseWrap } from "../editor/wrapColumn";
import { editorChrome, paneSearch } from "../editor/chrome";
import { multipleCursors } from "../editor/multiCursor";
import { testGutter } from "../editor/testGutter";
import { showTerminalRequest } from "../lib/revealLine";
import { completions } from "../lsp/completion";
import { useWorkspaceLsp } from "../lsp/useLsp";
import {
  diagnosticRanges,
  identifierAt,
  lspSupport,
  offsetToPosition,
  positionToOffset,
  setLspDiagnostics,
  type LspNavigationTarget,
  type RelatedLocationReading,
  type LocalSourceNavigation,
} from "../lsp/cmLsp";
import { lspFeatures, type ProviderPreviewRequest } from "../lsp/cmLspFeatures";
import type { UiOutcome } from "../lib/uiActions";
import type { CodeAction, LspLocation } from "../lsp/client";
import type { ReferenceReading } from "./ReferencesPanel";
import { openLocation, openProviderLocation, pathOfDocUri } from "../lib/revealLine";
import { forgetFileProblems, setFileProblems } from "../lib/fileProblems";
import { blameGutter } from "../editor/blameGutter";
import { useBlame } from "../editor/useBlame";
import { claimRevealPosition, onRevealLine } from "../lib/revealLine";
import { wrapGutterMarkers } from "../editor/wrapGutter";
import { createPlainSaver, type PlainSaveState } from "../lib/plainFileSave";
import { draftDisposition, useDraftKeeper } from "../lib/drafts";
import { onFlushSaves } from "../lib/flushSaves";
import { DiskChangeNotice, MergeView } from "./MergeView";
import { DebugStrip } from "../debug/DebugStrip";
import { usePausedElsewhere } from "../lib/pausedElsewhere";
import { debugEditor, debugStateEffects, revealLine } from "../debug/cmDebug";
import { isDebuggable } from "../debug/languages";
import { IDLE_SESSION } from "../debug/useDebugger";
import { isWorkspaceSource, useFollowingDebugSession } from "../debug/plainDebugHosts";
import {DebugSourceSignal,type DebugSourceCapability} from "../debug/SourceGuards";
import {documentUri} from "../lsp/documentUri";

function plainSourceChanges(state: EditorState, text: string) {
  const before = authoredSource(state);
  try {
    return authoredChanges(state, computeEdits(before, text).map(edit => ({ from: edit.start, to: edit.end, insert: edit.text })));
  } catch {
    // The diff may split a CRLF or a supplementary character. The complete
    // replacement still carries the exact authored bytes and is undoable.
    return authoredChanges(state, [{from: 0, to: before.length, insert: text}]);
  }
}

function syncPlainSource(view: EditorView, text: string) {
  if (authoredSource(view.state) === text) return;
  const transaction = view.state.update(plainSourceChanges(view.state, text));
  view.update([transaction]);
  const ranges: {from: number; to: number}[] = [];
  transaction.changes.iterChanges((_from, _to, from, to) => ranges.push({from, to}));
  flashSpans(view, ranges);
}

export function PlainFilePane({
  path,
  retainedResource = false,
  onAdopted,
  onReferences,
  captureNavigationContext,
  onRelatedLocations,
  onLocalNavigation,
  onProviderPreview,
  askText,
  askChoice,
  retainUnsaved = false,
  onUnsaved,
  onSource,
  proposalOwner, proposalTabId, startup, onRetryStartup, takePreparedSource,
}: {
  path: string;
  retainedResource?: boolean;
  onSource?: (path: string, source: string) => void;
  proposalOwner?: (tabId: string, read: () => RecordedPatchSource | null) => () => void;
  proposalTabId?: string;
  startup?: StartupPlainFile;
  onRetryStartup?: () => void;
  takePreparedSource?: (path: string) => PlainFile | undefined;
  /** Adoption succeeded: the file now has an owning document. The workspace
   * converts this very tab into a generated tab and opens the document. */
  onAdopted?: (adopted: AdoptResponse) => void;
  /** Find References asked from this file: the workspace shows the list. */
  onReferences?: (references: ReferenceReading) => void;
  captureNavigationContext?: () => () => boolean;
  onRelatedLocations?: (reading: RelatedLocationReading) => void;
  onLocalNavigation?: LocalSourceNavigation;
  onProviderPreview?: (request: ProviderPreviewRequest) => Promise<UiOutcome>;
  /** The window's prompt, for a rename's new name. */
  askText?: ReturnType<typeof usePrompt>["askText"];
  /** The window's prompt, for choosing a code action. */
  askChoice?: ReturnType<typeof usePrompt>["askChoice"];
  /** Keep a recovery draft for this already-named file. */
  retainUnsaved?: boolean;
  onUnsaved?: (
    dirty: boolean,
    actions: {
      save: () => Promise<boolean>;
      discard: () => void;
      retain: () => Promise<void>; revision?: () => unknown; isDirty?: () => boolean;
    },
  ) => void;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  // The live view AS STATE: a ref never re-renders, and the blame column has
  // to mount its fetch against the view that actually exists.
  const [railView, setRailView] = useState<EditorView | null>(null);
  const [file, setFile] = useState<PlainFile | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveState, setSaveState] = useState<PlainSaveState>({ kind: "idle" });
  const [adopting, setAdopting] = useState(false);
  const [adoptError, setAdoptError] = useState<string | null>(null);
  // Whether some document in the folder already writes this file.
  //
  // The tree normally routes such a file to the generated pane and this one
  // never opens for it. This is the second line of defence, for the ways it
  // can still get here — a path typed into the URL, a document created while
  // this tab was open — because offering to "make literate" a file that
  // already is, is worse than an extra request: the reader is told their
  // document is not what it plainly is.
  const [generatedBy, setGeneratedBy] = useState<string | null>(null);
  const startupReadOnly = !!startup && (!file || !!generatedBy);
  const startupAccess = useMemo(() => new Compartment(), []);
  const protectStartupSource = useMemo(() => [
    EditorState.readOnly.of(true), EditorView.editable.of(false),
    EditorState.transactionFilter.of(transaction => transaction.docChanged ? [] : transaction),
  ], []);
  useLayoutEffect(() => {
    if (railView && startup) {
      railView.dispatch({effects: startupAccess.reconfigure(startupReadOnly ? protectStartupSource : [])});
    }
  }, [railView, startup, startupReadOnly, startupAccess, protectStartupSource]);
  // A draft to put back into the buffer once the view exists. Held as state
  // rather than applied immediately because the load lands before the editor
  // is built.
  const [restored, setRestored] = useState<{ contents: string; detached: boolean; current: () => boolean } | null>(null);
  // Two versions of this file that both have changes worth keeping. Set when
  // a save conflicts, and when a restored draft finds the file has moved on.
  const [merge, setMerge] = useState<{
    base: string;
    ours: string;
    theirs: string;
    oursLabel: string;
    theirsLabel: string;
    bufferAtOpen: string;
    detached?: boolean;
  } | null>(null);
  const mergeRef = useRef(merge); mergeRef.current = merge;
  useEditorActions(merge ? null : railView, `file:${path}`);
  useEditorEditing(merge ? null : railView, { source: `file:${path}`, label: "file editor", current: () => !merge && !startupReadOnly, identity: () => [path, file?.hash, generatedBy] });
  const onAdoptedRef = useRef(onAdopted);
  onAdoptedRef.current = onAdopted;
  const onReferencesRef = useRef(onReferences);
  onReferencesRef.current = onReferences;
  const onRelatedLocationsRef = useRef(onRelatedLocations);
  onRelatedLocationsRef.current = onRelatedLocations;
  const onLocalNavigationRef = useRef(onLocalNavigation);
  onLocalNavigationRef.current = onLocalNavigation;
  const providerPreviewRef = useRef(onProviderPreview);
  providerPreviewRef.current = onProviderPreview;
  const askTextRef = useRef(askText);
  askTextRef.current = askText;
  const askChoiceRef = useRef(askChoice);
  askChoiceRef.current = askChoice;
  const onUnsavedRef = useRef(onUnsaved);
  onUnsavedRef.current = onUnsaved;
  const actionsRef = useRef<{
    save: () => Promise<boolean>;
    discard: () => void;
    retain: () => Promise<void>; revision?: () => unknown; isDirty?: () => boolean;
  }>({ save: async () => true, discard: () => {}, retain: async () => {} });
  // What the language server was last told — a refused rename, a code action
  // this editor cannot run — shown in the toolbar until the next one.
  const [notice, setNotice] = useState<string | null>(null);

  // ---- editor intelligence ------------------------------------------------
  //
  // The same language server the documents have, asked about this file at
  // its own path. Fed the LIVE text so positions match the screen; opened
  // only once the file has loaded, so the server never sees an empty file
  // stand in for a real one.
  const [liveText, setLiveTextState] = useState<string | null>(() => startup?.source.content ?? null);
  const sourceSignal=useMemo(()=>new DebugSourceSignal(),[]),sourceIdentity=useMemo(()=>({path}),[path]),sourceNamespace=useRef<string|null>(null),sourceWorkspace=useRef<string|null>(null),sourceMounted=useRef(true);
  const adoptionPending = useRef(false), recoveryPending = useRef(retainUnsaved), plainOwnership = useRef(false);
  const addressRef = useRef({path, guarded: false, retained: false, resumeOrdinary: false});
  const retainedAccess = useMemo(() => new Compartment(), []);
  const startupRef = useRef(startup);
  startupRef.current = startup;
  const addressBinding = useMemo(() => new Compartment(), []);
  const installedAddress = useRef<{path: string; language: string; client: typeof lsp.client} | null>(null);

  useEffect(()=>{sourceMounted.current=true;return()=>{sourceMounted.current=false;};},[path]);
  // The buffer's last text, readable after the view is gone. The draft
  // keeper's final flush runs during unmount, AFTER the effect that destroys
  // the view — React runs cleanups in declaration order — and reading an
  // empty string from a dead view wrote a draft of nothing, which the next
  // mount restored "silently" and then saved: a file emptied on disk by
  // switching tabs. Found by dogfooding on this repository, on a file with
  // 159 lines. Never read the buffer through the view alone.
  const lastTextRef = useRef(startup?.source.content ?? "");
  const onSourceRef = useRef(onSource);
  onSourceRef.current = onSource;
  const setLiveText = useCallback((text: string) => {
    lastTextRef.current=text;sourceSignal.changed();
    onSourceRef.current?.(addressRef.current.path, text);
    setLiveTextState(text);
  }, [sourceSignal]);
  const lsp = useWorkspaceLsp(file === null || retainedResource ? "" : path, liveText ?? "");
  const lspRef = useRef(lsp);
  lspRef.current = lsp;

  // The debugger, over the same workspace connection the language server
  // uses. The file is debugged as itself: a breakpoint on line 12 is a
  // breakpoint on line 12 of this file, and the program runs in its own
  // project. The session lives ABOVE this pane, in a workspace-level host
  // (debug/plainDebugHosts.tsx), because only the active tab is rendered
  // and a session held here would die the moment another tab was fronted.
  // The pane reads the session through a ref inside the editor's callbacks,
  // which were built once and must see the current state.
  const sourceCapability=useMemo<DebugSourceCapability>(()=>({read:uri=>{const text=lastTextRef.current,workspace=sourceNamespace.current,revision=sourceSignal.revision;if(addressRef.current.retained||!workspace||uri!==documentUri(path))throw Error('This source namespace is still loading or changed.');return {uri,workspace,namespace:workspace,identity:sourceIdentity,text,saved:!saver.hasPendingEdits()&&saver.baseContent()===text,current:()=>!addressRef.current.retained&&sourceMounted.current&&sourceNamespace.current===workspace&&sourceSignal.revision===revision&&lastTextRef.current===text};},subscribe:sourceSignal.subscribe}),[path,sourceIdentity,sourceSignal]);
  const debug = useFollowingDebugSession(retainedResource ? "" : path,undefined,undefined,undefined,sourceCapability) ?? IDLE_SESSION;
  const debugRef = useRef(debug);
  debugRef.current = debug;
  // Selecting a frame is asking to go there. A frame in this file is
  // revealed here; a frame in ANOTHER file of the folder — the callee in
  // `src/lib.rs` — is opened in its own tab at the adapter's line, since
  // this pane cannot show a line it does not have.
  const selectFrameAndReveal = useCallback((id: number) => {
    const session = debugRef.current;
    session.selectFrame(id);
    const frame = session.frames.find((candidate) => candidate.id === id);
    if (!frame) return;
    const view = viewRef.current;
    if (frame.line !== null && frame.line !== undefined) {
      if (view) revealLine(view, frame.line);
      return;
    }
    if (isWorkspaceSource(frame.source) && typeof frame.source_line === "number") {
      openLocation(frame.source, frame.source_line + 1);
    }
  }, []);

  const saver = useMemo(
    () =>
      createPlainSaver({
        put: (content, baseHash, force, guard) => {
          const {path, guarded, retained} = addressRef.current;
          if (retained) throw Error("This source has been deleted. Restore it before saving.");
          if (!startupRef.current && !guarded) return api.saveFile(path, content, baseHash, force);
          if (!guard) throw Error("Saved source authority is unavailable. Reload the exact file before saving.");
          return api.saveFile(path, content, baseHash, force, guard);
        },
        publish: async ({content, baseHash, guard, id}, saverCurrent) => {
          const namespace = sourceNamespace.current, workspace = sourceWorkspace.current;
          const address = addressRef.current, {path} = address;
          const current = () => !address.retained && !!namespace && !!workspace && sourceMounted.current && plainOwnership.current &&
            sourceNamespace.current === namespace && sourceWorkspace.current === workspace &&
            addressRef.current === address && saverCurrent();
          const hash = await historySourceHash(content);
          if (!current()) throw Error("The saved source address changed. Keep the draft and refresh Files.");
          const saved = await request<{
            version: number; history_protocol: number; path: string; hash: string; save_guard: string;
            source_hash: string; request_id: string; history: {available: boolean; outcome: string; id: string};
          }>("PUT", "/api/source-history/file", {version: 1, path, content, base_hash: baseHash, save_guard: guard, request_id: id});
          if (!current() || saved.version !== 1 || saved.history_protocol !== 1 || saved.path !== path ||
            saved.request_id !== id || saved.source_hash !== hash || typeof saved.hash !== "string" || !saved.hash ||
            typeof saved.save_guard !== "string" || !saved.save_guard || saved.history?.available !== true ||
            saved.history.outcome !== "published" || saved.history.id !== id) {
            throw Error("The guarded save receipt was not confirmed. The draft is still retained.");
          }
          return saved;
        },
        onState: (state) => {
          setSaveState(state);sourceSignal.changed();
          if (state.kind === "idle" || state.kind === "saved") {
            onUnsavedRef.current?.(false, actionsRef.current);
          }
        },
      }),
    [sourceSignal],
  );
  actionsRef.current = {
    revision: () => viewRef.current ? authoredSource(viewRef.current.state) : lastTextRef.current, isDirty: () => saver.hasPendingEdits(),
    save: () => !addressRef.current.retained && sourceMounted.current && viewRef.current?.dom.isConnected ? saver.saveNow() : Promise.resolve(!saver.hasPendingEdits()),
    discard: () => {
      const base = saver.discardPending();
      const view = viewRef.current;
      if (view) syncPlainSource(view, base);
    },
    retain: async () => {
      const contents = viewRef.current ? authoredSource(viewRef.current.state) : lastTextRef.current;
      if (startupReadOnly || addressRef.current.retained) return;
      await api.saveDraft({ path, contents, base: saver.baseContent(), saved_at: Date.now(), ...(saver.isDetached() ? {detached: true as const} : {}) });
    },
  };
  useEffect(() => () => saver.dispose(), [saver]);
  useEffect(() => proposalTabId && proposalOwner ? proposalOwner(proposalTabId, () => {
    const view = viewRef.current, namespace = sourceNamespace.current, workspace = sourceWorkspace.current;
    const guard = saver.baseGuard(), base = saver.baseContent(), revision = sourceSignal.revision;
    const address = addressRef.current;
    if (!view?.dom.isConnected || (view.state.readOnly && !address.retained) ||
        !namespace || !workspace || !guard || mergeRef.current) return null;
    const document = view.state.doc, text = authoredSource(view.state);
    const current = () => sourceMounted.current && viewRef.current === view && view.dom.isConnected
      && !mergeRef.current && view.state.doc === document && authoredSource(view.state) === text && sourceSignal.revision === revision
      && sourceNamespace.current === namespace && sourceWorkspace.current === workspace && addressRef.current === address;
    const writable = () => current() && plainOwnership.current && !adoptionPending.current && !recoveryPending.current &&
      !view.composing && !view.state.readOnly && view.state.facet(EditorView.editable);
    return { path, text, base, guard, revision, scope: {namespace, workspace}, current,
      prepareResource(after, saved, destination = path) {
        const plane = saver.prepareDetach(base, guard);
        const resumeOrdinary = !saver.isDetached();
        let releaseWrites: ((keepPrivate: boolean) => void) | null = null;
        const ownsEditor = () => address.retained ? destination !== null && current() && !view.composing : writable();
        const ready = () => ownsEditor() && (releaseWrites
          ? saver.baseContent() === base && saver.baseGuard() === guard
          : !!plane?.current());
        if (!ready()) return null;
        let adopted = false;
        return {
          current: ready,
          begin() {
            if (!ready()) throw Error("The open file changed before resource publication.");
            releaseWrites = saver.holdWrites();
            if (!releaseWrites) throw Error("Wait for the current file save before publishing resources.");
          },
          release() { releaseWrites?.(!adopted); },
          adopt(authority) {
            if (!ready()) throw Error("The open file changed during resource publication.");
            if (destination === null) {
              if (authority) throw Error("The deleted editor received a live file guard.");
              addressRef.current = {path, guarded: true, retained: true, resumeOrdinary};
              sourceSignal.changed();
              saver.restoreDetached();
              installedAddress.current = null;
              view.dispatch({effects: [
                retainedAccess.reconfigure([EditorState.readOnly.of(true), EditorView.editable.of(false)]),
                addressBinding.reconfigure(languageExtensions(fileRef.current?.language ?? "text")),
              ]});
              adopted = true;
              return;
            }
            if (!authority) throw Error("The restored editor needs a confirmed saved guard.");
            // Install the confirmed saved baseline before displaying its
            // draft. Programmatic adoption must never send a regular save.
            const transaction = view.state.update({
              ...plainSourceChanges(view.state, after),
              annotations: [isolateHistory.of("full"), Transaction.addToHistory.of(false)],
            });
            if (authoredSource(transaction.state) !== after) throw Error("The open editor cannot adopt the complete resource source.");
            if (destination !== path || address.retained) {
              addressRef.current = {path: destination, guarded: true, retained: false, resumeOrdinary: false};
              installedAddress.current = null;
            }
            saver.load(saved, authority.hash, authority.guard);
            if (address.retained && address.resumeOrdinary && after === saved) saver.resumeOrdinaryWrites();
            if (fileRef.current) {
              const next = {...fileRef.current, path: destination, content: saved,
                language: destination === path ? fileRef.current.language : languageFromPath(destination) ?? "text",
                hash: authority.hash, save_guard: authority.guard};
              fileRef.current = next;
              setFile(next);
            }
            if (after !== saved) {
              saver.restoreDetached();
              saver.changed(after);
            }
            if (text !== after) view.update([transaction]);
            if (destination !== path || address.retained) view.dispatch({
              effects: [retainedAccess.reconfigure([]), addressBinding.reconfigure([])],
            });
            onUnsavedRef.current?.(saver.hasPendingEdits(), actionsRef.current);
            adopted = true;
          },
        };
      },
      prepareDraft(after): WorkspaceDraftStage | null {
        if (!writable() || text === after) return null;
        const plane = saver.prepareDetach(base, guard);
        if (!plane) return null;
        const before = view.state, top = view.scrollDOM.scrollTop, left = view.scrollDOM.scrollLeft;
        const transaction = before.update({
          ...plainSourceChanges(before, after), annotations: isolateHistory.of("full"), userEvent: "input.workspace",
        });
        if (authoredSource(transaction.state) !== after) return null;
        let committed = false;
        const ready = () => writable() && view.state === before && plane.current();
        return {current: ready,
          commit() {
            if (!ready()) throw Error("The named plain editor changed. Review the draft again.");
            plane.commit(); committed = true;
            view.update([transaction]);
            if (view.state !== transaction.state) throw Error("The plain editor did not install its staged draft.");
          },
          rollback() {
            if (!committed) return;
            committed = false;
            try {
              if (view.state !== before) view.setState(before);
              view.scrollDOM.scrollTop = top;
              view.scrollDOM.scrollLeft = left;
              setLiveText(text);
            } finally {
              plane.rollback();
              onUnsavedRef.current?.(saver.hasPendingEdits(), actionsRef.current);
            }
          },
        };
      },
    };
  }) : undefined, [path, proposalOwner, proposalTabId, saver, setLiveText, sourceSignal, addressBinding, retainedAccess]);
  // File > Save All: this pane owns its saver, so it answers for its own
  // buffer. See lib/flushSaves.ts.
  useEffect(() => onFlushSaves(includeDetached => {
    if (addressRef.current.retained) return;
    if (includeDetached) void saver.saveNow();
    else void saver.flushNow();
  }), [saver]);

  // The initial read. The pane renders its refusals — binary, too large,
  // missing — as text where the editor would be: the tab is still an honest
  // place, it just has nothing to edit.
  const preparedRead = useRef<PlainFile | undefined>(undefined);
  const startupAuthority = startup?.authority;
  useEffect(() => {
    if (addressRef.current.guarded && addressRef.current.path === path && fileRef.current?.path === path) return;
    let live = true;
    recoveryPending.current = retainUnsaved;
    plainOwnership.current = false;
    if (!startup && preparedRead.current?.path !== path) preparedRead.current = takePreparedSource?.(path);
    const prepared = !startup && preparedRead.current?.path === path ? preparedRead.current : undefined;
    if (startupAuthority) setLoadError(null);
    const read = startupAuthority
      ? startupAuthority.then(authority => ({file: authority.file, authority}))
      : (prepared ? Promise.resolve(prepared) : api.file(path)).then(file => ({file, authority: null}));
    read.then(
      ({file: loaded, authority}) => {
        if (!live) return;
        if (startup && (!startup.current() || loaded.path !== path)) {
          setLoadError("The original file request changed. Reopen this read-only preview to check its current source.");
          return;
        }
        if (addressRef.current.path !== path) addressRef.current = {path, guarded: false, retained: false, resumeOrdinary: false};
        setFile(loaded);
        setLoadError(null);
        setLiveText(loaded.content);
        lastTextRef.current = loaded.content;
        saver.load(loaded.content, loaded.hash, loaded.save_guard);
        if (viewRef.current) syncPlainSource(viewRef.current, loaded.content);
        if (authority) {
          sourceNamespace.current = authority.namespace;
          sourceWorkspace.current = authority.workspace;
          plainOwnership.current = !authority.generatedBy;
          setGeneratedBy(authority.generatedBy);
          sourceSignal.changed();
        } else void api.files().then(
          (files) => {
            if (!live) return;sourceNamespace.current=files.root_path??null;sourceWorkspace.current=files.workspace_id??null;sourceSignal.changed();
            const find = (nodes: typeof files.tree): (typeof files.tree)[number] | undefined => {
              for (const node of nodes) {
                if (node.path === path) return node;
                const found = node.children ? find(node.children) : undefined;
                if (found) return found;
              }
            };
            const node = find(files.tree);
            plainOwnership.current = loaded.path === path && (files.folder_open === false
              ? !!loaded.save_guard : !!node && !node.dir && !node.doc_id && !node.generated_by);
            setGeneratedBy(node?.generated_by ?? null);
          },
          () => {
            // The listing is unavailable: the button stays, and the adopt
            // route's own refusal is the backstop.
          },
        );
        if (!retainUnsaved || authority?.generatedBy) { recoveryPending.current = false; return; }
        // Was this buffer holding unsaved work when the app last closed?
        //
        // Three answers, and only one of them interrupts anybody. The file is
        // as we left it: put the text back, still unsaved, silently — that is
        // the common case by a wide margin, and a dialog here would train
        // people to dismiss dialogs. The file already says the same thing:
        // the draft is stale, drop it. The file moved on and so did we: that
        // is a merge, and it is worth someone's attention.
        const recoveryCurrent = () => live && lastTextRef.current === loaded.content &&
          saver.baseContent() === loaded.content && saver.baseGuard() === (loaded.save_guard ?? null) && !saver.hasPendingEdits();
        void api.drafts().then(
          ({ drafts }) => {
            if (!live) return;
            const draft = drafts.find((d) => d.path === path);
            if (!draft || !recoveryCurrent()) { recoveryPending.current = false; return; }
            const next = draftDisposition(draft, loaded.content);
            if (next.kind === "clean") {
              recoveryPending.current = false;
              void api.discardDraft(path).catch(() => {});
              return;
            }
            if (next.kind === "restore") {
              setRestored({contents: next.contents, detached: draft.detached === true, current: recoveryCurrent});
              return;
            }
            setMerge({
              bufferAtOpen: loaded.content,
              base: next.base,
              ours: next.ours,
              theirs: next.theirs,
              oursLabel: "Your unsaved changes",
              theirsLabel: "The file on disk",
              detached: draft.detached === true,
            });
          },
          () => {
            if (live) recoveryPending.current = false;
            // No draft store on this machine: the file opens as it is on
            // disk, which is what would have happened anyway.
          },
        );
      },
      (e) => {
        if (!live) return;
        recoveryPending.current = false;
        setLoadError(e instanceof Error ? e.message : String(e));
      },
    );
    return () => {
      live = false;
    };
  }, [path, saver, retainUnsaved, startup, startupAuthority, takePreparedSource]);

  // The view is created ONCE, when the first load lands. Later content
  // arrives through syncPlainSource below — rebuilding the editor would throw
  // away the cursor and undo history mid-edit.
  const loaded = !!(file || startup);
  const fileRef = useRef(file);
  fileRef.current = file;
  const addressExtensions = useCallback((path: string, language: string, lspClient: typeof lsp.client) => {
    // Where a definition landed. This file: select it. Another: the
    // workspace opens that tab, and its pane does the rest.
    const goTo = (target: LspNavigationTarget | LspLocation, current?: () => boolean) => {
      if (current && !current()) return;
      const live = viewRef.current;
      if (target.uri === lspRef.current.uri && live) {
        const from = positionToOffset(live.state.doc, target.range.start);
        const to = positionToOffset(live.state.doc, target.range.end);
        live.dispatch({
          selection: { anchor: from, head: Math.max(to, from) },
          effects: EditorView.scrollIntoView(from, { y: "center" }),
        });
        live.focus();
        return;
      }
      const path = pathOfDocUri(target.uri.replace(/^hick-output:/,"hick:"));
      if (!path) return;
      const {line, character} = target.range.start;
      if (current) openProviderLocation(path, line + 1, character, current);
      else openLocation(path, line + 1, character);
    };
    const lspUri = documentUri(path);

    return [
      coverageGutter(() => path, () => fileRef.current?.content),
      performanceInline(() => path, () => fileRef.current?.content),
      wrapGutterMarkers(),
      // The breakpoint gutter, the paused line, the inline values — the
      // same layer a document's editor wears, told that every line here
      // is code. Only for a language hick can debug: a gutter that
      // accepts a dot it can never bind is a promise it cannot keep.
      ...(isDebuggable(language)
        ? debugEditor({
            language,
            lineNumbers: false,
            onToggleBreakpoint: (line) => debugRef.current.toggleBreakpoint(line),
            onSetBreakpointCondition: (line, patch) =>
              debugRef.current.setBreakpointCondition(line, patch),
            onSelectFrame: selectFrameAndReveal,
            evaluationOwner: () => inlineEvaluationOwner(debugRef.current),
            onEvaluate: (expression) => debugRef.current.query(expression),
            onAddWatch: (expression) => debugRef.current.addWatch(expression),
          })
        : []),
      ...languageExtensions(language),
      // A run mark beside every test the file's language has a shape
      // for; a click runs that test in a terminal named after it.
      ...testGutter({
        language,
        onError: setNotice,
        serializedSource: () => saver.baseContent(),
        onRun: async (mark, expected_source_hash, current) => {
          const session = await api.runTest({ path, name: mark.name, language, expected_source_hash });
          if (current() && session.source_checked_at_admission && session.source_hash === expected_source_hash) showTerminalRequest(session.id, session.title);
          return session;
        },
      }),
      ...(isMarkdownPath(path) ? [markdownStyling(), taskCheckboxes(), renderedMath()] : []),
      // Both kinds of completion: what is in scope here, from the
      // language server, and what this project calls things, from its
      // own text. The popup says which is which.
      completions({
        lsp: lspClient
          ? {
              client: lspClient,
              uri: lspUri,
              positionAt: (offset, state) => offsetToPosition(state.doc, offset),
            }
          : undefined,
        project: (prefix, around) =>
          api.complete(prefix, around).then((answer) => answer.suggestions),
      }),
      // Diagnostics, hover, definition, references — and the rest of the
      // server: colouring, inlay hints, signature help, folding, rename,
      // code actions. The same bundle a document's editor wears.
      ...lspSupport({
        client: lspClient,
        uri: lspUri,
        positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
        navigationIntent: true,
        captureNavigationContext,
        onRelatedLocations: onRelatedLocationsRef.current ? reading => onRelatedLocationsRef.current?.(reading) : undefined,
        onLocalNavigation: (view, departure) => onLocalNavigationRef.current?.(view, departure),
        chooseOutline: askChoiceRef.current ? (options, current) =>
          askChoiceRef.current?.("Structure", options, {agent: true, filterable: true, onlyWhenIdle: true, currentRevision: current}) ?? Promise.resolve(null) : undefined,
        onNavigate: goTo,
        onReferences: (locations, from, current) => {
          const live = viewRef.current;
          const at = live ? positionToOffset(live.state.doc, from.range.start) : 0;
          const query = live ? (identifierAt(live.state.doc.toString(), at) ?? "") : "";
          onReferencesRef.current?.({ locations, query, originUri: from.uri, current });
        },
        // While paused, a hover says what this holds as well as what it
        // is — the same merged tooltip a document's editor shows.
        runtimeValue: (word) => debugRef.current.valueAt(word),
      }),
      ...lspFeatures({
        client: lspClient,
        uri: lspUri,
        positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
        offsetAt: (position, view) => positionToOffset(view.state.doc, position),
        inlayHints: true,
        captureChangeContext: captureNavigationContext,
        onProviderPreview: providerPreviewRef.current ? request => providerPreviewRef.current!(request) : undefined,
        onRename: (current) =>
          askTextRef.current
            ? askTextRef.current(`Rename "${current}" to:`, current, { agent: true, currentRevision: () => viewRef.current?.state.doc })
            : null,
        onCodeActions: (actions: CodeAction[]) =>
          askChoiceRef.current
            ? askChoiceRef.current(
                "Code actions",
                actions.map((action) => ({ label: action.title, value: action })),
                { agent: true, currentRevision: () => viewRef.current?.state.doc },
              )
            : null,
        onMessage: setNotice,
      }),
      // Prose wraps at the measure; a fenced code block keeps its lines
      // and takes the whole pane. In a non-markdown file EVERY line is
      // code, which is exactly what `fencedCodeRanges` returning the whole
      // buffer expresses.
      proseWrap((state) =>
        isMarkdownPath(path)
          ? fencedCodeRanges(state.doc.toString())
          : [[0, state.doc.length] as [number, number]],
      ),
    ];
  }, [captureNavigationContext, saver, selectFrameAndReveal]);
  useEffect(() => {
    const host = hostRef.current;
    const initial = fileRef.current ?? (startup && {
      path, content: startup.source.content, language: languageFromPath(path) ?? "text",
    });
    if (!host || !initial) return;

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: normalizedSource(initial.content),
        extensions: [
          sourceText(initial.content),
          startupAccess.of(startup && !fileRef.current ? protectStartupSource : []),
          // The same chrome every editor wears; `code` is the look a
          // `hick:file` body has inside a document, so opening the file and
          // reading the block that writes it are not two different programs.
          editorChrome("code"),
          ...multipleCursors(),
          changeFlashField,
          // Before lineNumbers, which is what puts it to their LEFT:
          // CodeMirror lays gutters out in the order they are declared, and
          // the numbers stay against the text because they are the
          // coordinate everything else in this app refers to.
          blameGutter(),
          lineNumbers(),
          history(),
          retainedAccess.of([]),
          addressBinding.of(addressExtensions(initial.path, initial.language, lspRef.current.client)),
          paneSearch("file"),
          search({ top: true }),
          // Word motion first: bindings for one key run in registration
          // order and the first to return true wins, so these must arrive
          // before `defaultKeymap`'s own Ctrl+arrow. See editor/wordMotion.ts.
          keymap.of([
            ...wordMotionBindings,
            ...searchKeymap,
            ...defaultKeymap,
            ...historyKeymap,
            indentWithTab,
          ]),
          // Which buffer Print means. Not `markActiveEditor` — that one
          // answers "where does an Insert go?", and a hick element written
          // into a file this document generates would land in the woven
          // output, where it means nothing.
          EditorView.focusChangeEffect.of((_state, focusing) => {
            const live = viewRef.current;
            if (focusing && live) markFocusedEditor(live);
            return null;
          }),
          EditorView.updateListener.of((u) => {
            // Every change reaches the language server — a reload from disk
            // included, since the server should see what the screen shows.
            const text = authoredSource(u.state), edited = text !== authoredSource(u.startState);
            if (edited) setLiveText(text);
            // Only edits a person made: a programmatic reload is this pane
            // catching up with the disk, and saving it back would write
            // bytes nobody typed.
            if (
              edited &&
              u.transactions.some(
                (t) =>
                  t.isUserEvent("input") ||
                  t.isUserEvent("delete") ||
                  t.isUserEvent("move") ||
                  t.isUserEvent("undo") ||
                  t.isUserEvent("redo"),
              )
            ) {
              saver.changed(text);
              onUnsavedRef.current?.(saver.hasPendingEdits(), actionsRef.current);
            }
          }),
        ],
      }),
    });
    viewRef.current = view;
    installedAddress.current = {path: initial.path, language: initial.language, client: lspRef.current.client};
    setRailView(view);
    return () => {
      installedAddress.current = null;
      forgetFocusedEditor(view);
      view.destroy();
      viewRef.current = null;
      setRailView(null);
    };
    // Mounted once per load; the saver is stable per path.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loaded, saver]);
  useLayoutEffect(() => {
    if (!railView || addressRef.current.path !== path || addressRef.current.retained) return;
    const language = file?.language ?? languageFromPath(path) ?? "text";
    const installed = installedAddress.current;
    if (installed?.path === path && installed.language === language && installed.client === lsp.client) return;
    railView.dispatch({effects: addressBinding.reconfigure(addressExtensions(path, language, lsp.client))});
    installedAddress.current = {path, language, client: lsp.client};
  }, [addressBinding, addressExtensions, file?.language, lsp.client, path, railView, retainedResource]);
  useLayoutEffect(() => railView && !retainedResource ? registerSearchEditor(path, railView) : undefined, [path, railView, retainedResource]);

  /** Put text into the buffer and treat it as an unsaved edit — which is
   * exactly what it is: restored work that the file does not have yet. */
  const putInBuffer = useCallback(
    (text: string) => {
      const view = viewRef.current;
      if (!view) return;
      syncPlainSource(view, text);
      saver.changed(text);
      onUnsavedRef.current?.(saver.hasPendingEdits(), actionsRef.current);
    },
    [saver],
  );

  // The restored draft goes in once the view exists — the load that found it
  // lands before the editor is built.
  useEffect(() => {
    if (restored === null || !loaded) return;
    try {
      if (restored.current()) {
        if (restored.detached) saver.restoreDetached();
        putInBuffer(restored.contents);
      }
    } catch (error) {
      setNotice(error instanceof Error ? error.message : String(error));
    }
    recoveryPending.current = false;
    setRestored(null);
  }, [restored, loaded, putInBuffer, saver]);

  // Write the buffer down while it differs from the file. Read through a
  // callback so this costs nothing on the typing path — see lib/drafts.ts.
  useDraftKeeper({
    path: retainedResource ? null : path,
    enabled: file !== null && !startupReadOnly && retainUnsaved,
    read: () => ({
      // The view when it exists; what it last held when it does not. An
      // absent view is not an empty buffer.
      contents: viewRef.current ? authoredSource(viewRef.current.state) : lastTextRef.current,
      base: saver.baseContent(),
      detached: saver.isDetached() ? true : undefined,
    }),
  });

  // Who last touched each line, when the column is on.
  useBlame(railView, retainedResource ? null : path);

  // And the other side: a session owned by ANOTHER pane is paused in this
  // file. Drawn as the paused line here, because for reading it is one.
  const elsewhere = usePausedElsewhere();
  const pausedHere = elsewhere && elsewhere.path === path ? elsewhere.line : null;

  // Push the debugger's state into the editor: the gutter dots, the paused
  // line, and the values shown at the end of each line. `railView` is the
  // view as state, so this runs again once the editor exists.
  useEffect(() => {
    const view = viewRef.current;
    if (!view || !fileRef.current || !isDebuggable(fileRef.current.language)) return;
    const pausedLine = debug.pausedLine ?? pausedHere;
    view.dispatch({ effects: debugStateEffects({ ...debug, pausedLine }) });
    if (debug.pausedLine === null && pausedHere !== null) revealLine(view, pausedHere);
  }, [debug, pausedHere, railView]);

  // Diagnostics arrive in LSP line/character coordinates; translate against
  // the live buffer so they stay put while the user types. The workspace's
  // count and list read the same diagnostics from the file-problems store.
  useEffect(() => {
    setFileProblems(path, lsp.diagnostics);
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: setLspDiagnostics.of(
        diagnosticRanges(lsp.diagnostics, (p) => positionToOffset(view.state.doc, p)),
      ),
    });
  }, [lsp.diagnostics, path]);
  useEffect(() => () => forgetFileProblems(path), [path]);

  // A find hit asked for this file at a line. Claimed on mount as well as on
  // the event, because the request is usually made before this pane exists.
  useEffect(() => {
    const jump = () => {
      const view = viewRef.current;
      if (!view) return;
      const position = claimRevealPosition(path, authoredSource(view.state), view.dom);
      if (!position) return;
      const target = view.state.doc.line(Math.min(position.line, view.state.doc.lines));
      const anchor = Math.min(target.to, target.from + (position.character ?? 0));
      view.dispatch({
        selection: { anchor },
        // `center`, not `nearest`: a hit that lands on the last visible row
        // is technically shown and practically missed.
        effects: EditorView.scrollIntoView(anchor, { y: "center" }),
      });
      position.onRevealed?.();
      view.focus();
    };
    jump();
    return onRevealLine((asked) => {
      if (asked === path) jump();
    });
  });

  // Take the disk copy into the live buffer, flashing what changed.
  const adoptDiskCopy = useCallback(
    (fresh: PlainFile) => {
      setFile(fresh);
      saver.load(fresh.content, fresh.hash, fresh.save_guard);
      const view = viewRef.current;
      if (view) syncPlainSource(view, fresh.content);
    },
    [saver],
  );

  // Adopt this file into a literate document. The server proves the new
  // document weaves these exact bytes before writing anything, so a success
  // changes ownership and not one byte of content; a refusal (binary,
  // hick:-looking text, a taken document name) lands in the banner.
  const adopt = useCallback(async () => {
    if (adoptionPending.current || saver.isDetached() && saver.hasPendingEdits()) throw Error("Save or discard the private draft before changing this file's ownership.");
    adoptionPending.current = true;
    setAdopting(true); setAdoptError(null);
    try {
      const adopted = await api.adopt(path);
      plainOwnership.current = false;
      setGeneratedBy(adopted.doc_id);
      try { onAdoptedRef.current?.(adopted); return adopted; }
      catch (e) { const navigation_error = e instanceof Error ? e.message : String(e); setNotice(`Created ${adopted.doc_path}, but could not open it: ${navigation_error}`); return { ...adopted, navigation_error }; }
    } catch (e) { setAdoptError(e instanceof Error ? e.message : String(e)); throw e; }
    finally { adoptionPending.current = false; setAdopting(false); }
  }, [path, saver]);

  /** The merged text becomes the buffer, and the file is reloaded first so
   * the save that follows rides on the hash the disk actually has. */
  const acceptMerge = useCallback(
    async (text: string) => {
      const pending = mergeRef.current, view = viewRef.current;
      if (!pending || !view?.dom.isConnected || authoredSource(view.state) !== pending.bufferAtOpen) throw new Error("The merge buffer changed. Reopen the merge before accepting.");
      const captured = authoredSource(view.state);
      const fresh = await api.file(path);
      if (mergeRef.current !== pending || viewRef.current !== view || !view.dom.isConnected || authoredSource(view.state) !== captured) throw new Error("The merge owner changed while checking the disk. No merge was applied.");
      if (fresh.content !== pending.theirs) {
        setMerge({ ...pending, theirs: fresh.content });
        throw new Error("The file changed again on disk. Review the new merge before accepting.");
      }
      saver.resolve("reload");
      adoptDiskCopy(fresh);
      if (pending.detached) saver.restoreDetached();
      putInBuffer(text);
      recoveryPending.current = false;
      setMerge(null);
      return { buffer_applied: true, path, saved: false };
    },
    [saver, path, adoptDiskCopy, putInBuffer],
  );
  const launchProfile = async (choose=false) => {
    const source=viewRef.current && authoredSource(viewRef.current.state);
    if(source===null || !await saver.flushNow())return;
    if(!viewRef.current || authoredSource(viewRef.current.state)!==source)return;
    await launchWithProfiles({uri:`hick:///${path}`,expected_source:source},
      profile=>debug.start(undefined,{},undefined,profile),{choose,isCurrent:()=>!!viewRef.current&&authoredSource(viewRef.current.state)===source});
  };
  useUiReading("plain file status", { path, loaded: !!file, loaded_hash: file?.hash ?? null, generated_by: generatedBy, save: saveState, adopting, adoption_error: adoptError, notice, merge_open: !!merge, error: loadError, scope: "Last loaded file metadata plus current UI/save state; loaded_hash may predate later saves. Read editor source separately. Not an edit approval or verification." }, path);
  if (loadError && !startup) {
    return (
      <div className="plain-file-pane">
      <SurfaceInfo name={`File · ${path}`} editable destination="Edits save automatically to this file on disk." />
        <p className="error">{loadError}</p>
      </div>
    );
  }

  return (
    <div className="plain-file-pane">
      <SurfaceInfo name={`File · ${path}`} editable={!startupReadOnly && !retainedResource}
        destination={retainedResource ? "Deleted source. Restore it through Undo workspace change." : startupReadOnly
        ? "Read-only source preview." : saver.isDetached() ? "Use Save All to publish this private draft." : "Edits save automatically to this file on disk."} />
      <PaneHeaderContent part="actions"><div className="doc-tab-toolbar" role="toolbar" aria-label={`Actions for ${path}`}>
        {retainedResource ? <span className="save-state" role="status">Deleted source</span> : <>
          <MoreActions label={`More actions for ${path}`}>
            <PlainFileActions path={path} loaded={!!file && !startupReadOnly} generatedBy={generatedBy} saveKind={saveState.kind} adopting={adopting}
              sourceRevision={() => [fileRef.current?.hash, viewRef.current?.state.doc.toString()]} adopt={adopt} onError={setNotice} />
          </MoreActions>
          {notice && (
            <span className="muted plain-file__notice" role="status">
              {notice}
            </span>
          )}
          {saveState.kind === "draft" && <span className="save-state" role="status">Unsaved draft</span>}
          {(saveState.kind === "saving" || saveState.kind === "saved") && (
            <span
              className={`save-state save-state-${saveState.kind === "saving" ? "editing" : "saved"}`}
              role="status"
            >
              {saveState.kind === "saving" ? "Saving…" : "Saved"}
            </span>
          )}
        </>}
      </div></PaneHeaderContent>
      {adoptError && (
        <div className="banner banner-fail" role="alert">
          Could not adopt {path}: {adoptError}
        </div>
      )}
      {file && !startupReadOnly && !retainedResource && <DiskChangeNotice key={path} path={path} base={saver.baseContent()} source={liveText ?? file.content}
        read={async () => { const disk = await api.file(path); return { ...disk, source: disk.content }; }}
        apply={(disk, text) => {
          saver.resolve("reload"); adoptDiskCopy(disk);
          if (text !== disk.content) putInBuffer(text);
        }} />}
      {saveState.kind === "error" && (
        <div className="banner banner-fail" role="alert">
          Could not save {path}: {saveState.message}
        </div>
      )}
      {/* The debugger's chrome, above the file it debugs — nothing while
          idle. The same strip a document's block gets. */}
      <DebugStrip variableRoots={debug.variableRoots} valueNode={debug.valueNode} valueHistory={debug.valueHistory} valueNotice={debug.valueNotice} expandValue={debug.expandValue} collapseValue={debug.collapseValue} closeValues={debug.closeValues} refreshState={debug.refreshState} positionSourceBound={debug.positionSourceBound} sourceTestIntent={debug.sourceTestIntent} sourceTest={debug.sourceTest} releaseSourceTest={debug.releaseSourceTest} launchNotices={debug.launchNotices} breakpointNotice={debug.breakpointNotice} breakpointIntentions={debug.breakpointIntentions} onRunTo={debug.runTo} onJumpTo={debug.jumpTo} variables={debug.variables} threads={debug.threads} selectedThread={debug.selectedThread} threadStop={debug.threadStop} threadPending={debug.threadPending} threadSelecting={debug.threadSelecting} threadMessage={debug.threadMessage} frameVariablesPending={debug.frameVariablesPending} frameVariablesMessage={debug.frameVariablesMessage} stackMessage={debug.stackMessage} historicalFrames={debug.historicalFrames} observationScope={debug.observationScope} readThreads={debug.readThreads} selectThread={debug.selectThread} reading={debug.reading} readingChanged={observationSourceChanged(debug.reading, railView?.state.doc.toString())}
        editorView={railView}
        pausedLine={debug.pausedLine}
        status={debug.status}
        program={null}
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
        onSelectFrame={selectFrameAndReveal}
        onStep={debug.step}
        onJumpHere={() => {
          const view = viewRef.current;
          if (!view) return;
          debug.jumpTo(view.state.doc.lineAt(view.state.selection.main.head).number - 1);
        }}
        onStart={() => {void launchProfile();}}
        onStop={debug.stop}
        onAddWatch={() => {
          void askTextRef.current?.("Expression to watch:", "", { agent: true }).then((expression) => {
            if (expression) debug.addWatch(expression);
          });
        }}
        exceptionFilters={debug.exceptionFilters}
        onToggleExceptionFilter={debug.toggleExceptionFilter}
        onRemoveWatch={debug.removeWatch}
      />
      {!file && (startup
        ? loadError ? <ActionNotice kind="fail" retry={onRetryStartup && {label: "Retry source access", run: onRetryStartup}}>{loadError}</ActionNotice>
          : <p className="muted" role="status">Checking saved source authority…</p>
        : <p className="muted">Loading {path}…</p>)}
      {/* The merge REPLACES the editor rather than floating over it: the two
          sides plus their context need the whole pane to be readable, and a
          modal over the buffer would hide the very text being merged. The
          buffer is untouched underneath until the merge is accepted. */}
      {merge ? (
        <MergeView
          path={path}
          base={merge.base}
          ours={merge.ours}
          theirs={merge.theirs}
          oursLabel={merge.oursLabel}
          theirsLabel={merge.theirsLabel}
          sourceCurrent={() => !!viewRef.current?.dom.isConnected && authoredSource(viewRef.current.state) === merge.bufferAtOpen}
          onAccept={acceptMerge}
          onCancel={() => { recoveryPending.current = false; setMerge(null); }}
        />
      ) : null}
      <div className="output-editor with-right-rail" hidden={merge !== null}>
        <div ref={hostRef} className="editor-cm-host" />
        <RightRail view={railView} />
        <DebugLaunchRail view={railView} plainPath={path} enabled={!!file && !startupReadOnly && isDebuggable(file.language)} onStart={()=>{void launchProfile();}} onProfiles={()=>{void launchProfile(true);}} />
      </div>
    </div>
  );
}
