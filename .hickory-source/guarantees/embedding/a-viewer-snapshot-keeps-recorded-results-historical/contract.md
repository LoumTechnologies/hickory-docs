<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A viewer snapshot keeps recorded results historical

For: Host developers embedding the document editor without a native execution service.

Given a document that can be woven by the local application, exporting its viewer
snapshot captures source bytes, their SHA-256, capture time and the cell model
with recorded results. Export does not execute checks. The standalone browser
viewer uses the existing document embed and CellPanel to display that source,
stdout/stderr transcripts, statuses and expectation differences without a local
backend. It labels the results historical and does not certify current files.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-eaaafafcba7f"></a>
## `apps/web/src/embed/PublishedSnapshot.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A viewer snapshot keeps recorded results historical](contract.md).

<a id="f-eaaafafcba7f-hosts"></a>
### `{ hosts }`

<storycode:copy id="f-eaaafafcba7f-hosts">
// Guarantee: docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { PublishedSnapshot, type ViewerSnapshot } from "./PublishedSnapshot";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { sha256 } from "../lib/reviews";
const { hosts } = vi.hoisted(() => ({ hosts: [] as { resolveAsset: (path: string, signal: AbortSignal) => Promise<string> }[] }));
</storycode:copy>
<a id="f-eaaafafcba7f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-eaaafafcba7f-expression_statement">
vi.mock("./DocumentEmbed", () => ({ DocumentEmbed: ({ host }: { host: typeof hosts[number] }) => { hosts.push(host); return <div>Readonly document fixture</div>; } }));
</storycode:copy>
<a id="f-eaaafafcba7f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-eaaafafcba7f-expression_statement-2">
afterEach(() => { cleanup(); hosts.length = 0; vi.restoreAllMocks(); vi.unstubAllGlobals(); });
</storycode:copy>
<a id="f-eaaafafcba7f-snapshot"></a>
### `snapshot`

<storycode:copy id="f-eaaafafcba7f-snapshot">
const snapshot = (status: "passed" | "failed" = "passed"): ViewerSnapshot => ({ format: "hickory-viewer-snapshot", version: 1, path: "spec.md", source: "# Captured", source_hash: "hash", captured_at_ms: 0, execution: "recorded", cells: [],
  verification: { status, revision: "a".repeat(40), source_hash: "hash", dependencies: {}, assets: {}, environment: { hidden: "private environment" },
    commands: [{ command: ["run", "check"], exit_code: status === "passed" ? 0 : 1, stdout: "recorded stdout" }], requirements: [], origin: "supplied fixture", scope: "historical only" } });
</storycode:copy>
<a id="f-eaaafafcba7f-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-eaaafafcba7f-expression_statement-3">
it("shares publication disclosure and transcript state and resets a replacement capture", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const draw = (value: ViewerSnapshot) => <UiActionContext.Provider value={registry}><PublishedSnapshot snapshot={value} /></UiActionContext.Provider>;
  const view = render(draw(snapshot()));
  expect(screen.getByText(/Snapshot details/).closest("details")?.open).toBe(false);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const handle = (label: string) => actions().find(action => action.label === label)!.handle;
  const run = async (target: string, args: Record<string, boolean | number>) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: target, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(actions().some(action => action.label === "Read disclosed publication evidence bytes")).toBe(false);
  expect(JSON.stringify(actions())).not.toContain("private environment");
  await run(handle("Set publication evidence visibility: Environment and dependency identities"), { open: true });
  const reading = handle("Read disclosed publication evidence bytes");
  const bytes = await run(reading, { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(new Uint8Array(bytes.outcome.value.bytes))).toContain("private environment");
  const fold = screen.getByText("Environment and dependency identities").closest("details")!;
  fold.open = false; fireEvent(fold, new Event("toggle"));
  expect(await run(reading, { offset: 0, limit: 8192 })).not.toMatchObject({ status: "result" });
  await run(handle("Set published execution transcript visibility"), { shown: false });
  expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(false);
  const stale = handle("Set published execution transcript visibility");
  view.rerender(draw(snapshot("failed")));
  expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(true);
  expect(screen.getByText("recorded stdout").closest("details")!.open).toBe(true);
  expect(await run(stale, { shown: false })).not.toMatchObject({ status: "result" });
});
</storycode:copy>
<a id="f-eaaafafcba7f-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-eaaafafcba7f-expression_statement-4">
it("revokes a verified asset completing after its publication capture retires", async () => {
  const create = vi.fn(() => "blob:retired-asset"), revoke = vi.fn();
  vi.stubGlobal("URL", Object.assign(class extends URL {}, { createObjectURL: create, revokeObjectURL: revoke }));
  const value = snapshot(); value.verification!.assets["asset.png"] = await sha256("asset bytes");
  let resolve!: (response: Response) => void;
  vi.spyOn(globalThis, "fetch").mockImplementation(() => new Promise(done => { resolve = done; }));
  const view = render(<PublishedSnapshot snapshot={value} artifactUrl={`${location.origin}/publication/`} />);
  const pending = hosts.at(-1)!.resolveAsset("asset.png", new AbortController().signal);
  view.unmount();
  resolve(new Response("asset bytes", { headers: { "Content-Type": "image/png" } }));
  await expect(pending).rejects.toThrow("capture was retired");
  expect(create).toHaveBeenCalledTimes(1); expect(revoke).toHaveBeenCalledExactlyOnceWith("blob:retired-asset");
});
</storycode:copy>
<a id="f-eaaafafcba7f-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-eaaafafcba7f-expression_statement-5">
it("keeps saved text and SVG visible when execution transcripts are hidden", () => {
  const fetch = vi.spyOn(globalThis, "fetch");
  const value = snapshot();
  const svg = btoa('<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>');
  value.cells = [{ kind: "exec", id: "rich", command: "print('do not execute')", container: "python", span: [0, 1], status: "unrecorded", saved_output: `recorded text\n\x1eHICKORY_DISPLAY ${JSON.stringify({ kind: "image", mime: "image/svg+xml", data: svg })}\n` }];
  value.cells.push({ kind: "exec", id: "checked", command: "printf transcript-only", container: "shell", span: [1, 2], status: "ok", expect: { match: "exact", body: "transcript-only" }, transcript: [{ t: 0, kind: "out", data: "transcript-only" }] });
  const view = render(<PublishedSnapshot snapshot={value} />);
  expect(view.container.querySelector(".transcript")).toBeTruthy();
  expect(screen.getByText("recorded text")).toBeTruthy();
  const image = screen.getByRole("img", { name: "Python cell output" });
  expect(image.getAttribute("src")).toBe(`data:image/svg+xml;base64,${svg}`);
  expect(document.querySelector("script")).toBeNull();
  expect(fetch).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("checkbox"));
  expect(view.container.querySelector(".transcript")).toBeNull();
  expect(screen.getByText("recorded text")).toBeTruthy();
  expect(screen.getByRole("img", { name: "Python cell output" })).toBe(image);
  expect(fetch).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/embed/PublishedSnapshot.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/PublishedSnapshot.controls.test.tsx"><storycode:paste select="#f-eaaafafcba7f-hosts"/><storycode:paste select="#f-eaaafafcba7f-expression_statement"/><storycode:paste select="#f-eaaafafcba7f-expression_statement-2"/><storycode:paste select="#f-eaaafafcba7f-snapshot"/><storycode:paste select="#f-eaaafafcba7f-expression_statement-3"/><storycode:paste select="#f-eaaafafcba7f-expression_statement-4"/><storycode:paste select="#f-eaaafafcba7f-expression_statement-5"/></storycode:file>

<a id="output-48b9cf5d4873"></a>
## `apps/web/src/embed/PublishedSnapshot.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A viewer snapshot keeps recorded results historical](contract.md).

<a id="f-48b9cf5d4873-digest"></a>
### `digest`

<storycode:copy id="f-48b9cf5d4873-digest">
import { afterEach, expect, it, vi } from "vitest";
import { decodeSnapshot } from "./PublishedSnapshot";
const digest = vi.fn(async (_algorithm: string, bytes: Uint8Array) => {
  const out = new Uint8Array(32); bytes.forEach((byte, i) => { out[i % 32] ^= byte; }); return out.buffer;
});
</storycode:copy>
<a id="f-48b9cf5d4873-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-48b9cf5d4873-expression_statement">
afterEach(() => vi.unstubAllGlobals());
</storycode:copy>
<a id="f-48b9cf5d4873-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-48b9cf5d4873-expression_statement-2">
// Guarantee: docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md
it("rejects changed source and unsupported transcript events", async () => {
  vi.stubGlobal("crypto", { subtle: { digest } });
  const source = "original";
  const hash = Array.from(new Uint8Array(await digest("SHA-256", new TextEncoder().encode(source))), b => b.toString(16).padStart(2,"0")).join("");
  const snapshot = {format:"hickory-viewer-snapshot",version:1,path:"spec.md",source,source_hash:hash,captured_at_ms:0,execution:"recorded",cells:[]};
  expect((await decodeSnapshot(snapshot)).source).toBe(source);
  await expect(decodeSnapshot({...snapshot,source:"changed"})).rejects.toThrow("hash");
  await expect(decodeSnapshot({...snapshot,cells:[{kind:"exec",id:"a",container:"shell",command:"echo",span:[0,1],saved_output:{html:"untrusted"}}]})).rejects.toThrow("saved output");
  await expect(decodeSnapshot({...snapshot,cells:[{kind:"exec",id:"a",container:"shell",command:"echo",span:[0,1],transcript:[{kind:"injected",data:"x",t:0}]}]})).rejects.toThrow("transcript");
});
</storycode:copy>

### Assemble `apps/web/src/embed/PublishedSnapshot.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/PublishedSnapshot.test.ts"><storycode:paste select="#f-48b9cf5d4873-digest"/><storycode:paste select="#f-48b9cf5d4873-expression_statement"/><storycode:paste select="#f-48b9cf5d4873-expression_statement-2"/></storycode:file>

<a id="output-29151ba5d332"></a>
## `apps/web/src/embed/PublishedSnapshot.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A viewer snapshot keeps recorded results historical](contract.md), [Published executable results identify reviewed inputs](../publication-is-bound-to-reviewed-inputs/contract.md).

<a id="f-29151ba5d332-Verification"></a>
### `Verification`

<storycode:copy id="f-29151ba5d332-Verification">
import { UiLink } from "../components/UiLink";
import { useEffect, useId, useMemo, useState, type ReactNode } from "react";
import { UiActionsVisibility, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { ExecBlock } from "../api/types";
import { DocumentEmbed } from "./DocumentEmbed";
import { CellPanel } from "../components/CellPanel";
import { sha256 } from "../lib/reviews";
import { PredictionEvidence, predictionWitnesses, validatePredictionHistory, type PredictionHistory, type PredictionRevisionNavigation } from "./PredictionHistory";

export interface Verification {
  status: "passed" | "failed" | "not_run" | "unavailable" | "stale";
  revision: string; source_hash: string; dependencies: Record<string, string>; assets: Record<string, string>;
  environment: Record<string, unknown>; commands: { command: string[]; exit_code?: number; stdout?: string; stderr?: string; error?: string }[];
  execution_evidence?: { from_document: string[]; recording_keys: unknown[]; coverage: string } | null;
  requirements: { id: string; text: string; witnesses: number[] }[]; origin: string; scope: string;
}
</storycode:copy>
<a id="f-29151ba5d332-ViewerSnapshot"></a>
### `ViewerSnapshot`

<storycode:copy id="f-29151ba5d332-ViewerSnapshot">
export interface ViewerSnapshot {
  demonstration?: { capture_kind: "native-provider"; captured_revision: string; recording_asset: string; session_asset?: string; cuts?: string[] };
  format: "hickory-viewer-snapshot"; version: 1; path: string; source: string;
  verification?: Verification;
  prediction_history?: PredictionHistory;
  source_hash: string; captured_at_ms: number; execution: string; cells: (ExecBlock & { saved_output?: string })[];
}
</storycode:copy>
<a id="f-29151ba5d332-decodeSnapshot"></a>
### `decodeSnapshot`

Validate external snapshots before supplying them to product components.

<storycode:copy id="f-29151ba5d332-decodeSnapshot">
/** Validate external snapshots before supplying them to product components. */
export async function decodeSnapshot(value: unknown): Promise<ViewerSnapshot> {
  if (!value || typeof value !== "object") throw new Error("Expected a Hickory viewer snapshot");
  const v = value as ViewerSnapshot;
  if (v.format !== "hickory-viewer-snapshot" || v.version !== 1 || typeof v.source !== "string" || typeof v.path !== "string" || typeof v.execution !== "string" || !Number.isFinite(v.captured_at_ms) || !Array.isArray(v.cells)) throw new Error("Unsupported viewer snapshot");
  if (await sha256(v.source) !== v.source_hash) throw new Error("Snapshot source hash does not match");
  if (v.prediction_history !== undefined) {
    validatePredictionHistory(v.prediction_history);
    if (v.cells.length || v.demonstration || Object.keys(v.verification?.assets ?? {}).length || v.verification?.execution_evidence ||
        v.verification?.commands.some(command => command.stdout || command.stderr || command.error)) throw Error("Selected prediction snapshots must withhold target diagnostics and assets");
  }
  if (v.demonstration) {
    const demo = v.demonstration;
    if (demo.capture_kind !== "native-provider" || !/^[a-f0-9]{40}$/.test(demo.captured_revision) || typeof demo.recording_asset !== "string" || !/\.(mp4|webm)$/.test(demo.recording_asset) || !v.verification?.assets?.[demo.recording_asset] || demo.session_asset !== undefined && !v.verification?.assets?.[demo.session_asset] || demo.cuts !== undefined && (!Array.isArray(demo.cuts) || demo.cuts.some(note => typeof note !== "string"))) throw Error("Invalid reviewed demonstration assets");
  }
  if (v.verification) {
    const check = v.verification;
    if (!["passed", "failed", "not_run", "unavailable", "stale"].includes(check.status) || !/^[a-f0-9]{40}$/.test(check.revision) || check.source_hash !== v.source_hash || !Array.isArray(check.commands) || !Array.isArray(check.requirements) || typeof check.origin !== "string" || typeof check.scope !== "string" || !check.dependencies || !check.environment) throw new Error("Invalid revision-bound verification");
    for (const requirement of check.requirements) if (typeof requirement.id !== "string" || typeof requirement.text !== "string" || !Array.isArray(requirement.witnesses) || requirement.witnesses.some(i => !Number.isInteger(i) || i < 0)) throw new Error("Invalid requirement witness");
    for (const command of check.commands) if (!Array.isArray(command.command) || command.command.some(arg => typeof arg !== "string") || command.stdout !== undefined && typeof command.stdout !== "string" || command.stderr !== undefined && typeof command.stderr !== "string" || command.error !== undefined && typeof command.error !== "string") throw new Error("Invalid verification command");
  }
  for (const cell of v.cells) {
    if (cell.kind !== "exec" || typeof cell.id !== "string" || typeof cell.container !== "string" || typeof cell.command !== "string" || !Array.isArray(cell.span) || cell.span.length !== 2 || !cell.span.every(Number.isFinite) || cell.status !== undefined && !["ok", "failed", "stale", "unrecorded"].includes(cell.status)) throw new Error("Invalid snapshot cell");
    if (cell.saved_output !== undefined && typeof cell.saved_output !== "string") throw new Error("Invalid saved output");
    if (cell.expect && (!["exact", "regex-lines"].includes(cell.expect.match) || typeof cell.expect.body !== "string")) throw new Error("Invalid expectation");
    if (cell.transcript && (!Array.isArray(cell.transcript) || cell.transcript.some(event => !Number.isFinite(event.t) || (event.kind === "exit" ? !Number.isFinite(event.code) : !["cmd", "out", "err"].includes(event.kind) || typeof event.data !== "string")))) throw new Error("Invalid transcript");
  }
  return v;
}
</storycode:copy>
<a id="f-29151ba5d332-verifiedPublicationAsset"></a>
### `verifiedPublicationAsset`

Shared by the snapshot renderer and homepage; publication selection owns assets.

<storycode:copy id="f-29151ba5d332-verifiedPublicationAsset">
/** Shared by the snapshot renderer and homepage; publication selection owns assets. */
export async function verifiedPublicationAsset(snapshot: ViewerSnapshot, artifactUrl: string, key: string, signal?: AbortSignal): Promise<string> {
  const expected = snapshot.verification?.assets?.[key];
  const unsafe = !key || /^[a-z][a-z\d+.-]*:/i.test(key) || /[\\?#]/.test(key) || key.startsWith("/") || key.split("/").some(part => {
    try { const decoded = decodeURIComponent(part); return decoded === "." || decoded === ".." || /[\\/]/.test(decoded); } catch { return true; }
  });
  if (!expected || !/^[a-f0-9]{64}$/.test(expected) || unsafe) throw Error("Asset not selected for publication");
  const base = new URL("./", artifactUrl);
  const url = new URL(key, base);
  if (url.origin !== location.origin || !url.pathname.startsWith(base.pathname)) throw Error("External asset origin refused");
  const response = await fetch(url, { signal });
  if (!response.ok) throw Error("Published asset unavailable");
  const bytes = await response.arrayBuffer();
  const hash = Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)), b => b.toString(16).padStart(2, "0")).join("");
  if (hash !== expected) throw Error("Published asset hash mismatch");
  return URL.createObjectURL(new Blob([bytes], { type: response.headers.get("Content-Type") ?? "application/octet-stream" }));
}
</storycode:copy>
<a id="f-29151ba5d332-SnapshotProps"></a>
### `SnapshotProps`

<storycode:copy id="f-29151ba5d332-SnapshotProps">
type SnapshotProps = { snapshot: ViewerSnapshot; artifactUrl?: string; predictionNavigation?: PredictionRevisionNavigation };
</storycode:copy>
<a id="f-29151ba5d332-PublishedSnapshot"></a>
### `PublishedSnapshot`

<storycode:copy id="f-29151ba5d332-PublishedSnapshot">
export function PublishedSnapshot(props: SnapshotProps) {
  const owner = useMemo(() => crypto.randomUUID(), [props.snapshot, props.artifactUrl]);
  return <CapturedPublishedSnapshot key={owner} {...props} />;
}
</storycode:copy>
<a id="f-29151ba5d332-CapturedPublishedSnapshot"></a>
### `CapturedPublishedSnapshot`

<storycode:copy id="f-29151ba5d332-CapturedPublishedSnapshot">
function CapturedPublishedSnapshot({ snapshot, artifactUrl, predictionNavigation }: SnapshotProps) {
  const assets = useMemo(() => ({ urls: new Set<string>(), disposed: false }), [snapshot, artifactUrl]);
  useEffect(() => { assets.disposed = false; return () => { assets.disposed = true; for (const url of assets.urls) URL.revokeObjectURL(url); assets.urls.clear(); }; }, [assets]);
  const host = useMemo(() => ({ resolveAsset: async (path: string, signal: AbortSignal) => {
    if (!artifactUrl || !snapshot.verification) throw Error("No published asset manifest available");
    const sourcePath = new URL(snapshot.path, "https://publication.invalid/");
    const relative = new URL(path, sourcePath);
    if (relative.origin !== sourcePath.origin) throw Error("External assets are not approved");
    const key = decodeURIComponent(relative.pathname.slice(1));
    const immutable = await verifiedPublicationAsset(snapshot, artifactUrl, key, signal);
    if (signal.aborted || assets.disposed) { URL.revokeObjectURL(immutable); throw Error("The originating publication capture was retired"); }
    assets.urls.add(immutable);
    return immutable;
  } }), [snapshot, artifactUrl, assets]);
  const [replay, setReplay] = useState(true);
  const parent = `publication:${useId()}`;
  const revision = useUiRevision(() => [snapshot, artifactUrl, replay]);
  useUiReading("published snapshot status", { path: snapshot.path, captured: new Date(snapshot.captured_at_ms).toLocaleString(), source_hash: snapshot.source_hash,
    historical: true, transcripts_shown: replay, verification: snapshot.verification ? { status: snapshot.verification.status, revision: snapshot.verification.revision,
      origin: snapshot.verification.origin, scope: snapshot.verification.scope, requirements: snapshot.verification.requirements } : null,
    cells: snapshot.cells.map(cell => ({ id: cell.id, status: cell.status ?? "unrecorded" })), scope: "Supplied historical results; no current repository, execution or authenticity certification." });
  useUiCommands([{ key: `${parent}:transcripts`, command: { label: "Set published execution transcript visibility", parent, subject: parent, parameters: { shown: { type: "boolean" } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }), execute: async args => { setReplay(args.shown as boolean); return { status: "completed", value: { transcripts_requested: args.shown, execution_requested: false } }; },
  } }]);
  return <main className="published-snapshot">
    <header className="published-snapshot__heading">
      <h1>{snapshot.path}</h1>
      {snapshot.verification && <p className={`published-snapshot__status published-snapshot__status--${snapshot.verification.status}`}>Verification: {snapshot.verification.status}</p>}
    </header>
    <details className="snapshot-provenance">
      <summary>Snapshot details · captured {new Date(snapshot.captured_at_ms).toLocaleString()} · historical</summary>
      <div className="snapshot-provenance__contents">
        <p>Source SHA-256: <code>{snapshot.source_hash}</code></p>
        <p>This read-only snapshot displays recorded results. Opening or exporting it does not execute checks. It does not certify the current repository or authenticate the supplied results.</p>
        {snapshot.verification && <section aria-label="Revision-bound verification">
      <h2>Verification details</h2>
      <UiLink href={`https://github.com/LoumTechnologies/hickory-docs/commit/${snapshot.verification.revision}`}>Tested revision {snapshot.verification.revision}</UiLink>
      <p>{snapshot.verification.origin}</p>
      <SnapshotDisclosure label="Recording/replay evidence" value={snapshot.verification.execution_evidence ?? "Recording origin unavailable"}><pre>{JSON.stringify(snapshot.verification.execution_evidence ?? "Recording origin unavailable", null, 2)}</pre></SnapshotDisclosure><p>{snapshot.verification.scope}</p>
      <p>Historical success certifies only these source/dependency hashes; changed inputs need a new run.</p>
      <SnapshotDisclosure label="Environment and dependency identities" value={{ environment: snapshot.verification.environment, dependencies: snapshot.verification.dependencies, assets: snapshot.verification.assets }}><pre>{JSON.stringify({ environment: snapshot.verification.environment, dependencies: snapshot.verification.dependencies, assets: snapshot.verification.assets }, null, 2)}</pre></SnapshotDisclosure>
      {snapshot.verification.requirements.map((r, n) => <section key={`${r.id}:${n}`} id={`requirement-${n}`}><h3>{r.id}</h3><p>{r.text}</p>{r.witnesses.map(i => snapshot.cells[i] ? <UiLink key={i} href={`#witness-${i}`}>Witness {snapshot.cells[i].id} </UiLink> : <p key={i}>Witness {i} unavailable · not verified</p>)}{snapshot.prediction_history && predictionWitnesses(snapshot.prediction_history, r.id).map(attempt => <UiLink key={attempt} href={`#prediction-${attempt}`}>Recorded prediction attempt {attempt}</UiLink>)}</section>)}
      {snapshot.verification.commands.map((c, i) => <SnapshotDisclosure key={i} initialOpen={snapshot.verification!.status !== "passed"} label={`${c.command.join(" ")} · ${c.exit_code === undefined ? "unavailable" : `exit ${c.exit_code}`}`} value={{ stdout: c.stdout ?? "", stderr: c.stderr ?? c.error ?? "" }}><pre>{c.stdout}</pre><pre>{c.stderr ?? c.error}</pre></SnapshotDisclosure>)}
        </section>}
      </div>
    </details>
    {snapshot.prediction_history && <PredictionEvidence value={snapshot.prediction_history} sourceHash={snapshot.source_hash} navigation={predictionNavigation} />}
    <DocumentEmbed source={snapshot.source} revision={snapshot.source_hash} path={snapshot.path} host={host} readOnly />
    <h2>Recorded cell results</h2>
    <label><input type="checkbox" checked={replay} onChange={event => setReplay(event.target.checked)} /> Show execution transcripts</label>
    {!snapshot.cells.length && <p>No execution results recorded.</p>}
    {snapshot.cells.map((cell, index) => <section key={`${cell.id}:${index}`} id={`witness-${index}`}><h3>{cell.id} · {cell.status ?? "unrecorded"}</h3><CellPanel block={cell} command={cell.command} replay={replay} recordedOutput={cell.saved_output} /></section>)}
  </main>;
}

</storycode:copy>
<a id="f-29151ba5d332-SnapshotDisclosure"></a>
### `SnapshotDisclosure`

<storycode:copy id="f-29151ba5d332-SnapshotDisclosure">
function SnapshotDisclosure({ label, value, children, initialOpen = false }: { label: string; value: unknown; children: ReactNode; initialOpen?: boolean }) {
  const [open, setOpen] = useState(initialOpen);
  const parent = `publication-evidence:${useId()}`;
  const revision = useUiRevision(() => [label, value, open]);
  useUiReading("publication evidence summary", { label, open });
  useUiCommands([{ key: parent, command: { label: `Set publication evidence visibility: ${label.slice(0, 256)}`, parent, subject: parent,
    parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }),
    execute: async args => { setOpen(args.open as boolean); return { status: "completed", value: { visibility_requested: args.open } }; },
  } }]);
  return <details open={open} onToggle={event => setOpen(event.currentTarget.open)}><summary>{label}</summary><UiActionsVisibility visible={open}><SnapshotPayload value={value} />{children}</UiActionsVisibility></details>;
}
</storycode:copy>
<a id="f-29151ba5d332-SnapshotPayload"></a>
### `SnapshotPayload`

<storycode:copy id="f-29151ba5d332-SnapshotPayload">
function SnapshotPayload({ value }: { value: unknown }) { useUiReading("disclosed publication evidence", value); return null; }
</storycode:copy>

### Assemble `apps/web/src/embed/PublishedSnapshot.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/PublishedSnapshot.tsx"><storycode:paste select="#f-29151ba5d332-Verification"/><storycode:paste select="#f-29151ba5d332-ViewerSnapshot"/><storycode:paste select="#f-29151ba5d332-decodeSnapshot"/><storycode:paste select="#f-29151ba5d332-verifiedPublicationAsset"/><storycode:paste select="#f-29151ba5d332-SnapshotProps"/><storycode:paste select="#f-29151ba5d332-PublishedSnapshot"/><storycode:paste select="#f-29151ba5d332-CapturedPublishedSnapshot"/><storycode:paste select="#f-29151ba5d332-SnapshotDisclosure"/><storycode:paste select="#f-29151ba5d332-SnapshotPayload"/></storycode:file>

<a id="output-643910ecf3fe"></a>
## `apps/web/src/embed/SnapshotPage.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A viewer snapshot keeps recorded results historical](contract.md).

<a id="f-643910ecf3fe-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-643910ecf3fe-expression_statement">
// Guarantees: docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { SnapshotPage } from "./SnapshotPage";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { sha256 } from "../lib/reviews";
vi.mock("./PublishedSnapshot", async importOriginal => {
  const actual = await importOriginal<typeof import("./PublishedSnapshot")>();
  return { ...actual, PublishedSnapshot: ({ snapshot }: { snapshot: { path: string } }) => <p>Captured {snapshot.path}</p> };
});
</storycode:copy>
<a id="f-643910ecf3fe-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-643910ecf3fe-expression_statement-2">
afterEach(() => { cleanup(); vi.unstubAllGlobals(); window.history.replaceState(null, "", "/"); });
</storycode:copy>
<a id="f-643910ecf3fe-fixture"></a>
### `fixture`

<storycode:copy id="f-643910ecf3fe-fixture">
async function fixture(path = "spec.md") {
  const source = `# ${path}`;
  return { format: "hickory-viewer-snapshot", version: 1, path, source, source_hash: await sha256(source), captured_at_ms: 1, execution: "fixture", cells: [] };
}
</storycode:copy>
<a id="f-643910ecf3fe-setup"></a>
### `setup`

<storycode:copy id="f-643910ecf3fe-setup">
function setup() {
  const registry = new UiActions({ workspace: "static", window: "viewer" }, () => true, async () => {});
  const app = render(<UiActionContext.Provider value={registry}><SnapshotPage /></UiActionContext.Provider>);
  const run = async (label: string, args: Record<string, string> = {}) => {
    const action = registry.observe("publication-viewer", 64).actions.find(action => action.label === label)!;
    let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: action.handle, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  return { registry, app, run };
}
</storycode:copy>
<a id="f-643910ecf3fe-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-643910ecf3fe-expression_statement-3">
it("shares validated native/agent import and refuses invalid source without replacing retained evidence", async () => {
  const captured = await fixture(), { run } = setup();
  expect(await run("Import exported snapshot JSON", { json: JSON.stringify(captured) })).toMatchObject({ status: "result", outcome: { status: "completed", value: { historical: true, execution_requested: false } } });
  expect(screen.getByText("Captured spec.md")).toBeTruthy();
  expect(await run("Import exported snapshot JSON", { json: JSON.stringify({ ...captured, source: "tampered" }) })).toMatchObject({ outcome: { status: "unavailable" } });
  expect(screen.getByText("Captured spec.md")).toBeTruthy();
  const next = await fixture("native.md");
  fireEvent.change(screen.getByLabelText("Open exported snapshot"), { target: { files: [{ text: async () => JSON.stringify(next) }] } });
  await screen.findByText("Captured native.md");
});
</storycode:copy>
<a id="f-643910ecf3fe-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-643910ecf3fe-expression_statement-4">
it("cancels a pending fetch without adopting a late response and rejects external URLs before fetching", async () => {
  let finish!: (value: Response) => void;
  const fetcher = vi.fn(() => new Promise<Response>(resolve => { finish = resolve; })); vi.stubGlobal("fetch", fetcher);
  const { registry, run } = setup();
  expect(await run("Load same-origin publication snapshot", { path: "https://external.example/capture.json" })).toMatchObject({ outcome: { status: "unavailable" } });
  expect(fetcher).not.toHaveBeenCalled();
  const action = registry.observe("publication-viewer", 64).actions.find(action => action.label === "Load same-origin publication snapshot")!;
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: action.handle, arguments: { path: "/capture.json" }, request_id: "pending" }); });
  expect(await run("Cancel publication loading")).toMatchObject({ outcome: { value: { cancellation_requested: true } } });
  const snapshot = await fixture();
  await act(async () => { finish(new Response(JSON.stringify(snapshot))); expect(await pending).toMatchObject({ outcome: { status: "unavailable" } }); });
  expect(screen.queryByText("Captured spec.md")).toBeNull();
});

</storycode:copy>
<a id="f-643910ecf3fe-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-643910ecf3fe-expression_statement-5">
it("explains a non-JSON publication response without exposing parser jargon", async () => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response("<!doctype html><title>not a snapshot</title>")));
  const { run } = setup();
  expect(await run("Load same-origin publication index", { path: "/publications/index.json" })).toMatchObject({ outcome: { status: "unavailable" } });
  expect((await screen.findByRole("alert")).textContent).toContain("This link did not return snapshot JSON");
  expect(screen.getByRole("alert").textContent).not.toContain("Unexpected token");
});

</storycode:copy>
<a id="f-643910ecf3fe-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-643910ecf3fe-expression_statement-6">
it("binds index navigation to the selected source and tested revision", async () => {
  const captured = await fixture(), revision = "a".repeat(40);
  const index = { version: 1, revision, documents: [{ path: "spec.md", status: "passed", snapshot: "capture.json", revision, source_hash: captured.source_hash }] };
  const wrong = { ...captured, verification: { status: "passed", revision: "b".repeat(40), source_hash: captured.source_hash, dependencies: {}, assets: {}, environment: {}, commands: [], requirements: [], origin: "fixture", scope: "fixture" } };
  const fetcher = vi.fn().mockResolvedValueOnce(new Response(JSON.stringify(index))).mockResolvedValueOnce(new Response(JSON.stringify(wrong))); vi.stubGlobal("fetch", fetcher);
  const { run } = setup();
  expect(await run("Load same-origin publication index", { path: "/publications/index.json" })).toMatchObject({ outcome: { status: "completed" } });
  expect(await run("Open published document 0: spec.md")).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("identities differ") } });
  expect(screen.queryByText("Captured spec.md")).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/embed/SnapshotPage.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/SnapshotPage.test.tsx"><storycode:paste select="#f-643910ecf3fe-expression_statement"/><storycode:paste select="#f-643910ecf3fe-expression_statement-2"/><storycode:paste select="#f-643910ecf3fe-fixture"/><storycode:paste select="#f-643910ecf3fe-setup"/><storycode:paste select="#f-643910ecf3fe-expression_statement-3"/><storycode:paste select="#f-643910ecf3fe-expression_statement-4"/><storycode:paste select="#f-643910ecf3fe-expression_statement-5"/><storycode:paste select="#f-643910ecf3fe-expression_statement-6"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/SnapshotExport.tsx](../../../mechanisms/apps/web/src/components/SnapshotExport.tsx.md#output-0eb707422fe3)
- [apps/web/src/embed/PublishedSnapshot.tsx](contract.md#output-29151ba5d332)
- [apps/web/src/embed/snapshot.tsx](../../../mechanisms/apps/web/src/embed/snapshot.tsx.md#output-e9d2d7727ec4)
- [apps/web/vite.site.config.ts](../../../mechanisms/apps/web/vite.site.config.ts.md#output-2cea88e563a6)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/embed/PublishedSnapshot.controls.test.tsx](contract.md#output-eaaafafcba7f)
- [apps/web/src/embed/PublishedSnapshot.test.ts](contract.md#output-48b9cf5d4873)
- [apps/web/src/embed/SnapshotPage.test.tsx](contract.md#output-643910ecf3fe)
- [crates/hickory-cli/tests/serve_blame.rs](../../../mechanisms/crates/hickory-cli/tests/serve_blame.rs.md#output-39e5027e9d27)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
