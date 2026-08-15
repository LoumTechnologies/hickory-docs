import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, MOCK } from "../api/client";
import { WsRealtime, getSharedRealtime, type Realtime } from "../api/realtime";
import type { Block, Doc, ExecBlock } from "../api/types";
import { ChatDock } from "../components/ChatDock";
import { ReferencesPanel } from "../components/ReferencesPanel";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
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
import { DebugPanel } from "../debug/DebugPanel";
import {
  debugEditor,
  revealLine,
  stackMarksOf,
  setBreakpointMarks,
  setInlineValues,
  setPausedLine,
  setStackMarks,
} from "../debug/cmDebug";
import { positionToUtf16 } from "../lsp/positions";
import { sourcePositionAt, type OutputProvenance } from "../lsp/outputMapping";
import type { LspLocation } from "../lsp/client";
import { DocumentEditor } from "../editor/DocumentEditor";
import { SplitView } from "./SplitView";
import { ShellView } from "../shell/ShellView";
import { GeneratedFileView, OutputsTool } from "../shell/views";
import {
  freeform,
  open as openInLayout,
  paneFor,
  tab as makeTab,
  type Layout,
  type Region,
} from "../shell/layout";
import { layoutsFor, type LayoutChoice } from "../shell/layouts";
import { navigate } from "../router";

type Banner = { kind: "pending" | "pass" | "fail"; text: string } | null;

/** The Split (lineage) view needs real width for two panes + ribbons. */
// Three columns plus ribbons need real width — but 1200 was too greedy: a
// zoomed-in browser or a 13" laptop dropped straight to a single column with
// no way to ask for Split at all.
const SPLIT_MIN_WIDTH = "(min-width: 1024px)";

export function DocumentView({ docId }: { docId: string }) {
  const [doc, setDoc] = useState<Doc | null>(null);
  const [blocks, setBlocks] = useState<Block[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  // A failed weave is reported inline; only a failure to LOAD the document
  // (missing, forbidden) is fatal to the page.
  const [renderError, setRenderError] = useState<string | null>(null);
  // What the window is arranged as. Not a mode: either an arrangement this
  // person built (freeform: panes, tabs, splits) or one a document declared.
  // "Document & outputs" is the old Split view kept whole, because it draws
  // the provenance ribbons and nothing else does.
  //
  // See docs/specs/freeform/shell-layouts.md.
  //
  // Freeform by default, on every screen size. Opening a file should cost
  // what it costs in Notepad: the file, in an editor, immediately. An
  // arrangement someone else chose is a thing to explain before you have
  // typed anything, and the ribbons view — good as it is — is exactly that
  // when all you wanted was to read one document.
  const [choiceId, setChoiceId] = useState<string>("freeform");
  const [layout, setLayout] = useState<Layout>(freeform);
  const [declared, setDeclared] = useState<LayoutChoice[]>([]);
  // Split is only offered when the viewport is wide enough for two panes.
  const [wide, setWide] = useState<boolean>(() =>
    typeof window.matchMedia === "function" ? window.matchMedia(SPLIT_MIN_WIDTH).matches : false,
  );
  // null while unknown (don't disable the toggle on a flash of missing
  // data); false only once a check has actually come back empty.
  const [hasOutputs, setHasOutputs] = useState<boolean | null>(null);
  // The dock is part of the workspace, not a mode: it is always mounted and
  // remembers whether the log is expanded.
  const [chatCollapsed, setChatCollapsed] = useState(
    () => localStorage.getItem("hickory.chatCollapsed") === "1",
  );
  const [runningCells, setRunningCells] = useState<Set<string>>(new Set());
  const [banner, setBanner] = useState<Banner>(null);
  const [selectSpan, setSelectSpan] = useState<[number, number] | null>(null);
  const [dirtySource, setDirtySource] = useState<string | null>(null);
  // "saved" once the CRDT room's debounced persist has certainly landed and the
  // server render caught up; "editing" while the user is still typing.
  const [syncState, setSyncState] = useState<"idle" | "editing" | "saved">("idle");
  // Find-references results, and a pending "open this output file here" jump
  // produced by LSP navigation into a generated file.
  const [references, setReferences] = useState<{ locations: LspLocation[]; query: string } | null>(
    null,
  );
  const [outputTarget, setOutputTarget] = useState<{ path: string; span: [number, number] } | null>(
    null,
  );

  // Rename and code-action need an answer from the user before the edit
  // can be applied; this is that answer, in-app rather than in a modal.
  const { prompt, askText, askChoice, settle } = usePrompt();

  const checkRunRef = useRef<string | null>(null);
  // Fast local runs can finish (and emit their terminal WS message) before
  // the POST /check response delivers the run_id — remember terminals so
  // verify() can resolve the banner after the fact.
  const terminalStatusesRef = useRef<Map<string, string>>(new Map());
  const startedCellsRef = useRef<Set<string>>(new Set());

  const realtime: Realtime = useMemo(() => {
    // In mock mode a shared local bus is registered at startup so fake run
    // events reach this view without a network socket.
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
  const editorRef = useRef<import("@codemirror/view").EditorView | null>(null);

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
      ],
    });
  }, [debug.breakpoints, debug.pausedLine, debug.variables, debug.frames]);

  // Run events arrive in bursts (one terminal message per run, plus the
  // `verify` fallback path), and each used to trigger its own full render.
  // Coalesce them into one trailing fetch; `api.render` additionally collapses
  // anything still in flight.
  const renderTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const refreshOutputs = useCallback(() => {
    api.outputs(docId).then(
      (r) => setHasOutputs(r.files.length > 0),
      () => undefined,
    );
  }, [docId]);
  const scheduleRender = useCallback(() => {
    if (renderTimer.current !== null) return;
    renderTimer.current = setTimeout(() => {
      renderTimer.current = null;
      api.render(docId).then((r) => setBlocks(r.blocks), () => undefined);
      // A run event lands right about when new output files could exist.
      refreshOutputs();
    }, 150);
  }, [docId, refreshOutputs]);
  useEffect(
    () => () => {
      if (renderTimer.current !== null) clearTimeout(renderTimer.current);
    },
    [],
  );

  const refresh = useCallback(() => {
    api.doc(docId).then(setDoc, (e) => setError(String(e.message ?? e)));
    api.render(docId).then(
      (r) => {
        setBlocks(r.blocks);
        setRenderError(null);
      },
      // A document that will not weave — a syntax error, mid-edit — must
      // still OPEN. Replacing the page with the error made the one thing that
      // could fix it, the editor, unreachable.
      (e) => {
        setBlocks((prev) => prev ?? []);
        setRenderError(String(e.message ?? e));
      },
    );
  }, [docId]);

  useEffect(refresh, [refresh]);
  useEffect(refreshOutputs, [refreshOutputs]);

  // Track viewport width; Split degrades to Document when the window shrinks.
  useEffect(() => {
    if (typeof window.matchMedia !== "function") return;
    const mq = window.matchMedia(SPLIT_MIN_WIDTH);
    const onChange = () => setWide(mq.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);
  // The ribbons layout needs two panes' worth of width and something to put
  // in the second one. Losing either sends you to freeform, which needs
  // neither.
  useEffect(() => {
    if (!wide || hasOutputs === false) {
      setChoiceId((current) => (current === "ribbons" ? "freeform" : current));
    }
  }, [wide, hasOutputs]);


  // What this project declares. A folder can hold layouts; a document view is
  // still in a folder, so the same choices are offered here.
  useEffect(() => {
    let live = true;
    api.projects().then(
      async (projects) => {
        const project = projects[0];
        if (!project) return;
        const docs = await api.projectDocs(project.id).catch(() => []);
        const sources = await Promise.all(
          docs.map(async (entry) => ({
            path: entry.path,
            source: await api.doc(entry.id).then((d) => d.source).catch(() => ""),
          })),
        );
        if (!live) return;
        // `layoutsFor` puts Freeform first; the built-ins are added beside it
        // in `choices`, so only the declared ones are kept here.
        setDeclared(layoutsFor(sources).filter((entry) => entry.source));
      },
      () => {},
    );
    return () => {
      live = false;
    };
  }, [docId]);

  // Live run events: append to the matching cell's transcript.
  useEffect(() => {
    return realtime.onRunEvent((msg) => {
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
        // Pick up final statuses/transcripts from the server's render.
        scheduleRender();
        return;
      }
      const { exec_id, event } = msg;
      if (exec_id === "agent") return; // agent panel handles its own stream
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

  const runCell = async (execId: string) => {
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
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const runAll = async () => {
    setBanner(null);
    startedCellsRef.current = new Set();
    const ids = (blocks ?? []).filter((b) => b.kind === "exec").map((b) => b.id);
    setRunningCells(new Set(ids));
    try {
      await api.run(docId);
    } catch (e) {
      setRunningCells(new Set());
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const verify = async () => {
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
  };

  // The document has no Save button: edits go into the CRDT, which the server
  // persists (Postgres + git) 750 ms after typing stops. All this has to do is
  // pick the server's copy back up so the cell panels, provenance spans and
  // ribbons stop describing the previous version. Waiting longer than the
  // server's own debounce is deliberate — re-fetching sooner would read the
  // pre-persist source and render one keystroke behind forever.
  useEffect(() => {
    // The CRDT's first sync populates the buffer, which fires docChanged just
    // like typing does. A buffer identical to the server's copy is not an
    // edit, and reporting "Saved" for it is a lie the user has no way to check.
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

  // ---- the shell ----------------------------------------------------------
  //
  // Freeform and the ribbons view are built in; everything else is a document
  // in this project that declared regions. The picker lists them together
  // because to a person they are the same kind of choice.
  const choices: LayoutChoice[] = useMemo(() => {
    const builtIn: LayoutChoice[] = [
      {
        id: "freeform",
        name: "Freeform",
        detail: "One pane. Split it, fill it with tabs, arrange it yourself.",
        build: freeform,
      },
      {
        id: "ribbons",
        name: "Document & outputs",
        detail: "The document, its files, and ribbons drawing what came from where.",
        build: freeform,
      },
    ];
    return [...builtIn, ...declared];
  }, [declared]);

  const choice = choices.find((candidate) => candidate.id === choiceId) ?? choices[0];
  const regions: Region[] = useMemo(() => choice.regions ?? [], [choice]);

  // Picking a layout builds it and puts the document in it. A layout with
  // regions puts it where its globs say; freeform puts it in the only pane
  // there is.
  useEffect(() => {
    // A different document, before its own has loaded: the tabs on screen
    // belong to the one being left, and rendering them against the new id
    // asks the server for files that document does not generate.
    setLayout(freeform());
  }, [docId]);

  useEffect(() => {
    if (!doc || doc.id !== docId) return;
    const built = choice.build();
    const target = paneFor(built, choice.regions ?? [], doc.path);
    setLayout(openInLayout(built, makeTab("document", doc.path, doc.path.split("/").pop()), target));
    // Rebuilt only when the CHOICE or the document changes, never on every
    // render: a layout is session state, and rebuilding it would throw away
    // the arrangement the person just made.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [choiceId, docId, doc?.path, choices.length]);

  const openGenerated = useCallback(
    (path: string) => {
      setLayout((current) => {
        const target = paneFor(current, regions, path);
        return openInLayout(current, makeTab("generated", path, path.split("/").pop()), target);
      });
    },
    [regions],
  );

  // A span arriving from provenance or an LSP hit indexes the document's
  // BYTES; the editor selects by character.
  const onSelectSpan = useCallback(
    (span: [number, number]) => {
      setSelectSpan(doc ? [byteToChar(doc.source, span[0]), byteToChar(doc.source, span[1])] : span);
    },
    [doc],
  );

  // ---- editor intelligence ------------------------------------------------
  //
  // One LSP session per document, shared by both editors. It is fed the LIVE
  // text (unsaved edits included) so positions always match what is on screen.
  const liveSource = dirtySource ?? doc?.source ?? "";
  const lsp = useLsp(realtime, doc?.path ?? "", liveSource);

  const openTarget = useCallback(
    (target: LspNavigationTarget | LspLocation) => {
      setReferences(null);
      if (target.uri.startsWith("hick-output:///")) {
        // The bridge could not map this position back to prose — open the
        // generated file itself, positioned on the hit.
        const path = target.uri.slice("hick-output:///".length);
        setOutputTarget({
          path,
          // Output ranges are line/character; the pane resolves them against
          // its own buffer, so carry them as a line-anchored span.
          span: [target.range.start.line, target.range.end.line],
        });
        openGenerated(path);
        return;
      }
      const from = positionToUtf16(liveSource, target.range.start);
      const to = positionToUtf16(liveSource, target.range.end);
      setSelectSpan([from, Math.max(to, from)]);
    },
    [liveSource],
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
      // The rest of the server: colouring, hints, highlight, folding,
      // signature help, rename and code actions. Only the document editor
      // gets them — the output panes' coordinates travel back through
      // provenance, and a token painted through that mapping would land on
      // text the author did not write.
      // The gutter, the paused line and the inline values. Its own layer
      // rather than part of the LSP one: they answer different questions and
      // a document with no debugger running still has diagnostics.
      ...debugEditor({
        onToggleBreakpoint: (line) => debugRef.current.toggleBreakpoint(line),
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

  const execBlocks = (blocks ?? []).filter((b): b is ExecBlock => b.kind === "exec");

  if (error) {
    return (
      <div className="doc-page">
        <p className="error">{error}</p>
      </div>
    );
  }
  if (!doc || !blocks) {
    return (
      <div className="doc-page">
        <p className="muted">Loading document…</p>
      </div>
    );
  }

  return (
    <div
      className={`doc-page with-chat${choice.id === "freeform" ? "" : " wide-mode"}`}
    >
      <div className="doc-main">
        <header className="doc-toolbar">
          <button className="btn btn-link" onClick={() => navigate("/projects")}>
            ← Projects
          </button>
          <span className="doc-path mono">{doc.path}</span>
          <div className="toolbar-actions">
            {/* Only when there is a choice worth making. A control that
                offers one option is a control that asks a question it already
                knows the answer to. */}
            {choices.length > 1 && (
            <label className="layout-picker">
              <span className="layout-picker__label">Layout</span>
              <select
                value={choice.id}
                onChange={(event) => setChoiceId(event.target.value)}
                title={choice.detail}
              >
                {choices.map((candidate) => (
                  <option
                    key={candidate.id}
                    value={candidate.id}
                    disabled={candidate.id === "ribbons" && (!wide || hasOutputs === false)}
                  >
                    {candidate.name}
                  </option>
                ))}
              </select>
            </label>
            )}
            {syncState !== "idle" && (
              <span
                className={`save-state save-state-${syncState}`}
                role="status"
                title="Documents save themselves — edits sync to the project repo automatically."
              >
                {syncState === "editing" ? "Saving…" : "Saved"}
              </span>
            )}
            {/* The way to find a generated file when nothing is arranged for
                you. Absent rather than disabled when the document has not
                generated anything: there is nothing to explain. */}
            {hasOutputs && choice.id !== "ribbons" && (
              <button
                className="btn btn-quiet"
                onClick={() =>
                  setLayout((current) =>
                    openInLayout(current, makeTab("tool", "outputs", "Outputs")),
                  )
                }
                title="The files this document generates"
              >
                Outputs
              </button>
            )}
            <button className="btn" disabled={runningCells.size > 0} onClick={() => void runAll()}>
              Run all
            </button>
            <button className="btn btn-primary" onClick={() => void verify()}>
              Verify
            </button>
          </div>
        </header>
        {banner && (
          <div className={`banner banner-${banner.kind}`} role="status">
            {banner.text}
          </div>
        )}
        {renderError && (
          <div className="banner banner-fail" role="status">
            Could not weave this document — editing still works. {renderError}
          </div>
        )}
        {choice.id === "ribbons" ? (
          // Kept whole: this is the one view that draws the ribbons between a
          // document and the files it generates, and a pane cannot hold half
          // of a relationship.
          <SplitView
            docId={docId}
            docPath={doc.path}
            docSource={doc.source}
            editorKey={docId}
            realtime={realtime}
            onChange={setDirtySource}
            selectSpan={selectSpan}
            execBlocks={execBlocks}
            runningCells={runningCells}
            onRunCell={(id) => void runCell(id)}
            lspExtensions={lspExtensions}
            lspDiagnostics={lsp.diagnostics}
            makeOutputLsp={makeOutputLsp}
            outputTarget={outputTarget}
            onEditorReady={(view) => {
              editorRef.current = view;
            }}
          />
        ) : (
          <ShellView
            layout={layout}
            onLayout={setLayout}
            empty={
              <span>
                Nothing open here.
                <br />
                Open a generated file from the Outputs tab, or split another pane.
              </span>
            }
            render={(tab) => {
              if (tab.kind === "document") {
                return (
                  <DocumentEditor
                    key={docId}
                    docId={docId}
                    initialSource={doc.source}
                    realtime={realtime}
                    onChange={setDirtySource}
                    selectSpan={selectSpan}
                    execBlocks={execBlocks}
                    runningCells={runningCells}
                    onRunCell={(id) => void runCell(id)}
                    lspExtensions={lspExtensions}
                    lspDiagnostics={lsp.diagnostics}
                    onViewReady={(view) => {
                      editorRef.current = view;
                    }}
                  />
                );
              }
              if (tab.kind === "generated") {
                return (
                  <GeneratedFileView
                    docId={docId}
                    path={tab.target}
                    makeOutputLsp={makeOutputLsp}
                    onSelectSpan={onSelectSpan}
                  />
                );
              }
              return <OutputsTool docId={docId} onOpen={openGenerated} />;
            }}
          />
        )}
      </div>
      {references && (
        <ReferencesPanel
          locations={references.locations}
          query={references.query}
          onPick={openTarget}
          onClose={() => setReferences(null)}
        />
      )}
      {/* The debugger. Below the editor rather than beside it: the editor
          carries the gutter, the paused line and the values, and this is the
          part that is left. */}
      {(
        <DebugPanel
          status={debug.status}
          message={debug.message}
          capabilities={debug.capabilities}
          frames={debug.frames}
          variables={debug.variables}
          selectedFrame={debug.selectedFrame}
          onSelectFrame={(id) => {
            debug.selectFrame(id);
            // Selecting a frame is asking to go there — and a jump with no
            // sign of having moved leaves you hunting for what changed.
            const frame = debug.frames.find((candidate) => candidate.id === id);
            const view = editorRef.current;
            if (view && frame?.line !== null && frame?.line !== undefined) {
              revealLine(view, frame.line);
            }
          }}
          onStep={debug.step}
          onJumpHere={() => {
            // "Move here" acts on the caret, which is how a person says
            // *here* without a second control to pick a line.
            const editor = editorRef.current;
            if (!editor) return;
            const line = editor.state.doc.lineAt(editor.state.selection.main.head).number - 1;
            debug.jumpTo(line);
          }}
          onEvaluate={(expression) => debug.evaluate(expression)}
          lastValue={debug.lastValue}
          onStart={debug.start}
          onStop={debug.stop}
        />
      )}
      <PromptPanel prompt={prompt} onSettle={settle} />
      <ChatDock
        docId={docId}
        realtime={realtime}
        collapsed={chatCollapsed}
        onToggleCollapsed={() =>
          setChatCollapsed((v) => {
            localStorage.setItem("hickory.chatCollapsed", v ? "0" : "1");
            return !v;
          })
        }
        onAgentFinished={refresh}
      />
    </div>
  );
}
