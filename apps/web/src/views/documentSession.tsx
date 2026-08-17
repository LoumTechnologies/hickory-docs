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

import { api, MOCK } from "../api/client";
import { WsRealtime, getSharedRealtime, type Realtime } from "../api/realtime";
import type { Block, Doc, ExecBlock, OutputFile, RunWsMessage, SourceEdit } from "../api/types";
import { flashSpans } from "../editor/changeFlash";
import { usePrompt } from "../components/PromptPanel";
import { byteToChar } from "../lib/offsets";
import { useLsp } from "../lsp/useLsp";
import {
  lspSupport,
  offsetToPosition,
  positionToOffset,
  type LspNavigationTarget,
} from "../lsp/cmLsp";
import { lspFeatures } from "../lsp/cmLspFeatures";
import { useDebugger } from "../debug/useDebugger";
import {
  debugEditor,
  revealLine,
  stackMarksOf,
  setBreakpointMarks,
  setInlineValues,
  setPausedLine,
  setStackMarks,
  setWatchValues,
} from "../debug/cmDebug";
import { positionToUtf16 } from "../lsp/positions";
import { sourcePositionAt, type OutputProvenance } from "../lsp/outputMapping";
import type { LspLocation } from "../lsp/client";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { navigate } from "../router";

export type Banner = { kind: "pending" | "pass" | "fail"; text: string } | null;

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
  syncState: "idle" | "editing" | "saved";
  runningCells: Set<string>;
  execBlocks: ExecBlock[];
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
  menuSave: () => void;
  menuSaveAs: () => void;
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
}: {
  registry: SessionRegistry;
  docId: string;
  /** Open one of THIS document's generated files in the layout. */
  openGenerated: (docId: string, path: string) => void;
}) {
  const session = useDocumentSession(docId, openGenerated);
  useEffect(() => {
    registry.publish(session);
  });
  useEffect(() => () => registry.drop(docId), [registry, docId]);
  return null;
});

// ---------------------------------------------------------------------------
// The session itself (moved, near-verbatim, from views/DocumentView.tsx)
// ---------------------------------------------------------------------------

function useDocumentSession(
  docId: string,
  openGeneratedFor: (docId: string, path: string) => void,
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
  // "saved" once the CRDT room's debounced persist has certainly landed and
  // the server render caught up; "editing" while the user is still typing.
  const [syncState, setSyncState] = useState<"idle" | "editing" | "saved">("idle");
  const [references, setReferences] = useState<{ locations: LspLocation[]; query: string } | null>(
    null,
  );

  // Rename and code-action need an answer from the user before the edit can
  // be applied; this is that answer, in-app rather than in a modal.
  const { prompt, askText, askChoice, settle } = usePrompt();

  const checkRunRef = useRef<string | null>(null);
  // Fast local runs can finish (and emit their terminal WS message) before
  // the POST /check response delivers the run_id — remember terminals so
  // verify() can resolve the banner after the fact.
  const terminalStatusesRef = useRef<Map<string, string>>(new Map());
  const startedCellsRef = useRef<Set<string>>(new Set());

  const realtime: Realtime = useMemo(() => {
    // In mock mode a shared local bus is registered at startup so fake run
    // events reach this session without a network socket.
    const shared = getSharedRealtime();
    if (MOCK && shared) return shared;
    return new WsRealtime(`doc:${docId}`);
  }, [docId]);

  useEffect(() => {
    return () => {
      if (realtime !== getSharedRealtime()) realtime.close();
    };
  }, [realtime]);

  // The debugger. It is a READER: nothing it does can change the document,
  // its generated files, or a recorded transcript — the channel carries no
  // edit operation and the debuggee runs in a scratch copy.
  const debug = useDebugger(realtime, doc?.path ?? "");
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
  // Bumped by the up-loop's files_changed event: outputs changed on disk
  // without a run or a render in this window.
  const [outputsRefresh, setOutputsRefresh] = useState(0);

  // Push the debugger's state into the editor: the gutter dots, the paused
  // line, and the values shown at the end of each line.
  useEffect(() => {
    const view = editorRef.current;
    if (!view) return;
    view.dispatch({
      effects: [
        setBreakpointMarks.of(
          debug.breakpoints.map((breakpoint) => ({
            line: breakpoint.line,
            verified: breakpoint.verified,
            conditional: false,
            message: breakpoint.message,
          })),
        ),
        setPausedLine.of(debug.pausedLine),
        setInlineValues.of(debug.variables),
        // The rest of the stack, in the gutter: every frame below the one
        // execution is stopped at, on the line it will return to.
        setStackMarks.of(stackMarksOf(debug.frames, debug.pausedLine)),
        // Watches, drawn faded at the end of the line that mentions each one.
        setWatchValues.of(debug.watches),
      ],
    });
  }, [debug.breakpoints, debug.pausedLine, debug.variables, debug.frames, debug.watches, docEditor]);

  // Run events arrive in bursts (one terminal message per run, plus the
  // `verify` fallback path), and each used to trigger its own full render.
  // Coalesce them into one trailing fetch; `api.render` additionally
  // collapses anything still in flight.
  const renderTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const scheduleRender = useCallback(() => {
    if (renderTimer.current !== null) return;
    renderTimer.current = setTimeout(() => {
      renderTimer.current = null;
      api.render(docId).then((r) => setBlocks(r.blocks), () => undefined);
    }, 150);
  }, [docId]);
  useEffect(
    () => () => {
      if (renderTimer.current !== null) clearTimeout(renderTimer.current);
    },
    [],
  );

  const refresh = useCallback(() => {
    api.doc(docId).then(setDoc, (e) => setFatalError(String(e.message ?? e)));
    api.render(docId).then(
      (r) => {
        setBlocks(r.blocks);
        setRenderError(null);
      },
      // A document that will not weave — a syntax error, mid-edit — must
      // still OPEN. Replacing the page with the error made the one thing
      // that could fix it, the editor, unreachable.
      (e) => {
        setBlocks((prev) => prev ?? []);
        setRenderError(String(e.message ?? e));
      },
    );
  }, [docId]);

  useEffect(refresh, [refresh]);

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
        terminalStatusesRef.current.set(msg.run_id, msg.status);
        if (msg.run_id === checkRunRef.current) {
          checkRunRef.current = null;
          setBanner(
            msg.status === "ok"
              ? { kind: "pass", text: "Verification passed — no drift, all expectations met." }
              : { kind: "fail", text: "Verification failed — expectation mismatch or output drift." },
          );
        }
        setRunningCells(new Set());
        startedCellsRef.current = new Set();
        // The RUN's blocks, not a fresh render.
        //
        // `/render` re-weaves the document without executing, and a document
        // with no recorded transcript reports every cell as "never run" —
        // which is what it says a second after a successful run, because a
        // run does not record unless it was asked to. The run's own block
        // model has the transcripts in it, so that is what the cells show.
        api.runStatus(msg.run_id).then(
          (run) => {
            if (run.blocks && run.blocks.length > 0) setBlocks(run.blocks);
            else scheduleRender();
          },
          () => scheduleRender(),
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
          return { ...b, transcript, status };
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
        setRunningCells((prev) => new Set(prev).add(execId));
        try {
          await api.run(docId, [execId]);
        } catch (e) {
          setRunningCells((prev) => {
            const next = new Set(prev);
            next.delete(execId);
            return next;
          });
          setFatalError(e instanceof Error ? e.message : String(e));
        }
      })();
    },
    [docId],
  );

  const blocksRef = useRef(blocks);
  blocksRef.current = blocks;

  const runAll = useCallback(() => {
    void (async () => {
      setBanner(null);
      startedCellsRef.current = new Set();
      const ids = (blocksRef.current ?? []).filter((b) => b.kind === "exec").map((b) => b.id);
      // A document with no cells has nothing to run, and a button that
      // appears to do nothing is worse than one that says so: the run
      // happens either way (it re-weaves), but the only visible sign of it
      // is in cells that do not exist here.
      if (ids.length === 0) {
        setBanner({ kind: "pending", text: "Nothing to run — this document has no exec cells. Re-weaving it." });
      }
      setRunningCells(new Set(ids));
      try {
        await api.run(docId);
        if (ids.length === 0) {
          setBanner({ kind: "pass", text: "Re-woven. This document has no exec cells to run." });
          scheduleRender();
        }
      } catch (e) {
        setRunningCells(new Set());
        setBanner(null);
        setFatalError(e instanceof Error ? e.message : String(e));
      }
    })();
  }, [docId, scheduleRender]);

  const verify = useCallback(() => {
    void (async () => {
      setBanner({ kind: "pending", text: "Verifying — re-running pipeline against expectations…" });
      startedCellsRef.current = new Set();
      try {
        const { run_id } = await api.check(docId);
        const already = terminalStatusesRef.current.get(run_id);
        if (already !== undefined) {
          setBanner(
            already === "ok"
              ? { kind: "pass", text: "Verification passed — no drift, all expectations met." }
              : { kind: "fail", text: "Verification failed — expectation mismatch or output drift." },
          );
          scheduleRender();
        } else {
          checkRunRef.current = run_id;
        }
      } catch (e) {
        setBanner({ kind: "fail", text: e instanceof Error ? e.message : String(e) });
      }
    })();
  }, [docId, scheduleRender]);

  // The document has no Save button: edits go into the CRDT, which the
  // server persists (Postgres + git) 750 ms after typing stops. All this has
  // to do is pick the server's copy back up so the cell panels, provenance
  // spans and ribbons stop describing the previous version. Waiting longer
  // than the server's own debounce is deliberate — re-fetching sooner would
  // read the pre-persist source and render one keystroke behind forever.
  useEffect(() => {
    // The CRDT's first sync populates the buffer, which fires docChanged
    // just like typing does. A buffer identical to the server's copy is not
    // an edit, and reporting "Saved" for it is a lie the user cannot check.
    if (dirtySource === null || dirtySource === doc?.source) return;
    setSyncState("editing");
    const timer = setTimeout(() => {
      Promise.all([api.doc(docId), api.render(docId)]).then(
        ([d, r]) => {
          setDoc(d);
          setBlocks(r.blocks);
          setSyncState("saved");
        },
        () => undefined,
      );
    }, 1200);
    return () => clearTimeout(timer);
  }, [dirtySource, doc?.source, docId]);

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
  const menuSave = useCallback(() => {
    const { doc, dirtySource } = menuStateRef.current;
    if (!doc) return;
    const source = dirtySource ?? doc.source;
    api.saveDoc(doc.id, source).then(
      (saved) => {
        setDoc(saved);
        setSyncState("saved");
      },
      (e) => setBanner({ kind: "fail", text: e instanceof Error ? e.message : String(e) }),
    );
  }, []);
  const menuSaveAs = useCallback(() => {
    const { doc, dirtySource } = menuStateRef.current;
    if (!doc) return;
    const source = dirtySource ?? doc.source;
    void (async () => {
      const path = await askText("Save a copy as (relative path):", doc.path);
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
  const lsp = useLsp(realtime, doc?.path ?? "", liveSource);

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
      const from = positionToUtf16(liveSource, target.range.start);
      const to = positionToUtf16(liveSource, target.range.end);
      setSelectSpan([from, Math.max(to, from)]);
    },
    [liveSource, openGenerated],
  );

  const wordAt = (offset: number) => {
    const m = /[A-Za-z_][A-Za-z0-9_]*/y;
    let start = offset;
    while (start > 0 && /[A-Za-z0-9_]/.test(liveSource[start - 1] ?? "")) start--;
    m.lastIndex = start;
    return m.exec(liveSource)?.[0] ?? "";
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
            query: wordAt(positionToUtf16(liveSource, from.range.start)),
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
        // A gutter stack mark is a caller; clicking it shows that frame.
        onSelectFrame: selectFrameAndReveal,
        // The inline eval at the paused line, answered in the selected frame.
        onEvaluate: (expression) => debugRef.current.query(expression),
        onAddWatch: (expression) => debugRef.current.addWatch(expression),
      }),
      ...lspFeatures({
        client: lsp.client,
        uri: lsp.uri,
        positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
        offsetAt: (position, view) => positionToOffset(view.state.doc, position),
        inlayHints: true,
        onRename: (current) => askText(`Rename "${current}" to:`, current),
        onCodeActions: (actions) =>
          askChoice(
            "Code actions",
            actions.map((action) => ({ label: action.title, value: action })),
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
    (provenance: OutputProvenance[]) =>
      lspSupport({
        client: lsp.client,
        uri: lsp.uri,
        positionAt: (offset) =>
          sourcePositionAt(offset, provenance, doc?.path ?? "", liveSource),
        onNavigate: openTarget,
        onReferences: (locations) => setReferences({ locations, query: "" }),
      }),
    [lsp.client, lsp.uri, openTarget, doc?.path, liveSource],
  );

  const registerOutputView = useCallback((path: string, view: EditorView | null) => {
    setOpenOutputs((current) => {
      const next = new Map(current);
      if (view) next.set(path, view);
      else next.delete(path);
      return next;
    });
  }, []);

  const onDocViewReady = useCallback((view: EditorView | null) => {
    editorRef.current = view;
    setDocEditor(view);
  }, []);

  const clearReferences = useCallback(() => setReferences(null), []);

  const execBlocks = useMemo(
    () => (blocks ?? []).filter((b): b is ExecBlock => b.kind === "exec"),
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
    runningCells,
    execBlocks,
    outputs,
    openOutputs,
    docEditor,
    realtime,
    debug,
    selectSpan,
    references,
    prompt,
    settle,
    askText,
    lspDiagnostics: lsp.diagnostics,
    lspExtensions,
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
  };
}
