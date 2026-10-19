import { useEffect, useRef, useState } from "react";
import { decodeSnapshot, verifiedPublicationAsset, type ViewerSnapshot } from "../embed/PublishedSnapshot";
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
  useEffect(() => () => { for (const url of urls.current) URL.revokeObjectURL(url); }, []);
  async function load() {
    if (busy) return;
    setBusy(true);
    for (const url of urls.current) URL.revokeObjectURL(url);
    urls.current = []; setVideo(""); setSession(""); setSnapshot(null); setArtifact("");
    emit({ name: "interest_clicked", interest_id: "agent-engineering", link_id: "load-recording" });
    setStatus("Loading reviewed publication metadata…");
    try {
      const indexUrl = new URL("./publications/index.json", location.href);
      const response = await fetch(indexUrl);
      if (!response.ok) throw Error("Public results are unavailable");
      const index = await response.json() as { version: number; documents: { path: string; snapshot: string; source_hash: string; revision: string }[] };
      if (index.version !== 1 || !Array.isArray(index.documents)) throw Error("Invalid public-results index");
      const entry = index.documents.find(entry => entry.path === "examples/ai-engineer-deadline.md");
      if (!entry) throw Error("This scenario has no selected public result");
      const url = new URL(entry.snapshot, indexUrl);
      if (url.origin !== location.origin || !url.pathname.startsWith(indexUrl.pathname.replace(/index\.json$/, ""))) throw Error("External/unselected snapshot refused");
      const captured = await fetch(url); if (!captured.ok) throw Error("Selected snapshot unavailable");
      const checked = await decodeSnapshot(await captured.json());
      if (checked.source_hash !== entry.source_hash || checked.verification?.revision !== entry.revision) throw Error("Publication index and snapshot revisions differ");
      setSnapshot(checked); setArtifact(url.href);
      if (!checked.demonstration) { setStatus("A real native/provider recording has not been published for this result. Inspect the retained results or try the explicitly labelled browser example below."); return; }
      const media = await verifiedPublicationAsset(checked, new URL("./", url).href, checked.demonstration.recording_asset);
      urls.current.push(media); setVideo(media); setStatus("Reviewed recording loaded. Seeded inputs and declared cuts remain separate from actual provider/native execution.");
    } catch (error) { setStatus(String(error)); }
    finally { setBusy(false); }
  }
  async function loadSession() {
    const key = snapshot?.demonstration?.session_asset;
    if (!snapshot || !key || busy) return;
    setBusy(true);
    try {
      const checked = await verifiedPublicationAsset(snapshot, new URL("./", artifact).href, key);
      urls.current.push(checked); setSession(checked);
      setStatus("Reviewed public session export verified. It is historical evidence, not a new provider request.");
    } catch (error) { setStatus(String(error)); }
    finally { setBusy(false); }
  }
  return <section id="watch-workflow" aria-label="Recorded agent workflow">
    <h2>Investigate first. Review the evidence. Then change the code.</h2>
    <p>Delegate a retry-budget bug: two 80ms attempts share one 100ms deadline. Inspect the agent's failure and paused source, decide that retries consume the same budget, then review a focused correction and its checks.</p>
    <button disabled={busy} onClick={() => void load()}>{busy ? "Loading reviewed recording…" : "Load reviewed workflow recording"}</button>
    <p role="status">{status}</p>
    {video && <video controls preload="metadata" src={video} onPlay={() => emit({ name: "demo_engaged", demo_id: "agent-first-recording", step: "play" })} onError={() => setStatus("The recording could not be decoded by this browser; inspect its matching source and retained results.")} />}
    {video && <p><a href={video} download={snapshot?.demonstration?.recording_asset.split("/").at(-1)}>Download this verified recording</a></p>}
    {snapshot?.demonstration?.session_asset && <p>{session ? <a href={session} download={snapshot.demonstration.session_asset.split("/").at(-1)}>Download reviewed public session export</a> : <button disabled={busy} onClick={() => void loadSession()}>Load reviewed public session export</button>}</p>}
    {snapshot && <p>Published source <code>{snapshot.source_hash}</code> · tested revision <code>{snapshot.verification?.revision}</code> · result {snapshot.verification?.status ?? "not_run"}. <a href={`./snapshot.html?src=${encodeURIComponent(artifact)}`}>Inspect this exact result, including failures and stale evidence</a></p>}
    {snapshot?.demonstration && <><p>Recording build: <code>{snapshot.demonstration.captured_revision}</code>. Publication and recording revisions are reported separately.</p><ul>{snapshot.demonstration.cuts?.map((note, index) => <li key={index}>{note}</li>)}</ul></>}
    <p>These deterministic checks establish budget allocation, not real network latency or OS scheduling. A recorded browser proposal is not a live provider request.</p>
  </section>;
}
