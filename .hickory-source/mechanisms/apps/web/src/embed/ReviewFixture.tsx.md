<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/ReviewFixture.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d12194d73d05"></a>
## `apps/web/src/embed/ReviewFixture.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d12194d73d05-ReviewFixture"></a>
### `ReviewFixture`

Acceptance host: product review UI and attribution on the supported document embed.

<storycode:copy id="f-d12194d73d05-ReviewFixture">
// Acceptance host: product review UI and attribution on the supported document embed.
import { providerObservationContext } from "../debug/providerObservations";
import { useEffect, useMemo, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { DocumentEmbed } from "./DocumentEmbed";
import { ReviewPanel } from "../components/ReviewPanel";
import { LineageSettings } from "../components/LineageSettings";
import { installAttributionProvider } from "../lib/attribution";
import { blameGutter } from "../editor/blameGutter";
import { useBlame } from "../editor/useBlame";
import "../styles.css";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
export function ReviewFixture() {
  const [githubContext, setGithubContext] = useState("");
  const [view, setView] = useState<EditorView | null>(null);
  const [source, setSource] = useState("Human requirement\nAgent claim about total deadline\n");
  const extensions = useMemo(() => [blameGutter()], []);
  useEffect(() => installAttributionProvider({ id: "fixture-receipts", label: "Recorded roles", lines: async () => [
    {line:1,text:"Human requirement",author:"Sam",email:"sam@example.com",commit:"one",time:0,summary:"",uncommitted:false,origin:"human",provider:"Recorded role"},
    {line:2,text:"Agent claim about total deadline",author:"Sam",email:"sam@example.com",commit:"one",time:0,summary:"",uncommitted:false,origin:"agent",provider:"Recorded agent write"},
  ] }), []);
  useBlame(view, "spec.md");
  const revision = useUiRevision(() => [source, githubContext]);
  useUiReading("review acceptance selected GitHub context", githubContext);
  const invoke = useUiCommands([{ key: "review-fixture:context", command: { label: "Inspect selected GitHub context", parent: "review-fixture", subject: "review-fixture:context", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async () => { const context = JSON.stringify(providerObservationContext("spec.md", source)); setGithubContext(context); return { status: "completed", value: { context, scope: "Recorded provider selection; no runtime execution" } }; } } }]);
  return <main style={{padding:24}}><h1>Review acceptance workspace</h1><ReviewPanel document="spec.md" source={source} view={view} /><LineageSettings />
    <button onClick={() => void invoke("review-fixture:context").catch(reason => setGithubContext(String(reason)))}>Inspect selected GitHub context</button><output aria-label="Selected GitHub context">{githubContext}</output>
    <DocumentEmbed path="spec.md" revision={source} source={source} onChange={change => setSource(change.source)} onViewReady={setView} extensions={extensions} />
  </main>;
}
</storycode:copy>

### Assemble `apps/web/src/embed/ReviewFixture.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/ReviewFixture.tsx"><storycode:paste select="#f-d12194d73d05-ReviewFixture"/></storycode:file>

</storycode:doc>
