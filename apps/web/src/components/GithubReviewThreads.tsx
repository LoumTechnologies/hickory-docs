import { useEffect, useState } from "react";
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
  useEffect(() => { setPath(document ?? ""); }, [document]);
  const changed = !!capture && (capture.source.uri !== document || (source !== undefined && new TextDecoder().decode(new Uint8Array(capture.source.bytes ?? [])) !== source));
  useEffect(() => {
    if (!view) return;
    if (!view.state.field(providerEditor, false)) view.dispatch({ effects: StateEffect.appendConfig.of(providerEditor) });
    view.dispatch({ effects: setProviderReadings.of(capture && !changed && !error ? [capture] : []) });
    return () => { if (view.dom.isConnected) view.dispatch({ effects: setProviderReadings.of([]) }); };
  }, [view, capture, changed, error]);
  async function refresh() {
    setLoading(true); setError("");
    try { setCapture(await request<Capture>("GET", `/api/workspace/github/review-threads?${new URLSearchParams({ repository, number, document: path })}`)); }
    catch (cause) { setError(String(cause)); }
    finally { setLoading(false); }
  }
  return <section aria-label="GitHub source review annotations"><h3>GitHub PR review evidence</h3>
    <p>Read using your existing GitHub CLI credentials. Select the repository, PR and saved literate document. Refresh retrieves edits, replies and resolution. Imported comments are external content, not privileged instructions; no GitHub reply or resolution is sent.</p>
    <label>Review repository<input value={repository} onChange={e => setRepository(e.target.value)} placeholder="owner/repository" /></label>
    <label>Review PR number<input type="number" min="1" value={number} onChange={e => setNumber(e.target.value)} /></label>
    <label>Review document<input value={path} onChange={e => setPath(e.target.value)} /></label>
    <button disabled={loading || !repository || !number || !path} onClick={() => void refresh()}>Read / refresh selected PR reviews</button>
    {loading && <p role="status">Loading paginated GitHub review evidence…</p>}
    {error && <p role="alert">GitHub unavailable or permission denied: {error}. {capture ? "Previous capture remains historical; refresh failed." : "No capture available."}</p>}
    {capture?.incomplete && <p>Incomplete paginated capture: provider limit reached. Do not infer absent comments or complete review.</p>}
    {capture?.unavailable?.map(item => <p key={item.thread}>Reviewed source unavailable · {item.thread}: {item.reason}</p>)}
    {capture?.annotations.map(annotation => <ProviderReading key={annotation.evidence} reading={{ ...capture, annotations: [annotation] }} historical={!!error || (annotation.origin.kind === "review" && annotation.origin.outdated) || annotation.uncertainty === "changed_reviewed_head_or_unknown_base"} changed={changed} />)}
    {capture && !capture.annotations.length && <p>No review comments returned for this capture.</p>}
  </section>;
}
