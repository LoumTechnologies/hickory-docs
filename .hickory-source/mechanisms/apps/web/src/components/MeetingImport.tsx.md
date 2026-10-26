<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MeetingImport.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c47a539793f2"></a>
## `apps/web/src/components/MeetingImport.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Meeting recordings remain recoverable local evidence](../../../../../guarantees/authoring/recordings-stay-local/contract.md).

<a id="f-c47a539793f2-Imported"></a>
### `Imported`

<storycode:copy id="f-c47a539793f2-Imported">
import { MeetingRecording } from "./MeetingRecording";
import { useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { request } from "../api/client";

interface Imported {
  doc_id: string; path: string; source: string; duplicate: boolean; format: string; source_sha256: string;
  utterances: { id: string; speaker: string | null; at: string | null;
    text: string; start_utf8: number; end_utf8: number; line: number }[];
}

</storycode:copy>
<a id="f-c47a539793f2-MeetingImport"></a>
### `MeetingImport`

Import is deterministic local parsing; no summary-generation side effect.

<storycode:copy id="f-c47a539793f2-MeetingImport">
/** Import is deterministic local parsing; no summary-generation side effect. */
export function MeetingImport({ onOpen, onOpenNote }: { onOpen: (path: string, line?: number) => void; onOpenNote?: (id: string) => void }) {
  const [open, setOpen] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Imported | null>(null);
  const [error, setError] = useState("");
  const generation = useRef(0);
  const id = useId();
  const parent = `meeting-import:${id}`;
  const submitting = useRef(false);
  const revision = useUiRevision(() => [open, file?.name, file?.size, busy, result, error]);
  async function ingest(selected: { name: string; size: number; arrayBuffer: () => Promise<ArrayBuffer> }, signal?: AbortSignal): Promise<UiOutcome> {
    if (submitting.current) return { status: "unavailable", reason: "Import already running" };
    submitting.current = true;
    const epoch = ++generation.current;
    setBusy(true); setError(""); setResult(null);
    try {
      if (!/\.(vtt|srt|sbv|txt|md)$/i.test(selected.name))
        throw new Error("Choose VTT, SRT, SBV or text. Transcribe audio/video first.");
      if (selected.size > 1024 * 1024) throw new Error("Choose a transcript smaller than 1 MB.");
      const text = new TextDecoder("utf-8", { fatal: true }).decode(await selected.arrayBuffer());
      if (generation.current !== epoch || signal?.aborted) return { status: "unavailable", reason: "Cancelled before upload" };
      const imported = await request<Imported>("POST", "/api/meetings/import", { name: selected.name, text });
      if (generation.current === epoch) setResult(imported);
      return { status: "completed", value: { doc_id: imported.doc_id, path: imported.path, source: imported.source, source_sha256: imported.source_sha256, duplicate: imported.duplicate, utterances: imported.utterances.length, cancellation: "Once uploaded, retained evidence is not rolled back" } };
    } catch (reason) {
      if (generation.current === epoch) setError(String(reason));
      return { status: "unavailable", reason: String(reason) };
    } finally { submitting.current = false; if (generation.current === epoch) setBusy(false); }
  }
  function cancel() { generation.current++; setFile(null); setError(""); setResult(null); setOpen(false); }
  const commands: { key: string; command: UiCommand }[] = [];
  function command(key: string, label: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"], enabled = true) {
    commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled, ...(!enabled ? { reason: "Open import, select evidence or wait for the active import" } : {}) }), execute } });
  }
  command("visibility", "Set transcript import visibility", { open: { type: "boolean" } }, async args => { setOpen(args.open as boolean); return { status: "completed", value: args.open }; });
  if (open) {
    command("import-text", "Import transcript text", { name: { type: "string", maxLength: 255 }, text: { type: "string", maxLength: 1048576 } }, async (args, _actor, signal) => {
      const bytes = new TextEncoder().encode(args.text as string);
      return ingest({ name: args.name as string, size: bytes.byteLength, arrayBuffer: async () => bytes.buffer }, signal);
    }, !busy);
    command("import-file", "Import selected transcript file", {}, async (_args, _actor, signal) => file ? ingest(file, signal) : { status: "unavailable", reason: "Select a transcript file or use Import transcript text" }, !!file && !busy);
    command("cancel", "Cancel transcript import panel", {}, async () => { cancel(); return { status: "completed", value: true }; }, !busy);
    command("read", "Read transcript import evidence", { offset: { type: "number", minimum: 0, maximum: 10485760, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ selected_file: file ? { name: file.name, bytes: file.size } : null, busy, error, result }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside evidence" };
      // Exact UTF-8 byte pages may split a code point; byte arrays preserve it.
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", total_bytes: bytes.length, offset, next_offset: end < bytes.length ? end : null } };
    });
    if (result) {
      command("note", "Open imported meeting note", {}, async () => { onOpenNote ? onOpenNote(result.doc_id) : onOpen(result.path); return { status: "completed", value: { navigation_requested: result.path } }; });
      command("original", "Open retained transcript original", {}, async () => { onOpen(result.source); return { status: "completed", value: { navigation_requested: result.source } }; });
      command("utterance", "Open transcript utterance source", { id: { type: "string", maxLength: 255 } }, async args => {
        const utterance = result.utterances.find(item => item.id === args.id);
        if (!utterance) return { status: "unavailable", reason: "Observe a current utterance identity first" };
        onOpen(result.source, utterance.line);
        return { status: "completed", value: { navigation_requested: result.source, line: utterance.line, source_sha256: result.source_sha256 } };
      });
    }
  }
  const invoke = useUiCommands(commands);
  return <><MeetingRecording onOpenNote={onOpenNote} /><details open={open} onToggle={(event) => setOpen(event.currentTarget.open)}>
    <summary>Import meeting transcript</summary>
    <p>Import locally without a model. Original bytes and fingerprints are retained.
      Summaries and action items are generated declarations, refreshed separately;
      citations and passing checks do not establish their accuracy.</p>
    <label>Transcript file <input type="file" accept=".vtt,.srt,.sbv,.txt,.md"
      disabled={busy} onChange={(event) => { setFile(event.target.files?.[0] ?? null); setResult(null); setError(""); }} /></label>
    <button disabled={!file || busy} onClick={() => void invoke(`${parent}:import-file`)}>{busy ? "Importing…" : "Import transcript"}</button>
    {!busy && <button onClick={() => void invoke(`${parent}:cancel`)}>Cancel</button>}
    {busy && <p role="status">Import submitted; retained evidence may complete even if this panel closes.</p>}
    {error && <p role="alert">{error}</p>}
    {result && <section aria-label="Imported meeting evidence">
      <p role="status">{result.duplicate ? "Already imported; opened the existing note." : "Transcript imported."}</p>
      <button onClick={() => void invoke(`${parent}:note`)}>Open meeting note</button>
      <button onClick={() => void invoke(`${parent}:original`)}>Open retained original</button>
      <p>Source SHA-256: <code>{result.source_sha256}</code></p>
      <p>Parsed {result.format}; references below use offsets into the retained UTF-8 source.
        Edit corrections in the note; the retained original stays unchanged.</p>
      <ol>{result.utterances.map((utterance) => <li key={utterance.id}>
        <button onClick={() => void invoke(`${parent}:utterance`, { id: utterance.id })} aria-label={`Open source for ${utterance.id}`}>
          {utterance.id} · {utterance.speaker ?? "Unknown speaker"} {utterance.at ?? ""}
        </button> <span>{utterance.text}</span>
        <small> source bytes {utterance.start_utf8}–{utterance.end_utf8}</small>
        <p>Cite this passage: <code>[{utterance.id}]({result.source}#L{utterance.line})</code></p>
      </li>)}</ol>
    </section>}
  </details></>;
}
</storycode:copy>

### Assemble `apps/web/src/components/MeetingImport.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MeetingImport.tsx"><storycode:paste select="#f-c47a539793f2-Imported"/><storycode:paste select="#f-c47a539793f2-MeetingImport"/></storycode:file>

</storycode:doc>
