import { useRef, useState } from "react";
import { request } from "../api/client";

interface Imported {
  doc_id: string; path: string; source: string; duplicate: boolean; format: string; source_sha256: string;
  utterances: { id: string; speaker: string | null; at: string | null;
    text: string; start_utf8: number; end_utf8: number; line: number }[];
}

/** Import is deterministic local parsing; no summary-generation side effect. */
export function MeetingImport({ onOpen, onOpenNote }: { onOpen: (path: string, line?: number) => void; onOpenNote?: (id: string) => void }) {
  const [open, setOpen] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Imported | null>(null);
  const [error, setError] = useState("");
  const generation = useRef(0);
  async function ingest() {
    if (!file || busy) return;
    const selected = file;
    const epoch = ++generation.current;
    setBusy(true); setError(""); setResult(null);
    try {
      if (!/\.(vtt|srt|sbv|txt|md)$/i.test(selected.name))
        throw new Error("Choose VTT, SRT, SBV or text. Transcribe audio/video first.");
      if (selected.size > 1024 * 1024) throw new Error("Choose a transcript smaller than 1 MB.");
      const text = new TextDecoder("utf-8", { fatal: true }).decode(await selected.arrayBuffer());
      // Cancel before upload prevents any write. Once submitted, the local
      // server may complete; hiding the panel cannot undo retained evidence.
      if (generation.current !== epoch) return;
      const imported = await request<Imported>("POST", "/api/meetings/import", { name: selected.name, text });
      if (generation.current === epoch) setResult(imported);
    } catch (reason) { if (generation.current === epoch) setError(String(reason)); }
    finally { if (generation.current === epoch) setBusy(false); }
  }
  return <details open={open} onToggle={(event) => setOpen(event.currentTarget.open)}>
    <summary>Import meeting transcript</summary>
    <p>Import locally without a model. Original bytes and fingerprints are retained.
      Summaries and action items are generated declarations, refreshed separately;
      citations and passing checks do not establish their accuracy.</p>
    <label>Transcript file <input type="file" accept=".vtt,.srt,.sbv,.txt,.md"
      disabled={busy} onChange={(event) => { setFile(event.target.files?.[0] ?? null); setResult(null); setError(""); }} /></label>
    <button disabled={!file || busy} onClick={() => void ingest()}>{busy ? "Importing…" : "Import transcript"}</button>
    {!busy && <button onClick={() => { generation.current++; setFile(null); setError(""); setResult(null); setOpen(false); }}>Cancel</button>}
    {busy && <p role="status">Import submitted; retained evidence may complete even if this panel closes.</p>}
    {error && <p role="alert">{error}</p>}
    {result && <section aria-label="Imported meeting evidence">
      <p role="status">{result.duplicate ? "Already imported; opened the existing note." : "Transcript imported."}</p>
      <button onClick={() => onOpenNote ? onOpenNote(result.doc_id) : onOpen(result.path)}>Open meeting note</button>
      <button onClick={() => onOpen(result.source)}>Open retained original</button>
      <p>Source SHA-256: <code>{result.source_sha256}</code></p>
      <p>Parsed {result.format}; references below use offsets into the retained UTF-8 source.
        Edit corrections in the note; the retained original stays unchanged.</p>
      <ol>{result.utterances.map((utterance) => <li key={utterance.id}>
        <button onClick={() => onOpen(result.source, utterance.line)} aria-label={`Open source for ${utterance.id}`}>
          {utterance.id} · {utterance.speaker ?? "Unknown speaker"} {utterance.at ?? ""}
        </button> <span>{utterance.text}</span>
        <small> source bytes {utterance.start_utf8}–{utterance.end_utf8}</small>
        <p>Cite this passage: <code>[{utterance.id}]({result.source}#L{utterance.line})</code></p>
      </li>)}</ol>
    </section>}
  </details>;
}
