import { useEffect, useRef, useState } from "react";
import { selectProviderReading, type ProviderReading as Reading } from "./providerObservations";

function ReviewContent({ payload }: { payload: unknown }) {
  const value = payload as { thread?: { comments?: { id: string; body: string; author?: { login?: string }; createdAt?: string; updatedAt?: string; url?: string; diffHunk?: string }[] }; comment?: { id: string; body: string; author?: { login?: string }; createdAt?: string; updatedAt?: string; url?: string }; reviewed_source?: { uri: string }; mapping_state?: string };
  const comments = value.thread?.comments ?? (value.comment ? [value.comment] : []);
  const safe = (url?: string) => url?.startsWith("https://github.com/") ? url : undefined;
  return <><p>External review · {value.mapping_state}</p>{comments.map(comment => <article key={comment.id}>
    <p>{comment.author?.login ?? "unknown author"} · {comment.createdAt} · edited {comment.updatedAt}</p>
    <p style={{ whiteSpace: "pre-wrap" }}>{comment.body}</p>
    {safe(comment.url) && <a href={safe(comment.url)} target="_blank" rel="noopener noreferrer">Exact GitHub discussion · {comment.id}</a>}
    {"diffHunk" in comment && typeof comment.diffHunk === "string" && comment.diffHunk && <details><summary>Original reviewed diff</summary><pre>{comment.diffHunk}</pre></details>}
  </article>)}{safe(value.reviewed_source?.uri) && <a href={safe(value.reviewed_source?.uri)} target="_blank" rel="noopener noreferrer">Exact reviewed source revision</a>}
    <details><summary>Complete external evidence lineage</summary><pre>{JSON.stringify(payload, null, 2)}</pre></details></>;
}

/** General provider view; runtime and external reviews retain their own origin. */
export function ProviderReading({ reading, historical = false, changed = false }: { reading: Reading; historical?: boolean; changed?: boolean }) {
  const owner = useRef(Symbol("provider-reading"));
  const [expanded, setExpanded] = useState(false);
  const [pinned, setPinned] = useState(false);
  useEffect(() => {
    selectProviderReading(owner.current, reading, expanded || pinned, historical || changed);
    return () => selectProviderReading(owner.current, reading, false, historical);
  }, [reading, expanded, pinned, historical, changed]);
  const text = reading.source.bytes && new TextDecoder().decode(new Uint8Array(reading.source.bytes));
  return <section aria-label="Provider source observations">
    <label><input type="checkbox" checked={pinned} onChange={event => setPinned(event.target.checked)} />Pin provider evidence in next context</label>
    <details onToggle={event => setExpanded(event.currentTarget.open)}>
      <summary>Provider observations · {changed ? "source changed" : historical ? "historical" : "captured"}</summary>
      <p>{reading.source.uri} · revision {reading.source.revision ?? "unavailable"}. Expanded/pinned scope selects context; it is not a delivery receipt or action permission.</p>
      {reading.annotations.map(annotation => <article key={annotation.evidence} data-evidence={annotation.evidence}>
        {annotation.location && text && <code>[{annotation.location.id}] {text.split("\n")[annotation.location.line]}</code>}
        <p>{annotation.provider} · {annotation.evidence} · source precision {annotation.source_precision}</p>
        {annotation.origin.kind === "review" && <p>GitHub state: {annotation.origin.resolved ? "resolved" : "open"} · {annotation.origin.outdated ? "outdated" : "reviewed anchor"} · {annotation.origin.side}</p>}<pre>{JSON.stringify(annotation.origin, null, 2)}</pre>{annotation.provider.startsWith("github-pr-") ? <ReviewContent payload={annotation.payload} /> : <pre>{JSON.stringify(annotation.payload, null, 2)}</pre>}
        <p>Measurement: {annotation.measurement_semantics ?? "not supplied"} · uncertainty {annotation.uncertainty ?? "not supplied"}</p>
      </article>)}
      <p>Unsupported: {reading.unsupported.join(", ") || "none declared"}</p>
    </details>
  </section>;
}
