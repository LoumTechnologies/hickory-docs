import { GithubReviewThreads } from "./GithubReviewThreads";
import type { EditorView } from "@codemirror/view";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { useEffect, useState, useSyncExternalStore } from "react";
import { investigationConversations, sendInvestigation } from "../lib/reviewInvestigations";
import { request } from "../api/client";
import { openLocation } from "../lib/revealLine";
import { REVIEW_TARGET, REVIEW_ERROR, reviewSelection, investigationPrompt, type ReviewEvent, type ReviewTarget } from "../lib/reviews";
import "./ReviewPanel.css";
interface Investigation { event: string; thread: string; link: { id: string; conversation: string; turn?: string }; state: string; response?: string; session?: string; error?: string }
interface History { scope?: string; events: ReviewEvent[]; targets: { thread: string; current_state: string }[]; investigations?: Investigation[] }
export function ReviewPanel({ document, source, view }: { document?: string; source?: string; view?: EditorView | null } = {}) {
  const [shown, setShown] = useState(false);
  const [target, setTarget] = useState<ReviewTarget | null>(null);
  const [history, setHistory] = useState<History>({ events: [], targets: [] });
  const [thread, setThread] = useState<string | null>(null);
  const [body, setBody] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [prompt, setPrompt] = useState("");
  const conversations = useSyncExternalStore(investigationConversations.subscribe, investigationConversations.snapshot);
  const [conversation, setConversation] = useState("");
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
  const investigations = (history.investigations ?? []).filter((item, index, all) => item.thread === thread &&
    !all.slice(index + 1).some(later => later.link.id === item.link.id));
  const pending = investigations.some(item => item.state === "queued" || item.state === "running");
  useEffect(() => {
    if (!shown || !pending) return;
    const timer = setInterval(() => void load().catch(e => setError(String(e))), 1000);
    return () => clearInterval(timer);
  }, [shown, pending]);
  const investigate = async () => {
    if (!thread || !root || !conversation || busy) return;
    setBusy(true); setError("");
    const scope = history.scope;
    const selectedThread = thread;
    const id = crypto.randomUUID();
    const record = async (state: string, turn?: string, error?: string) => request("POST", "/api/reviews", {
      scope, thread: selectedThread, target: null, action: "investigate",
      body: JSON.stringify({ version: 1, id, conversation, state, turn, error }),
    });
    try {
      await record("queued");
      const comparison = history.targets.find(item => item.thread === selectedThread)?.current_state ?? "unavailable";
      const text = investigationPrompt(root, entries) + `\n\nCurrent saved-source comparison: ${comparison}. Historical quote remains authoritative evidence of the original objection; reread current bytes before editing.\nReview thread: ${selectedThread}; investigation: ${id}`;
      const turn = await sendInvestigation(conversation, text);
      await record("running", turn);
      await load();
    } catch (e) {
      setError(String(e));
      await record("failed", undefined, String(e)).catch(() => undefined);
      await load().catch(() => undefined);
    } finally { setBusy(false); }
  };
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
        {thread && <section aria-label="Agent investigations">
          <label>Investigation conversation <select value={conversation} disabled={busy} onChange={event => setConversation(event.target.value)}>
            <option value="">Choose an open agent conversation</option>
            {conversations.map(item => <option key={item.id} value={item.id}>{item.label}</option>)}
          </select></label>
          <button disabled={busy || !conversations.some(item => item.id === conversation)} onClick={() => void investigate()}>Send investigation to selected conversation</button>
          {investigations.map(item => <article key={item.event}>
            <p>Investigation {item.state} · {item.link.conversation} · turn {item.link.turn ?? "not yet accepted"}</p>
            {item.response && <blockquote>{item.response}</blockquote>}
            {item.error && <p>{item.error}</p>}
            {item.session && <button onClick={() => openLocation(item.session!, 1)}>Open response, proposed edits and executed check receipts</button>}
            <p>Edits use the agent pane’s existing approval flow. Model prose is not a check receipt. Human review remains {resolved ? "resolved" : "open"}.</p>
          </article>)}
        </section>}
      </> : <p>Select a claim and choose Comment on selection, or right-click a ribbon or brace. Use Alt+Enter on a focused connection for keyboard access.</p>}
      <GithubReviewThreads document={document} source={source} view={view} />
      <p className="muted">Stored locally with this workspace. Comments are declarations; passing checks do not resolve them. Original targets and earlier comments remain.</p>
    </section>}
  </div>;
}
