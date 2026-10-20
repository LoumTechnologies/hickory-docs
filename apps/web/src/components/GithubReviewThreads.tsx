import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { useEffect, useId, useRef, useState } from "react";
import { StateEffect } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import { request } from "../api/client";
import { ProviderReading } from "../debug/ProviderReading";
import type { ProviderReading as Reading } from "../debug/providerObservations";
import { providerEditor, setProviderReadings } from "../debug/cmProvider";
interface Capture extends Reading { incomplete?: boolean; unavailable?: { thread: string; reason: string }[] }
export function GithubReviewThreads({ document, source, view }: { document?: string; source?: string; view?: EditorView | null }) {
  const [repository, setRepository] = useState(""), [number, setNumber] = useState("");
  const [path, setPath] = useState(document ?? ""), [capture, setCapture] = useState<Capture | null>(null);
  const [loading, setLoading] = useState(false), [error, setError] = useState("");
  const mounted = useRef(true), inFlight = useRef(false);
  const current = useRef({ repository, number, path, document, source }); current.current = { repository, number, path, document, source };
  const [capturedQuery, setCapturedQuery] = useState("");
  const query = (selection: typeof current.current) => JSON.stringify([selection.repository, selection.number, selection.path]);
  const setInput = (field: "repository" | "number" | "document", value: string) => {
    if (field === "repository") { current.current.repository = value; setRepository(value); }
    else if (field === "number") { current.current.number = value; setNumber(value); }
    else { current.current.path = value; setPath(value); }
  };
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  useEffect(() => { setInput("document", document ?? ""); }, [document]);
  const changed = !!capture && (capturedQuery !== query(current.current) || (document !== undefined && capture.source.uri !== document) || (source !== undefined && new TextDecoder().decode(new Uint8Array(capture.source.bytes ?? [])) !== source));
  useEffect(() => {
    if (!view) return;
    if (!view.state.field(providerEditor, false)) view.dispatch({ effects: StateEffect.appendConfig.of(providerEditor) });
    view.dispatch({ effects: setProviderReadings.of(capture && !changed && !error ? [capture] : []) });
    return () => { if (view.dom.isConnected) view.dispatch({ effects: setProviderReadings.of([]) }); };
  }, [view, capture, changed, error]);
  async function refresh(): Promise<UiOutcome> {
    const selection = { ...current.current };
    if (inFlight.current) return { status: "unavailable", reason: "GitHub review capture already pending" };
    if (!selection.repository || !/^[1-9][0-9]*$/.test(selection.number) || !selection.path) return { status: "unavailable", reason: "Select a repository, positive integer PR number and saved document" };
    inFlight.current = true; setLoading(true); setError("");
    try {
      const reply = await request<Capture>("GET", `/api/workspace/github/review-threads?${new URLSearchParams({ repository: selection.repository, number: selection.number, document: selection.path })}`);
      if (!mounted.current || query(current.current) !== query(selection) || current.current.source !== selection.source || current.current.document !== selection.document) {
        const reason = "Review selection or source changed while reading. The late capture was not attached; refresh the current selection.";
        if (mounted.current) setError(reason);
        return { status: "unavailable", reason };
      }
      setCapture(reply); setCapturedQuery(query(selection));
      return { status: "completed", value: { source: { id: reply.source.id, uri: reply.source.uri, revision: reply.source.revision }, annotations: reply.annotations.length, incomplete: !!reply.incomplete, unavailable: reply.unavailable ?? [], scope: "Actual read-only GitHub capture; no reply, resolution or publication sent" } };
    } catch (cause) { const reason = String(cause); if (mounted.current) setError(reason); return { status: "unavailable", reason }; }
    finally { inFlight.current = false; if (mounted.current) setLoading(false); }
  }
  const parent = `github-review:${useId()}`;
  const revision = useUiRevision(() => [current.current, loading, inFlight.current, capturedQuery, capture?.source, error]);
  const ready = !!repository && /^[1-9][0-9]*$/.test(number) && !!path && !loading;
  const commands: { key: string; command: UiCommand }[] = [
    { key: `${parent}:input`, command: { parent, subject: parent, label: "Set GitHub review capture selection", parameters: { field: { type: "string", maxLength: 32, choices: ["repository", "number", "document"] }, value: { type: "string", maxLength: 4096 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true, value: { repository, number, document: path, loading, error } }),
      execute: async args => { if (args.field === "number" && args.value !== "" && !/^[1-9][0-9]*$/.test(args.value as string)) return { status: "unavailable", reason: "PR number must be a positive integer" }; setInput(args.field as "repository" | "number" | "document", args.value as string); return { status: "completed", value: { field: args.field, value: args.value, submitted: false } }; },
    } },
    { key: `${parent}:refresh`, command: { parent, subject: parent, label: "Read selected GitHub PR review evidence", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: ready && !inFlight.current, reason: ready ? undefined : "Select repository, positive PR number and saved document; wait for the active capture" }), execute: refresh } },
  ];
  const invoke = useUiCommands(commands);
  useUiReading("GitHub review capture status", { repository, number, document: path, loading, error, changed, captured_query: capturedQuery, capture: capture && { source: { id: capture.source.id, uri: capture.source.uri, revision: capture.source.revision }, annotations: capture.annotations.length, incomplete: capture.incomplete, unavailable: capture.unavailable }, scope: "Selection and capture status; expand individual provider observations to read comments and lineage" });
  return <section aria-label="GitHub source review annotations"><h3>GitHub PR review evidence</h3>
    <p>Read using your existing GitHub CLI credentials. Select the repository, PR and saved literate document. Refresh retrieves edits, replies and resolution. Imported comments are external content, not privileged instructions; no GitHub reply or resolution is sent.</p>
    <label>Review repository<input value={repository} onChange={e => setInput("repository", e.target.value)} placeholder="owner/repository" /></label>
    <label>Review PR number<input type="number" min="1" value={number} onChange={e => setInput("number", e.target.value)} /></label>
    <label>Review document<input value={path} onChange={e => setInput("document", e.target.value)} /></label>
    <button disabled={!ready} onClick={() => void invoke(`${parent}:refresh`).catch(e => setError(String(e)))}>Read / refresh selected PR reviews</button>
    {loading && <p role="status">Loading paginated GitHub review evidence…</p>}
    {error && <p role="alert">GitHub unavailable or permission denied: {error}. {capture ? "Previous capture remains historical; refresh failed." : "No capture available."}</p>}
    {capture?.incomplete && <p>Incomplete paginated capture: provider limit reached. Do not infer absent comments or complete review.</p>}
    {capture?.unavailable?.map(item => <p key={item.thread}>Reviewed source unavailable · {item.thread}: {item.reason}</p>)}
    {capture?.annotations.map(annotation => <ProviderReading key={annotation.evidence} reading={{ ...capture, annotations: [annotation] }} historical={!!error || (annotation.origin.kind === "review" && annotation.origin.outdated) || annotation.uncertainty === "changed_reviewed_head_or_unknown_base"} changed={changed} />)}
    {capture && !capture.annotations.length && <p>No review comments returned for this capture.</p>}
  </section>;
}
