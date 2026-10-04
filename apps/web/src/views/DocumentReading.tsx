import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { DocumentEditor } from "../editor/DocumentEditor";
import { LocalRealtime } from "../api/realtime";
import { useReading } from "../lib/readingViews";
import { CommitReading } from "./CommitReading";
import "./DocumentReading.css";
import { languageFromPath } from "../editor/languages";
const noCells = new Set<string>();

/** A reading has no save path or execution binding. Its bytes never enter a document room. */
export function DocumentComparison({ id, path, before, after }: { id: string; path: string; before: string; after: string }) {
  const text = (value: string) => {
    if (!path.includes(".") || /\.(md|hick)$/i.test(path)) return value;
    const longest = Math.max(2, ...[...`${before}${after}`.matchAll(/`+/g)].map(match => match[0].length));
    const fence = "`".repeat(longest + 1);
    return `${fence}${languageFromPath(path) ?? "text"}\n${value}\n${fence}\n`;
  };
  const source = text(after), base = text(before);
  const realtime = useMemo(() => new LocalRealtime(), [id]);
  const editor = useRef<EditorView | null>(null);
  const ready = useCallback((view: EditorView | null) => { editor.current = view; }, []);
  useEffect(() => {
    const view = editor.current;
    if (view && view.state.doc.toString() !== source) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: source } });
  }, [source]);
  useEffect(() => () => realtime.close(), [realtime]);
  return <DocumentEditor docId={`reading:${id}`} initialSource={source} realtime={realtime}
    preserveBytes readOnly comparisonBase={base} execBlocks={[]} runningCells={noCells} onRunCell={() => {}}
    onViewReady={ready} />;
}

export function DocumentReading({ id }: { id: string }) {
  const reading = useReading(id);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  if (id.startsWith("commit:")) return <CommitReading sha={id.slice(7)} />;
  if (!reading) return <p className="muted">This proposal is no longer connected. Reopen its conversation to review changes.</p>;
  const decide = async (accepted: boolean) => {
    setBusy(true); setError("");
    try { await reading.decide?.(accepted); } catch (e) { setError(e instanceof Error ? e.message : String(e)); }
    finally { setBusy(false); }
  };
  return <section className="document-reading" aria-label="Document change review">
    <header className="doc-tab-toolbar"><span>{reading.title} · {reading.path} · {reading.status}</span>
      {reading.status === "pending" && <><button className="btn" disabled={busy} onClick={() => void decide(true)}>Accept change</button>
        <button className="btn" disabled={busy} onClick={() => void decide(false)}>Reject change</button></>}
    </header>
    {error && <p role="alert">{error}</p>}
    <DocumentComparison id={id} path={reading.path} before={reading.before} after={reading.after} />
  </section>;
}
