import { useEffect, useMemo, useState } from "react";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { EditingSurface } from "../embed/EditingSurface";
import { WsRealtime } from "../api/realtime";
import type { Project, DocSummary, Doc, RenderResponse, Run } from "../api/types";
import { action, desktop, DraftWriter, NativeSocket, native, type Draft, type Initial } from "./bridge";

const blank = (): Draft => ({ id: crypto.randomUUID(), title: "Untitled", source: "", revision: 0, remote: null, y_state: null });
export function MobileApp() {
  const [initial, setInitial] = useState<Initial | null>(null);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const [machine, setMachine] = useState("");
  const [connected, setConnected] = useState("");
  const [reach, setReach] = useState("default");
  const [address, setAddress] = useState("");
  const [phrase, setPhrase] = useState("");
  const [invitation, setInvitation] = useState("");
  const [projects, setProjects] = useState<Project[]>([]);
  const [docs, setDocs] = useState<DocSummary[]>([]);
  async function refresh() { setInitial(await action<Initial>("init")); }
  useEffect(() => { void refresh().catch(error => setNotice(String(error))); }, []);
  async function perform(work: () => Promise<void>) {
    setBusy(true); setNotice("");
    try { await work(); } catch (error) { setNotice(String(error)); } finally { setBusy(false); }
  }
  if (draft) return <DraftEditor key={draft.id} initial={draft} connected={connected} onBack={async () => { await refresh(); setDraft(null); }} />;
  return <main className="mobile-app">
    <header><h1>Hickory Docs</h1><p>{native() ? "Documents on this phone" : "Browser preview · drafts stay in this browser"}</p></header>
    {notice && <p role="alert">{notice}</p>}
    <section aria-labelledby="documents"><h2 id="documents">Your documents</h2>
      <button disabled={busy || !initial} onClick={() => setDraft(blank())}>New document</button>
      {initial?.drafts.map(item => <button className="document-item" key={item.id} onClick={() => setDraft(item)}>{item.title || "Untitled"}<small>{item.remote ? `Recovery copy · ${item.remote.machine}` : "Saved locally"}</small></button>)}
    </section>
    <details><summary>Connect a desktop</summary>
      <p>Use the desktop’s fleet pairing and peer session. Execution remains off until granted on that desktop.</p>
      <label>Reachability<select value={reach} onChange={event => setReach(event.target.value)}><option value="default">Direct with relay fallback</option><option value="direct">Direct only</option></select></label>
      <p>{reach === "default" ? "Addresses are published to number0 discovery; traffic may use its encrypted relays." : "No relay or public address publication. Some networks will be unreachable."}</p>
      <label>Or your HTTPS relay<input placeholder="https://relay.example" onChange={event => { if (event.target.value) setReach(event.target.value); else setReach("default"); }} /></label>
      <label>Desktop pairing phrase<input value={phrase} onChange={event => setPhrase(event.target.value)} /></label>
      <button disabled={busy || !native() || !phrase.trim()} onClick={() => void perform(async () => { await action("pair", { phrase, reach }); await refresh(); setNotice("Paired. Choose the desktop and paste its session address."); })}>Pair</button>
      <details><summary>Pair using invitations instead</summary><p>Exchange invitations in both directions. This invitation contains only the phone’s public identity.</p><textarea aria-label="Phone invitation" readOnly value={initial?.invitation ?? ""} /><label>Desktop invitation<textarea value={invitation} onChange={event => setInvitation(event.target.value)} /></label><button disabled={busy || !native() || !invitation} onClick={() => void perform(async () => { await action("accept", { invitation }); await refresh(); setInvitation(""); })}>Accept desktop invitation</button></details>
      <label>Desktop<select value={machine} onChange={event => setMachine(event.target.value)}><option value="">Choose a paired desktop</option>{initial?.machines.map(item => <option key={item.public_key} value={item.name}>{item.name}</option>)}</select></label>
      <label>Session address JSON<textarea value={address} onChange={event => setAddress(event.target.value)} placeholder="Paste the address printed by the desktop peer session" /></label>
      <button disabled={busy || !native() || !machine || !address} onClick={() => void perform(async () => {
        const result = await action<{ reachability: string }>("connect", { machine, address, reach });
        setConnected(machine); setDocs([]); setProjects(await desktop<Project[]>("/api/projects"));
        setNotice(`Connected to ${machine}: ${result.reachability}. This is live access, not repository synchronization.`);
      })}>Connect</button>
      {connected && <button disabled={busy} onClick={() => void perform(async () => { await action("disconnect"); setConnected(""); setProjects([]); setDocs([]); })}>Disconnect {connected}</button>}
    </details>
    {connected && <section><h2>On {connected}</h2>{projects.map(project => <button disabled={busy} key={project.id} onClick={() => void perform(async () => setDocs(await desktop<DocSummary[]>(`/api/projects/${encodeURIComponent(project.id)}/docs`)))}>{project.name}</button>)}
      {docs.map(doc => <button className="document-item" disabled={busy} key={doc.id} onClick={() => void perform(async () => {
        const existing = initial?.drafts.find(d => d.remote?.machine === connected && d.remote.doc_id === doc.id);
        if (existing) { setDraft(existing); return; }
        const value = await desktop<Doc>(`/api/docs/${encodeURIComponent(doc.id)}`);
        setDraft({ ...blank(), title: value.path, source: value.source, remote: { machine: connected, doc_id: doc.id } });
      })}>{doc.path}</button>)}
    </section>}
  </main>;
}
function DraftEditor({ initial, connected, onBack }: { initial: Draft; connected: string; onBack: () => Promise<void> }) {
  const [title, setTitle] = useState(initial.title);
  const [source, setSource] = useState(initial.source);
  const [status, setStatus] = useState(initial.revision ? "Recovery copy available locally" : "Not saved yet");
  const [notice, setNotice] = useState("");
  const [synced, setSynced] = useState(false);
  const [working, setWorking] = useState(false);
  const [rendered, setRendered] = useState<RenderResponse | null>(initial.outputs?.render ?? null);
  const [runId, setRunId] = useState("");
  const session = useMemo(() => {
    const doc = new Y.Doc();
    if (initial.y_state) Y.applyUpdate(doc, new Uint8Array(initial.y_state));
    const awareness = new Awareness(doc);
    return { doc, awareness, collab: { ytext: doc.getText("source"), awareness }, writer: new DraftWriter(initial), latest: initial, serial: 0, realtime: null as WsRealtime | null };
  }, [initial]);
  const remote = initial.remote;
  const available = !!remote && connected === remote.machine;
  const writable = !remote || !!initial.y_state || synced;
  function persist(next: Draft) {
    session.latest = next;
    const serial = ++session.serial;
    setStatus("Saving on this device…");
    void session.writer.save(next).then(saved => {
      session.latest = { ...session.latest, revision: saved.revision };
      if (serial === session.serial) setStatus(native() ? "Saved on this phone" : "Saved in this browser");
    }).catch(error => { if (serial === session.serial) setStatus(`Not saved: ${String(error)}`); });
  }
  useEffect(() => {
    let active = true;
    const changed = () => {
      const next = session.doc.getText("source").toString();
      setSource(next);
      persist({ ...session.latest, source: next, y_state: Array.from(Y.encodeStateAsUpdate(session.doc)) });
    };
    if (remote) session.doc.on("update", changed);
    let realtime: WsRealtime | undefined;
    if (available && remote) {
      realtime = new WsRealtime(`doc:${remote.doc_id}`, url => new NativeSocket(url, (message, online) => { if (active) { setNotice(message); setSynced(false); if (online) void realtime?.whenSynced().then(() => { if (active) setSynced(true); }); } }));
      session.realtime = realtime;
      realtime.bindDoc(session.doc, session.awareness);
      void realtime.whenSynced().then(() => { if (active) setSynced(true); });
    }
    const visibility = () => {
      if (document.hidden) { realtime?.close(); setSynced(false); }
      else { realtime?.reopen(); if (realtime) void realtime.whenSynced().then(() => { if (active) setSynced(true); }); }
    };
    document.addEventListener("visibilitychange", visibility);
    return () => { active = false; document.removeEventListener("visibilitychange", visibility); realtime?.close(); session.doc.off("update", changed); session.awareness.destroy(); session.doc.destroy(); };
    // One editor lifetime owns one CRDT and one desktop target.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [session]);
  async function leave() {
    setWorking(true);
    try { session.realtime?.close(); await session.writer.flush(); await onBack(); } catch (error) { session.realtime?.reopen(); setNotice(`Keep this document open: ${String(error)}`); } finally { setWorking(false); }
  }
  async function review() {
    if (!remote) return;
    setWorking(true);
    try { const render = await desktop<RenderResponse>(`/api/docs/${encodeURIComponent(remote.doc_id)}/render`); setRendered(render); persist({ ...session.latest, outputs: { captured_at: new Date().toISOString(), render } }); setNotice("Snapshot retrieved from desktop. Refresh after edits or runs; this view may become stale."); }
    catch (error) { setNotice(String(error)); } finally { setWorking(false); }
  }
  async function run() {
    if (!remote || !synced) return;
    setWorking(true);
    try {
      const bytes = new TextEncoder().encode(session.doc.getText("source").toString());
      const hash = Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))).map(n => n.toString(16).padStart(2,"0")).join("");
      const result = await desktop<{run_id: string}>(`/api/docs/${encodeURIComponent(remote.doc_id)}/run`, "POST", { expected_source_hash: hash });
      setRunId(result.run_id); setNotice(`Run ${result.run_id} accepted on ${remote.machine}. Refresh its status to inspect completion.`);
    } catch (error) { setNotice(`Run was not confirmed: ${String(error)}. It may have started; refresh desktop state before retrying.`); }
    finally { setWorking(false); }
  }
  return <main className="mobile-app mobile-document">
    <header><button disabled={working} onClick={() => void leave()}>Documents</button><label>Document title<input value={title} onChange={event => { const value = event.target.value; setTitle(value); persist({ ...session.latest, title: value }); }} /></label><p role="status">{status}</p><p>{remote ? `Desktop: ${remote.machine} · ${available ? "Live session available" : "Offline recovery copy"}` : "Local document"}</p></header>
    {notice && <p role="alert">{notice}</p>}
    {!writable && <p>Connect to this desktop once to initialize the live editing room. The fetched text below is a read-only snapshot.</p>}
    {remote && !writable ? <pre className="mobile-snapshot">{initial.source}</pre> : <EditingSurface ariaLabel="Document source" hick value={source} collab={remote ? session.collab : undefined} readOnly={!writable || working} onChange={remote ? undefined : value => { setSource(value); persist({ ...session.latest, source: value }); }} />}
    <footer>
      <button disabled={working} onClick={() => persist(session.latest)}>Save on this device</button>
      <button disabled={working || !available} onClick={() => void review()}>Refresh desktop outputs</button>
      <button disabled={working || !available || !synced} onClick={() => void run()}>Run on {remote?.machine ?? "desktop"}</button>
      <p>{remote ? "Live edits use the desktop’s edit grant. Running also needs its execute grant; pairing leaves it off." : "Process execution, terminals, language servers and debugging require a connected desktop."}</p>
      {runId && <button disabled={working || !available} onClick={() => { setWorking(true); void desktop<Run>(`/api/runs/${encodeURIComponent(runId)}`).then(result => {
        setNotice(`${remote?.machine}: ${result.status}${result.error ? ` — ${result.error}` : ""}`);
        if (result.blocks.length) {
          const render = { blocks: result.blocks }; setRendered(render);
          persist({ ...session.latest, outputs: { captured_at: new Date().toISOString(), render } });
        }
      }).catch(error => setNotice(String(error))).finally(() => setWorking(false)); }}>Refresh run status</button>}
    </footer>
    {rendered && <section aria-label="Desktop output snapshot"><h2>Desktop output snapshot</h2><p>Cached snapshot. The save status above applies to this recovery copy. It may be stale; connect and refresh to check current state.</p>{rendered.blocks.filter(block => block.kind === "exec" || block.kind === "file").map((block, index) => <pre key={index}>{block.kind === "exec" ? `${block.id}: ${block.status ?? "unrecorded"}
${block.transcript?.map(event => "data" in event ? event.data : `exit ${event.code}`).join("\n") ?? "No recorded output"}` : block.kind === "file" ? `${block.path}
${block.body}` : ""}</pre>)}</section>}
  </main>;
}
