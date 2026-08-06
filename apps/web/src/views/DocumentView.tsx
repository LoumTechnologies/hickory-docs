import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, MOCK } from "../api/client";
import { WsRealtime, getSharedRealtime, type Realtime } from "../api/realtime";
import type { Block, Doc, ExecBlock, SourceEdit } from "../api/types";
import { AgentPanel } from "../components/AgentPanel";
import { DocumentEditor } from "../editor/DocumentEditor";
import { OutputView } from "./OutputView";
import { navigate } from "../router";

type Banner = { kind: "pending" | "pass" | "fail"; text: string } | null;

export function DocumentView({ docId }: { docId: string }) {
  const [doc, setDoc] = useState<Doc | null>(null);
  const [blocks, setBlocks] = useState<Block[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<"document" | "output">("document");
  const [showAgent, setShowAgent] = useState(false);
  const [runningCells, setRunningCells] = useState<Set<string>>(new Set());
  const [banner, setBanner] = useState<Banner>(null);
  const [selectSpan, setSelectSpan] = useState<[number, number] | null>(null);
  const [dirtySource, setDirtySource] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  // Bumped when an /outputs/edit rewrote the source out-of-band so the
  // Document editor reseeds its Y.Doc from the freshly fetched source.
  const [editorEpoch, setEditorEpoch] = useState(0);

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

  const refresh = useCallback(() => {
    api.doc(docId).then(setDoc, (e) => setError(String(e.message ?? e)));
    api.render(docId).then(
      (r) => setBlocks(r.blocks),
      (e) => setError(String(e.message ?? e)),
    );
  }, [docId]);

  useEffect(refresh, [refresh]);

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
        api.render(docId).then((r) => setBlocks(r.blocks), () => undefined);
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
  }, [realtime, docId]);

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
        api.render(docId).then((r) => setBlocks(r.blocks), () => undefined);
      } else {
        checkRunRef.current = run_id;
      }
    } catch (e) {
      setBanner({ kind: "fail", text: e instanceof Error ? e.message : String(e) });
    }
  };

  const save = async () => {
    if (dirtySource === null || !doc) return;
    setSaving(true);
    try {
      const updated = await api.saveDoc(docId, dirtySource);
      setDoc(updated);
      setDirtySource(null);
      const r = await api.render(docId);
      setBlocks(r.blocks);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  const onSelectSpan = (span: [number, number]) => {
    setView("document");
    setSelectSpan(span);
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
    <div className={`doc-page${showAgent ? " with-agent" : ""}`}>
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
            </div>
            {view === "document" && (
              <button className="btn" disabled={dirtySource === null || saving} onClick={() => void save()}>
                {saving ? "Saving…" : "Save"}
              </button>
            )}
            <button className="btn" disabled={runningCells.size > 0} onClick={() => void runAll()}>
              Run all
            </button>
            <button className="btn btn-primary" onClick={() => void verify()}>
              Verify
            </button>
            <button
              className={`btn${showAgent ? " on" : ""}`}
              aria-pressed={showAgent}
              onClick={() => setShowAgent((v) => !v)}
            >
              Agent
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
          />
        ) : (
          <OutputView docId={docId} onSourceEdited={onSourceEdited} onSelectSpan={onSelectSpan} />
        )}
      </div>
      {showAgent && <AgentPanel docId={docId} realtime={realtime} />}
    </div>
  );
}
