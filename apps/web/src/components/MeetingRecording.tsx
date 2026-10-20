import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { request } from "../api/client";
type Recording = { id: string; mime: string; bytes: number; sha256: string; complete: boolean; created: string };
type Adapter = { mode: string; command: string; args: string[]; endpoint: string; model: string };
type Catalogue = { directory: string; max_bytes: number; recordings: Recording[]; adapter: Adapter; retention: string };
const empty: Adapter = { mode: "local", command: "", args: ["{audio}"], endpoint: "", model: "" };
export function MeetingRecording({ onOpenNote }: { onOpenNote?: (id: string) => void }) {
  const [open, setOpen] = useState(false);
  const parent = `meeting-recording:${useId()}`;
  const transcription = useRef<string | null>(null);
  const [catalogue, setCatalogue] = useState<Catalogue | null>(null);
  const [adapter, setAdapter] = useState<Adapter>(empty);
  const [argsText, setArgsText] = useState(JSON.stringify(empty.args));
  const mounted = useRef(true);
  const [status, setStatus] = useState("Idle");
  const [error, setError] = useState("");
  const [active, setActive] = useState<Recording | null>(null);
  const [consent, setConsent] = useState(false);
  const [transcribing, setTranscribing] = useState<string | null>(null);
  const [evidence, setEvidence] = useState<{ recording: Recording; transcript: string; transcript_sha256: string } | null>(null);
  const [correction, setCorrection] = useState("");
  const recorder = useRef<MediaRecorder | null>(null);
  const captured = useRef<Recording | null>(null);
  const uploads = useRef<Promise<void>>(Promise.resolve());
  const stopped = useRef(false);
  async function refresh() {
    const data = await request<Catalogue>("GET", "/api/meetings/recordings");
    setCatalogue(data); setAdapter(data.adapter.mode ? data.adapter : empty); setArgsText(JSON.stringify(data.adapter.mode ? data.adapter.args : empty.args));
    return data;
  }
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; stopped.current = true; if (recorder.current?.state === "recording") recorder.current.stop(); recorder.current?.stream.getTracks().forEach(t => t.stop()); }; }, []);
  async function start(): Promise<UiOutcome> {
    setError(""); setStatus("Requesting microphone permission");
    try {
      if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === "undefined") throw new Error("Microphone capture is unavailable in this browser. System audio is unsupported.");
      const mime = ["audio/webm", "audio/ogg", "audio/mp4"].find(m => MediaRecorder.isTypeSupported(m));
      if (!mime) throw new Error("No supported audio recording format");
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
      if (!mounted.current) { stream.getTracks().forEach(t => t.stop()); return { status: "unavailable", reason: "Recording view closed before capture started" }; }
      try {
        const initial = await request<Recording>("POST", "/api/meetings/recordings", { mime, offset: 0, data: "" });
        if (!mounted.current) { stream.getTracks().forEach(t => t.stop()); return { status: "unavailable", reason: "Recording view closed before capture started" }; }
        captured.current = initial; setActive(initial); stopped.current = false; uploads.current = Promise.resolve();
        const media = new MediaRecorder(stream, { mimeType: mime }); recorder.current = media;
        media.ondataavailable = event => {
          if (!event.data.size) return;
          uploads.current = uploads.current.then(async () => {
            const bytes = new Uint8Array(await event.data.arrayBuffer());
            for (let at = 0; at < bytes.length; at += 512 * 1024) {
              const chunk = bytes.subarray(at, at + 512 * 1024);
              let binary = ""; for (const byte of chunk) binary += String.fromCharCode(byte);
              const current = captured.current!;
              const saved = await request<Recording>("POST", "/api/meetings/recordings", { id: current.id, offset: current.bytes, data: btoa(binary) });
              captured.current = saved; setActive(saved);
            }
          }).catch(reason => { setError(`Audio retention failed: ${reason}. Stop and recover retained chunks.`); if (media.state === "recording") media.stop(); });
        };
        media.onerror = () => { setError("Microphone capture failed. Retained chunks remain recoverable."); if (media.state === "recording") media.stop(); };
        media.onstop = () => {
          stream.getTracks().forEach(t => t.stop());
          void uploads.current.then(async () => {
            const current = captured.current!;
            const saved = await request<Recording>("POST", "/api/meetings/recordings", { id: current.id, offset: current.bytes, data: "", complete: true });
            captured.current = saved; setActive(saved); setStatus("Stopped · audio retained locally"); await refresh();
          }).catch(reason => { setError(String(reason)); setStatus("Stopped · recovery required"); });
        };
        media.start(5000); setStatus("Recording microphone");
        return { status: "running", job: initial.id, value: { recording: initial.id, scope: "Microphone capture; final retention/hash pending stop" } };
      } catch (reason) { stream.getTracks().forEach(t => t.stop()); throw reason; }
    } catch (reason) { setError(String(reason)); setStatus("Capture refused or unavailable"); return { status: "unavailable", reason: String(reason) }; }
  }
  async function transcribe(record: Recording, signal?: AbortSignal): Promise<UiOutcome> {
    if (transcription.current) return { status: "unavailable", reason: "A transcription is already in flight" };
    if (signal?.aborted) return { status: "unavailable", reason: "Cancelled before transcription submission" };
    transcription.current = record.id;
    setError(""); setTranscribing(record.id); setEvidence(null);
    const cancel = () => { void request("POST", `/api/meetings/recordings/${record.id}/cancel`, {}).catch(reason => setError(String(reason))); };
    signal?.addEventListener("abort", cancel, { once: true });
    try {
      const result = await request<{ recording: Recording; transcript: string; transcript_sha256: string }>("POST", "/api/meetings/transcribe", { id: record.id, remote_consent: catalogue?.adapter.mode === "remote" && consent, expected_adapter: catalogue?.adapter });
      setEvidence(result); setCorrection(result.transcript);
      return { status: "completed", value: { recording: result.recording.id, audio_sha256: result.recording.sha256, transcript_sha256: result.transcript_sha256, transcript_bytes: new TextEncoder().encode(result.transcript).length, scope: "Actual adapter transcription; not human-corrected or independently verified" } };
    } catch (reason) { setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
    finally { signal?.removeEventListener("abort", cancel); transcription.current = null; setTranscribing(null); setConsent(false); }
  }
  async function importTranscript(): Promise<UiOutcome> {
    if (!evidence) return { status: "unavailable", reason: "Transcribe retained audio first" };
    try {
      const imported = await request<{ doc_id: string }>("POST", "/api/meetings/import", { name: `recording-${evidence.recording.id}.txt`, text: correction });
      // Import receives evidence separately; corrections never overwrite the retained transcription.
      await request("POST", "/api/meetings/recordings/" + evidence.recording.id + "/link", { doc_id: imported.doc_id, transcript_sha256: evidence.transcript_sha256, correction });
      onOpenNote?.(imported.doc_id);
      return { status: "completed", value: { doc_id: imported.doc_id, recording: evidence.recording.id, transcript_sha256: evidence.transcript_sha256, navigation_requested: !!onOpenNote, correctness: "not established" } };
    } catch (reason) { setError(String(reason)); return { status: "unavailable", reason: `${String(reason)}; inspect retained note/link before retrying because import may already have completed` }; }
  }
  function stopCapture(): UiOutcome {
    if (recorder.current?.state !== "recording") return { status: "unavailable", reason: "Microphone is not currently recording" };
    stopped.current = true; recorder.current.stop(); setStatus("Stopping · retaining final audio");
    return { status: "completed", value: { recording: captured.current?.id ?? null, stop_requested: true, retention_complete: "Observe final retained status/hash" } };
  }
  async function saveConfiguration(): Promise<UiOutcome> {
    try {
      const args: unknown = JSON.parse(argsText);
      if (!Array.isArray(args) || !args.every(a => typeof a === "string")) throw Error("Arguments must be a JSON string array");
      await request("POST", "/api/meetings/transcription", { ...adapter, args }); setConsent(false);
      const actual = await refresh();
      return { status: "completed", value: { mode: actual.adapter.mode, model: actual.adapter.model, scope: "Saved host configuration; no transcription or upload requested" } };
    } catch (reason) { setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
  }
  const revision = useUiRevision(() => [open, catalogue, adapter, argsText, status, error, active, consent, transcribing, evidence, correction]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, enabled: boolean, execute: UiCommand["execute"], parameters: UiCommand["parameters"] = {}) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: ["read", "cancel", "visibility"].includes(key) ? `${parent}:${key}` : parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled }), execute } });
  add("visibility", "Set meeting recording visibility", true, async args => { setOpen(args.open as boolean); if (args.open && !catalogue) await refresh(); return { status: "completed", value: args.open }; }, { open: { type: "boolean" } });
  if (open) {
    add("refresh", "Refresh retained meeting recordings", true, async () => { const actual = await refresh(); return { status: "completed", value: { recordings: actual.recordings.length } }; });
    add("start", "Start microphone recording", !!catalogue && !/^(Recording|Requesting|Stopping)/.test(status), async (_args, actor) => actor.kind === "agent" ? { status: "requires_input", request: `${parent}:microphone`, reason: "A human starts capture and grants actual browser/OS microphone consent" } : start());
    add("stop", "Stop microphone recording", recorder.current?.state === "recording", async () => stopCapture());
    add("config", "Set transcription configuration draft", true, async args => { setAdapter({ mode: args.mode as string, command: args.command as string, endpoint: args.endpoint as string, model: args.model as string, args: [] }); setArgsText(args.args as string); setConsent(false); return { status: "completed", value: { draft_changed: true, saved: false, upload_authorized: false } }; }, { mode: { type: "string", maxLength: 6, choices: ["local", "remote"] }, command: { type: "string", maxLength: 4096 }, endpoint: { type: "string", maxLength: 8192 }, model: { type: "string", maxLength: 4096 }, args: { type: "string", maxLength: 16384 } });
    add("save", "Save transcription configuration", true, async () => saveConfiguration());
    if (adapter.mode === "remote") add("consent", "Set remote transcription consent", true, async (args, actor) => {
      if (actor.kind === "agent" && args.checked) return { status: "requires_input", request: `${parent}:remote-consent`, reason: "The human authorizes the selected audio upload and possible charges for this transcription" };
      setConsent(args.checked as boolean); return { status: "completed", value: args.checked };
    }, { checked: { type: "boolean" } });
    const recordId = { id: { type: "string" as const, maxLength: 4096 } };
    add("transcribe", "Transcribe retained meeting audio", !transcribing && !!catalogue?.recordings.some(r => r.complete && r.bytes > 0), async (args, actor, signal) => {
      const record = catalogue?.recordings.find(r => r.id === args.id);
      if (!record?.complete || !record.bytes) return { status: "unavailable", reason: "Observe a stopped, nonempty retained recording" };
      if (catalogue?.adapter.mode === "remote" && actor.kind === "agent") return { status: "requires_input", request: `${parent}:remote-transcribe:${record.id}:${record.sha256}`, reason: "A human must authorize and submit this particular remote audio upload and charges" };
      if (catalogue?.adapter.mode === "remote" && !consent) return { status: "unavailable", reason: "Remote consent is not established" };
      return transcribe(record, signal);
    }, recordId);
    if (transcribing) add("cancel", "Cancel meeting transcription", true, async () => {
      const actual = await request<{ cancelled: string; coverage: string }>("POST", `/api/meetings/recordings/${transcribing}/cancel`, {});
      return { status: "completed", value: { cancellation_requested: transcribing, host: actual, scope: "Cancellation request is not rollback or proof a provider avoided processing/charges" } };
    });
    for (const operation of ["recover", "delete", "download"] as const) add(operation, `${operation === "recover" ? "Recover" : operation === "delete" ? "Delete" : "Download"} retained meeting audio`, operation === "download" || !transcribing, async args => {
      const record = catalogue?.recordings.find(r => r.id === args.id);
      if (!record) return { status: "unavailable", reason: "Observe a current retained recording identity" };
      if (operation !== "download" && captured.current?.id === record.id && (recorder.current?.state === "recording" || /^(Recording|Stopping)/.test(status))) return { status: "unavailable", reason: "Stop capture and wait for final retention before recovering/deleting its audio" };
      if (operation === "download") { const link = document.createElement("a"); link.href = `/api/meetings/recordings/${record.id}`; link.download = ""; link.click(); return { status: "completed", value: { browser_download_requested: record.id, file_saved: "Browser-owned; not observed" } }; }
      if (operation === "recover" && record.complete) return { status: "unavailable", reason: "Recording is already complete" };
      const actual = await request<Recording | { deleted: string }>(operation === "delete" ? "DELETE" : "POST", `/api/meetings/recordings/${record.id}${operation === "recover" ? "/recover" : ""}`, ...(operation === "delete" ? [] : [{}]));
      await refresh(); return { status: "completed", value: JSON.parse(JSON.stringify(actual)) };
    }, recordId);
    if (evidence) {
      add("correction", "Set meeting transcript correction draft", true, async args => { setCorrection(args.text as string); return { status: "completed", value: { draft_changed: true, imported: false, human_review: "not established" } }; }, { text: { type: "string", maxLength: 1048576 } });
      add("import", "Import reviewed meeting transcription", true, async (_args, actor) => actor.kind === "agent" ? { status: "requires_input", request: `${parent}:review:${evidence.transcript_sha256}`, reason: "The human reviews transcript corrections and imports through the actual control" } : importTranscript());
    }
    add("read", "Read meeting recording state bytes", true, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ catalogue, adapter_draft: { ...adapter, args_text: argsText }, status, error, active, consent, transcribing, evidence, correction, scope: "Actual UI/host state; transcription and model claims are not human review" }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside recording state" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    }, { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } });
  }
  const invoke = useUiCommands(commands);
  const click = (key: string, args: Parameters<typeof invoke>[1] = {}) => { void invoke(`${parent}:${key}`, args).catch(reason => setError(String(reason))); };
  return <details open={open} onToggle={e => { setOpen(e.currentTarget.open); if (e.currentTarget.open && !catalogue) void refresh().catch(r => setError(String(r))); }}>
    <summary>Record a meeting</summary>
    <p>Microphone only; system audio is unavailable. {catalogue?.retention ?? "Audio stays local until explicit deletion."}</p>
    <p>Save destination: <code>{catalogue?.directory ?? "Loading local application data location"}</code>. Limit: 256 MiB per recording.</p>
    <p role="status">{status}{active ? ` · ${active.bytes} bytes · ${active.sha256 || "hash finalized on stop"}` : ""}</p>
    <button disabled={!catalogue || status.startsWith("Recording") || status.startsWith("Requesting") || status.startsWith("Stopping")} onClick={() => click("start")}>Start microphone recording</button>
    <button disabled={recorder.current?.state !== "recording"} onClick={() => click("stop")}>Stop recording</button>
    <fieldset><legend>Transcription configuration · save before capture/upload</legend>
      <label>Processing <select value={adapter.mode} onChange={e => { setAdapter(a => ({ ...a, mode: e.target.value })); setConsent(false); }}><option value="local">Local adapter · runs on this machine</option><option value="remote">Remote adapter · uploads audio and may charge</option></select></label>
      {adapter.mode === "local" ? <><label>Adapter executable <input value={adapter.command} onChange={e => setAdapter(a => ({ ...a, command: e.target.value }))} /></label><label>Arguments (JSON array; {'{audio}'} and {'{model}'} substituted)<input value={argsText} onChange={e => setArgsText(e.target.value)} /></label></> : <><label>HTTPS adapter endpoint <input value={adapter.endpoint} onChange={e => setAdapter(a => ({ ...a, endpoint: e.target.value }))} /></label><label><input type="checkbox" checked={consent} onChange={e => setConsent(e.target.checked)} /> I authorize uploading the selected recording and possible provider charges for this transcription.</label></>}
      <label>Model/settings identity <input value={adapter.model} onChange={e => setAdapter(a => ({ ...a, model: e.target.value }))} /></label>
      <p>Saved adapter used for transcription: {catalogue?.adapter.mode || "unconfigured"} · {catalogue?.adapter.endpoint || catalogue?.adapter.command} · {catalogue?.adapter.model}</p>
      <button onClick={() => click("save")}>Save transcription configuration</button>
    </fieldset>
    {catalogue?.recordings.map(record => <section key={record.id} aria-label={`Recording ${record.id}`}>
      <p>{record.created} · {record.bytes} bytes · {record.complete ? "Stopped" : "Interrupted/in progress"} · <code>{record.sha256}</code></p>
      <a href={`/api/meetings/recordings/${record.id}`} download onClick={event => { event.preventDefault(); click("download", { id: record.id }); }}>Save retained audio</a>
      {!record.complete && <button onClick={() => click("recover", { id: record.id })}>Recover and stop retained audio</button>}
      <button disabled={!record.complete || !record.bytes || !!transcribing || (catalogue?.adapter.mode === "remote" && !consent)} onClick={() => click("transcribe", { id: record.id })}>Transcribe / retry</button>
      <button disabled={!!transcribing} onClick={() => click("delete", { id: record.id })}>Delete retained audio and transcripts</button>
    </section>)}
    {transcribing && <p role="status">Transcribing retained audio… <button onClick={() => click("cancel")}>Cancel transcription</button></p>}
    {error && <p role="alert">{error}</p>}
    {evidence && <section aria-label="Transcription evidence"><p>Audio {evidence.recording.sha256} · transcript {evidence.transcript_sha256}. Speaker identity is unknown unless supplied or confirmed. Review corrections separately below.</p><label>Transcript corrections<textarea value={correction} onChange={e => setCorrection(e.target.value)} /></label><button onClick={() => click("import")}>Import reviewed transcript</button></section>}
  </details>;
}
