<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/PlainFilePane.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2f891a49445f"></a>
## `apps/web/src/components/PlainFilePane.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Opens, Edits, And Never Silently Loses A Byte](../../../../../guarantees/authoring/a-plain-file-opens-and-saves/contract.md), [Adopt a plain file without changing its bytes](../../../../../guarantees/authoring/adoption-is-byte-exact-or-refused/contract.md), [Conflict actions name the write](../../../../../guarantees/authoring/conflict-actions-name-the-write/contract.md), [Markdown Quotes And Task Lists Are Read As Prose](../../../../../guarantees/authoring/markdown-quotes-and-task-lists-are-read-as-prose/contract.md), [Several Cursors Edit At Once](../../../../../guarantees/authoring/several-cursors-edit-at-once/contract.md), [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [Word Navigation Is The Person's Choice](../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [Choose a launch without replacing the editor being read](../../../../../guarantees/debugging/debug-controls-preserve-editor-space/contract.md), [A Plain File Has The Same Language Server A Document Has](../../../../../guarantees/editor-intelligence/a-plain-file-has-the-same-language-server/contract.md), [Keep editor actions attached to the view they affect](../../../../../guarantees/editor-intelligence/actions-belong-to-their-subject/contract.md), [A Test Runs From The Line It Is Written On](../../../../../guarantees/execution/a-test-runs-from-the-line-it-is-written-on/contract.md).

<a id="f-2f891a49445f-PlainFilePane"></a>
### `PlainFilePane`

Observe disk even while edits are pending. The hash-guarded saver refuses
external conflicts; the disk notice then lets the reader ignore without a write,
reload or merge against the last acknowledged content. A clean buffer also
requires a choice unless safe automatic reload is enabled.


Plain and generated file search uses the same header icon and modal as document search. Replacement still passes through the existing read-only, region ownership and source-revision checks.

The window can prepare a private draft at an exact mounted plain editor. The
prepared transaction retains its authored line endings, native history and saved
guard. It switches off autosave before dispatch. The same owner publishes only
through an explicit guarded Save and rejects a receipt for a different source.
Pending group rollback restores the original editor state and the original save
plane. A marked recovery draft reenters the private plane when the window has
enabled draft recovery.

<storycode:copy id="f-2f891a49445f-PlainFilePane">
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
import { Compartment, EditorState } from "@codemirror/state";
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
} from "../lsp/cmLsp";
import { lspFeatures } from "../lsp/cmLspFeatures";
import type { CodeAction, LspLocation } from "../lsp/client";
import { openLocation, pathOfDocUri } from "../lib/revealLine";
import { forgetFileProblems, setFileProblems } from "../lib/fileProblems";
import { blameGutter } from "../editor/blameGutter";
import { useBlame } from "../editor/useBlame";
import { claimReveal, onRevealLine } from "../lib/revealLine";
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
  onAdopted,
  onReferences,
  askText,
  askChoice,
  retainUnsaved = false,
  onUnsaved,
  onSource,
  proposalOwner, proposalTabId, startup,
}: {
  path: string;
  onSource?: (path: string, source: string) => void;
  proposalOwner?: (tabId: string, read: () => RecordedPatchSource | null) => () => void;
  proposalTabId?: string;
  startup?: StartupPlainFile;
  /** Adoption succeeded: the file now has an owning document. The workspace
   * converts this very tab into a generated tab and opens the document. */
  onAdopted?: (adopted: AdoptResponse) => void;
  /** Find References asked from this file: the workspace shows the list. */
  onReferences?: (references: { locations: LspLocation[]; query: string }) => void;
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
  const sourceSignal=useMemo(()=>new DebugSourceSignal(),[path]),sourceIdentity=useMemo(()=>({path}),[path]),sourceNamespace=useRef<string|null>(null),sourceWorkspace=useRef<string|null>(null),sourceMounted=useRef(true);
  const adoptionPending = useRef(false), recoveryPending = useRef(retainUnsaved), plainOwnership = useRef(false);
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
    onSourceRef.current?.(path, text);
    setLiveTextState(text);
  }, [path]);
  const lsp = useWorkspaceLsp(file === null ? "" : path, liveText ?? "");
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
  const sourceCapability=useMemo<DebugSourceCapability>(()=>({read:uri=>{const text=lastTextRef.current,workspace=sourceNamespace.current,revision=sourceSignal.revision;if(!workspace||uri!==documentUri(path))throw Error('This source namespace is still loading or changed.');return {uri,workspace,namespace:workspace,identity:sourceIdentity,text,saved:!saver.hasPendingEdits()&&saver.baseContent()===text,current:()=>sourceMounted.current&&sourceNamespace.current===workspace&&sourceSignal.revision===revision&&lastTextRef.current===text};},subscribe:sourceSignal.subscribe}),[path,sourceIdentity,sourceSignal]);
  const debug = useFollowingDebugSession(path,undefined,undefined,undefined,sourceCapability) ?? IDLE_SESSION;
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
          if (!startup) return api.saveFile(path, content, baseHash, force);
          if (!guard) throw Error("Saved source authority is unavailable. Reload the exact file before saving.");
          return api.saveFile(path, content, baseHash, force, guard);
        },
        publish: async ({content, baseHash, guard, id}, saverCurrent) => {
          const namespace = sourceNamespace.current, workspace = sourceWorkspace.current;
          const current = () => !!namespace && !!workspace && sourceMounted.current && plainOwnership.current &&
            sourceNamespace.current === namespace && sourceWorkspace.current === workspace && saverCurrent();
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
    [path, startup],
  );
  actionsRef.current = {
    revision: () => viewRef.current ? authoredSource(viewRef.current.state) : lastTextRef.current, isDirty: () => saver.hasPendingEdits(),
    save: () => sourceMounted.current && viewRef.current?.dom.isConnected ? saver.saveNow() : Promise.resolve(!saver.hasPendingEdits()),
    discard: () => {
      const base = saver.discardPending();
      const view = viewRef.current;
      if (view) syncPlainSource(view, base);
    },
    retain: async () => {
      const contents = viewRef.current ? authoredSource(viewRef.current.state) : lastTextRef.current;
      if (startupReadOnly) return;
      await api.saveDraft({ path, contents, base: saver.baseContent(), saved_at: Date.now(), ...(saver.isDetached() ? {detached: true as const} : {}) });
    },
  };
  useEffect(() => () => saver.dispose(), [saver]);
  useEffect(() => proposalTabId && proposalOwner ? proposalOwner(proposalTabId, () => {
    const view = viewRef.current, namespace = sourceNamespace.current, workspace = sourceWorkspace.current;
    const guard = saver.baseGuard(), base = saver.baseContent(), revision = sourceSignal.revision;
    if (!view?.dom.isConnected || view.state.readOnly || !namespace || !workspace || !guard || mergeRef.current) return null;
    const document = view.state.doc, text = authoredSource(view.state);
    const current = () => sourceMounted.current && viewRef.current === view && view.dom.isConnected
      && !mergeRef.current && view.state.doc === document && authoredSource(view.state) === text && sourceSignal.revision === revision
      && sourceNamespace.current === namespace && sourceWorkspace.current === workspace;
    return { path, text, base, guard, revision, scope: {namespace, workspace}, current,
      prepareDraft(after): WorkspaceDraftStage | null {
        const writable = () => current() && plainOwnership.current && !adoptionPending.current && !recoveryPending.current &&
          !view.composing && !view.state.readOnly && view.state.facet(EditorView.editable);
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
  }) : undefined, [path, proposalOwner, proposalTabId, saver, setLiveText, sourceSignal]);
  // File > Save All: this pane owns its saver, so it answers for its own
  // buffer. See lib/flushSaves.ts.
  useEffect(() => onFlushSaves(includeDetached => {
    if (includeDetached) void saver.saveNow();
    else void saver.flushNow();
  }), [saver]);

  // The initial read. The pane renders its refusals — binary, too large,
  // missing — as text where the editor would be: the tab is still an honest
  // place, it just has nothing to edit.
  useEffect(() => {
    let live = true;
    recoveryPending.current = retainUnsaved;
    plainOwnership.current = false;
    const read = startup
      ? startup.authority.then(authority => ({file: authority.file, authority}))
      : api.file(path).then(file => ({file, authority: null}));
    read.then(
      ({file: loaded, authority}) => {
        if (!live) return;
        if (startup && (!startup.current() || loaded.path !== path)) {
          setLoadError("The original file request changed. Reopen this read-only preview to check its current source.");
          return;
        }
        setFile(loaded);
        setLoadError(null);
        setLiveText(loaded.content);
        lastTextRef.current = loaded.content;
        saver.load(loaded.content, loaded.hash, loaded.save_guard);
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
            plainOwnership.current = loaded.path === path && !!node && !node.dir && !node.doc_id && !node.generated_by;
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
  }, [path, saver, retainUnsaved, startup]);

  // The view is created ONCE, when the first load lands. Later content
  // arrives through syncPlainSource below — rebuilding the editor would throw
  // away the cursor and undo history mid-edit.
  const loaded = !!(file || startup);
  const fileRef = useRef(file);
  fileRef.current = file;
  useEffect(() => {
    const host = hostRef.current;
    const initial = fileRef.current ?? (startup && {
      path, content: startup.source.content, language: languageFromPath(path) ?? "text",
    });
    if (!host || !initial) return;

    // Where a definition landed. This file: select it. Another: the
    // workspace opens that tab, and its pane does the rest.
    const goTo = (target: LspNavigationTarget | LspLocation) => {
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
      const other = pathOfDocUri(target.uri) ?? target.uri.replace(/^hick-output:\/\/\//, "");
      openLocation(other, target.range.start.line + 1);
    };
    const lspClient = lspRef.current.client;
    const lspUri = documentUri(path);

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
          coverageGutter(() => initial.path, () => fileRef.current?.content),
          performanceInline(() => initial.path, () => fileRef.current?.content),
          wrapGutterMarkers(),
          // The breakpoint gutter, the paused line, the inline values — the
          // same layer a document's editor wears, told that every line here
          // is code. Only for a language hick can debug: a gutter that
          // accepts a dot it can never bind is a promise it cannot keep.
          ...(isDebuggable(initial.language)
            ? debugEditor({
                language: initial.language,
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
          ...languageExtensions(initial.language),
          // A run mark beside every test the file's language has a shape
          // for; a click runs that test in a terminal named after it.
          ...testGutter({
            language: initial.language,
            onError: setNotice,
            serializedSource: () => saver.baseContent(),
            onRun: async (mark, expected_source_hash, current) => {
              const session = await api.runTest({ path: initial.path, name: mark.name, language: initial.language, expected_source_hash });
              if (current() && session.source_checked_at_admission && session.source_hash === expected_source_hash) showTerminalRequest(session.id, session.title);
              return session;
            },
          }),
          ...(isMarkdownPath(initial.path) ? [markdownStyling(), taskCheckboxes(), renderedMath()] : []),
          history(),
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
            onNavigate: goTo,
            onReferences: (locations, from) => {
              const live = viewRef.current;
              const at = live ? positionToOffset(live.state.doc, from.range.start) : 0;
              const query = live ? (identifierAt(live.state.doc.toString(), at) ?? "") : "";
              onReferencesRef.current?.({ locations, query });
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
          // Prose wraps at the measure; a fenced code block keeps its lines
          // and takes the whole pane. In a non-markdown file EVERY line is
          // code, which is exactly what `fencedCodeRanges` returning the whole
          // buffer expresses.
          proseWrap((state) =>
            isMarkdownPath(initial.path)
              ? fencedCodeRanges(state.doc.toString())
              : [[0, state.doc.length] as [number, number]],
          ),
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
    const unregisterSearch = registerSearchEditor(path, view);
    setRailView(view);
    return () => {
      unregisterSearch();
      forgetFocusedEditor(view);
      view.destroy();
      viewRef.current = null;
      setRailView(null);
    };
    // Mounted once per load; the saver is stable per path.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loaded, saver]);

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
    path,
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
  useBlame(railView, path);

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
      const line = claimReveal(path);
      if (line === null) return;
      const target = view.state.doc.line(Math.min(line, view.state.doc.lines));
      view.dispatch({
        selection: { anchor: target.from },
        // `center`, not `nearest`: a hit that lands on the last visible row
        // is technically shown and practically missed.
        effects: EditorView.scrollIntoView(target.from, { y: "center" }),
      });
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
      <SurfaceInfo name={`File · ${path}`} editable={!startupReadOnly} destination={startupReadOnly
        ? "Read-only source preview." : saver.isDetached() ? "Use Save All to publish this private draft." : "Edits save automatically to this file on disk."} />
      <PaneHeaderContent part="actions"><div className="doc-tab-toolbar" role="toolbar" aria-label={`Actions for ${path}`}>
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
      </div></PaneHeaderContent>
      {adoptError && (
        <div className="banner banner-fail" role="alert">
          Could not adopt {path}: {adoptError}
        </div>
      )}
      {file && !startupReadOnly && <DiskChangeNotice key={path} path={path} base={saver.baseContent()} source={liveText ?? file.content}
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
        ? <p className="muted" role="status">{loadError ?? "Checking saved source authority…"}</p>
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
</storycode:copy>

### Assemble `apps/web/src/components/PlainFilePane.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PlainFilePane.tsx"><storycode:paste select="#f-2f891a49445f-PlainFilePane"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-d923b2fdae48"></a>
## `apps/web/src/components/PlainFilePane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [Choose a launch without replacing the editor being read](../../../../../guarantees/debugging/debug-controls-preserve-editor-space/contract.md), [A Plain File Has The Same Language Server A Document Has](../../../../../guarantees/editor-intelligence/a-plain-file-has-the-same-language-server/contract.md).

<a id="f-d923b2fdae48-ON_DISK"></a>
### `ON_DISK`

<storycode:copy id="f-d923b2fdae48-ON_DISK">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { EditorView } from "@codemirror/view";
// Unsaved work coming back, and what happens when the file moved on while the
// app was closed.
//
// Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
//
// Driven through the pane against a mocked API client, because the decision
// being tested — restore quietly, drop a stale draft, or open a merge — is
// made from what those two requests answer.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { PlainFilePane } from "./PlainFilePane";
import { undo } from "@codemirror/commands";
import { api } from "../api/client";
import * as apiClient from "../api/client";
import type { RecordedPatchSource } from "../simple/AgentProposalAdapter";
import { authoredSource } from "../simple/SourceText";
import { historySourceHash } from "../simple/SourceHistory";
import type { WorkspaceDraft } from "../api/types";
import type { StartupPlainFile } from "../views/folderPane";
import { setSharedRealtime, type Realtime } from "../api/realtime";
import { createLspChannel, encodeLspFrame, type JsonRpcMessage } from "../lsp/channel";
import { resetWorkspaceLsp } from "../lsp/useLsp";
import { resetWorkspaceDebugger } from "../debug/useDebugger";
import { publishPausedElsewhere, resetPausedElsewhere } from "../lib/pausedElsewhere";
import { PlainDebugHosts, resetPlainDebugHosts } from "../debug/plainDebugHosts";
import { allFileProblems, resetFileProblems } from "../lib/fileProblems";

const ON_DISK = "one\ntwo\nthree\n";

</storycode:copy>
<a id="f-d923b2fdae48-serve"></a>
### `serve`

<storycode:copy id="f-d923b2fdae48-serve">
function serve({ content, drafts }: { content: string; drafts: WorkspaceDraft[] }) {
  vi.spyOn(api, "file").mockResolvedValue({
    path: "notes.md",
    language: "markdown",
    content,
    hash: `h:${content}`,
  });
  vi.spyOn(api, "drafts").mockResolvedValue({ drafts });
  vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
  vi.spyOn(api, "saveFile").mockResolvedValue({ path: "notes.md", hash: "h:saved" });
  vi.spyOn(api, "files").mockResolvedValue({
    root: "notebook",
    tree: [{ name: "notes.md", path: "notes.md", dir: false }],
  });
}

</storycode:copy>
<a id="f-d923b2fdae48-draft"></a>
### `draft`

<storycode:copy id="f-d923b2fdae48-draft">
const draft = (over: Partial<WorkspaceDraft> = {}): WorkspaceDraft => ({
  path: "notes.md",
  contents: "one\nMINE\nthree\n",
  base: ON_DISK,
  saved_at: 1,
  ...over,
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement">
beforeEach(() => {
  vi.restoreAllMocks();
  resetWorkspaceLsp();
  resetWorkspaceDebugger();
  resetPausedElsewhere();
  resetPlainDebugHosts();
  resetFileProblems();
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement-2">
afterEach(() => {
  cleanup();
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement-3">
it("promotes the same startup editor and includes its physical guard in later saves", async () => {
  const path = "startup.py", namespace = "/workspace", workspace = "a".repeat(32), content = "value = 1\n";
  let confirm!: (value: Awaited<StartupPlainFile["authority"]>) => void;
  const authority = new Promise<Awaited<StartupPlainFile["authority"]>>(resolve => { confirm = resolve; });
  const save = vi.spyOn(api, "saveFile").mockRejectedValue(Object.assign(Error("Source file was replaced"), {status: 409}));
  const startup: StartupPlainFile = {source: {version: 1, path, content, root_path: namespace, workspace_id: workspace},
    authority, current: () => true};
  const mounted = render(<PlainFilePane path={path} startup={startup} />);
  const editor = EditorView.findFromDOM(mounted.container.querySelector<HTMLElement>(".cm-editor")!)!;
  expect(editor.state.doc.toString()).toBe(content);
  expect(editor.state.readOnly).toBe(true);
  act(() => editor.dispatch({changes: {from: 0, insert: "blocked"}, userEvent: "input.type", selection: {anchor: 2}}));
  expect(editor.state.doc.toString()).toBe(content);
  expect(save).not.toHaveBeenCalled();
  act(() => editor.dispatch({selection: {anchor: 2, head: 7}}));
  const selection = editor.state.selection;
  expect(selection.main.empty).toBe(false);

  await act(async () => confirm({file: {path, content, language: "python", hash: "base", save_guard: "original-source"},
    namespace, workspace, generatedBy: null}));
  expect(EditorView.findFromDOM(mounted.container.querySelector<HTMLElement>(".cm-editor")!)).toBe(editor);
  expect(editor.state.readOnly).toBe(false);
  expect(editor.state.selection.eq(selection)).toBe(true);
  act(() => editor.dispatch({changes: {from: 8, to: 9, insert: "2"}, userEvent: "input.type"}));
  await waitFor(() => expect(save).toHaveBeenCalledWith(path, "value = 2\n", "base", false, "original-source"));
  expect(editor.state.doc.toString()).toBe("value = 2\n");
  expect(save).toHaveBeenCalledTimes(1);
});

describe("reopening a file that had unsaved changes", () => {
  it("keeps a private marked recovery draft off the ordinary autosave path", async () => {
    const recovered = draft({ detached: true });
    serve({ content: ON_DISK, drafts: [recovered] });
    const view = render(<PlainFilePane path="notes.md" retainUnsaved />);
    await screen.findByText("Unsaved draft");
    const editor = EditorView.findFromDOM(view.container.querySelector<HTMLElement>(".cm-editor")!)!;
    expect(authoredSource(editor.state)).toBe(recovered.contents);
    await act(async () => { await new Promise(resolve => setTimeout(resolve, 650)); });
    expect(api.saveFile).not.toHaveBeenCalled();
    await act(async () => view.unmount());
    expect(api.saveDraft).toHaveBeenCalledWith(expect.objectContaining({ contents: recovered.contents, base: ON_DISK, detached: true }));
    expect(api.saveFile).not.toHaveBeenCalled();
  });

  it("prepares a byte-exact detached transaction and publishes only the explicit guarded Save", async () => {
    const base = "one\r\ntwo 😀\r\nthree\n", after = "one\r\nchanged 😀\r\nthree\n";
    const path = "notes.md", workspace = "a".repeat(32);
    serve({ content: base, drafts: [] });
    vi.mocked(api.file).mockResolvedValue({path, content: base, language: "markdown", hash: "original-hash", save_guard: "original-guard"});
    vi.mocked(api.files).mockResolvedValue({root: "notebook", root_path: "/workspace", workspace_id: workspace, tree: [{name: path, path, dir: false}]});
    const originalRequest = apiClient.request;
    const publish = vi.fn(async (_method: string, _route: string, raw?: unknown) => {
      const body = raw as {request_id: string; content: string};
      return {version: 1, history_protocol: 1, path, hash: "saved-hash", save_guard: "next-guard", request_id: body.request_id,
        source_hash: await historySourceHash(body.content), history: {available: true, outcome: "published", id: body.request_id}};
    });
    vi.spyOn(apiClient, "request").mockImplementation(async <T,>(method: string, route: string, raw?: unknown) =>
      route === "/api/source-history/file" ? await publish(method, route, raw) as T : originalRequest<T>(method, route, raw));
    let read = (): RecordedPatchSource | null => null;
    let save = async () => false;
    const view = render(<PlainFilePane path={path} proposalTabId="original-tab"
      proposalOwner={(_tab, owner) => { read = owner; return () => {}; }}
      onUnsaved={(_dirty, actions) => { save = actions.save; }} />);
    await waitFor(() => expect(read()?.prepareDraft?.(after)?.current()).toBe(true));
    const editor = EditorView.findFromDOM(view.container.querySelector<HTMLElement>(".cm-editor")!)!;
    act(() => editor.dispatch({selection: {anchor: 1, head: 3}}));
    const originalSelection = editor.state.selection;
    const aborted = read()!.prepareDraft!(after)!;
    act(() => { aborted.commit(); aborted.rollback(); });
    expect(EditorView.findFromDOM(view.container.querySelector<HTMLElement>(".cm-editor")!)).toBe(editor);
    expect(editor.state.selection.eq(originalSelection)).toBe(true);
    expect(authoredSource(editor.state)).toBe(base);
    act(() => { expect(undo(editor)).toBe(false); });
    act(() => read()!.prepareDraft!(after)!.commit());
    expect(authoredSource(editor.state)).toBe(after);
    expect(EditorView.findFromDOM(view.container.querySelector<HTMLElement>(".cm-editor")!)).toBe(editor);
    act(() => { expect(undo(editor)).toBe(true); });
    expect(authoredSource(editor.state)).toBe(base);
    act(() => read()!.prepareDraft!(after)!.commit());
    await act(async () => { await new Promise(resolve => setTimeout(resolve, 650)); });
    expect(api.saveFile).not.toHaveBeenCalled();
    expect(publish).not.toHaveBeenCalled();
    await act(async () => { expect(await save()).toBe(true); });
    expect(publish).toHaveBeenCalledOnce();
    expect(publish).toHaveBeenCalledWith("PUT", "/api/source-history/file", expect.objectContaining({
      version: 1, path, content: after, base_hash: "original-hash", save_guard: "original-guard",
    }));
    expect(read()?.base).toBe(after);
    expect(read()?.guard).toBe("next-guard");
    expect(api.saveFile).not.toHaveBeenCalled();
  });

  it("puts the unsaved text back, without asking", () => {
    // The common case by a wide margin. A dialog here would train people to
    // dismiss dialogs, and the work is theirs — nothing was lost or decided.
    serve({ content: ON_DISK, drafts: [draft()] });
    render(<PlainFilePane path="notes.md" retainUnsaved />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("MINE");
      expect(screen.queryByTestId("merge-view")).toBeNull();
    });
  });

  it("throws away a draft the file already agrees with", () => {
    // Somebody saved the same text from elsewhere; there is nothing unsaved.
    const discard = vi.fn().mockResolvedValue({ ok: true });
    serve({ content: ON_DISK, drafts: [draft({ contents: ON_DISK, base: "older\n" })] });
    vi.spyOn(api, "discardDraft").mockImplementation(discard);
    render(<PlainFilePane path="notes.md" retainUnsaved />);
    return waitFor(() => expect(discard).toHaveBeenCalledWith("notes.md"));
  });

  it("opens a merge when the file moved on and the buffer did too", () => {
    serve({
      content: "one\nTHEIRS\nthree\n",
      drafts: [draft({ contents: "one\nMINE\nthree\n", base: ON_DISK })],
    });
    render(<PlainFilePane path="notes.md" retainUnsaved />);
    return waitFor(() => {
      const merge = screen.getByTestId("merge-view");
      expect(merge.textContent).toMatch(/changed while you were away/i);
      expect(merge.textContent).toMatch(/still needs you/i);
    });
  });

  it("opens normally when there is no draft for this file", () => {
    serve({ content: ON_DISK, drafts: [draft({ path: "somewhere/else.md" })] });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("two");
      expect(screen.queryByTestId("merge-view")).toBeNull();
    });
  });

  it("opens normally when the draft store is unreachable", () => {
    // A machine with no data directory, or a read-only home. The file opens
    // as it is on disk, which is what would have happened anyway.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "drafts").mockRejectedValue(new Error("no store"));
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("two");
    });
  });
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement-4">
describe("a file that is already the output of a document", () => {
  it("is not offered a button to make it literate", () => {
    // It already is. Offering the verb tells the reader their document is not
    // what it plainly is.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "files").mockResolvedValue({
      root: "notebook",
      tree: [{ name: "notes.md", path: "notes.md", dir: false, generated_by: "d1" }],
    });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(screen.queryByRole("button", { name: /make literate/i })).toBeNull();
      expect(screen.getByText(/already literate/i)).toBeTruthy();
    });
  });

  it("keeps the button for a file nobody writes", () => {
    serve({ content: ON_DISK, drafts: [] });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() =>
      expect(screen.getByRole("button", { name: /make literate/i })).toBeTruthy(),
    );
  });

  it("keeps the button when the listing is unavailable", () => {
    // The adopt route's own refusal is the backstop; a failed listing must
    // not remove a verb that might be valid.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "files").mockRejectedValue(new Error("no listing"));
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() =>
      expect(screen.getByRole("button", { name: /make literate/i })).toBeTruthy(),
    );
  });
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-5"></a>
### `expression_statement`

Protects docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md

<storycode:copy id="f-d923b2fdae48-expression_statement-5">
// Protects docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md
describe("a plain file and the language server", () => {
  /** A realtime whose language channel is a loopback wire this test holds
   * both ends of. Not server-authoritative, which is how the workspace
   * connection is told to reuse it rather than open a socket. */
  function wire() {
    const outbound: JsonRpcMessage[] = [];
    const channel = createLspChannel({
      send: (frame) => {
        outbound.push(JSON.parse(new TextDecoder().decode(frame.subarray(1))));
      },
    });
    const realtime: Realtime = {
      serverAuthoritative: false,
      whenSynced: () => Promise.resolve(),
      bindDoc: () => {},
      onRunEvent: () => () => {},
      lsp: () => channel,
      close: () => {},
      reopen: () => {},
    };
    setSharedRealtime(realtime);
    const inject = (msg: JsonRpcMessage) => channel.handleFrame(encodeLspFrame(msg));
    return { outbound, inject };
  }

  it("opens the file with the language server at its own path, once loaded", async () => {
    const { outbound } = wire();
    serve({ content: ON_DISK, drafts: [] });
    render(<PlainFilePane path="notes.md" />);
    await waitFor(() => {
      const opened = outbound.find((m) => "method" in m && m.method === "textDocument/didOpen");
      expect(opened).toBeDefined();
      // The file's own path, and the text the screen shows — never an empty
      // stand-in sent before the load landed.
      expect(opened).toMatchObject({
        params: { textDocument: { uri: "hick:///notes.md", text: ON_DISK } },
      });
    });
  });

  it("puts what the server says is wrong into the file-problems store", async () => {
    const { outbound, inject } = wire();
    serve({ content: ON_DISK, drafts: [] });
    render(<PlainFilePane path="notes.md" />);
    await waitFor(() =>
      expect(outbound.some((m) => "method" in m && m.method === "textDocument/didOpen")).toBe(true),
    );
    inject({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: {
        uri: "hick:///notes.md",
        diagnostics: [
          {
            range: { start: { line: 1, character: 0 }, end: { line: 1, character: 3 } },
            severity: 1,
            message: "two is not a number",
          },
        ],
      },
    });
    await waitFor(() => {
      expect(allFileProblems()).toEqual([
        expect.objectContaining({
          path: "notes.md",
          diagnostics: [expect.objectContaining({ message: "two is not a number" })],
        }),
      ]);
      // And drawn in the buffer, where the reader is.
      expect(document.querySelector(".cm-lsp-diagnostic, .cm-lsp-error, [class*=\"cm-lsp\"]")).not.toBeNull();
    });
  });
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-6"></a>
### `expression_statement`

Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md

<storycode:copy id="f-d923b2fdae48-expression_statement-6">
// Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
describe("closing the pane", () => {
  it("never writes a draft of nothing for a buffer that was never edited", async () => {
    // The final flush runs after the view is destroyed. Reading "" from the
    // dead view used to record an empty draft, which the next mount restored
    // and saved — a 159-line file emptied on disk by switching tabs.
    serve({ content: ON_DISK, drafts: [] });
    const saveDraft = vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
    const view = render(<PlainFilePane path="notes.md" />);
    await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("two"));
    view.unmount();
    expect(saveDraft).not.toHaveBeenCalled();
    expect(saveDraft.mock.calls.every((call) => call[0].contents !== "")).toBe(true);
  });
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-7"></a>
### `expression_statement`

Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md

<storycode:copy id="f-d923b2fdae48-expression_statement-7">
// Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
describe("a plain file and the debugger", () => {
  const APP = "import os\n\ndef main():\n    print(os.getcwd())\n\nmain()\n";

  /** A realtime whose debug channel this test holds both ends of. */
  function debugWire() {
    const sent: Record<string, unknown>[] = [];
    let handler: ((frame: Uint8Array) => boolean) | null = null;
    const realtime: Realtime = {
      serverAuthoritative: false,
      whenSynced: () => Promise.resolve(),
      bindDoc: () => {},
      onRunEvent: () => () => {},
      lsp: () => null,
      close: () => {},
      reopen: () => {},
      debug: () => ({
        send: (frame) => {
          sent.push(JSON.parse(new TextDecoder().decode(frame.subarray(1))));
        },
      }),
      onDebugFrame: (h) => {
        handler = h;
      },
    };
    setSharedRealtime(realtime);
    const deliver = (event: Record<string, unknown>) => {
      const body = new TextEncoder().encode(JSON.stringify(event));
      const frame = new Uint8Array(body.length + 1);
      frame[0] = 0x03;
      frame.set(body, 1);
      handler?.(frame);
    };
    return { sent, deliver };
  }

  function serveFile(path: string, language: string, content: string) {
    vi.spyOn(api, "file").mockResolvedValue({ path, language, content, hash: `h:${content}` });
    vi.spyOn(api, "drafts").mockResolvedValue({ drafts: [] });
    vi.spyOn(api, "files").mockResolvedValue({ root: "repo", tree: [{ name: path, path, dir: false }] });
  }

  it("offers Debug and a breakpoint gutter for a language hick can debug", async () => {
    debugWire();
    serveFile("tools/app.py", "python", APP);
    render(<><PlainDebugHosts /><PlainFilePane path="tools/app.py" /></>);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Debug tools/app.py" })).toBeDefined();
      expect(document.querySelector(".cm-breakpoint-gutter")).not.toBeNull();
    });
  });

  it("offers neither for a file hick cannot debug", async () => {
    // A gutter that takes a dot it can never bind is a promise the pane
    // cannot keep, and a Debug button that only ever fails is worse.
    debugWire();
    serveFile("notes.md", "markdown", ON_DISK);
    render(<><PlainDebugHosts /><PlainFilePane path="notes.md" /></>);
    await waitFor(() => expect(document.querySelector(".cm-content")).not.toBeNull());
    expect(screen.queryByRole("button", { name: "Debug tools/app.py" })).toBeNull();
    expect(document.querySelector(".cm-breakpoint-gutter")).toBeNull();
  });

  it("draws the paused line when another pane's session is stopped in this file", async () => {
    // The pane for `app.py` owns the session and stepped into `helpers.py`.
    // This pane holds helpers.py; for reading, the paused line is its own.
    debugWire();
    serveFile("tools/helpers.py", "python", "def double(x):\n    return x * 2\n");
    render(<><PlainDebugHosts /><PlainFilePane path="tools/helpers.py" /></>);
    await waitFor(() => expect(document.querySelector(".cm-breakpoint-gutter")).not.toBeNull());
    publishPausedElsewhere("tools/app.py", { path: "tools/helpers.py", line: 1 });
    await waitFor(() => expect(document.querySelector(".cm-paused-arrow")).not.toBeNull());
    // Only the owner clears it, and it does when the program moves on.
    publishPausedElsewhere("tools/other.py", null);
    expect(document.querySelector(".cm-paused-arrow")).not.toBeNull();
    publishPausedElsewhere("tools/app.py", null);
    await waitFor(() => expect(document.querySelector(".cm-paused-arrow")).toBeNull());
  });

  it("starts from the gutter at the file's own path and exposes stepping", async () => {
    const { sent, deliver } = debugWire();
    vi.spyOn(apiClient, "request").mockResolvedValue({
      fingerprint: "plain-project", viewed: "tools/app.py", workspace: "repo",
      configuration: ".hick-launch.json",
      profiles: [{id: "app:tools/app.py", name: "Application", can_debug: true}],
    });
    serveFile("tools/app.py", "python", APP);
    render(<><PlainDebugHosts /><PlainFilePane path="tools/app.py" /></>);
    const button = await screen.findByRole("button", { name: "Debug tools/app.py" });
    button.click();
    await waitFor(() =>
      expect(sent[0]).toMatchObject({ op: "start", doc: "hick:///tools/app.py", breakpoints: [], profile: {id: "app:tools/app.py", fingerprint: "plain-project"} }),
    );
    // An answer for ANOTHER pane's file is not this pane's business.
    deliver({
      event: "started",
      session: "dbg-7",
      doc: "hick:///other.py",
      capabilities: {},
      breakpoints: [],
    });
    deliver({
      event: "started",
      session: "dbg-0",
      doc: "hick:///tools/app.py",
      capabilities: {},
      breakpoints: [],
    });
    deliver({
      event: "stopped",
      session: "dbg-0",
      reason: "breakpoint",
      line: 3,
      frames: [{ id: 1, name: "main", line: 3, source: "tools/app.py", in_document: true }],
      variables: [],
    });
    await waitFor(() => {
      const strip = screen.getByRole("toolbar", { name: "Debugger" });
      expect(strip.textContent).toContain("paused");
    });
    // The launch action stays in the gutter; stepping uses the session controls.
    expect(screen.getByRole("button", { name: "Debug tools/app.py" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Step over" })).toBeDefined();
  });
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement-8">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reopens the merge against newer disk text instead of applying an old decision", async () => {
  serve({ content: "one\nTHEIRS\nthree\n", drafts: [draft()] });
  render(<PlainFilePane path="notes.md" retainUnsaved />);
  await screen.findByTestId("merge-view");
  await waitFor(() => expect((screen.getByRole("button", { name: /accept, keeping mine/i }) as HTMLButtonElement).disabled).toBe(false));
  vi.mocked(api.file).mockResolvedValue({ path: "notes.md", language: "markdown", content: "one\nNEW DISK\nthree\n", hash: "new-disk" });
  fireEvent.click(screen.getByRole("button", { name: /accept, keeping mine/i }));
  await waitFor(() => expect(screen.getByTestId("merge-view").textContent).toContain("NEW DISK"));
  expect(api.saveFile).not.toHaveBeenCalled();
  expect(api.discardDraft).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement-9">
it("applies a reviewed merge draft only after the observed disk side still matches", async () => {
  serve({ content: "one\nTHEIRS\nthree\n", drafts: [draft()] });
  render(<PlainFilePane path="notes.md" retainUnsaved />);
  await screen.findByTestId("merge-view");
  await waitFor(() => expect((screen.getByRole("button", { name: /accept, keeping mine/i }) as HTMLButtonElement).disabled).toBe(false));
  fireEvent.click(screen.getByRole("button", { name: /accept, keeping mine/i }));
  await waitFor(() => expect(screen.queryByTestId("merge-view")).toBeNull());
  expect(document.querySelector(".cm-content")?.textContent).toContain("MINE");
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement-10">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("keeps newer buffer edits when an explicit disk reload returns late", async () => {
  serve({ content: ON_DISK, drafts: [] });
  vi.mocked(api.saveFile).mockRejectedValue(Object.assign(new Error("disk changed"), { status: 409 }));
  render(<PlainFilePane path="notes.md" />);
  await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("two"));
  const editor = EditorView.findFromDOM(document.querySelector<HTMLElement>(".cm-editor")!)!;
  act(() => { editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "FIRST EDIT" }, userEvent: "input" }); });
  vi.mocked(api.file).mockResolvedValue({ path: "notes.md", language: "markdown", content: "new disk", hash: "new-disk-hash" });
  fireEvent(window, new Event("hickory:files-changed"));
  await screen.findByRole("button", { name: "Reload and discard unsaved changes" });
  let finish!: (value: Awaited<ReturnType<typeof api.file>>) => void;
  vi.mocked(api.file).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  fireEvent.click(screen.getByRole("button", { name: "Reload and discard unsaved changes" }));
  act(() => { editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "NEWER EDIT" }, userEvent: "input" }); });
  await act(async () => { finish({ path: "notes.md", language: "markdown", content: "new disk", hash: "new-disk-hash" }); });
  await screen.findByText(/newer edits were kept/);
  expect(editor.state.doc.toString()).toBe("NEWER EDIT");
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement-11">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retains actual adoption success if opening its created document fails", async () => {
  serve({ content: ON_DISK, drafts: [] });
  const adopted = { doc_id: "created-doc", doc_path: "notes.literate.md", file_path: "notes.md", output_path: "notes.md", created: true };
  const adopt = vi.spyOn(api, "adopt").mockResolvedValue(adopted);
  const onAdopted = vi.fn(() => { throw new Error("navigation refused"); });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PlainFilePane path="notes.md" onAdopted={onAdopted} /></UiActionContext.Provider>);
  await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("two"));
  await act(async () => { fireEvent.click(screen.getByLabelText("More actions for notes.md")); });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  const handle = actions().find(a => a.label === "Make current file literate")!.handle;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle, arguments: {}, request_id: "adopt" }); });
  expect(result).toMatchObject({ status: "result", outcome: { status: "completed", value: { owner_reply: JSON.stringify({ ...adopted, navigation_error: "navigation refused" }) } } });
  expect(adopt).toHaveBeenCalledWith("notes.md");
  expect(onAdopted).toHaveBeenCalledWith(adopted);
  expect(screen.getByText(/Created notes.literate.md, but could not open it/)).toBeTruthy();
  expect(screen.getByText(/already literate/)).toBeTruthy();
  expect(actions().some(a => a.label === "Make current file literate")).toBe(false);
});

</storycode:copy>
<a id="f-d923b2fdae48-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-d923b2fdae48-expression_statement-12">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("agent editor input uses the existing guarded plain-file autosave pipeline", async () => {
  serve({ content: ON_DISK, drafts: [] });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PlainFilePane path="notes.md" /></UiActionContext.Provider>);
  await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("two"));
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  await waitFor(() => expect(actions().some(a => a.label === "Replace file editor range")).toBe(true));
  const replace = actions().find(a => a.label === "Replace file editor range")!;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: replace.handle, arguments: { from: 4, to: 7, text: "AGENT" }, request_id: "input" }); });
  expect(result).toMatchObject({ status: "result", outcome: { value: { source_changed: true, approved: false, verified: false } } });
  await waitFor(() => expect(api.saveFile).toHaveBeenCalledWith("notes.md", "one\nAGENT\nthree\n", `h:${ON_DISK}`, false));
  expect(screen.getByText("Saved")).toBeTruthy();
});
</storycode:copy>

### Assemble `apps/web/src/components/PlainFilePane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PlainFilePane.test.tsx"><storycode:paste select="#f-d923b2fdae48-ON_DISK"/><storycode:paste select="#f-d923b2fdae48-serve"/><storycode:paste select="#f-d923b2fdae48-draft"/><storycode:paste select="#f-d923b2fdae48-expression_statement"/><storycode:paste select="#f-d923b2fdae48-expression_statement-2"/><storycode:paste select="#f-d923b2fdae48-expression_statement-3"/><storycode:paste select="#f-d923b2fdae48-expression_statement-4"/><storycode:paste select="#f-d923b2fdae48-expression_statement-5"/><storycode:paste select="#f-d923b2fdae48-expression_statement-6"/><storycode:paste select="#f-d923b2fdae48-expression_statement-7"/><storycode:paste select="#f-d923b2fdae48-expression_statement-8"/><storycode:paste select="#f-d923b2fdae48-expression_statement-9"/><storycode:paste select="#f-d923b2fdae48-expression_statement-10"/><storycode:paste select="#f-d923b2fdae48-expression_statement-11"/><storycode:paste select="#f-d923b2fdae48-expression_statement-12"/></storycode:file>


</storycode:doc>
