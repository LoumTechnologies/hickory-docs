import { UiLink } from "../components/UiLink";
import { useEffect, useRef, useState } from "react";
import { decodeSnapshot, verifiedPublicationAsset, type ViewerSnapshot } from "../embed/PublishedSnapshot";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { useMediaActions } from "../components/useMediaActions";
import type { UiOutcome, UiValue } from "../lib/uiActions";
import { emit } from "../analytics/events";

/** Media is delivered only through #61's selected, hash-reviewed asset manifest. */
export function WorkflowRecording() {
  const [snapshot, setSnapshot] = useState<ViewerSnapshot | null>(null);
  const [artifact, setArtifact] = useState("");
  const [video, setVideo] = useState("");
  const [session, setSession] = useState("");
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("Recording has not been loaded. Public results and the browser example are available without installation.");
  const urls = useRef<string[]>([]);
  const owner = useRef({ alive: true, busy: false, controller: null as AbortController | null });
  const media = useRef<HTMLVideoElement>(null);
  useMediaActions(media, video);
  useEffect(() => { owner.current.alive = true; return () => { owner.current.alive = false; owner.current.controller?.abort(); for (const url of urls.current) URL.revokeObjectURL(url); }; }, []);
  async function perform(operation: (check: () => void, signal: AbortSignal) => Promise<UiValue>, signal?: AbortSignal): Promise<UiOutcome> {
    if (!owner.current.alive || signal?.aborted) return { status: "unavailable", reason: "The originating recording request is unavailable or cancelled" };
    if (owner.current.busy) return { status: "unavailable", reason: "Another reviewed-asset request is pending" };
    const controller = new AbortController(), abort = () => controller.abort();
    owner.current.busy = true; owner.current.controller = controller; setBusy(true);
    signal?.addEventListener("abort", abort, { once: true });
    const check = () => { if (!owner.current.alive || controller.signal.aborted) throw Error("The originating recording request was cancelled or disposed; no late asset was adopted"); };
    try { const value = await operation(check, controller.signal); check(); return { status: "completed", value }; }
    catch (error) { if (owner.current.alive) setStatus(String(error)); return { status: "unavailable", reason: String(error) }; }
    finally { signal?.removeEventListener("abort", abort); owner.current.busy = false; owner.current.controller = null; if (owner.current.alive) setBusy(false); }
  }
  const retain = (url: string, check: () => void) => { try { check(); } catch (error) { URL.revokeObjectURL(url); throw error; } urls.current.push(url); return url; };
  async function load(signal?: AbortSignal): Promise<UiOutcome> { return perform(async (check, signal): Promise<UiValue> => {
    for (const url of urls.current) URL.revokeObjectURL(url);
    urls.current = []; setVideo(""); setSession(""); setSnapshot(null); setArtifact("");
    emit({ name: "interest_clicked", interest_id: "agent-engineering", link_id: "load-recording" });
    setStatus("Loading reviewed publication metadata…");
      const indexUrl = new URL("./publications/index.json", location.href);
      const response = await fetch(indexUrl, { signal });
      if (!response.ok) throw Error("Public results are unavailable");
      const index = await response.json() as { version: number; documents: { path: string; snapshot: string; source_hash: string; revision: string }[] };
      if (index.version !== 1 || !Array.isArray(index.documents)) throw Error("Invalid public-results index");
      const entry = index.documents.find(entry => entry.path === "examples/ai-engineer-deadline.md");
      if (!entry) throw Error("This scenario has no selected public result");
      const url = new URL(entry.snapshot, indexUrl);
      if (url.origin !== location.origin || !url.pathname.startsWith(indexUrl.pathname.replace(/index\.json$/, ""))) throw Error("External/unselected snapshot refused");
      const captured = await fetch(url, { signal }); if (!captured.ok) throw Error("Selected snapshot unavailable");
      const checked = await decodeSnapshot(await captured.json());
      if (checked.source_hash !== entry.source_hash || checked.verification?.revision !== entry.revision) throw Error("Publication index and snapshot revisions differ");
      check(); setSnapshot(checked); setArtifact(url.href);
      if (!checked.demonstration) { setStatus("A real native/provider recording has not been published for this result. Inspect the retained results or try the explicitly labelled browser example below."); return { recording_loaded: false, metadata_loaded: true, historical: true }; }
      const media = await verifiedPublicationAsset(checked, new URL("./", url).href, checked.demonstration.recording_asset, signal);
      retain(media, check); setVideo(media); setStatus("Reviewed recording loaded. Seeded inputs and declared cuts remain separate from actual provider/native execution.");
      return { recording_loaded: true, historical: true, captured_revision: checked.demonstration.captured_revision, source_hash: checked.source_hash };
    }, signal); }
  async function loadSession(signal?: AbortSignal): Promise<UiOutcome> {
    const key = snapshot?.demonstration?.session_asset;
    if (!snapshot || !key) return { status: "unavailable", reason: "Load publication metadata with a selected public session first" };
    return perform(async (check, signal) => {
      const checked = await verifiedPublicationAsset(snapshot, new URL("./", artifact).href, key, signal);
      retain(checked, check); setSession(checked);
      setStatus("Reviewed public session export verified. It is historical evidence, not a new provider request.");
      return { session_asset_loaded: true, historical: true, source_hash: snapshot.source_hash };
    }, signal);
  }
  const revision = useUiRevision(() => [snapshot, artifact, video, session, busy, status]);
  useUiReading("reviewed workflow recording", { busy, status, video_loaded: !!video, session_loaded: !!session,
    publication: snapshot ? { source_hash: snapshot.source_hash, verification: snapshot.verification ? { revision: snapshot.verification.revision, status: snapshot.verification.status } : null, demonstration: snapshot.demonstration ?? null } : null });
  useUiCommands([
    { key: "workflow-recording:load", command: { label: "Load reviewed workflow recording", parent: "workflow-recording", subject: "workflow-recording:load", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !owner.current.busy }), execute: async (_args, _actor, signal) => load(signal) } },
    ...(snapshot?.demonstration?.session_asset ? [{ key: "workflow-recording:session", command: { label: "Load reviewed public session export", parent: "workflow-recording", subject: "workflow-recording:load", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !owner.current.busy && !session }), execute: async (_args: Record<string, UiValue>, _actor: unknown, signal: AbortSignal) => loadSession(signal) } }] : []),
    { key: "workflow-recording:cancel", command: { label: "Cancel reviewed workflow asset loading", parent: "workflow-recording", subject: "workflow-recording:cancel", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: owner.current.busy }), execute: async () => { owner.current.controller?.abort(); return { status: "completed", value: { cancellation_requested: true, rollback: false } }; } } },
  ]);
  return <section id="watch-workflow" aria-label="Recorded agent workflow">
    <h2>Investigate first. Review the evidence. Then change the code.</h2>
    <p>Delegate a retry-budget bug: two 80ms attempts share one 100ms deadline. Inspect the agent's failure and paused source, decide that retries consume the same budget, then review a focused correction and its checks.</p>
    <button disabled={busy} onClick={() => void load()}>{busy ? "Loading reviewed recording…" : "Load reviewed workflow recording"}</button>
    <p role="status">{status}</p>
    {video && <video ref={media} controls preload="metadata" src={video} onPlay={() => emit({ name: "demo_engaged", demo_id: "agent-first-recording", step: "play" })} onError={() => setStatus("The recording could not be decoded by this browser; inspect its matching source and retained results.")} />}
    {video && <p><UiLink href={video} download={snapshot?.demonstration?.recording_asset.split("/").at(-1)}>Download this verified recording</UiLink></p>}
    {snapshot?.demonstration?.session_asset && <p>{session ? <UiLink href={session} download={snapshot.demonstration.session_asset.split("/").at(-1)}>Download reviewed public session export</UiLink> : <button disabled={busy} onClick={() => void loadSession()}>Load reviewed public session export</button>}</p>}
    {snapshot && <p>Published source <code>{snapshot.source_hash}</code> · tested revision <code>{snapshot.verification?.revision}</code> · result {snapshot.verification?.status ?? "not_run"}. <UiLink href={`./snapshot.html?src=${encodeURIComponent(artifact)}`}>Inspect this exact result, including failures and stale evidence</UiLink></p>}
    {snapshot?.demonstration && <><p>Recording build: <code>{snapshot.demonstration.captured_revision}</code>. Publication and recording revisions are reported separately.</p><ul>{snapshot.demonstration.cuts?.map((note, index) => <li key={index}>{note}</li>)}</ul></>}
    <p>These deterministic checks establish budget allocation, not real network latency or OS scheduling. A recorded browser proposal is not a live provider request.</p>
  </section>;
}
