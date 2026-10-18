import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { useEffect, useState } from "react";
import { request } from "../api/client";
import { openLocation } from "../lib/revealLine";
import { REVIEW_TARGET, REVIEW_ERROR, reviewSelection, investigationPrompt, type ReviewEvent, type ReviewTarget } from "../lib/reviews";
import "./ReviewPanel.css";
interface History { scope?: string; events: ReviewEvent[]; targets: { thread: string; current_state: string }[] }
export function ReviewPanel() {
  const [shown, setShown] = useState(false);
  const [target, setTarget] = useState<ReviewTarget | null>(null);
  const [history, setHistory] = useState<History>({ events: [], targets: [] });
  const [thread, setThread] = useState<string | null>(null);
  const [body, setBody] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [prompt, setPrompt] = useState("");
  const load = async () => setHistory(await request<History>("GET", "/api/reviews"));
  useEffect(() => {
    const open = (event: Event) => { setTarget((event as CustomEvent<ReviewTarget>).detail); setThread(null); setBody(""); setPrompt(""); setShown(true); };
    const failure = (event: Event) => { setError((event as CustomEvent<string>).detail); setShown(true); };
    window.addEventListener(REVIEW_ERROR, failure);
    window.addEventListener(REVIEW_TARGET, open);
    return () => { window.removeEventListener(REVIEW_TARGET, open); window.removeEventListener(REVIEW_ERROR, failure); };
  }, []);
  useEffect(() => {
    if (!shown) return;
    const refresh = () => { void load().catch(e => setError(String(e))); };
    refresh();
    window.addEventListener(FILES_CHANGED_EVENT, refresh);
    return () => window.removeEventListener(FILES_CHANGED_EVENT, refresh);
  }, [shown, target?.source_hash, target?.path]);
  const entries = history.events.filter(event => event.thread === thread);
  const root = entries.find(event => event.target)?.target ?? target;
  const resolved = [...entries].reverse().find(event => event.action === "resolve" || event.action === "reopen")?.action === "resolve";
  const save = async (action: ReviewEvent["action"]) => {
    setBusy(true); setError("");
    try {
      const event = await request<ReviewEvent>("POST", "/api/reviews", { scope: history.scope, thread, target: thread ? null : target, action, body });
      setThread(event.thread); setTarget(null); setBody(""); await load();
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  };
  return <div className="review-tools">
    <button className="btn-link" onMouseDown={event => event.preventDefault()} onClick={() => void reviewSelection().catch(e => { setError(String(e)); setShown(true); })}>Comment on selection</button>
    <button className="btn-link" onClick={() => setShown(!shown)}>Review threads</button>
    {shown && <section className="review-panel" role="region" aria-label="Review threads">
      <header><strong>Review threads</strong><button onClick={() => setShown(false)} aria-label="Close review threads">Close</button></header>
      {error && <p role="alert">{error}</p>}
      <nav aria-label="Saved review threads">{history.events.filter(event => event.target).map(event => <button key={event.id} onClick={() => { setThread(event.thread); setTarget(null); setPrompt(""); }}>
        {event.target!.path}: {event.body.slice(0, 70)}
      </button>)}</nav>
      {root ? <>
        <p>{root.kind} · <button className="btn-link" onClick={() => openLocation(root.path, root.lines[0])}>{root.path}:{root.lines.join("–")}</button></p>
        <blockquote>{root.quote}</blockquote>
        {root.relationship !== undefined && <details><summary>Questioned relationship</summary><pre>{JSON.stringify(root.relationship, null, 2)}</pre></details>}
        <details><summary>Captured source revision</summary><code>{root.source_hash}</code></details>
        <p>Saved source: {history.targets.find(item => item.thread === thread)?.current_state ?? "captured editor text; may include unsaved changes"}. {resolved ? "Resolved by user" : "Open"}.</p>
        {entries.map(event => <article key={event.id}><small>{event.actor} · {event.action} · {new Date(Number(event.recorded_at) / (event.recorded_at.length > 13 ? 1e6 : 1)).toLocaleString()}</small><p>{event.body}</p></article>)}
        <form onSubmit={event => { event.preventDefault(); void save("comment"); }}>
          <label>Comment or reason<textarea required value={body} onChange={event => setBody(event.target.value)} /></label>
          <button disabled={busy || !body.trim()}>Save comment</button>
          {thread && <button type="button" disabled={busy || !body.trim()} onClick={() => void save(resolved ? "reopen" : "resolve")}>{resolved ? "Reopen" : "Resolve after review"}</button>}
        </form>
        {thread && <button disabled={busy} onClick={() => {
          const text = investigationPrompt(root, entries); setPrompt(text);
        }}>Prepare agent investigation</button>}
        {prompt && <label>Investigation prompt · review and send in the agent pane<textarea readOnly value={prompt} /><button onClick={() => void navigator.clipboard.writeText(prompt).catch(e => setError(String(e)))}>Copy prompt</button></label>}
      </> : <p>Select a claim and choose Comment on selection, or right-click a ribbon or brace. Use Alt+Enter on a focused connection for keyboard access.</p>}
      <p className="muted">Stored locally with this workspace. Comments are declarations; passing checks do not resolve them. Original targets and earlier comments remain.</p>
    </section>}
  </div>;
}
