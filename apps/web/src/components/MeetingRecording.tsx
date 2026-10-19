import { useEffect, useRef, useState } from "react";
import { request } from "../api/client";
type Recording = { id: string; mime: string; bytes: number; sha256: string; complete: boolean; created: string };
type Adapter = { mode: string; command: string; args: string[]; endpoint: string; model: string };
type Catalogue = { directory: string; max_bytes: number; recordings: Recording[]; adapter: Adapter; retention: string };
const empty: Adapter = { mode: "local", command: "", args: ["{audio}"], endpoint: "", model: "" };
export function MeetingRecording({ onOpenNote }: { onOpenNote?: (id: string) => void }) {
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
  }
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; stopped.current = true; if (recorder.current?.state === "recording") recorder.current.stop(); recorder.current?.stream.getTracks().forEach(t => t.stop()); }; }, []);
  async function start() {
    setError(""); setStatus("Requesting microphone permission");
    try {
      if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === "undefined") throw new Error("Microphone capture is unavailable in this browser. System audio is unsupported.");
      const mime = ["audio/webm", "audio/ogg", "audio/mp4"].find(m => MediaRecorder.isTypeSupported(m));
      if (!mime) throw new Error("No supported audio recording format");
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
      if (!mounted.current) { stream.getTracks().forEach(t => t.stop()); return; }
      try {
        const initial = await request<Recording>("POST", "/api/meetings/recordings", { mime, offset: 0, data: "" });
        if (!mounted.current) { stream.getTracks().forEach(t => t.stop()); return; }
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
      } catch (reason) { stream.getTracks().forEach(t => t.stop()); throw reason; }
    } catch (reason) { setError(String(reason)); setStatus("Capture refused or unavailable"); }
  }
  async function transcribe(record: Recording) {
    setError(""); setTranscribing(record.id); setEvidence(null);
    try {
      const result = await request<{ recording: Recording; transcript: string; transcript_sha256: string }>("POST", "/api/meetings/transcribe", { id: record.id, remote_consent: catalogue?.adapter.mode === "remote" && consent, expected_adapter: catalogue?.adapter });
      setEvidence(result); setCorrection(result.transcript);
    } catch (reason) { setError(String(reason)); } finally { setTranscribing(null); }
  }
  async function importTranscript() {
    if (!evidence) return;
    try {
      const imported = await request<{ doc_id: string }>("POST", "/api/meetings/import", { name: `recording-${evidence.recording.id}.txt`, text: correction });
      // Import receives evidence separately; corrections never overwrite the retained transcription.
      await request("POST", "/api/meetings/recordings/" + evidence.recording.id + "/link", { doc_id: imported.doc_id, transcript_sha256: evidence.transcript_sha256, correction });
      onOpenNote?.(imported.doc_id);
    } catch (reason) { setError(String(reason)); }
  }
  return <details onToggle={e => { if (e.currentTarget.open && !catalogue) void refresh().catch(r => setError(String(r))); }}>
    <summary>Record a meeting</summary>
    <p>Microphone only; system audio is unavailable. {catalogue?.retention ?? "Audio stays local until explicit deletion."}</p>
    <p>Save destination: <code>{catalogue?.directory ?? "Loading local application data location"}</code>. Limit: 256 MiB per recording.</p>
    <p role="status">{status}{active ? ` · ${active.bytes} bytes · ${active.sha256 || "hash finalized on stop"}` : ""}</p>
    <button disabled={!catalogue || status.startsWith("Recording") || status.startsWith("Requesting") || status.startsWith("Stopping")} onClick={() => void start()}>Start microphone recording</button>
    <button disabled={recorder.current?.state !== "recording"} onClick={() => { stopped.current = true; recorder.current?.stop(); setStatus("Stopping · retaining final audio"); }}>Stop recording</button>
    <fieldset><legend>Transcription configuration · save before capture/upload</legend>
      <label>Processing <select value={adapter.mode} onChange={e => { setAdapter(a => ({ ...a, mode: e.target.value })); setConsent(false); }}><option value="local">Local adapter · runs on this machine</option><option value="remote">Remote adapter · uploads audio and may charge</option></select></label>
      {adapter.mode === "local" ? <><label>Adapter executable <input value={adapter.command} onChange={e => setAdapter(a => ({ ...a, command: e.target.value }))} /></label><label>Arguments (JSON array; {'{audio}'} and {'{model}'} substituted)<input value={argsText} onChange={e => setArgsText(e.target.value)} /></label></> : <><label>HTTPS adapter endpoint <input value={adapter.endpoint} onChange={e => setAdapter(a => ({ ...a, endpoint: e.target.value }))} /></label><label><input type="checkbox" checked={consent} onChange={e => setConsent(e.target.checked)} /> I authorize uploading the selected recording and possible provider charges for this transcription.</label></>}
      <label>Model/settings identity <input value={adapter.model} onChange={e => setAdapter(a => ({ ...a, model: e.target.value }))} /></label>
      <p>Saved adapter used for transcription: {catalogue?.adapter.mode || "unconfigured"} · {catalogue?.adapter.endpoint || catalogue?.adapter.command} · {catalogue?.adapter.model}</p>
      <button onClick={() => { try { const args: unknown = JSON.parse(argsText); if (!Array.isArray(args) || !args.every(a => typeof a === "string")) throw new Error("Arguments must be a JSON string array"); void request("POST", "/api/meetings/transcription", { ...adapter, args }).then(refresh).catch(r => setError(String(r))); } catch (r) { setError(String(r)); } }}>Save transcription configuration</button>
    </fieldset>
    {catalogue?.recordings.map(record => <section key={record.id} aria-label={`Recording ${record.id}`}>
      <p>{record.created} · {record.bytes} bytes · {record.complete ? "Stopped" : "Interrupted/in progress"} · <code>{record.sha256}</code></p>
      <a href={`/api/meetings/recordings/${record.id}`} download>Save retained audio</a>
      {!record.complete && <button onClick={() => void request("POST", `/api/meetings/recordings/${record.id}/recover`, {}).then(refresh).catch(r => setError(String(r)))}>Recover and stop retained audio</button>}
      <button disabled={!record.complete || !record.bytes || !!transcribing || (catalogue?.adapter.mode === "remote" && !consent)} onClick={() => { void transcribe(record); setConsent(false); }}>Transcribe / retry</button>
      <button disabled={!!transcribing} onClick={() => void request("DELETE", `/api/meetings/recordings/${record.id}`).then(refresh).catch(r => setError(String(r)))}>Delete retained audio and transcripts</button>
    </section>)}
    {transcribing && <p role="status">Transcribing retained audio… <button onClick={() => void request("POST", `/api/meetings/recordings/${transcribing}/cancel`, {}).catch(r => setError(String(r)))}>Cancel transcription</button></p>}
    {error && <p role="alert">{error}</p>}
    {evidence && <section aria-label="Transcription evidence"><p>Audio {evidence.recording.sha256} · transcript {evidence.transcript_sha256}. Speaker identity is unknown unless supplied or confirmed. Review corrections separately below.</p><label>Transcript corrections<textarea value={correction} onChange={e => setCorrection(e.target.value)} /></label><button onClick={() => void importTranscript()}>Import reviewed transcript</button></section>}
  </details>;
}
