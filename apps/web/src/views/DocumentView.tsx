import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, MOCK } from "../api/client";
import { WsRealtime, getSharedRealtime, type Realtime } from "../api/realtime";
import type { Block, Doc } from "../api/types";
import { BlockRenderer } from "../components/BlockRenderer";
import { AgentPanel } from "../components/AgentPanel";
import { SourceEditor } from "../editor/SourceEditor";
import { navigate } from "../router";

type Banner = { kind: "pending" | "pass" | "fail"; text: string } | null;

export function DocumentView({ docId }: { docId: string }) {
  const [doc, setDoc] = useState<Doc | null>(null);
  const [blocks, setBlocks] = useState<Block[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<"notebook" | "source">("notebook");
  const [showAgent, setShowAgent] = useState(false);
  const [runningCells, setRunningCells] = useState<Set<string>>(new Set());
  const [banner, setBanner] = useState<Banner>(null);
  const [selectSpan, setSelectSpan] = useState<[number, number] | null>(null);
  const [dirtySource, setDirtySource] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

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
    setView("source");
    setSelectSpan(span);
  };

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
                aria-selected={view === "notebook"}
                className={view === "notebook" ? "on" : ""}
                onClick={() => setView("notebook")}
              >
                Notebook
              </button>
              <button
                role="tab"
                aria-selected={view === "source"}
                className={view === "source" ? "on" : ""}
                onClick={() => setView("source")}
              >
                Source
              </button>
            </div>
            {view === "source" && (
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
        {view === "notebook" ? (
          <BlockRenderer
            blocks={blocks}
            runningCells={runningCells}
            onRunCell={(id) => void runCell(id)}
            onSelectSpan={onSelectSpan}
          />
        ) : (
          <SourceEditor
            docId={docId}
            initialSource={doc.source}
            realtime={realtime}
            onChange={setDirtySource}
            selectSpan={selectSpan}
          />
        )}
      </div>
      {showAgent && <AgentPanel docId={docId} realtime={realtime} />}
    </div>
  );
}
