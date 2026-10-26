<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ReviewPanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-da8b14773733"></a>
## `apps/web/src/components/ReviewPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Review threads preserve the questioned target](../../../../../guarantees/authoring/review-threads-preserve-the-questioned-target/contract.md).

<a id="f-da8b14773733-Investigation"></a>
### `Investigation`

<storycode:copy id="f-da8b14773733-Investigation">
import { GithubReviewThreads } from "./GithubReviewThreads";
import type { EditorView } from "@codemirror/view";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { useEffect, useId, useState, useSyncExternalStore } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";
import { investigationConversations, sendInvestigation } from "../lib/reviewInvestigations";
import { request } from "../api/client";
import { openLocation } from "../lib/revealLine";
import { REVIEW_TARGET, REVIEW_ERROR, reviewSelection, investigationPrompt, type ReviewEvent, type ReviewTarget } from "../lib/reviews";
import "./ReviewPanel.css";
interface Investigation { event: string; thread: string; link: { id: string; conversation: string; turn?: string }; state: string; response?: string; session?: string; error?: string }
</storycode:copy>
<a id="f-da8b14773733-History"></a>
### `History`

<storycode:copy id="f-da8b14773733-History">
interface History { scope?: string; events: ReviewEvent[]; targets: { thread: string; current_state: string }[]; investigations?: Investigation[] }
</storycode:copy>
<a id="f-da8b14773733-ReviewPanel"></a>
### `ReviewPanel`

<storycode:copy id="f-da8b14773733-ReviewPanel">
export function ReviewPanel({ document, source, view }: { document?: string; source?: string; view?: EditorView | null } = {}) {
  const instance = useId();
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
  const investigate = async (agent = false) => {
    if (!thread || !root || !conversation || busy) return { error: "Select a thread and conversation; wait for the current operation" };
    setBusy(true); setError("");
    const scope = history.scope;
    const selectedThread = thread;
    const id = crypto.randomUUID();
    const record = async (state: string, turn?: string, error?: string) => request("POST", "/api/reviews", {
      scope, thread: selectedThread, target: null, action: "investigate", ...(agent ? { agent: true } : {}),
      body: JSON.stringify({ version: 1, id, conversation, state, turn, error }),
    });
    try {
      await record("queued");
      const comparison = history.targets.find(item => item.thread === selectedThread)?.current_state ?? "unavailable";
      const text = investigationPrompt(root, entries) + `\n\nCurrent saved-source comparison: ${comparison}. Historical quote remains authoritative evidence of the original objection; reread current bytes before editing.\nReview thread: ${selectedThread}; investigation: ${id}`;
      const turn = await sendInvestigation(conversation, text);
      await record("running", turn);
      await load();
      return { turn };
    } catch (e) {
      setError(String(e));
      await record("failed", undefined, String(e)).catch(() => undefined);
      await load().catch(() => undefined);
      return { error: String(e) };
    } finally { setBusy(false); }
  };
  const save = async (action: ReviewEvent["action"], agent = false) => {
    setBusy(true); setError("");
    try {
      const event = await request<ReviewEvent>("POST", "/api/reviews", { scope: history.scope, thread, target: thread ? null : target, action, body, ...(agent ? { agent: true } : {}) });
      setThread(event.thread); setTarget(null); setBody(""); await load();
      return { event };
    } catch (e) { setError(String(e)); return { error: String(e) }; } finally { setBusy(false); }
  };
  const copyPrompt = async (): Promise<import("../lib/uiActions").UiOutcome> => {
    if (!prompt) return { status: "unavailable", reason: "Prepare the investigation prompt first" };
    setError("");
    try {
      if (!navigator.clipboard?.writeText) throw Error("Clipboard is unavailable");
      await navigator.clipboard.writeText(prompt);
      return { status: "completed", value: { copied: true, investigation_sent: false } };
    } catch (cause) {
      const reason = `${String(cause)}. Browser clipboard access may require a human gesture; the prepared prompt remains readable in the review draft.`;
      setError(reason);
      return { status: "requires_input", request: "review-prompt-clipboard", reason };
    }
  };
  const parent = `reviews:${instance}`;
  const revision = useUiRevision(() => [shown, target, history, thread, body, error, busy, prompt, conversation, conversations]);
  const commands: { key: string; command: UiCommand }[] = [];
  const done = (value: UiValue = null) => ({ status: "completed" as const, value });
  function register(id: string, label: string, execute: UiCommand["execute"], parameters: Record<string, UiField> = {}, enabled = true, reason?: string) {
    commands.push({ key: `${parent}/${id}`, command: { label, parent: id === "disclosure" ? "workspace-reviews" : parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled, reason }), execute } });
  }
  register("disclosure", "Set review threads visibility", async args => { setShown(Boolean(args.open)); return done({ open: Boolean(args.open) }); }, { open: { type: "boolean" } });
  if (shown) {
    register("read", "Read review threads and selected target", async args => {
      const collections = { threads: { scope: history.scope, targets: history.targets, roots: history.events.filter(event => event.target).map(event => ({ id: event.id, thread: event.thread, path: event.target?.path, actor: event.actor })) }, target: root, events: entries, investigations, conversations: { selected: conversation, available: conversations }, draft: { body, prompt, thread, resolved, busy, error } };
      const collection = String(args.collection) as keyof typeof collections;
      const bytes = new TextEncoder().encode(JSON.stringify(collections[collection] ?? null));
      const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds this review collection; observe current thread state" };
      return done({ collection, encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], total_bytes: bytes.length, next: end < bytes.length ? end : null });
    }, { collection: { type: "string", maxLength: 32, choices: ["threads", "target", "events", "investigations", "conversations", "draft"] }, offset: { type: "number", minimum: 0, maximum: 100000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } });
    register("refresh", "Refresh review threads", async () => { await load(); return done({ dispatch: "loaded", result: "Observe current review state" }); });
    register("select", "Select saved review thread", async args => {
      const id = String(args.thread);
      if (!history.events.some(event => event.thread === id && event.target)) return { status: "unavailable", reason: "Thread is not in the observed workspace history" };
      setThread(id); setTarget(null); setPrompt(""); return done();
    }, { thread: { type: "string", maxLength: 256 } });
    register("comment-text", "Set review comment or reason", async args => { setBody(String(args.text)); return done(); }, { text: { type: "string", maxLength: 32768 } }, !busy, "Wait for the current review operation");
    if (root) {
      register("source", "Open selected review source", async () => { openLocation(root.path, root.lines[0]); return done(); });
      register("prepare", "Prepare selected review investigation", async () => { const text = investigationPrompt(root, entries); setPrompt(text); return done({ text, sent: false }); }, {}, !!thread, "Save the original review thread first");
      if (prompt) register("copy-prompt", "Copy prepared investigation prompt", copyPrompt, {}, !busy, "Wait for the current review operation");
      register("conversation", "Select investigation conversation", async args => {
        const id = String(args.conversation);
        if (!conversations.some(item => item.id === id)) return { status: "unavailable", reason: "Choose a currently open conversation" };
        setConversation(id); return done();
      }, { conversation: { type: "string", maxLength: 256 } }, !busy);
      register("investigate", "Send investigation to selected conversation", async (_args, actor) => {
        const result = await investigate(actor.kind === "agent");
        return result.turn ? { status: "queued", job: result.turn, value: { conversation, thread, result: "Observe the actual investigation state; model response is not verification" } }
          : { status: "unavailable", reason: result.error ?? "Investigation was not accepted" };
      }, {}, !!thread && !busy && conversations.some(item => item.id === conversation), "Select a saved thread and open conversation; this makes a real provider request");
      register("save", "Save review comment", async (_args, actor) => {
        const result = await save("comment", actor.kind === "agent");
        return result.event ? done(JSON.parse(JSON.stringify(result.event))) : { status: "unavailable", reason: result.error ?? "Comment was not saved" };
      }, {}, !busy && !!body.trim(), "Enter a comment and wait for any current operation");
      register("resolve", resolved ? "Reopen reviewed thread" : "Resolve after human review", async (_args, actor) => {
        if (actor.kind === "agent") return { status: "requires_input", reason: "Resolution is a human judgment. Use the actual review form and inspect its retained resolve/reopen event" };
        const result = await save(resolved ? "reopen" : "resolve");
        return result.event ? done(JSON.parse(JSON.stringify(result.event))) : { status: "unavailable", reason: result.error ?? "Review action was not saved" };
      }, {}, !!thread && !busy && !!body.trim(), "Select a saved thread and enter the review reason");
    }
  }
  const invoke = useUiCommands(commands);
  return <div className="review-tools">
    <button className="btn-link" onMouseDown={event => event.preventDefault()} onClick={() => void reviewSelection().catch(e => { setError(String(e)); setShown(true); })}>Comment on selection</button>
    <button className="btn-link" onClick={() => setShown(!shown)}>Review threads</button>
    {shown && <section className="review-panel" role="region" aria-label="Review threads" data-coach-target="review">
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
        {prompt && <label>Investigation prompt · review and send in the agent pane<textarea readOnly value={prompt} /><button disabled={busy} onClick={() => void invoke(`${parent}/copy-prompt`).catch(e => setError(String(e)))}>Copy prompt</button></label>}
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
</storycode:copy>

### Assemble `apps/web/src/components/ReviewPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ReviewPanel.tsx"><storycode:paste select="#f-da8b14773733-Investigation"/><storycode:paste select="#f-da8b14773733-History"/><storycode:paste select="#f-da8b14773733-ReviewPanel"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-d6bd48d3bc5c"></a>
## `apps/web/src/components/ReviewPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Review investigations link observed agent work without resolving human review](../../../../../guarantees/authoring/review-investigations-link-observed-agent-work/contract.md), [Review threads preserve the questioned target](../../../../../guarantees/authoring/review-threads-preserve-the-questioned-target/contract.md).

<a id="f-d6bd48d3bc5c-request"></a>
### `request`

<storycode:copy id="f-d6bd48d3bc5c-request">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { afterEach, expect, it, vi } from "vitest";
const request = vi.hoisted(() => vi.fn());
</storycode:copy>
<a id="f-d6bd48d3bc5c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d6bd48d3bc5c-expression_statement">
vi.mock("../api/client", () => ({ request }));
</storycode:copy>
<a id="f-d6bd48d3bc5c-import_statement"></a>
### `import_statement`

<storycode:copy id="f-d6bd48d3bc5c-import_statement">
import { ReviewPanel } from "./ReviewPanel";
</storycode:copy>
<a id="f-d6bd48d3bc5c-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-d6bd48d3bc5c-import_statement-2">
import { openReview } from "../lib/reviews";
</storycode:copy>
<a id="f-d6bd48d3bc5c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d6bd48d3bc5c-expression_statement-2">
afterEach(() => { cleanup(); request.mockReset(); });
</storycode:copy>
<a id="f-d6bd48d3bc5c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d6bd48d3bc5c-expression_statement-3">
// Guarantee: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
it("attributes an agent comment without granting human resolution", async () => {
  const target = { kind: "claim", path: "spec.md", quote: "Claim", source_hash: "a".repeat(64), lines: [1, 1] };
  const original = { id: "original", thread: "thread", target, action: "comment", body: "Human criterion", actor: "user-declared", recorded_at: "1" };
  request.mockImplementation(async (method: string) => method === "GET" ? { scope: "workspace", events: [original], targets: [] } : { id: "agent-comment", thread: "thread", actor: "agent-declared" });
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><ReviewPanel /></UiActionContext.Provider>);
  fireEvent.click(screen.getByRole("button", { name: "Review threads" }));
  fireEvent.click(await screen.findByRole("button", { name: "spec.md: Human criterion" }));
  let n = 0;
  const invoke = async (label: string, arguments_ = {}) => {
    const parent = registry.groups().groups.find(group => group.startsWith("reviews:"))!;
    const action = registry.observe(parent).actions.find(action => action.label === label)!;
    let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: arguments_, request_id: String(++n) }); }); return result;
  };
  await invoke("Set review comment or reason", { text: "Actual agent finding" });
  expect(await invoke("Resolve after human review")).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(await invoke("Save review comment")).toMatchObject({ status: "result", outcome: { value: { id: "agent-comment", actor: "agent-declared" } } });
  expect(request).toHaveBeenCalledWith("POST", "/api/reviews", { scope: "workspace", thread: "thread", target: null, action: "comment", body: "Actual agent finding", agent: true });
  expect(request.mock.calls.some(call => call[0] === "POST" && call[2].action === "resolve")).toBe(false);
});
</storycode:copy>
<a id="f-d6bd48d3bc5c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-d6bd48d3bc5c-expression_statement-4">
// Guarantee: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
it("posts the captured target and workspace scope and retains its quotation", async () => {
  request.mockImplementation(async (method: string) => method === "GET" ? { scope:"workspace",events:[],targets:[] } : {thread:"review-1"});
  render(<ReviewPanel />);
  const target={kind:"claim",path:"spec.md",quote:"The whole operation is bounded",source_hash:"a".repeat(64),lines:[1,1] as [number,number]};
  openReview(target);
  await screen.findByText(target.quote);
  await waitFor(() => expect(request).toHaveBeenCalledWith("GET","/api/reviews"));
  fireEvent.change(screen.getByLabelText("Comment or reason"),{target:{value:"Only each attempt is bounded"}});
  fireEvent.click(screen.getByRole("button",{name:"Save comment"}));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST","/api/reviews",{scope:"workspace",thread:null,target,action:"comment",body:"Only each attempt is bounded"}));
});

</storycode:copy>
<a id="f-d6bd48d3bc5c-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-d6bd48d3bc5c-expression_statement-5">
// Guarantee: docs/guarantees/authoring/review-investigations-link-observed-agent-work.md
it("dispatches only to the explicitly selected conversation and keeps human review open", async () => {
  const { registerInvestigationConversation } = await import("../lib/reviewInvestigations");
  const first = vi.fn(async (_prompt: string) => "turn-a");
  const second = vi.fn(async (_prompt: string) => "turn-b");
  const unregisterA = registerInvestigationConversation({ id: "conversation-a", label: "Conversation A", send: first });
  const unregisterB = registerInvestigationConversation({ id: "conversation-b", label: "Conversation B", send: second });
  const target = { kind: "claim", path: "spec.md", quote: "Total deadline", source_hash: "a".repeat(64), lines: [1, 1] };
  const events = [{ id: "original", thread: "thread-1", target, action: "comment", body: "Only each attempt is bounded", actor: "user-declared", recorded_at: "1" }];
  request.mockImplementation(async (method: string) => method === "GET" ? {
    scope: "workspace", events, targets: [{ thread: "thread-1", current_state: "ambiguous" }], investigations: [],
  } : { thread: "thread-1" });
  try {
    render(<ReviewPanel />);
    fireEvent.click(screen.getByRole("button", { name: "Review threads" }));
    fireEvent.click(await screen.findByRole("button", { name: /spec.md: Only each attempt/ }));
    expect(first).not.toHaveBeenCalled(); expect(second).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Investigation conversation"), { target: { value: "conversation-b" } });
    fireEvent.click(screen.getByRole("button", { name: "Send investigation to selected conversation" }));
    await waitFor(() => expect(second).toHaveBeenCalledTimes(1));
    expect(first).not.toHaveBeenCalled();
    expect(second.mock.calls[0]?.[0]).toContain("ambiguous");
    expect(second.mock.calls[0]?.[0]).toContain("Only each attempt is bounded");
    await waitFor(() => expect(request.mock.calls.some((call: unknown[]) => call[0] === "POST" &&
      String((call[2] as { body: string }).body).includes('"turn":"turn-b"'))).toBe(true));
    expect(request.mock.calls.some((call: unknown[]) => call[0] === "POST" &&
      (call[2] as { action: string }).action === "resolve")).toBe(false);
  } finally { unregisterA(); unregisterB(); }
});

</storycode:copy>
<a id="f-d6bd48d3bc5c-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-d6bd48d3bc5c-expression_statement-6">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares prepared-prompt copying and retains an explicit browser-permission request", async () => {
  const originalClipboard = Object.getOwnPropertyDescriptor(navigator, "clipboard");
  const copy = vi.fn().mockRejectedValue(new Error("Permission denied"));
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: copy } });
  try {
    const target = { kind: "claim", path: "spec.md", quote: "Claim", source_hash: "a".repeat(64), lines: [1, 1] };
    request.mockResolvedValue({ scope: "workspace", events: [{ id: "original", thread: "thread", target, action: "comment", body: "Human criterion", actor: "user-declared", recorded_at: "1" }], targets: [] });
    const registry = new UiActions({ workspace: "practice", window: "copy-prompt" }, () => true, async () => {});
    render(<UiActionContext.Provider value={registry}><ReviewPanel /></UiActionContext.Provider>);
    fireEvent.click(screen.getByRole("button", { name: "Review threads" }));
    fireEvent.click(await screen.findByRole("button", { name: "spec.md: Human criterion" }));
    fireEvent.click(screen.getByRole("button", { name: "Prepare agent investigation" }));
    const parent = registry.groups().groups.find(group => group.startsWith("reviews:"))!;
    const command = registry.observe(parent, 64).actions.find(action => action.label === "Copy prepared investigation prompt")!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { kind: "agent", id: "predictor" }, { handle: command.handle, arguments: {}, request_id: "copy-requires-gesture" }); });
    expect(result).toMatchObject({ status: "result", outcome: { status: "requires_input", request: "review-prompt-clipboard" } });
    expect(screen.getByRole("alert").textContent).toContain("Permission denied");
    copy.mockResolvedValue(undefined);
    fireEvent.click(screen.getByRole("button", { name: "Copy prompt" }));
    await waitFor(() => expect(copy).toHaveBeenCalledTimes(2));
    expect(copy.mock.calls[0][0]).toBe(copy.mock.calls[1][0]);
    expect(copy.mock.calls[1][0]).toContain("Human criterion");
    expect(request.mock.calls.every(call => call[0] === "GET")).toBe(true);
  } finally {
    if (originalClipboard) Object.defineProperty(navigator, "clipboard", originalClipboard);
    else Reflect.deleteProperty(navigator, "clipboard");
  }
});
</storycode:copy>

### Assemble `apps/web/src/components/ReviewPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ReviewPanel.test.tsx"><storycode:paste select="#f-d6bd48d3bc5c-request"/><storycode:paste select="#f-d6bd48d3bc5c-expression_statement"/><storycode:paste select="#f-d6bd48d3bc5c-import_statement"/><storycode:paste select="#f-d6bd48d3bc5c-import_statement-2"/><storycode:paste select="#f-d6bd48d3bc5c-expression_statement-2"/><storycode:paste select="#f-d6bd48d3bc5c-expression_statement-3"/><storycode:paste select="#f-d6bd48d3bc5c-expression_statement-4"/><storycode:paste select="#f-d6bd48d3bc5c-expression_statement-5"/><storycode:paste select="#f-d6bd48d3bc5c-expression_statement-6"/></storycode:file>


</storycode:doc>
