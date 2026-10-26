<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Published executable results identify reviewed inputs

For: Host developers embedding the document editor without a native execution service.

Given the explicit publication selection,
when CI generates public verification artifacts, then it copies only selected
source/dependencies/assets whose SHA-256 values match their recorded review.
It runs the existing native weave/test commands in an isolated temporary input
tree with an isolated home and no inherited provider credentials. It preserves
failed command details and still generates the static index and snapshot.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-a3ecda5eca43"></a>
## `apps/web/e2e/predictionPublication.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Published executable results identify reviewed inputs](contract.md).

<a id="f-a3ecda5eca43-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a3ecda5eca43-expression_statement">
import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { resolve, sep } from "node:path";

// Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
// Actual recorded host-process/DAP exports; static hosting is a test fixture.
test("inspect the recorded authorization pair without a backend and refuse missing evidence", async ({ page }) => {
  test.skip(!process.env.HICKORY_PREDICTION_PUBLICATIONS, "Generate the explicitly selected authorization publications first");
  const root = resolve(process.env.HICKORY_PREDICTION_PUBLICATIONS!);
  const index = JSON.parse(readFileSync(resolve(root, "index.json"), "utf8"));
  const api: string[] = [];
  let removeEvidence = false;
  await page.route(url => url.pathname.startsWith("/api/"), route => { api.push(route.request().url()); return route.abort(); });
  await page.route("**/publications/review/**", async route => {
    const relative = new URL(route.request().url()).pathname.split("/publications/review/")[1];
    const path = resolve(root, relative);
    if (!path.startsWith(root + sep)) return route.abort();
    const value = JSON.parse(readFileSync(path, "utf8"));
    if (removeEvidence && value.prediction_history) value.prediction_history.history.ordering = value.prediction_history.history.ordering.filter((entry: { event: { kind: string } }) => entry.event.kind !== "finished");
    return route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(value) });
  });
  for (const name of ["vulnerable", "patched"]) {
    const document = index.documents.find((entry: { path: string }) => entry.path.endsWith(`${name}.md`));
    expect(document).toBeTruthy();
    await page.goto("/snapshot.html?index=publications/review/index.json");
    await page.getByRole("link", { name: `${document.path} · ${document.status}`, exact: true }).click();
    await expect(page.getByRole("heading", { name: `Verification: ${name === "vulnerable" ? "failed" : "passed"}`, exact: true })).toBeVisible();
    await expect(page.getByRole("heading", { name: /^execution_rejected · attempt/ })).toBeHidden();
    const requirement = page.locator("#requirement-0");
    await requirement.getByRole("link", { name: /^Recorded prediction attempt/ }).first().click();
    await expect(page.getByRole("heading", { name: /^execution_rejected · attempt/ })).toBeVisible();
    await expect(page.getByRole("heading", { name: /^verified_after_correction · attempt/ })).toBeVisible();
    await expect(page.getByText(/matches this captured document/)).toHaveCount(1);
    await expect(page.getByText(/different revision; source bytes not included/)).toHaveCount(1);
    await expect(page.getByText(/Current certification: unavailable/)).toHaveCount(2);
    await expect(page.getByText(/recorded capture; browsing does not execute a debugger/)).toHaveCount(2);
    await expect(page.getByRole("link", { name: /^Tested revision/ })).toHaveAttribute("href", `https://github.com/LoumTechnologies/hickory-docs/commit/${document.revision}`);
    const other = index.documents.find((entry: { source_hash: string; path: string }) => entry.path.includes("security-authorization/") && entry.source_hash !== document.source_hash);
    await page.getByRole("button", { name: `Inspect captured source revision ${other.source_hash}`, exact: true }).click();
    await expect(page.getByRole("heading", { name: `Verification: ${other.status}`, exact: true })).toBeVisible();
  }
  removeEvidence = true;
  await page.goto(`/snapshot.html?src=${encodeURIComponent(`publications/review/${index.documents.at(-1).snapshot}`)}`);
  await expect(page.getByRole("alert")).toContainText("Invalid selected prediction history");
  await expect(page.getByRole("heading", { name: "Verification: passed", exact: true })).toHaveCount(0);
  expect(api).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/e2e/predictionPublication.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/predictionPublication.spec.ts"><storycode:paste select="#f-a3ecda5eca43-expression_statement"/></storycode:file>

<a id="output-e9c434f32625"></a>
## `apps/web/src/embed/PredictionHistory.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Published executable results identify reviewed inputs](contract.md).

<a id="f-e9c434f32625-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e9c434f32625-expression_statement">
// Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import recorded from "../../../../examples/security-authorization/predictions.json";
import { PredictionEvidence, validatePredictionHistory } from "./PredictionHistory";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);
</storycode:copy>
<a id="f-e9c434f32625-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-e9c434f32625-expression_statement-2">
it("accepts the recorded host pair and refuses raw answers, stale certification and missing seals", () => {
  validatePredictionHistory(recorded);
  expect(recorded.history.attempts.map(attempt => attempt.classification)).toEqual(["execution_rejected", "verified_after_correction"]);
  for (const corrupt of [
    (value: typeof recorded) => { value.history.ordering.find(entry => entry.event.kind === "finished")!.event.actual = "PRIVATE ANSWER"; },
    (value: typeof recorded) => { value.history.attempts[1].current_certification = true; },
    (value: typeof recorded) => { value.history.attempts[1].seal = "missing"; },
    (value: typeof recorded) => { value.history.ordering[1].sequence = value.history.ordering[0].sequence; },
    (value: typeof recorded) => { value.history.ordering = value.history.ordering.filter(entry => entry.event.kind !== "finished"); },
  ]) {
    const copy = structuredClone(recorded); corrupt(copy);
    expect(() => validatePredictionHistory(copy)).toThrow("Invalid selected prediction history");
  }
  expect(() => validatePredictionHistory({ ...recorded, actual: "PRIVATE ANSWER" })).toThrow();
});
</storycode:copy>
<a id="f-e9c434f32625-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-e9c434f32625-expression_statement-3">
it("discloses both revisions and reports a different source without certifying a current pass", async () => {
  validatePredictionHistory(recorded);
  const registry = new UiActions({ workspace: "published", window: "viewer" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PredictionEvidence value={recorded} sourceHash={"f".repeat(64)} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  expect(screen.getByText("Inspect sealed predictions and retained attempts").closest("details")!.open).toBe(false);
  expect(actions().some(action => action.label === "Read disclosed selected prediction chain bytes")).toBe(false);
  const disclosure = registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Set selected prediction history visibility")!;
  await act(async () => { await registry.act(registry.scope, { kind: "agent", id: "viewer-test" }, { handle: disclosure.handle, arguments: { open: true }, request_id: "inspect" }); });
  expect(screen.getByText(/execution_rejected · attempt/)).toBeTruthy();
  expect(screen.getByText(/verified_after_correction · attempt/)).toBeTruthy();
  expect(screen.getAllByText(/different revision; source bytes not included/)).toHaveLength(2);
  expect(screen.getAllByText(/Current certification: unavailable/)).toHaveLength(2);
  const details = screen.getByText("Inspect sealed predictions and retained attempts").closest("details")!;
  details.open = false; fireEvent(details, new Event("toggle"));
  expect(details.open).toBe(false);
  expect(actions().some(action => action.label === "Read disclosed selected prediction chain bytes")).toBe(false);
});

</storycode:copy>
<a id="f-e9c434f32625-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-e9c434f32625-expression_statement-4">
it("retains answer-free future-stop coordinates while refusing embedded prior visible values", () => {
  const copy = structuredClone(recorded) as unknown as Record<string, unknown>;
  const history = copy.history as { ordering: {event: Record<string, unknown>}[] };
  const event = history.ordering.find(entry => entry.event.kind === "sealed")!.event;
  event.future_stop = { run: "retained-host-run", thread: 1, frame: 2, hit: 1, prior_visible_digest: "a".repeat(64) };
  validatePredictionHistory(copy);
  (event.future_stop as Record<string, unknown>).prior_visible = "PRIVATE PREVIOUS STATE";
  expect(() => validatePredictionHistory(copy)).toThrow();
});
</storycode:copy>

### Assemble `apps/web/src/embed/PredictionHistory.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/PredictionHistory.test.tsx"><storycode:paste select="#f-e9c434f32625-expression_statement"/><storycode:paste select="#f-e9c434f32625-expression_statement-2"/><storycode:paste select="#f-e9c434f32625-expression_statement-3"/><storycode:paste select="#f-e9c434f32625-expression_statement-4"/></storycode:file>

<a id="output-1c9757635083"></a>
## `apps/web/src/landing/WorkflowRecording.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Published executable results identify reviewed inputs](contract.md).

<a id="f-1c9757635083-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1c9757635083-expression_statement">
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Test runtime has Node crypto; browser code does not import it.
import { webcrypto } from "node:crypto";
import { WorkflowRecording } from "./WorkflowRecording";
import { sha256 } from "../lib/reviews";
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });
</storycode:copy>
<a id="f-1c9757635083-setup"></a>
### `setup`

<storycode:copy id="f-1c9757635083-setup">
async function setup(recording = false, wrongRevision = false) {
  vi.stubGlobal("crypto", webcrypto);
  const source = "# Public requirement\n", hash = await sha256(source);
  const snapshot = { format: "hickory-viewer-snapshot", version: 1, path: "examples/ai-engineer-deadline.md", source, source_hash: hash, captured_at_ms: 1, execution: "Test fixture", cells: [], verification: { status: "failed", revision: "a".repeat(40), source_hash: hash, commands: [], requirements: [], origin: "Test fixture", scope: "Test fixture", dependencies: {}, assets: {}, environment: {} }, ...(recording ? { demonstration: { capture_kind: "native-provider", captured_revision: "b".repeat(40), recording_asset: "unselected.webm" } } : {}) };
  const fetcher = vi.fn().mockResolvedValueOnce(new Response(JSON.stringify({ version: 1, documents: [{ path: snapshot.path, snapshot: "selected/snapshot.json", source_hash: hash, revision: (wrongRevision ? "c" : "a").repeat(40) }] }), { status: 200 })).mockResolvedValueOnce(new Response(JSON.stringify(snapshot), { status: 200 }));
  vi.stubGlobal("fetch", fetcher); return fetcher;
}
</storycode:copy>
<a id="f-1c9757635083-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-1c9757635083-expression_statement-2">
// Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
it("does not manufacture a recording or turn a published failure into a pass", async () => {
  const fetcher = await setup(); render(<WorkflowRecording />);
  expect(fetcher).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Load reviewed workflow recording" }));
  await screen.findByText(/A real native\/provider recording has not been published/);
  expect(screen.getByText(/result failed/)).toBeTruthy();
  expect(document.querySelector("video")).toBeNull(); expect(fetcher).toHaveBeenCalledTimes(2);
});
</storycode:copy>
<a id="f-1c9757635083-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-1c9757635083-expression_statement-3">
it("rejects media absent from the reviewed manifest before downloading it", async () => {
  const fetcher = await setup(true); render(<WorkflowRecording />);
  fireEvent.click(screen.getByRole("button", { name: "Load reviewed workflow recording" }));
  await screen.findByText(/Invalid reviewed demonstration assets/);
  expect(document.querySelector("video")).toBeNull(); expect(fetcher).toHaveBeenCalledTimes(2);
});
</storycode:copy>
<a id="f-1c9757635083-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-1c9757635083-expression_statement-4">
it("rejects mismatched publication revisions", async () => {
  const fetcher = await setup(false, true); render(<WorkflowRecording />);
  fireEvent.click(screen.getByRole("button", { name: "Load reviewed workflow recording" }));
  await screen.findByText(/Publication index and snapshot revisions differ/);
  expect(document.querySelector("video")).toBeNull(); expect(fetcher).toHaveBeenCalledTimes(2);
});
</storycode:copy>

### Assemble `apps/web/src/landing/WorkflowRecording.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/landing/WorkflowRecording.test.tsx"><storycode:paste select="#f-1c9757635083-expression_statement"/><storycode:paste select="#f-1c9757635083-setup"/><storycode:paste select="#f-1c9757635083-expression_statement-2"/><storycode:paste select="#f-1c9757635083-expression_statement-3"/><storycode:paste select="#f-1c9757635083-expression_statement-4"/></storycode:file>

<a id="output-3bd89dadc75d"></a>
## `scripts/publish-spec-results.py`

Related contracts: [Published executable results identify reviewed inputs](contract.md).

<a id="f-3bd89dadc75d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3bd89dadc75d-expression_statement">
#!/usr/bin/env python3
"""Publish only explicitly selected, reviewed material using the existing CLI/viewer.

No globbing, sessions, credentials or implicit assets are copied. Run in CI or a
clean checkout. Each command runs in a temporary copy of selected inputs.
"""
</storycode:copy>
<a id="f-3bd89dadc75d-argparse"></a>
### `argparse`

<storycode:copy id="f-3bd89dadc75d-argparse">
import argparse
</storycode:copy>
<a id="f-3bd89dadc75d-datetime"></a>
### `datetime`

<storycode:copy id="f-3bd89dadc75d-datetime">
from datetime import datetime, timezone
</storycode:copy>
<a id="f-3bd89dadc75d-hashlib"></a>
### `hashlib`

<storycode:copy id="f-3bd89dadc75d-hashlib">
import hashlib
</storycode:copy>
<a id="f-3bd89dadc75d-json"></a>
### `json`

<storycode:copy id="f-3bd89dadc75d-json">
import json
</storycode:copy>
<a id="f-3bd89dadc75d-os"></a>
### `os`

<storycode:copy id="f-3bd89dadc75d-os">
import os
</storycode:copy>
<a id="f-3bd89dadc75d-Path"></a>
### `Path`

<storycode:copy id="f-3bd89dadc75d-Path">
from pathlib import Path
</storycode:copy>
<a id="f-3bd89dadc75d-platform"></a>
### `platform`

<storycode:copy id="f-3bd89dadc75d-platform">
import platform
</storycode:copy>
<a id="f-3bd89dadc75d-subprocess"></a>
### `subprocess`

<storycode:copy id="f-3bd89dadc75d-subprocess">
import subprocess
</storycode:copy>
<a id="f-3bd89dadc75d-tempfile"></a>
### `tempfile`

<storycode:copy id="f-3bd89dadc75d-tempfile">
import tempfile
</storycode:copy>
<a id="f-3bd89dadc75d-importlib-util"></a>
### `importlib.util`

<storycode:copy id="f-3bd89dadc75d-importlib-util">
import importlib.util

</storycode:copy>
<a id="f-3bd89dadc75d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3bd89dadc75d-expression_statement-2">
_prediction_spec = importlib.util.spec_from_file_location('prediction_publication', Path(__file__).with_name('prediction-publication.py'))
</storycode:copy>
<a id="f-3bd89dadc75d-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-3bd89dadc75d-expression_statement-3">
_prediction_module = importlib.util.module_from_spec(_prediction_spec)
</storycode:copy>
<a id="f-3bd89dadc75d-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-3bd89dadc75d-expression_statement-4">
_prediction_spec.loader.exec_module(_prediction_module)


</storycode:copy>
<a id="f-3bd89dadc75d-digest"></a>
### `digest`

<storycode:copy id="f-3bd89dadc75d-digest">
def digest(data):
    return hashlib.sha256(data).hexdigest()


</storycode:copy>
<a id="f-3bd89dadc75d-demonstration"></a>
### `demonstration`

<storycode:copy id="f-3bd89dadc75d-demonstration">
def demonstration(entry, assets):
    value = entry.get('demonstration')
    if value is None:
        return None
    if not isinstance(value, dict) or value.get('reviewed') is not True:
        raise ValueError('A demonstration requires explicit review')
    if value.get('capture_kind') != 'native-provider':
        raise ValueError('Launch recordings must declare an actual native/provider capture')
    revision = value.get('captured_revision', '')
    if not isinstance(revision, str) or len(revision) != 40 or any(c not in '0123456789abcdef' for c in revision):
        raise ValueError('Recording needs its captured build revision')
    recording = value.get('recording_asset')
    if not isinstance(recording, str) or recording not in assets or not recording.endswith(('.mp4', '.webm')):
        raise ValueError('Recording must be a separately selected and hash-reviewed video asset')
    session = value.get('session_asset')
    if session is not None and (not isinstance(session, str) or session not in assets):
        raise ValueError('A shared session must be an explicitly reviewed public export')
    notes = value.get('cuts', [])
    if not isinstance(notes, list) or any(not isinstance(note, str) for note in notes):
        raise ValueError('Recording cuts must be declared')
    return {k: value[k] for k in ('capture_kind', 'captured_revision', 'recording_asset', 'session_asset', 'cuts') if k in value}


</storycode:copy>
<a id="f-3bd89dadc75d-selected"></a>
### `selected`

<storycode:copy id="f-3bd89dadc75d-selected">
def selected(root, relative):
    if not isinstance(relative, str) or Path(relative).is_absolute() or '..' in Path(relative).parts:
        raise ValueError('Publication inputs must be workspace-relative paths without parent traversal')
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()) or not path.is_file():
        raise ValueError(f'Publication input is unavailable or outside root: {relative}')
    if any(part in ('sessions', '.git', '.hick-cache', '.aws', '.ssh') or part.startswith('.env') for part in Path(relative).parts):
        raise ValueError(f'Private/session input is not publishable: {relative}')
    return path


</storycode:copy>
<a id="f-3bd89dadc75d-publish"></a>
### `publish`

<storycode:copy id="f-3bd89dadc75d-publish">
def publish(root, selection, destination, hick, runner=subprocess.run):
    if selection.get('version') != 1:
        raise ValueError('Unsupported publication selection')
    revision = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
    runner_hash = None
    if hick.is_file():
        hasher = hashlib.sha256()
        with hick.open('rb') as binary:
            for chunk in iter(lambda: binary.read(1024 * 1024), b''):
                hasher.update(chunk)
        runner_hash = hasher.hexdigest()
    index = dict(version=1, revision=revision, documents=[])
    destination.mkdir(parents=True, exist_ok=True)
    failed = False
    for entry in selection['documents']:
        if entry.get('reviewed_output') is not True:
            raise ValueError('Each published document/output must be explicitly reviewed')
        path = entry['path']
        expected_status = entry.get('expected_status', 'passed')
        if expected_status not in ('passed', 'failed') or expected_status == 'failed' and not entry.get('expected_status_reason'):
            raise ValueError('An expected negative fixture needs an explicit reviewed reason; unavailable/stale evidence cannot be waived')
        source = selected(root, path).read_bytes()
        inputs = {path: source}
        for p in [*entry.get('dependencies', []), *entry.get('assets', [])]:
            inputs[p] = selected(root, p).read_bytes()
        prediction_path = entry.get('prediction_history')
        if prediction_path is not None:
            if prediction_path in inputs:
                raise ValueError('Prediction history must be selected separately from executable inputs/assets')
            inputs[prediction_path] = selected(root, prediction_path).read_bytes()
        if any(entry.get('reviewed_inputs', {}).get(p) != digest(data) for p, data in inputs.items()):
            raise ValueError('Selected input changed since publication review; review bytes and update reviewed_inputs before publishing')
        dependencies = {p: digest(inputs[p]) for p in entry.get('dependencies', [])}
        assets = {p: digest(inputs[p]) for p in entry.get('assets', [])}
        demo = demonstration(entry, assets)
        predictions = _prediction_module.selected_history(json.loads(inputs[prediction_path])) if prediction_path is not None else None
        if predictions is not None and (assets or demo):
            raise ValueError('Answer-withheld prediction publication cannot include media or other assets')
        identity = digest(path.encode())[:16]
        with tempfile.TemporaryDirectory(prefix='hickory-publication-') as temporary:
            scratch = Path(temporary)
            for p in [path, *dependencies, *assets]:
                target = scratch / p
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(inputs[p])
            # These selected examples run locally; no inherited keys/provider config.
            environment = {'PATH': os.environ.get('PATH', ''), 'HOME': str(scratch / 'home'),
                           'HICKORY_EXECUTOR': 'local', 'HICKORY_ENGINE_WORKER': '1', 'LANG': 'C.UTF-8'}
            commands = [] if predictions is not None else [[str(hick), 'weave', path], [str(hick), 'test', path, '--json']]
            results = []
            blocks = []
            execution_evidence = None
            status = _prediction_module.reported_status(predictions, digest(source)) if predictions is not None else 'not_run'
            try:
                for command in commands:
                    result = runner(command, cwd=scratch, env=environment, capture_output=True, text=True, timeout=180)
                    results.append(dict(command=command, exit_code=result.returncode,
                                        stdout=result.stdout, stderr=result.stderr))
                    if command[1] == 'weave' and result.returncode:
                        status = 'unavailable'
                        break
                    if command[1] == 'test':
                        status = {0: 'passed', 1: 'stale', 2: 'unavailable', 3: 'failed'}.get(result.returncode, 'failed')
                        try:
                            model = json.loads(result.stdout)
                            model = model[0] if isinstance(model, list) else model
                            blocks = model['blocks']
                            execution_evidence = model.get('execution_evidence')
                        except (ValueError, KeyError, IndexError, TypeError):
                            if result.returncode == 0:
                                status = 'unavailable'
            except (OSError, subprocess.TimeoutExpired) as error:
                status = 'unavailable'
                results.append(dict(command=commands[len(results)], error=str(error)))
            cells = [b for b in blocks if b.get('kind') == 'exec']
            if status == 'passed' and predictions is None:
                if not cells:
                    status = 'not_run'
                elif any(not isinstance(i, int) or i < 0 or i >= len(cells) or not cells[i].get('expect') for r in entry.get('requirements', []) for i in r['witnesses']):
                    status = 'unavailable'
            failed |= status != expected_status
            text = source.decode('utf-8')
            snapshot = dict(format='hickory-viewer-snapshot', version=1, path=path,
                source=text, source_hash=digest(source), captured_at_ms=int(datetime.now(timezone.utc).timestamp()*1000),
                execution='CI verification command executed; see exact command/outcome. Cell playback origin is unavailable unless separately recorded.',
                cells=cells,
                verification=dict(status=status, revision=revision, source_hash=digest(source), dependencies=dependencies,
                    assets=assets, execution_evidence=execution_evidence, environment=dict(platform=platform.platform(), executor='local', runner_sha256=runner_hash, inherited_credentials=False),
                    commands=results, requirements=entry.get('requirements', []),
                    origin='executed-command; per-cell recorded/replayed origin unavailable',
                    scope='Only listed witnesses; deterministic allocation is not real network timing or arbitrary prose verification.'))
            if demo is not None:
                snapshot['demonstration'] = demo
            if predictions is not None:
                snapshot['prediction_history'] = predictions
                snapshot['cells'] = []
                snapshot['verification']['commands'] = [{k: result[k] for k in ('command', 'exit_code') if k in result} for result in results]
                snapshot['verification']['execution_evidence'] = None
                snapshot['verification']['origin'] = 'recorded host-ledger comparisons; no publication-time target execution'
                snapshot['verification']['environment'] = dict(publication_platform=platform.platform(), comparison_environment='See independently captured sealed runtime identities; publication executes no target', inherited_credentials=False)
                snapshot['execution'] = 'Recorded private output/debugger comparisons; publication does not execute the target or expose actual values.'
                snapshot['verification']['scope'] += ' Runtime transcripts, target values, errors and assets are intentionally withheld. Selected prediction verdicts are historical host reports, not current certification.'
            if expected_status == 'failed':
                snapshot['verification']['scope'] += ' Expected negative fixture: ' + entry['expected_status_reason']
            # Asset references remain relative to the original document path.
            # Host an isolated publication subtree and use a matching snapshot path.
            artifact = destination / identity
            artifact.mkdir(exist_ok=True)
            for p in assets:
                output = artifact / p
                output.parent.mkdir(parents=True, exist_ok=True)
                output.write_bytes(inputs[p])
            filename = f'{identity}/snapshot.json'
            (destination / filename).write_text(json.dumps(snapshot, indent=2) + '\n')
            index['documents'].append(dict(path=path, status=status, expected_status=expected_status, snapshot=filename, revision=revision,
                                            source_hash=digest(source)))
    (destination / 'index.json').write_text(json.dumps(index, indent=2) + '\n')
    return not failed


</storycode:copy>
<a id="f-3bd89dadc75d-main"></a>
### `main`

<storycode:copy id="f-3bd89dadc75d-main">
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--selection', type=Path, default=Path('publications/selected.json'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--hick', type=Path, required=True)
    args = parser.parse_args()
    root = Path.cwd().resolve()
    success = publish(root, json.loads(args.selection.read_text()), args.output, args.hick.resolve())
    raise SystemExit(0 if success else 1)


</storycode:copy>
<a id="f-3bd89dadc75d-if_statement"></a>
### `if_statement`

<storycode:copy id="f-3bd89dadc75d-if_statement">
if __name__ == '__main__':
    main()
</storycode:copy>

### Assemble `scripts/publish-spec-results.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/publish-spec-results.py"><storycode:paste select="#f-3bd89dadc75d-expression_statement"/><storycode:paste select="#f-3bd89dadc75d-argparse"/><storycode:paste select="#f-3bd89dadc75d-datetime"/><storycode:paste select="#f-3bd89dadc75d-hashlib"/><storycode:paste select="#f-3bd89dadc75d-json"/><storycode:paste select="#f-3bd89dadc75d-os"/><storycode:paste select="#f-3bd89dadc75d-Path"/><storycode:paste select="#f-3bd89dadc75d-platform"/><storycode:paste select="#f-3bd89dadc75d-subprocess"/><storycode:paste select="#f-3bd89dadc75d-tempfile"/><storycode:paste select="#f-3bd89dadc75d-importlib-util"/><storycode:paste select="#f-3bd89dadc75d-expression_statement-2"/><storycode:paste select="#f-3bd89dadc75d-expression_statement-3"/><storycode:paste select="#f-3bd89dadc75d-expression_statement-4"/><storycode:paste select="#f-3bd89dadc75d-digest"/><storycode:paste select="#f-3bd89dadc75d-demonstration"/><storycode:paste select="#f-3bd89dadc75d-selected"/><storycode:paste select="#f-3bd89dadc75d-publish"/><storycode:paste select="#f-3bd89dadc75d-main"/><storycode:paste select="#f-3bd89dadc75d-if_statement"/></storycode:file>

<a id="output-d6d3bf5aa2fb"></a>
## `scripts/test-publish-spec-results.py`

Related contracts: [Published executable results identify reviewed inputs](contract.md).

<a id="f-d6d3bf5aa2fb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d6d3bf5aa2fb-expression_statement">
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch
spec = importlib.util.spec_from_file_location('publisher', Path(__file__).with_name('publish-spec-results.py'))
</storycode:copy>
<a id="f-d6d3bf5aa2fb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d6d3bf5aa2fb-expression_statement-2">
publisher = importlib.util.module_from_spec(spec)
</storycode:copy>
<a id="f-d6d3bf5aa2fb-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d6d3bf5aa2fb-expression_statement-3">
spec.loader.exec_module(publisher)

</storycode:copy>
<a id="f-d6d3bf5aa2fb-PublicationTests"></a>
### `PublicationTests`

<storycode:copy id="f-d6d3bf5aa2fb-PublicationTests">
class PublicationTests(unittest.TestCase):
    # Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
    def test_prediction_projection_withholds_command_answers_and_requires_reviewed_export(self):
        recorded = json.loads((Path(__file__).resolve().parents[1] / 'examples/security-authorization/predictions.json').read_text())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = b'# Selected source\n'
            history = json.dumps(recorded).encode()
            (root/'spec.md').write_bytes(source); (root/'predictions.json').write_bytes(history)
            selection = dict(version=1, documents=[dict(path='spec.md', reviewed_output=True, prediction_history='predictions.json', reviewed_inputs={'spec.md':publisher.digest(source), 'predictions.json':publisher.digest(history)})])
            def run(command, **kwargs):
                self.fail('Recorded prediction publication must not execute the held-out target')
            with patch.object(publisher.subprocess,'check_output',return_value='a'*40):
                publisher.publish(root,selection,root/'out',Path('/hick'),run)
                index=json.loads((root/'out/index.json').read_text())
                snapshot=json.loads((root/'out'/index['documents'][0]['snapshot']).read_text())
                self.assertEqual(snapshot['prediction_history'], recorded)
                self.assertEqual(snapshot['cells'], [])
                self.assertNotIn('PRIVATE HELD-OUT ANSWER',json.dumps(snapshot))
                self.assertNotIn('stdout',json.dumps(snapshot['verification']['commands']))
                (root/'predictions.json').write_text('{}')
                with self.assertRaises(ValueError): publisher.publish(root,selection,root/'changed',Path('/hick'),run)

    def test_recorded_authorization_pair_keeps_the_expected_failure_and_exact_source_identities(self):
        examples=Path(__file__).resolve().parents[1] / 'examples/security-authorization'
        history=(examples/'predictions.json').read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); (root/'predictions.json').write_bytes(history)
            documents=[]
            for name,status in [('vulnerable','failed'),('patched','passed')]:
                source=(examples/f'{name}.md').read_bytes(); path=f'{name}.md';(root/path).write_bytes(source)
                documents.append(dict(path=path, reviewed_output=True, prediction_history='predictions.json', expected_status=status,
                    expected_status_reason='Synthetic cross-tenant predicate must fail before the patch' if status=='failed' else '',
                    reviewed_inputs={path:publisher.digest(source),'predictions.json':publisher.digest(history)}))
            with patch.object(publisher.subprocess,'check_output',return_value='a'*40):
                self.assertTrue(publisher.publish(root,dict(version=1,documents=documents),root/'out',Path('/hick'),lambda *_args,**_kwargs:self.fail('No target execution')))
                index=json.loads((root/'out/index.json').read_text())
                self.assertEqual([doc['status'] for doc in index['documents']],['failed','passed'])
                self.assertNotEqual(index['documents'][0]['source_hash'],index['documents'][1]['source_hash'])
                # Declaring an expected failure cannot waive unavailable source evidence.
                source=b'# Changed without matching prediction evidence\n';(root/'vulnerable.md').write_bytes(source)
                documents[0]['reviewed_inputs']['vulnerable.md']=publisher.digest(source)
                self.assertFalse(publisher.publish(root,dict(version=1,documents=documents),root/'stale',Path('/hick')))

    def test_future_stop_projection_retains_coordinates_without_prior_visible_state(self):
        recorded=json.loads((Path(__file__).resolve().parents[1] / 'examples/security-authorization/predictions.json').read_text())
        event=next(entry['event'] for entry in recorded['history']['ordering'] if entry['event']['kind']=='sealed')
        event['future_stop']=dict(run='retained-host-run',thread=1,frame=2,hit=1,prior_visible_digest='a'*64)
        publisher._prediction_module.selected_history(recorded)
        event['future_stop']['prior_visible']='PRIVATE PREVIOUS STATE'
        with self.assertRaises(ValueError): publisher._prediction_module.selected_history(recorded)

    def test_prediction_selection_refuses_private_actual_and_unknown_payload(self):
        recorded=json.loads((Path(__file__).resolve().parents[1] / 'examples/security-authorization/predictions.json').read_text())
        changed=json.loads(json.dumps(recorded))
        next(entry['event'] for entry in changed['history']['ordering'] if entry['event']['kind']=='finished')['actual']='PRIVATE ANSWER'
        with self.assertRaises(ValueError): publisher._prediction_module.selected_history(changed)
        with self.assertRaises(ValueError): publisher._prediction_module.selected_history({**recorded, 'private_evidence':'PRIVATE ANSWER'})
        missing=json.loads(json.dumps(recorded))
        missing['history']['ordering']=[entry for entry in missing['history']['ordering'] if entry['event']['kind']!='finished']
        with self.assertRaises(ValueError): publisher._prediction_module.selected_history(missing)
        if recorded.get('debugger_contexts'):
            context=json.loads(json.dumps(recorded))
            context['debugger_contexts'][0]['hits'][0]['values']=['PRIVATE TARGET']
            with self.assertRaises(ValueError): publisher._prediction_module.selected_history(context)
    def test_recordings_reuse_selected_reviewed_assets_without_implicit_sessions(self):
        video = 'examples/public-demo.webm'
        demo = dict(reviewed=True, capture_kind='native-provider', captured_revision='a'*40, recording_asset=video, cuts=['One declared cut'])
        self.assertEqual(publisher.demonstration(dict(demonstration=demo), {video:'b'*64})['recording_asset'], video)
        for change in (dict(reviewed=False), dict(capture_kind='browser-replay'), dict(recording_asset='sessions/private.webm'), dict(session_asset='sessions/private.md'), dict(captured_revision=123), dict(recording_asset=[])):
            with self.assertRaises(ValueError): publisher.demonstration(dict(demonstration={**demo, **change}), {video:'b'*64})
        self.assertIsNone(publisher.demonstration({}, {}))
    # Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
    def test_failed_run_publishes_details_hashes_and_missing_witness(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'spec.md').write_text('# Requirement\n'); (root / 'lock').write_text('deps')
            selection = dict(version=1, documents=[dict(path='spec.md', reviewed_output=True, dependencies=['lock'], reviewed_inputs={'spec.md':publisher.digest(b'# Requirement\n'),'lock':publisher.digest(b'deps')}, requirements=[dict(id='R1', text='budget', witnesses=[9])])])
            def run(command, **kwargs):
                self.assertNotIn('OPENAI_API_KEY', kwargs['env'])
                self.assertNotEqual(kwargs['cwd'], root)
                return SimpleNamespace(returncode=0 if command[1] == 'weave' else 3,
                    stdout=json.dumps([dict(blocks=[])]), stderr='expected 100; actual 160')
            with patch.object(publisher.subprocess, 'check_output', return_value='a'*40):
                self.assertFalse(publisher.publish(root, selection, root/'out', Path('/hick'), run))
            index = json.loads((root/'out/index.json').read_text())
            snapshot = json.loads((root/'out'/index['documents'][0]['snapshot']).read_text())
            self.assertEqual(index['documents'][0]['status'], 'failed')
            self.assertIn('actual 160', snapshot['verification']['commands'][-1]['stderr'])
            self.assertEqual(snapshot['source_hash'], publisher.digest(b'# Requirement\n'))
            self.assertEqual(snapshot['verification']['dependencies']['lock'], publisher.digest(b'deps'))
    # Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
    def test_explicit_selection_rejects_escape_private_input_and_unreviewed_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); (root/'spec.md').write_text('x'); (root/'.env').write_text('secret')
            for path in ('../escape', str(root/'spec.md'), '.env'):
                with self.assertRaises(ValueError): publisher.selected(root,path)
            with patch.object(publisher.subprocess,'check_output',return_value='b'*40):
                with self.assertRaises(ValueError): publisher.publish(root,dict(version=1,documents=[dict(path='spec.md')]),root/'out',Path('/hick'))

</storycode:copy>
<a id="f-d6d3bf5aa2fb-if_statement"></a>
### `if_statement`

<storycode:copy id="f-d6d3bf5aa2fb-if_statement">
if __name__ == '__main__': unittest.main()
</storycode:copy>

### Assemble `scripts/test-publish-spec-results.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/test-publish-spec-results.py"><storycode:paste select="#f-d6d3bf5aa2fb-expression_statement"/><storycode:paste select="#f-d6d3bf5aa2fb-expression_statement-2"/><storycode:paste select="#f-d6d3bf5aa2fb-expression_statement-3"/><storycode:paste select="#f-d6d3bf5aa2fb-PublicationTests"/><storycode:paste select="#f-d6d3bf5aa2fb-if_statement"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/embed/PublishedSnapshot.tsx](../a-viewer-snapshot-keeps-recorded-results-historical/contract.md#output-29151ba5d332)
- [apps/web/src/embed/snapshot.tsx](../../../mechanisms/apps/web/src/embed/snapshot.tsx.md#output-e9d2d7727ec4)
- [scripts/publish-spec-results.py](contract.md#output-3bd89dadc75d)
- [scripts/test-publish-spec-results.py](contract.md#output-d6d3bf5aa2fb)

### Tests and fixtures

- [apps/web/e2e/predictionPublication.spec.ts](contract.md#output-a3ecda5eca43)
- [apps/web/src/embed/PredictionHistory.test.tsx](contract.md#output-e9c434f32625)
- [apps/web/src/landing/WorkflowRecording.test.tsx](contract.md#output-1c9757635083)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

## Focused CI witness

This check exercises refusal of escaping paths, private inputs and unreviewed
publication output. It is one named test, not certification of every publication
requirement. The project policy selects its stable ID; the command stays here
beside the behavior it checks.

<storycode:copy id="ci-publication-witness">
{
  "version": 1,
  "checks": [
    {
      "id": "publication-input-safety",
      "owner": ".hickory-source/guarantees/embedding/publication-is-bound-to-reviewed-inputs/contract.md",
      "cwd": ".",
      "argv": [
        "{python}",
        "scripts/test-publish-spec-results.py",
        "PublicationTests.test_explicit_selection_rejects_escape_private_input_and_unreviewed_output"
      ],
      "needs": [],
      "assertion": "exit-zero",
      "timeout_seconds": 30
    }
  ]
}
</storycode:copy>
<storycode:file path="ci/checks/publication.json"><storycode:paste select="#ci-publication-witness"/></storycode:file>

</storycode:doc>
