<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/EmbeddingFixture.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7c085f088935"></a>
## `apps/web/src/embed/EmbeddingFixture.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7c085f088935-EmbeddingFixture"></a>
### `EmbeddingFixture`

<storycode:copy id="f-7c085f088935-EmbeddingFixture">
import { useEffect, useState } from "react";
import { DocumentEmbed, DebuggableDocument } from "./index";
import { measurePortableCore } from "./measure";
import { browserExample } from "../landing/demos/BrowserDebugDemo";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";

export function EmbeddingFixture() {
  const [metrics, setMetrics] = useState("");
  const [runtimeEvidence, setRuntimeEvidence] = useState("");
  useEffect(() => { void measurePortableCore().then((value) => setMetrics(JSON.stringify(value))); }, []);
  const [left, setLeft] = useState("# First note\n\nAn ordinary note.\n\n![asset](images/mark.svg)\n<hick:unknown untouched=\"yes\">🦀 raw</hick:unknown>\n");
  const [right, setRight] = useState("# Second note\n\nIndependent text.\n");
  const [source, setSource] = useState(browserExample("ts"));
  const [revision, setRevision] = useState(0);
  const [readOnly, setReadOnly] = useState(false);
  const [mounted, setMounted] = useState(true);
  const [secondDebug, setSecondDebug] = useState(false);
  const [saved, setSaved] = useState("");
  const [error, setError] = useState("");
  const revisionToken = useUiRevision(() => [left, source, revision, readOnly, mounted, secondDebug, saved]);
  useUiReading("embedding fixture status", { metrics, saved, readOnly, mounted, secondDebug, debug_source: source, error });
  const invoke = useUiCommands([
    { key: "embedding-fixture:readonly", command: { label: "Toggle read-only", parent: "embedding-fixture", subject: "embedding-fixture:first", parameters: {}, prerequisites: [], read: () => ({ revision: revisionToken(), enabled: true, value: readOnly }), execute: async () => { setReadOnly(value => !value); return { status: "completed", value: { read_only: !readOnly } }; } } },
    { key: "embedding-fixture:mount", command: { label: "Mount/unmount", parent: "embedding-fixture", subject: "embedding-fixture:documents", parameters: {}, prerequisites: [], read: () => ({ revision: revisionToken(), enabled: true, value: mounted }), execute: async () => { setMounted(value => !value); return { status: "completed", value: { mounted: !mounted } }; } } },
    { key: "embedding-fixture:replace", command: { label: "Replace source", parent: "embedding-fixture", subject: "embedding-fixture:first", parameters: {}, prerequisites: [], read: () => ({ revision: revisionToken(), enabled: true }), execute: async () => { setLeft("# Host replacement\n"); setRevision(value => value + 1); return { status: "completed", value: { replaced: true, saved: false } }; } } },
    { key: "embedding-fixture:save", command: { label: "Save explicitly", parent: "embedding-fixture", subject: "embedding-fixture:first", parameters: {}, prerequisites: [], read: () => ({ revision: revisionToken(), enabled: true }), execute: async () => { setSaved(left); return { status: "completed", value: { saved: left, scope: "Fixture host memory; no persistence backend" } }; } } },
    { key: "embedding-fixture:second", command: { label: "Add/remove second debugger", parent: "embedding-fixture", subject: "embedding-fixture:debuggers", parameters: {}, prerequisites: [], read: () => ({ revision: revisionToken(), enabled: true, value: secondDebug }), execute: async () => { setSecondDebug(value => !value); return { status: "completed", value: { second_debugger: !secondDebug } }; } } },
    { key: "embedding-fixture:source", command: { label: "Edit fixture debug source", parent: "embedding-fixture", subject: "embedding-fixture:debug-source", parameters: { source: { type: "string", maxLength: 262144 } }, prerequisites: [], read: () => ({ revision: revisionToken(), enabled: true }), execute: async args => { setSource(String(args.source)); setRevision(value => value + 1); return { status: "completed", value: { edited: true, executed: false, saved: false } }; } } },
  ]);
  const act = (key: string, args = {}) => { void invoke(`embedding-fixture:${key}`, args).catch(reason => setError(String(reason))); };
  return <main style={{ maxWidth: 1000, margin: "30px auto", fontFamily: "system-ui" }}>
    <h1>A host that owns its documents</h1>
    {error && <p role="alert">{error}</p>}
    <output aria-label="Browser measurements">{metrics}</output>
    <button onClick={() => act("readonly")}>Toggle read-only</button>
    <button onClick={() => act("mount")}>Mount/unmount</button>
    <button onClick={() => act("replace")}>Replace source</button>
    <button onClick={() => act("save")}>Save explicitly</button>
    <output aria-label="Saved document">{saved}</output>
    {mounted && <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, alignItems: "start" }}>
      <DocumentEmbed path="first.md" source={left} revision={String(revision)} readOnly={readOnly} onChange={(change) => { setLeft(change.source); setRevision((r) => r + 1); }}
        host={{ resolveAsset: async () => "data:image/svg+xml," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40"><circle cx="20" cy="20" r="18" fill="orange"/></svg>') }} />
      <DocumentEmbed path="second.md" source={right} revision="0" onChange={(change) => setRight(change.source)} />
    </div>}
    <h2>Local debugger</h2>
    <button onClick={() => act("second")}>Add/remove second debugger</button>
    <textarea aria-label="Debug source" style={{ width: "100%", height: 100 }} value={source} onChange={(event) => act("source", { source: event.target.value })} />
    {secondDebug && <DebuggableDocument path="second-debug.md" revision="1" source={'<hick:file path="second.js">console.log(7);</hick:file>'} />}
    <output aria-label="Runtime evidence" hidden>{runtimeEvidence}</output>
    <DebuggableDocument onRuntime={(run) => setRuntimeEvidence(JSON.stringify(run))} source={source} path="fixture.md" revision={String(revision)} onChange={(change) => { setSource(change.source); setRevision((r) => r + 1); }} />
  </main>;
}
</storycode:copy>

### Assemble `apps/web/src/embed/EmbeddingFixture.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/EmbeddingFixture.tsx"><storycode:paste select="#f-7c085f088935-EmbeddingFixture"/></storycode:file>

</storycode:doc>
