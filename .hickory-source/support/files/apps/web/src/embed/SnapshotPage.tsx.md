<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/SnapshotPage.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a3b8cc5dce078292a382130e6bce4f2ef0f8feeedbd5e8ba05b579d8ba053f89`.

<book:entry path="apps/web/src/embed/SnapshotPage.tsx">
import { useEffect, useState } from "react";
import { useRef } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";
import { decodeSnapshot, PublishedSnapshot, type ViewerSnapshot } from "./PublishedSnapshot";
interface PublicationIndex { version: number; revision: string; documents: { path: string; status: string; snapshot: string; revision: string; source_hash: string }[] }
interface SnapshotLoadError { title: string; detail: string; recovery: string }

function presentSnapshotLoadError(failure: unknown): SnapshotLoadError {
  const message = String(failure);
  if (failure instanceof SyntaxError || /Unexpected token|JSON/.test(message)) return { title: "This link did not return snapshot JSON", detail: "The viewer received a web page or malformed data instead of a Hickory snapshot.", recovery: "Check the publication link, or choose an exported .json snapshot." };
  if (/Publication download failed: 404/.test(message)) return { title: "This published snapshot was not found", detail: "The publication may have moved or is not available on this site.", recovery: "Choose another publication link or open an exported .json snapshot." };
  if (/cancelled|aborted/i.test(message)) return { title: "Snapshot loading was cancelled", detail: "No new snapshot was opened.", recovery: "Try the publication link again when you are ready." };
  return { title: "Could not open this snapshot", detail: "The file is unavailable or is not a supported Hickory viewer snapshot.", recovery: "Choose a valid exported .json snapshot or check the publication link." };
}
export function SnapshotPage() {
  const [index, setIndex] = useState<PublicationIndex | null>(null);
  const [artifactUrl, setArtifactUrl] = useState<string | undefined>();
  const [indexUrl, setIndexUrl] = useState("");
  const [snapshot, setSnapshot] = useState<ViewerSnapshot | null>(null);
  const [error, setError] = useState<SnapshotLoadError | null>(null);
  const [busy, setBusy] = useState(false), [capture, setCapture] = useState(0);
  const generation = useRef(0), active = useRef<AbortController | null>(null), alive = useRef(true);
  useEffect(() => { alive.current = true; return () => { alive.current = false; ++generation.current; active.current?.abort(); }; }, []);
  async function load(read: (signal: AbortSignal) => Promise<{ snapshot?: ViewerSnapshot; artifact?: string; index?: PublicationIndex; indexUrl?: string }>, signal?: AbortSignal): Promise<UiOutcome> {
    if (signal?.aborted) return { status: "unavailable", reason: "Publication request was cancelled before loading" };
    active.current?.abort();
    const controller = new AbortController(), captured = ++generation.current;
    active.current = controller;
    const abort = () => controller.abort(); signal?.addEventListener("abort", abort, { once: true });
    setBusy(true); setError(null);
    try {
      const result = await read(controller.signal);
      if (!alive.current || controller.signal.aborted || captured !== generation.current) return { status: "unavailable", reason: "The originating publication request was cancelled or replaced; its result was not adopted" };
      if (result.index) { setIndex(result.index); setIndexUrl(result.indexUrl!); setSnapshot(null); setArtifactUrl(undefined); }
      else { setSnapshot(result.snapshot!); setArtifactUrl(result.artifact); setCapture(captured); }
      return { status: "completed", value: { publication_loaded: true, historical: true, execution_requested: false, source_hash: result.snapshot?.source_hash ?? null } };
    } catch (failure) {
      const reason = String(failure);
      if (alive.current && captured === generation.current) setError(presentSnapshotLoadError(failure));
      return { status: "unavailable", reason };
    } finally {
      signal?.removeEventListener("abort", abort);
      if (captured === generation.current) { active.current = null; if (alive.current) setBusy(false); }
    }
  }
  const importText = (text: () => Promise<string>, signal?: AbortSignal) => load(async () => ({ snapshot: await decodeSnapshot(JSON.parse(await text())) }), signal);
  function publicationUrl(path: string, base = location.href) {
    const url = new URL(path, base);
    if (url.origin !== location.origin || !["http:", "https:"].includes(url.protocol)) throw Error("Host publications on the same origin as this viewer.");
    return url;
  }
  const loadUrl = (path: string, kind: "snapshot" | "index", signal?: AbortSignal, expected?: PublicationIndex["documents"][number]) => load(async signal => {
    const url = publicationUrl(path), response = await fetch(url, { signal });
    if (!response.ok) throw Error(`Publication download failed: ${response.status}`);
    const value = await response.json();
    if (kind === "snapshot") {
      const snapshot = await decodeSnapshot(value);
      if (expected && (snapshot.source_hash !== expected.source_hash || snapshot.verification?.revision !== expected.revision || snapshot.path !== expected.path)) throw Error("Publication index and snapshot identities differ");
      return { snapshot, artifact: new URL("./", url).href };
    }
    const index = value as PublicationIndex;
    if (!index || index.version !== 1 || typeof index.revision !== "string" || !Array.isArray(index.documents) || index.documents.some(d => !d || typeof d.path !== "string" || typeof d.snapshot !== "string" || typeof d.revision !== "string" || typeof d.source_hash !== "string" || !["passed", "failed", "not_run", "unavailable", "stale"].includes(d.status))) throw Error("Invalid publication index");
    return { index, indexUrl: url.href };
  }, signal);
  useEffect(() => {
    const query = new URL(location.href).searchParams, src = query.get("src"), selected = query.get("index");
    if (src || selected) void loadUrl((src ?? selected)!, src ? "snapshot" : "index");
  }, []);
  const cancel = () => { active.current?.abort(); ++generation.current; active.current = null; setBusy(false); };
  const revision = useUiRevision(() => [index, indexUrl, snapshot, artifactUrl, busy, error]);
  useUiReading("publication viewer", { busy, error, index, current: snapshot ? { path: snapshot.path, source_hash: snapshot.source_hash, historical: true } : null });
  const command = (label: string, execute: (args: Record<string, import("../lib/uiActions").UiValue>, signal: AbortSignal) => Promise<UiOutcome>, parameters: import("../lib/uiActions").UiCommand["parameters"]) => ({ key: `publication-viewer:${label}`, command: {
    label, parent: "publication-viewer", subject: "publication-viewer:load", parameters, prerequisites: ["Imported results are historical; no checks are executed."],
    read: () => ({ revision: revision(), enabled: !busy, reason: busy ? "A publication is loading; cancel it before replacing it" : undefined }),
    execute: (args: Record<string, import("../lib/uiActions").UiValue>, _actor: unknown, signal: AbortSignal) => execute(args, signal),
  } });
  useUiCommands([
    command("Import exported snapshot JSON", (args, signal) => importText(async () => args.json as string, signal), { json: { type: "string", maxLength: 1048576 } }),
    ...(["snapshot", "index"] as const).map(kind => command(`Load same-origin publication ${kind}`, (args, signal) => loadUrl(args.path as string, kind, signal), { path: { type: "string", maxLength: 4096 } })),
    { key: "publication-viewer:cancel", command: { label: "Cancel publication loading", parent: "publication-viewer", subject: "publication-viewer:cancel", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: busy }), execute: async () => { cancel(); return { status: "completed", value: { cancellation_requested: true, retained_publication_unchanged: true } }; } } },
    ...(index?.documents ?? []).map((document, i) => command(`Open published document ${i}: ${document.path}`, (_args, signal) => loadUrl(new URL(document.snapshot, indexUrl).href, "snapshot", signal, document), {})),
  ]);
  return <><header className="snapshot-header"><h1>Hickory Docs snapshot viewer</h1><label>Open exported snapshot <input type="file" accept=".json,application/json" onChange={event => { const file = event.target.files?.[0]; if (file) void importText(() => file.text()); }} /></label>{busy && <button onClick={cancel}>Cancel publication loading</button>}</header>
    {index && <main className="published-snapshot"><h2>Published executable specifications</h2><p>Selected public evidence at {index.revision}; each result is revision-bound.</p><ul>{index.documents.map((d, i) => { let src: URL; try { src = publicationUrl(d.snapshot, indexUrl); } catch { return <li key={i}>External artifact refused</li>; } return <li key={i}>{src.origin === location.origin ? <a href={`?src=${encodeURIComponent(src.href)}`} onClick={event => { event.preventDefault(); void loadUrl(src.href, "snapshot", undefined, d); }}>{d.path} · {d.status}</a> : <span>External artifact refused</span>} · {d.source_hash}</li>; })}</ul></main>}
    {error && <section className="snapshot-load-error" role="alert"><h2>{error.title}</h2><p>{error.detail}</p><p>{error.recovery}</p></section>}{snapshot && <PublishedSnapshot key={capture} snapshot={snapshot} artifactUrl={artifactUrl} predictionNavigation={{
      available: (index?.documents ?? []).filter(document => index!.documents.filter(candidate => candidate.source_hash === document.source_hash).length === 1).map(document => document.source_hash),
      open: async hash => {
        if (busy) return { status: "unavailable", reason: "A publication is loading; wait or cancel before navigating" };
        const matching = index?.documents.filter(document => document.source_hash === hash) ?? [];
        if (matching.length !== 1) return { status: "unavailable", reason: "The selected source revision is missing or ambiguous in this publication index" };
        const document = matching[0];
        return loadUrl(new URL(document.snapshot, indexUrl).href, "snapshot", undefined, document);
      },
    }} />}</>;
}

</book:entry>

</book:doc>
