import { registerSearchEditor } from "../lib/workspaceSearch";
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

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { search, searchKeymap } from "@codemirror/search";
import { wordMotionBindings } from "../editor/wordMotion";

import { api } from "../api/client";
import type { AdoptResponse, PlainFile } from "../api/types";
import { changeFlashField, syncAndFlash } from "../editor/changeFlash";
import { languageExtensions } from "../editor/languages";
import { forgetFocusedEditor, markFocusedEditor } from "../editor/activeEditor";
import { fencedCodeRanges, isMarkdownPath, markdownStyling } from "../editor/markdownStyling";
import { taskCheckboxes } from "../editor/taskList";
import { renderedMath } from "../editor/mathRender";
import { proseWrap } from "../editor/wrapColumn";
import { editorChrome } from "../editor/chrome";
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
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { createPlainSaver, type PlainSaveState } from "../lib/plainFileSave";
import { draftDisposition, useDraftKeeper } from "../lib/drafts";
import { onFlushSaves } from "../lib/flushSaves";
import { MergeView } from "./MergeView";
import { DivergedBanner } from "./DivergedBanner";
import { DebugStrip } from "../debug/DebugStrip";
import { usePausedElsewhere } from "../lib/pausedElsewhere";
import { debugEditor, debugStateEffects, revealLine } from "../debug/cmDebug";
import { isDebuggable } from "../debug/languages";
import { IDLE_SESSION } from "../debug/useDebugger";
import { isWorkspaceSource, usePlainDebugSession } from "../debug/plainDebugHosts";

export function PlainFilePane({
  path,
  onAdopted,
  onReferences,
  askText,
  askChoice,
  retainUnsaved = false,
  onUnsaved,
}: {
  path: string;
  /** Adoption succeeded: the file now has an owning document. The workspace
   * converts this very tab into a generated tab and opens the document. */
  onAdopted?: (adopted: AdoptResponse) => void;
  /** Find References asked from this file: the workspace shows the list. */
  onReferences?: (references: { locations: LspLocation[]; query: string }) => void;
  /** The window's prompt, for a rename's new name. */
  askText?: (title: string, initial: string) => Promise<string | null>;
  /** The window's prompt, for choosing a code action. */
  askChoice?: <T>(title: string, options: { label: string; value: T }[]) => Promise<T | null>;
  /** Keep a recovery draft for this already-named file. */
  retainUnsaved?: boolean;
  onUnsaved?: (
    dirty: boolean,
    actions: {
      save: () => Promise<boolean>;
      discard: () => void;
      retain: () => Promise<void>;
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
  // A draft to put back into the buffer once the view exists. Held as state
  // rather than applied immediately because the load lands before the editor
  // is built.
  const [restored, setRestored] = useState<string | null>(null);
  // Two versions of this file that both have changes worth keeping. Set when
  // a save conflicts, and when a restored draft finds the file has moved on.
  const [merge, setMerge] = useState<{
    base: string;
    ours: string;
    theirs: string;
    oursLabel: string;
    theirsLabel: string;
  } | null>(null);
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
    retain: () => Promise<void>;
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
  const [liveText, setLiveText] = useState<string | null>(null);
  // The buffer's last text, readable after the view is gone. The draft
  // keeper's final flush runs during unmount, AFTER the effect that destroys
  // the view — React runs cleanups in declaration order — and reading an
  // empty string from a dead view wrote a draft of nothing, which the next
  // mount restored "silently" and then saved: a file emptied on disk by
  // switching tabs. Found by dogfooding on this repository, on a file with
  // 159 lines. Never read the buffer through the view alone.
  const lastTextRef = useRef<string>("");
  const lsp = useWorkspaceLsp(liveText === null ? "" : path, liveText ?? "");
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
  const debug = usePlainDebugSession(path) ?? IDLE_SESSION;
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
        put: (content, baseHash, force) => api.saveFile(path, content, baseHash, force),
        onState: (state) => {
          setSaveState(state);
          if (state.kind === "idle" || state.kind === "saved") {
            onUnsavedRef.current?.(false, actionsRef.current);
          }
        },
      }),
    [path],
  );
  actionsRef.current = {
    save: () => saver.flushNow(),
    discard: () => {
      const base = saver.discardPending();
      const view = viewRef.current;
      if (view && view.state.doc.toString() !== base) syncAndFlash(view, base);
    },
    retain: async () => {
      const contents = viewRef.current?.state.doc.toString() ?? lastTextRef.current;
      await api.saveDraft({ path, contents, base: saver.baseContent(), saved_at: Date.now() });
    },
  };
  useEffect(() => () => saver.dispose(), [saver]);
  // File > Save All: this pane owns its saver, so it answers for its own
  // buffer. See lib/flushSaves.ts.
  useEffect(() => onFlushSaves(() => saver.flushNow()), [saver]);

  // The initial read. The pane renders its refusals — binary, too large,
  // missing — as text where the editor would be: the tab is still an honest
  // place, it just has nothing to edit.
  useEffect(() => {
    let live = true;
    api.file(path).then(
      (loaded) => {
        if (!live) return;
        setFile(loaded);
        setLoadError(null);
        setLiveText(loaded.content);
        lastTextRef.current = loaded.content;
        saver.load(loaded.content, loaded.hash);
        void api.files().then(
          (files) => {
            if (!live) return;
            const find = (nodes: typeof files.tree): string | null => {
              for (const node of nodes) {
                if (node.path === path) return node.generated_by ?? null;
                const found = node.children ? find(node.children) : null;
                if (found) return found;
              }
              return null;
            };
            setGeneratedBy(find(files.tree));
          },
          () => {
            // The listing is unavailable: the button stays, and the adopt
            // route's own refusal is the backstop.
          },
        );
        if (!retainUnsaved) return;
        // Was this buffer holding unsaved work when the app last closed?
        //
        // Three answers, and only one of them interrupts anybody. The file is
        // as we left it: put the text back, still unsaved, silently — that is
        // the common case by a wide margin, and a dialog here would train
        // people to dismiss dialogs. The file already says the same thing:
        // the draft is stale, drop it. The file moved on and so did we: that
        // is a merge, and it is worth someone's attention.
        void api.drafts().then(
          ({ drafts }) => {
            if (!live) return;
            const draft = drafts.find((d) => d.path === path);
            if (!draft) return;
            const next = draftDisposition(draft, loaded.content);
            if (next.kind === "clean") {
              void api.discardDraft(path).catch(() => {});
              return;
            }
            if (next.kind === "restore") {
              setRestored(next.contents);
              return;
            }
            setMerge({
              base: next.base,
              ours: next.ours,
              theirs: next.theirs,
              oursLabel: "Your unsaved changes",
              theirsLabel: "The file on disk",
            });
          },
          () => {
            // No draft store on this machine: the file opens as it is on
            // disk, which is what would have happened anyway.
          },
        );
      },
      (e) => live && setLoadError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [path, saver, retainUnsaved]);

  // The view is created ONCE, when the first load lands. Later content
  // arrives through syncAndFlash below — rebuilding the editor would throw
  // away the cursor and undo history mid-edit.
  const loaded = file !== null;
  const fileRef = useRef(file);
  fileRef.current = file;
  useEffect(() => {
    const host = hostRef.current;
    const initial = fileRef.current;
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
    const lspUri = lspRef.current.uri;

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: initial.content,
        extensions: [
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
                onEvaluate: (expression) => debugRef.current.query(expression),
                onAddWatch: (expression) => debugRef.current.addWatch(expression),
              })
            : []),
          ...languageExtensions(initial.language),
          // A run mark beside every test the file's language has a shape
          // for; a click runs that test in a terminal named after it.
          ...testGutter({
            language: initial.language,
            onRun: (mark) =>
              void api
                .runTest({ path: initial.path, name: mark.name, language: initial.language })
                .then(
                  (session) => showTerminalRequest(session.id, session.title),
                  (e) => setNotice(e instanceof Error ? e.message : String(e)),
                ),
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
                ? askTextRef.current(`Rename "${current}" to:`, current)
                : null,
            onCodeActions: (actions: CodeAction[]) =>
              askChoiceRef.current
                ? askChoiceRef.current(
                    "Code actions",
                    actions.map((action) => ({ label: action.title, value: action })),
                  )
                : null,
            onMessage: setNotice,
          }),
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
            if (u.docChanged) {
              const text = u.state.doc.toString();
              lastTextRef.current = text;
              setLiveText(text);
            }
            // Only edits a person made: a programmatic reload is this pane
            // catching up with the disk, and saving it back would write
            // bytes nobody typed.
            if (
              u.docChanged &&
              u.transactions.some(
                (t) =>
                  t.isUserEvent("input") ||
                  t.isUserEvent("delete") ||
                  t.isUserEvent("move") ||
                  t.isUserEvent("undo") ||
                  t.isUserEvent("redo"),
              )
            ) {
              onUnsavedRef.current?.(true, actionsRef.current);
              saver.changed(u.state.doc.toString());
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
      if (view.state.doc.toString() !== text) syncAndFlash(view, text);
      saver.changed(text);
    },
    [saver],
  );

  // The restored draft goes in once the view exists — the load that found it
  // lands before the editor is built.
  useEffect(() => {
    if (restored === null || !loaded) return;
    putInBuffer(restored);
    setRestored(null);
  }, [restored, loaded, putInBuffer]);

  // Write the buffer down while it differs from the file. Read through a
  // callback so this costs nothing on the typing path — see lib/drafts.ts.
  useDraftKeeper({
    path,
    enabled: loaded && retainUnsaved,
    read: () => ({
      // The view when it exists; what it last held when it does not. An
      // absent view is not an empty buffer.
      contents: viewRef.current ? viewRef.current.state.doc.toString() : lastTextRef.current,
      base: saver.baseContent(),
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
      saver.load(fresh.content, fresh.hash);
      const view = viewRef.current;
      if (view && view.state.doc.toString() !== fresh.content) {
        syncAndFlash(view, fresh.content);
      }
    },
    [saver],
  );

  // External changes: refetch on window focus and on the files-changed
  // announcement — unless this pane's own edits are still unsent, in which
  // case the disk copy predates them and adopting it would eat keystrokes.
  // (Their save will 409 against the moved disk, which is the conflict
  // banner's moment, not this one.)
  useEffect(() => {
    const refresh = () => {
      if (!viewRef.current || saver.hasPendingEdits()) return;
      api.file(path).then(
        (fresh) => {
          if (fresh.hash !== fileRef.current?.hash) adoptDiskCopy(fresh);
        },
        () => {}, // deleted or unreadable now; the buffer stays as evidence
      );
    };
    window.addEventListener("focus", refresh);
    window.addEventListener(FILES_CHANGED_EVENT, refresh);
    return () => {
      window.removeEventListener("focus", refresh);
      window.removeEventListener(FILES_CHANGED_EVENT, refresh);
    };
  }, [path, saver, adoptDiskCopy]);

  // Adopt this file into a literate document. The server proves the new
  // document weaves these exact bytes before writing anything, so a success
  // changes ownership and not one byte of content; a refusal (binary,
  // hick:-looking text, a taken document name) lands in the banner.
  const adopt = useCallback(() => {
    setAdopting(true);
    setAdoptError(null);
    api.adopt(path).then(
      (adopted) => {
        setAdopting(false);
        onAdoptedRef.current?.(adopted);
      },
      (e) => {
        setAdopting(false);
        setAdoptError(e instanceof Error ? e.message : String(e));
      },
    );
  }, [path]);

  const overwrite = useCallback(() => saver.resolve("overwrite"), [saver]);

  /** The third answer to a conflict, and the one that does not throw work
   * away: three-way merge the buffer against the disk, over the bytes this
   * session started from. */
  const openMerge = useCallback(() => {
    const view = viewRef.current;
    if (!view) return;
    const ours = view.state.doc.toString();
    const base = saver.baseContent();
    void api.file(path).then(
      (fresh) =>
        setMerge({
          base,
          ours,
          theirs: fresh.content,
          oursLabel: "Your unsaved changes",
          theirsLabel: "The file on disk",
        }),
      (e) => setSaveState({ kind: "error", message: e instanceof Error ? e.message : String(e) }),
    );
  }, [saver, path]);

  /** The merged text becomes the buffer, and the file is reloaded first so
   * the save that follows rides on the hash the disk actually has. */
  const acceptMerge = useCallback(
    (text: string) => {
      setMerge(null);
      saver.resolve("reload");
      void api.file(path).then(
        (fresh) => {
          adoptDiskCopy(fresh);
          putInBuffer(text);
        },
        (e) => setSaveState({ kind: "error", message: e instanceof Error ? e.message : String(e) }),
      );
    },
    [saver, path, adoptDiskCopy, putInBuffer],
  );
  const reload = useCallback(() => {
    saver.resolve("reload");
    api.file(path).then(adoptDiskCopy, (e) =>
      setSaveState({ kind: "error", message: e instanceof Error ? e.message : String(e) }),
    );
  }, [saver, path, adoptDiskCopy]);

  if (loadError) {
    return (
      <div className="plain-file-pane">
        <p className="error">{loadError}</p>
      </div>
    );
  }

  return (
    <div className="plain-file-pane">
      <div className="doc-tab-toolbar" role="toolbar" aria-label={`Actions for ${path}`}>
        {generatedBy ? (
          // Not a disabled button: there is nothing to enable. This file is
          // already the output of a literate document, and saying which one
          // is more useful than a greyed-out verb.
          <span className="muted plain-file__generated" role="status">
            Written by a literate document — it is already literate.
          </span>
        ) : (
        <button
          className="btn"
          // Not while a save is in flight or parked on a conflict: the
          // server adopts the DISK bytes, and those must be this buffer's.
          disabled={adopting || !file || saveState.kind === "saving" || saveState.kind === "conflict"}
          onClick={adopt}
          data-tip="Wrap this file in a new literate document, byte-exactly — the file itself does not change, its ownership does"
        >
          {adopting ? "Adopting…" : "Make literate"}
        </button>
        )}
        {/* The way into a session, for a file hick can debug. The strip's
            own "Debug again" takes over once a session has run, so this is
            only shown while there is none. */}
        {file && isDebuggable(file.language) && debug.status === "idle" && (
          <button
            className="btn"
            onClick={() => debug.start()}
            data-tip="Run this file under the debugger, in its own project, stopping at your breakpoints"
          >
            Debug
          </button>
        )}
        {notice && (
          <span className="muted plain-file__notice" role="status">
            {notice}
          </span>
        )}
        {(saveState.kind === "saving" || saveState.kind === "saved") && (
          <span
            className={`save-state save-state-${saveState.kind === "saving" ? "editing" : "saved"}`}
            role="status"
          >
            {saveState.kind === "saving" ? "Saving…" : "Saved"}
          </span>
        )}
      </div>
      {adoptError && (
        <div className="banner banner-fail" role="alert">
          Could not adopt {path}: {adoptError}
        </div>
      )}
      {saveState.kind === "conflict" && (
        // The same surface a diverged generated file gets, with the same
        // three ways out. Here "mine" is the buffer and "theirs" is the
        // disk, because it was another program that wrote the file.
        <DivergedBanner
          what={path}
          reason="Another program wrote the file while you were editing; your text is still in this buffer, unsaved."
          mine="your unsaved text"
          theirs="the file on disk"
          onKeepMine={overwrite}
          onTakeTheirs={reload}
          onMerge={openMerge}
        />
      )}
      {saveState.kind === "error" && (
        <div className="banner banner-fail" role="alert">
          Could not save {path}: {saveState.message}
        </div>
      )}
      {/* The debugger's chrome, above the file it debugs — nothing while
          idle. The same strip a document's block gets. */}
      <DebugStrip
        status={debug.status}
        program={null}
        message={debug.message}
        offerInstall={debug.offerInstall}
        onInstall={async (offer) => {
          await api.installTool(offer.kind, offer.language);
          // Straight back into the session that failed: installing and then
          // asking somebody to press Debug again is the same missing step
          // this button exists to remove.
          debug.start();
        }}
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
        onStart={() => debug.start()}
        onStop={debug.stop}
        onAddWatch={() => {
          void askTextRef.current?.("Expression to watch:", "").then((expression) => {
            if (expression) debug.addWatch(expression);
          });
        }}
        exceptionFilters={debug.exceptionFilters}
        onToggleExceptionFilter={debug.toggleExceptionFilter}
        onRemoveWatch={debug.removeWatch}
      />
      {!file && <p className="muted">Loading {path}…</p>}
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
          onAccept={acceptMerge}
          onCancel={() => setMerge(null)}
        />
      ) : null}
      <div className="output-editor" hidden={merge !== null}>
        <div ref={hostRef} className="editor-cm-host" />
      </div>
    </div>
  );
}
