<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/ProviderReading.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-80c29adca86c"></a>
## `apps/web/src/debug/ProviderReading.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-80c29adca86c-ReviewContent"></a>
### `ReviewContent`

<storycode:copy id="f-80c29adca86c-ReviewContent">
import { EvidenceDisclosure } from "../components/EvidenceDisclosure";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { selectProviderReading, type ProviderReading as Reading } from "./providerObservations";

function ReviewContent({ payload }: { payload: unknown }) {
  const value = (payload && typeof payload === "object" ? payload : {}) as { thread?: { comments?: { id: string; body: string; author?: { login?: string }; createdAt?: string; updatedAt?: string; url?: string; diffHunk?: string }[] }; comment?: { id: string; body: string; author?: { login?: string }; createdAt?: string; updatedAt?: string; url?: string }; reviewed_source?: { uri: string }; mapping_state?: string };
  const comments = value.thread?.comments ?? (value.comment ? [value.comment] : []);
  const safe = (url?: string) => url?.startsWith("https://github.com/") ? url : undefined;
  return <><p>External review · {value.mapping_state}</p>{comments.map(comment => <article key={comment.id}>
    <p>{comment.author?.login ?? "unknown author"} · {comment.createdAt} · edited {comment.updatedAt}</p>
    <p style={{ whiteSpace: "pre-wrap" }}>{comment.body}</p>
    {safe(comment.url) && <ProviderLink url={safe(comment.url)!} label={`Exact GitHub discussion · ${comment.id}`} />}
    {"diffHunk" in comment && typeof comment.diffHunk === "string" && comment.diffHunk && <EvidenceDisclosure label={`Original reviewed diff · ${comment.id}`} value={comment.diffHunk}><pre>{comment.diffHunk}</pre></EvidenceDisclosure>}
  </article>)}{safe(value.reviewed_source?.uri) && <ProviderLink url={safe(value.reviewed_source?.uri)!} label="Exact reviewed source revision" />}
    <EvidenceDisclosure label="Complete external evidence lineage" value={payload}><pre>{JSON.stringify(payload, null, 2)}</pre></EvidenceDisclosure></>;
}

</storycode:copy>
<a id="f-80c29adca86c-ProviderReading"></a>
### `ProviderReading`

General provider view; runtime and external reviews retain their own origin.

<storycode:copy id="f-80c29adca86c-ProviderReading">
/** General provider view; runtime and external reviews retain their own origin. */
export function ProviderReading({ reading, historical = false, changed = false }: { reading: Reading; historical?: boolean; changed?: boolean }) {
  const owner = useRef(Symbol("provider-reading"));
  const [expanded, setExpanded] = useState(false);
  const [pinned, setPinned] = useState(false);
  const [actionError, setActionError] = useState("");
  const projected = useMemo(() => ({ ...reading, annotations: reading.annotations.map(annotation => annotation.provider.startsWith("github-pr-") ? { ...annotation, payload_schema: "hickory.github-review-visible.v1", payload: { original_payload_schema: annotation.payload_schema, visible: visibleReview(annotation.payload), disclosure: "Visible review comment projection; original diff and raw lineage require explicit nested reading" } } : annotation) }), [reading]);
  useEffect(() => {
    selectProviderReading(owner.current, projected, expanded || pinned, historical || changed);
    return () => selectProviderReading(owner.current, projected, false, historical);
  }, [projected, expanded, pinned, historical, changed]);
  const flags = useRef({ expanded, pinned }); flags.current = { expanded, pinned };
  const select = (key: "expanded" | "pinned", value: boolean) => {
    flags.current[key] = value;
    if (key === "expanded") setExpanded(value); else setPinned(value);
    selectProviderReading(owner.current, projected, flags.current.expanded || flags.current.pinned, historical || changed);
  };
  const parent = `provider-reading:${useId()}`;
  const revision = useUiRevision(() => [reading, historical, changed, flags.current, actionError]);
  const invoke = useUiCommands([{ key: `${parent}:pin`, command: { parent, subject: parent, label: "Set provider evidence pin", parameters: { pinned: { type: "boolean" } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true, value: { pinned: flags.current.pinned, expanded: flags.current.expanded, historical, changed, action_error: actionError, source: reading.source.uri, evidence: reading.annotations.map(a => a.evidence) } }),
    execute: async args => { select("pinned", args.pinned as boolean); return { status: "completed", value: { pinned: args.pinned, scope: "Selected captured evidence for future context; not a delivery receipt or action permission" } }; },
  } }]);
  const text = reading.source.bytes && new TextDecoder().decode(new Uint8Array(reading.source.bytes));
  const visible = { source: { id: reading.source.id, uri: reading.source.uri, revision: reading.source.revision }, historical, changed, unsupported: reading.unsupported,
    annotations: reading.annotations.map(annotation => ({ ...annotation, payload: annotation.provider.startsWith("github-pr-") ? visibleReview(annotation.payload) : annotation.payload, excerpt: annotation.location && text ? text.split("\n")[annotation.location.line] : null })) };
  return <section aria-label="Provider source observations">
    <label><input type="checkbox" checked={pinned} onChange={event => void invoke(`${parent}:pin`, { pinned: event.target.checked }).catch(e => setActionError(String(e)))} />Pin provider evidence in next context</label>
    {actionError && <p role="alert">{actionError}</p>}
    <EvidenceDisclosure label={`Provider observations · ${changed ? "source changed" : historical ? "historical" : "captured"}`} value={visible} onOpenChange={open => select("expanded", open)}>
      <p>{reading.source.uri} · revision {reading.source.revision ?? "unavailable"}. Expanded/pinned scope selects visible evidence for context; original diffs and complete raw lineage require their nested reading. Selection is not a delivery receipt or action permission.</p>
      {reading.annotations.map(annotation => <article key={annotation.evidence} data-evidence={annotation.evidence}>
        {annotation.location && text && <code>[{annotation.location.id}] {text.split("\n")[annotation.location.line]}</code>}
        <p>{annotation.provider} · {annotation.evidence} · source precision {annotation.source_precision}</p>
        {annotation.origin.kind === "review" && <p>GitHub state: {annotation.origin.resolved ? "resolved" : "open"} · {annotation.origin.outdated ? "outdated" : "reviewed anchor"} · {annotation.origin.side}</p>}<pre>{JSON.stringify(annotation.origin, null, 2)}</pre>{annotation.provider.startsWith("github-pr-") ? <ReviewContent payload={annotation.payload} /> : <pre>{JSON.stringify(annotation.payload, null, 2)}</pre>}
        <p>Measurement: {annotation.measurement_semantics ?? "not supplied"} · uncertainty {annotation.uncertainty ?? "not supplied"}</p>
      </article>)}
      <p>Unsupported: {reading.unsupported.join(", ") || "none declared"}</p>
    </EvidenceDisclosure>
  </section>;
}

</storycode:copy>
<a id="f-80c29adca86c-visibleReview"></a>
### `visibleReview`

<storycode:copy id="f-80c29adca86c-visibleReview">
function visibleReview(payload: unknown) {
  const value = (payload && typeof payload === "object" ? payload : {}) as { thread?: { comments?: { id: string; body: string; author?: unknown; createdAt?: string; updatedAt?: string; url?: string }[] }; comment?: { id: string; body: string; author?: unknown; createdAt?: string; updatedAt?: string; url?: string }; reviewed_source?: { uri: string }; mapping_state?: string };
  return { mapping_state: value.mapping_state, reviewed_source_uri: value.reviewed_source?.uri,
    comments: (value.thread?.comments ?? (value.comment ? [value.comment] : [])).map(({ id, body, author, createdAt, updatedAt, url }) => ({ id, body, author, createdAt, updatedAt, url })) };
}
</storycode:copy>
<a id="f-80c29adca86c-ProviderLink"></a>
### `ProviderLink`

<storycode:copy id="f-80c29adca86c-ProviderLink">
function ProviderLink({ url, label }: { url: string; label: string }) {
  const parent = `provider-link:${useId()}`;
  const revision = useUiRevision(() => [url, label]);
  const open = async () => {
    try {
      const result = window.open(url, "_blank", "noopener,noreferrer");
      return { status: "completed" as const, value: { url, navigation_requested: true, tab_handle_returned: !!result, scope: "Browser opening requested; browser/OS may require user interaction and this is not a page-load witness" } };
    } catch (e) { return { status: "unavailable" as const, reason: e instanceof Error ? e.message : String(e) }; }
  };
  useUiCommands([{ key: parent, command: { parent, subject: parent, label, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { url } }), execute: open } }]);
  return <a href={url} target="_blank" rel="noopener noreferrer" onClick={event => { event.preventDefault(); void open(); }}>{label}</a>;
}
</storycode:copy>

### Assemble `apps/web/src/debug/ProviderReading.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/ProviderReading.tsx"><storycode:paste select="#f-80c29adca86c-ReviewContent"/><storycode:paste select="#f-80c29adca86c-ProviderReading"/><storycode:paste select="#f-80c29adca86c-visibleReview"/><storycode:paste select="#f-80c29adca86c-ProviderLink"/></storycode:file>

</storycode:doc>
