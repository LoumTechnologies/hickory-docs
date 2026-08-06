import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, MOCK } from "../api/client";
import { WsRealtime, getSharedRealtime, type Realtime } from "../api/realtime";
import type { Block, Doc, ExecBlock, SourceEdit } from "../api/types";
import { ChatDock } from "../components/ChatDock";
import { ReferencesPanel } from "../components/ReferencesPanel";
import { byteToChar } from "../lib/offsets";
import { useLsp } from "../lsp/useLsp";
import { lspSupport, offsetToPosition, type LspNavigationTarget } from "../lsp/cmLsp";
import { positionToUtf16 } from "../lsp/positions";
import { sourcePositionAt, type OutputProvenance } from "../lsp/outputMapping";
import type { LspLocation } from "../lsp/client";
import { DocumentEditor } from "../editor/DocumentEditor";
import { OutputView } from "./OutputView";
import { SplitView } from "./SplitView";
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
  // Split is the workspace: document, the files it generates, and the
  // generated text, all editable and linked. It needs width, so narrow
  // viewports start on the document and the effect below keeps them there.
  const [view, setView] = useState<"document" | "output" | "split">(() =>
    typeof window.matchMedia === "function" && window.matchMedia(SPLIT_MIN_WIDTH).matches
      ? "split"
      : "document",
  );
  // Split is only offered when the viewport is wide enough for two panes.
  const [wide, setWide] = useState<boolean>(() =>
    typeof window.matchMedia === "function" ? window.matchMedia(SPLIT_MIN_WIDTH).matches : false,
  );
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
  // Bumped when an /outputs/edit rewrote the source out-of-band so the
  // Document editor reseeds its Y.Doc from the freshly fetched source.
  const [editorEpoch, setEditorEpoch] = useState(0);
  // Find-references results, and a pending "open this output file here" jump
  // produced by LSP navigation into a generated file.
  const [references, setReferences] = useState<{ locations: LspLocation[]; query: string } | null>(
    null,
  );
  const [outputTarget, setOutputTarget] = useState<{ path: string; span: [number, number] } | null>(
    null,
  );

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

  // Run events arrive in bursts (one terminal message per run, plus the
  // `verify` fallback path), and each used to trigger its own full render.
  // Coalesce them into one trailing fetch; `api.render` additionally collapses
  // anything still in flight.
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
    api.doc(docId).then(setDoc, (e) => setError(String(e.message ?? e)));
    api.render(docId).then(
      (r) => setBlocks(r.blocks),
      (e) => setError(String(e.message ?? e)),
    );
  }, [docId]);

  useEffect(refresh, [refresh]);

  // Track viewport width; Split degrades to Document when the window shrinks.
  useEffect(() => {
    if (typeof window.matchMedia !== "function") return;
    const mq = window.matchMedia(SPLIT_MIN_WIDTH);
    const onChange = () => setWide(mq.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);
  useEffect(() => {
    if (!wide) setView((v) => (v === "split" ? "document" : v));
  }, [wide]);


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

  // Spans arriving from provenance / source-edit responses are BYTE offsets
  // into the doc source; the editor selects by char position.
  const onSelectSpan = (span: [number, number]) => {
    setView((v) => (v === "split" ? v : "document"));
    setSelectSpan(
      doc ? [byteToChar(doc.source, span[0]), byteToChar(doc.source, span[1])] : span,
    );
  };

  // An output edit rewrote the source through provenance: re-fetch doc and
  // render, and reseed the Document editor from the new source.
  const onSourceEdited = (_edits: SourceEdit[]) => {
    setDirtySource(null);
    api.doc(docId).then(
      (d) => {
        setDoc(d);
        setEditorEpoch((n) => n + 1);
      },
      () => undefined,
    );
    api.render(docId).then((r) => setBlocks(r.blocks), () => undefined);
  };

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
        setView((v) => (v === "split" ? v : "output"));
        return;
      }
      const from = positionToUtf16(liveSource, target.range.start);
      const to = positionToUtf16(liveSource, target.range.end);
      setView((v) => (v === "split" ? v : "document"));
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
    () =>
      lspSupport({
        client: lsp.client,
        uri: lsp.uri,
        positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
        onNavigate: openTarget,
        onReferences: (locations, from) =>
          setReferences({
            locations,
            query: wordAt(positionToUtf16(liveSource, from.range.start)),
          }),
      }),
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
      className={`doc-page with-chat${view === "split" ? " split-mode" : ""}`}
    >
      <div className="doc-main">
        <header className="doc-toolbar">
          <button className="btn btn-link" onClick={() => navigate("/projects")}>
            ← Projects
          </button>
          <span className="doc-path mono">{doc.path}</span>
          <div className="toolbar-actions">
            <div className="segmented" role="tablist">
              <button
                role="tab"
                aria-selected={view === "document"}
                className={view === "document" ? "on" : ""}
                onClick={() => setView("document")}
              >
                Document
              </button>
              <button
                role="tab"
                aria-selected={view === "output"}
                className={view === "output" ? "on" : ""}
                onClick={() => setView("output")}
              >
                Output
              </button>
              {wide && (
                <button
                  role="tab"
                  aria-selected={view === "split"}
                  className={view === "split" ? "on" : ""}
                  onClick={() => setView("split")}
                >
                  Split
                </button>
              )}
            </div>
            {view !== "output" && syncState !== "idle" && (
              <span
                className={`save-state save-state-${syncState}`}
                role="status"
                title="Documents save themselves — edits sync to the project repo automatically."
              >
                {syncState === "editing" ? "Saving…" : "Saved"}
              </span>
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
        {view === "document" ? (
          <DocumentEditor
            key={`${docId}:${editorEpoch}`}
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
          />
        ) : view === "split" ? (
          <SplitView
            docId={docId}
            docPath={doc.path}
            docSource={doc.source}
            editorKey={`${docId}:${editorEpoch}`}
            realtime={realtime}
            onChange={setDirtySource}
            selectSpan={selectSpan}
            execBlocks={execBlocks}
            runningCells={runningCells}
            onRunCell={(id) => void runCell(id)}
            onSourceEdited={onSourceEdited}
            lspExtensions={lspExtensions}
            lspDiagnostics={lsp.diagnostics}
            makeOutputLsp={makeOutputLsp}
            outputTarget={outputTarget}
          />
        ) : (
          <OutputView
            docId={docId}
            onSourceEdited={onSourceEdited}
            onSelectSpan={onSelectSpan}
            makeOutputLsp={makeOutputLsp}
            outputTarget={outputTarget}
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
