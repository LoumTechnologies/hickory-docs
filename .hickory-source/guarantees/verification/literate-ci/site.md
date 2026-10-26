<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Read a local CI run without a runner

For: maintainers sharing a selected run as a static website.

For a new run and export in one command, use
`python3 scripts/literate-ci.py --execute --site-output target/ci-site`.

You can also run the policy with `python3 scripts/literate-ci.py --execute`. The receipt
captures each owning document before execution. Then run
`python3 scripts/export-ci-site.py --run target/literate-ci/RUN-ID --output target/ci-site`.
The exporter builds the shared editor viewer and writes an independent directory
that can be served over HTTP at any path. It never executes the checks again.
Failed checks remain visible and do not prevent exporting their evidence.

Only check IDs selected in `ci/public-preview.json` are exported. Selection is an
explicit public-content decision: review those owners and their command logs
before hosting the bundle. This pilot has no publishing service or access-control
implementation. Private hosting must protect the entire exported directory.
Arbitrary environment variables, unselected documents, filesystem assets and the
raw receipt are not copied. Captured documents and log text are still real source
and output; the exporter does not promise automatic secret redaction.

## Capture the source that the run actually saw

The runner stores original source bytes with their hashes before launching checks.
If an owner no longer matches the planned hash, execution refuses. The exporter
uses this capture, never the current checkout's version. Edits in the viewer create
a local draft and explicitly cease to represent tested source. Downloading the
draft changes no repository and launches no commands.

## Export selected evidence and static assets

Validate IDs, owner hashes and log paths before writing the destination. Refuse
missing captures and escaping symlinks. Omitted logs remain omitted. The preview
normalizes host-specific executable paths; it makes no new assertion about a
commit when the original receipt has no tested revision.

Build a portable data model solely from explicitly selected captured material.

<book:copy id="ci-export-selection">
#!/usr/bin/env python3
"""Export selected local CI evidence as a static Hickory viewer."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys


def digest(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def contained_file(root, relative):
    if not isinstance(relative, str) or Path(relative).is_absolute():
        raise ValueError("Expected a relative captured log path")
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()) or not path.is_file():
        raise ValueError("Captured log is missing or escapes the run")
    return path


def bundle(run, selection):
    report = json.loads((run / "report.json").read_text())
    if report.get("version") != 1 or selection.get("version") != 1:
        raise ValueError("Unsupported run or selection version")
    selected = selection.get("checks")
    if not isinstance(selected, list) or not selected or len(set(selected)) != len(selected):
        raise ValueError("Select unique public check IDs")
    captured = report.get("documents", {})
    checks = {c["id"]: c for c in report["checks"]}
    if len(checks) != len(report["checks"]):
        raise ValueError("Duplicate captured check IDs")
    published = []
    for identity in selected:
        if not isinstance(identity, str) or not re.fullmatch(r"[a-z][a-z0-9-]*", identity) or identity not in checks:
            raise ValueError("Selected check is absent from the receipt")
        check = checks[identity]
        if check["status"] not in ("passed", "failed", "blocked", "timed_out", "unavailable"):
            raise ValueError("Incomplete check outcome")
        document = captured.get(check["owner"])
        if not document or digest(document["source"]) != document["source_hash"] or report["definition_hashes"].get(check["owner"]) != document["source_hash"]:
            raise ValueError("Missing or changed captured owner")
        logs = {}
        for kind in ("stdout", "stderr"):
            if selection.get("include_logs") is True and check.get(kind):
                data = contained_file(run, check[kind]).read_bytes()
                if len(data) > 1024 * 1024:
                    raise ValueError("Selected log exceeds the 1 MiB preview limit")
                logs[kind] = data.decode("utf-8", errors="replace")
        argv = list(check["argv"])
        if argv and Path(argv[0]).is_absolute():
            argv[0] = Path(argv[0]).name
        published.append(dict(id=identity, owner=check["owner"], source=document["source"],
            source_hash=document["source_hash"], status=check["status"], argv=argv,
            needs=check.get("needs", []), assertion=check["assertion"],
            exit_code=check.get("exit_code"), elapsed_seconds=check.get("elapsed_seconds"), logs=logs))
    return dict(format="hickory-ci-publication", version=1, title="Literate CI, run locally",
                captured_at=report["finished_at"], base_revision=report["base_revision"],
                tested_revision=report.get("tested_revision"), checks=published,
                scope="Selected historical command outcomes; opening this page executes no checks.")


</book:copy>

The build includes only the CI entry and its imported assets. A new destination prevents accidental replacement of an existing publication.

<book:copy id="ci-export-package">
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--selection", type=Path, default=Path("ci/public-preview.json"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    value = bundle(args.run.resolve(), json.loads(args.selection.read_text()))
    destination = args.output.resolve()
    if destination.exists():
        raise ValueError("Use a fresh output directory")
    subprocess.run(["npm", "exec", "--", "vite", "build", "--config", "vite.ci.config.ts"], cwd=root / "apps/web", check=True)
    shutil.copytree(root / "apps/web/dist-ci", destination)
    (destination / "index.html").write_bytes((destination / "ci.html").read_bytes())
    data = destination / "ci-preview/bundle.json"
    data.parent.mkdir()
    data.write_text(json.dumps(value, indent=2) + "\n")
    print("Static CI site: " + str(destination))
    print("Serve this directory over HTTP, or upload it to your static host.")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
        print("Export refused: " + str(error), file=sys.stderr)
        sys.exit(1)
</book:copy>

This reviewed pilot selects the runner safety tests and one publication refusal test. Logs are explicitly included; adding another check requires a public-content decision here.

<book:copy id="ci-public-selection">
{
  "version": 1,
  "checks": [
    "ci-runner-safety",
    "publication-input-safety"
  ],
  "include_logs": true
}
</book:copy>

Relative asset URLs support hosting under a subdirectory. Excluding the public directory prevents unrelated site content from entering standalone exports.

<book:copy id="ci-portable-build">
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [react()],
  base: "./",
  publicDir: false,
  build: {
    outDir: "dist-ci",
    rollupOptions: { input: resolve(import.meta.dirname, "ci.html") },
  },
});
</book:copy>

The static entry loads the same React document surface used by embeds. There is no engine connection or browser-triggered CI execution.

<book:copy id="ci-page-entry">
<!doctype html>
<html lang="en" data-theme="dark"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="color-scheme" content="dark"><title>Literate CI · Hickory Docs</title></head><body><div id="root"></div><script type="module" src="./src/embed/ci.tsx"></script></body></html>
</book:copy>

Reuse the static UI session for the editor’s commands without attaching a native host.

<book:copy id="ci-page-mount">
import { createRoot } from "react-dom/client";
import { CiPublicationPage } from "./CiPublication";
import { StaticUiSurface } from "./StaticUiSurface";
import "../styles.css";
import "./ci.css";
createRoot(document.getElementById("root")!).render(<StaticUiSurface><CiPublicationPage /></StaticUiSurface>);
</book:copy>

Downloaded evidence is untrusted data. Validate its shape and source hashes before handing it to the shared editor. Text logs render as text, never as HTML.

<book:copy id="ci-publication-decoder">
import { useEffect, useState } from "react";
import { DocumentEmbed } from "./DocumentEmbed";
import { sha256 } from "../lib/reviews";

export interface CiCheck {
  id: string; owner: string; source: string; source_hash: string;
  status: "passed" | "failed" | "blocked" | "timed_out" | "unavailable";
  argv: string[]; needs: string[]; assertion: string; exit_code: number | null;
  elapsed_seconds: number | null; logs: { stdout?: string; stderr?: string };
}
export interface CiPublication {
  format: "hickory-ci-publication"; version: 1; title: string; captured_at: string;
  base_revision: string; tested_revision: string | null; scope: string; checks: CiCheck[];
}
export async function decodeCiPublication(value: unknown): Promise<CiPublication> {
  const v = value as CiPublication;
  if (!v || v.format !== "hickory-ci-publication" || v.version !== 1 || typeof v.title !== "string" || typeof v.scope !== "string" || typeof v.captured_at !== "string" || !/^[a-f0-9]{40}$/.test(v.base_revision) || v.tested_revision !== null && !/^[a-f0-9]{40}$/.test(v.tested_revision) || !Array.isArray(v.checks) || !v.checks.length) throw Error("Unsupported CI publication");
  const ids = new Set<string>();
  for (const c of v.checks) {
    if (!c || typeof c.id !== "string" || !/^[a-z][a-z0-9-]*$/.test(c.id) || ids.has(c.id) || typeof c.owner !== "string" || typeof c.source !== "string" || !/^[a-f0-9]{64}$/.test(c.source_hash) || !["passed", "failed", "blocked", "timed_out", "unavailable"].includes(c.status) || !Array.isArray(c.argv) || c.argv.some(a => typeof a !== "string") || !Array.isArray(c.needs) || c.needs.some(n => typeof n !== "string") || c.assertion !== "exit-zero" || c.exit_code !== null && !Number.isInteger(c.exit_code) || c.elapsed_seconds !== null && (typeof c.elapsed_seconds !== "number" || !Number.isFinite(c.elapsed_seconds) || c.elapsed_seconds < 0) || !c.logs || [c.logs.stdout, c.logs.stderr].some(log => log !== undefined && typeof log !== "string")) throw Error("Invalid CI check");
    if (await sha256(c.source) !== c.source_hash) throw Error("Captured source hash differs");
    ids.add(c.id);
  }
  return v;
}

</book:copy>

A check opens its captured owner beside its outcome. Editing is a disposable local draft: results stay historical, resetting restores the captured bytes, and downloading uses a browser blob without a server save.

<book:copy id="ci-publication-view">
export function CiPublicationView({ publication }: { publication: CiPublication }) {
  const [selected, setSelected] = useState(publication.checks[0].id);
  const check = publication.checks.find(c => c.id === selected)!;
  const [draft, setDraft] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const passed = publication.checks.filter(c => c.status === "passed").length;
  function choose(id: string) { setSelected(id); setDraft(null); setEditing(false); }
  function download() {
    const url = URL.createObjectURL(new Blob([draft ?? check.source], { type: "text/markdown;charset=utf-8" }));
    const link = document.createElement("a"); link.href = url; link.download = check.owner.split("/").pop() ?? "document.md"; link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  return <div className="ci-publication">
    <header className="ci-masthead"><a href="https://hickorydocs.com">Hickory Docs</a><span>Local execution · static evidence</span></header>
    <section className="ci-intro"><p className="ci-eyebrow">A working experiment</p><h1>{publication.title}</h1><p>Read the requirement, inspect its implementation, and see the checks that ran against it. This page is an artifact of a local run.</p>
      <div className="ci-summary"><strong>{passed} / {publication.checks.length} checks passed</strong><span>Captured {new Date(publication.captured_at).toLocaleString()}</span></div>
      <p className="ci-muted">{publication.tested_revision ? `Tested commit ${publication.tested_revision}` : `Working-tree run based on ${publication.base_revision.slice(0, 12)} · no tested commit associated`}</p>
    </section>
    <div className="ci-workspace"><aside aria-label="Checks"><h2>Checks and their documents</h2>{publication.checks.map(c => <button key={c.id} aria-pressed={selected === c.id} onClick={() => choose(c.id)}><span className={`ci-status ci-status-${c.status}`}>{c.status.replaceAll("_", " ")}</span><strong>{c.id}</strong><small>{c.owner.split("/").slice(-2).join("/")}</small></button>)}<p className="ci-muted">{publication.scope}</p></aside>
    <main><section className="ci-result" aria-label="Check result"><div><span className={`ci-status ci-status-${check.status}`}>{check.status.replaceAll("_", " ")}</span><h2>{check.id}</h2></div><code>{check.argv.join(" ")}</code><p>Assertion: command exits with zero · observed exit {check.exit_code ?? "unavailable"} · {check.elapsed_seconds ?? "—"} seconds</p>
      {check.needs.length > 0 && <p>Depends on: {check.needs.join(", ")}</p>}
      <details><summary>Captured command logs</summary><h3>Standard output</h3><pre>{check.logs.stdout || "No output captured."}</pre><h3>Standard error</h3><pre>{check.logs.stderr || "No error output captured."}</pre></details>
    </section><section aria-label="Owning document"><div className="ci-document-heading"><div><p className="ci-eyebrow">The document behind this check</p><h2>{check.owner.split("/").slice(-2).join("/")}</h2></div><div className="ci-actions"><button onClick={() => { setEditing(!editing); if (editing) setDraft(null); }}>{editing ? "Return to captured source" : "Try editing a local draft"}</button><button onClick={download}>Download {editing ? "draft" : "source"}</button></div></div>
      {editing && <p className="ci-draft" role="status">Local draft · these edits have not been tested or saved. Switching checks discards this draft; download it to keep it.</p>}
      <DocumentEmbed key={`${check.id}:${editing}`} path={check.owner} revision={check.source_hash} source={draft ?? check.source} readOnly={!editing} onChange={change => setDraft(change.source)} />
      <p className="ci-muted ci-hash">Captured source SHA-256: {check.source_hash}</p>
    </section></main></div><footer>Built with Hickory’s document editor. Reading and editing here run no CI commands.</footer>
  </div>;
}

export function CiPublicationPage() {
  const [publication, setPublication] = useState<CiPublication | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    const controller = new AbortController();
    void fetch(new URL("ci-preview/bundle.json", location.href), { signal: controller.signal })
      .then(response => { if (!response.ok) throw Error(`Publication unavailable (${response.status})`); return response.json(); })
      .then(decodeCiPublication).then(value => { if (!controller.signal.aborted) setPublication(value); })
      .catch(failure => { if (!controller.signal.aborted) setError(String(failure)); });
    return () => controller.abort();
  }, []);
  if (error) return <main className="ci-loading" role="alert"><h1>Could not open this CI publication</h1><p>{error}</p></main>;
  return publication ? <CiPublicationView publication={publication} /> : <p className="ci-loading" role="status">Loading captured checks and documents…</p>;
}
</book:copy>

Keep navigation and outcomes scannable while the actual document receives the reading space. A single column on narrow screens preserves all controls and log text.

<book:copy id="ci-publication-layout">
body { margin:0; }
.ci-publication { --ci-border:#343c45; --ci-muted:#a4afbb; background:#171b20; color:#e4eaf0; min-height:100vh; font:15px/1.6 system-ui,sans-serif; }
.ci-publication a { color:#80c6ff; }
.ci-masthead { display:flex; justify-content:space-between; gap:1rem; padding:1rem 3vw; border-bottom:1px solid var(--ci-border); color:var(--ci-muted); }
.ci-masthead a { color:inherit; text-decoration:none; font-weight:700; }
.ci-intro { max-width:74rem; margin:0 auto; padding:3rem 3vw 2rem; }
.ci-intro h1 { font-size:clamp(2rem,5vw,3.5rem); letter-spacing:-.04em; line-height:1.1; margin:.5rem 0 1rem; }
.ci-intro > p { max-width:48rem; }
.ci-eyebrow { color:#72d4c5; font-size:.75rem; letter-spacing:.13em; text-transform:uppercase; margin:0; }
.ci-summary { display:flex; gap:1.5rem; align-items:center; flex-wrap:wrap; margin-top:1.5rem; }
.ci-summary strong { color:#9fdfa9; }
.ci-muted { color:var(--ci-muted); font-size:.85rem; }
.ci-workspace { display:grid; grid-template-columns:16rem minmax(0,1fr); gap:2rem; max-width:92rem; padding:0 3vw; margin:auto; }
.ci-workspace aside { align-self:start; position:sticky; top:1rem; }
.ci-workspace aside h2 { font-size:.9rem; }
.ci-workspace aside button { display:flex; flex-direction:column; align-items:flex-start; gap:.5rem; width:100%; padding:1rem; margin-bottom:.75rem; text-align:left; background:#20262d; color:inherit; border:1px solid var(--ci-border); border-radius:.5rem; cursor:pointer; overflow-wrap:anywhere; }
.ci-workspace aside button[aria-pressed=true] { border-color:#72d4c5; background:#23332f; }
.ci-workspace aside small { color:var(--ci-muted); }
.ci-status { display:inline-block; font:600 .75rem/1.5 system-ui,sans-serif; padding:.15rem .5rem; border:1px solid #727b86; border-radius:1rem; color:#d4dbe3; }
.ci-status-passed { color:#a6e7b1; border-color:#537a5b; }
.ci-status-failed,.ci-status-timed_out { color:#ffb6b6; border-color:#a46c6c; }
.ci-result { border:1px solid var(--ci-border); padding:1.25rem; border-radius:.5rem; background:#20262d; margin-bottom:2rem; }
.ci-result > div { display:flex; gap:.75rem; align-items:center; }
.ci-result h2 { margin:0; font-size:1.1rem; }
.ci-result code { display:block; margin-top:1rem; overflow-wrap:anywhere; white-space:pre-wrap; }
.ci-result p { color:var(--ci-muted); font-size:.85rem; }
.ci-result summary { cursor:pointer; }
.ci-result pre { white-space:pre-wrap; overflow-wrap:anywhere; max-height:24rem; overflow:auto; font-size:.8rem; }
.ci-document-heading { display:flex; justify-content:space-between; align-items:center; gap:1rem; flex-wrap:wrap; margin-bottom:1rem; }
.ci-document-heading h2 { font-size:1rem; margin:.2rem 0; overflow-wrap:anywhere; }
.ci-actions { display:flex; gap:.5rem; flex-wrap:wrap; }
.ci-actions button { cursor:pointer; border:1px solid var(--ci-border); border-radius:.3rem; background:#252d36; color:inherit; padding:.5rem .75rem; }
.ci-draft { background:#3e331e; color:#f3d8a0; padding:.75rem; border-radius:.3rem; }
.ci-hash { overflow-wrap:anywhere; }
.ci-publication footer { text-align:center; padding:3rem 1rem; color:var(--ci-muted); font-size:.85rem; }
.ci-loading { padding:3rem; }
@media(max-width:800px) { .ci-workspace { grid-template-columns:minmax(0,1fr); } .ci-workspace aside { position:static; } .ci-masthead { flex-wrap:wrap; } .ci-intro { padding-top:2rem; } }
</book:copy>

### Assemble `scripts/export-ci-site.py`

<book:file path="scripts/export-ci-site.py"><book:paste select="#ci-export-selection"/><book:paste select="#ci-export-package"/></book:file>

### Assemble `ci/public-preview.json`

<book:file path="ci/public-preview.json"><book:paste select="#ci-public-selection"/></book:file>

### Assemble `apps/web/vite.ci.config.ts`

<book:file path="apps/web/vite.ci.config.ts"><book:paste select="#ci-portable-build"/></book:file>

### Assemble `apps/web/ci.html`

<book:file path="apps/web/ci.html"><book:paste select="#ci-page-entry"/></book:file>

### Assemble `apps/web/src/embed/ci.tsx`

<book:file path="apps/web/src/embed/ci.tsx"><book:paste select="#ci-page-mount"/></book:file>

### Assemble `apps/web/src/embed/CiPublication.tsx`

<book:file path="apps/web/src/embed/CiPublication.tsx"><book:paste select="#ci-publication-decoder"/><book:paste select="#ci-publication-view"/></book:file>

### Assemble `apps/web/src/embed/ci.css`

<book:file path="apps/web/src/embed/ci.css"><book:paste select="#ci-publication-layout"/></book:file>


Focused fixtures prove export uses captured bytes, preserves failures, excludes unselected content, and refuses a changed owner hash or an escaping log.

<book:copy id="ci-export-tests">
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("export_ci", Path(__file__).with_name("export-ci-site.py"))
export = importlib.util.module_from_spec(spec)
spec.loader.exec_module(export)


class ExportTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.run = Path(temporary.name)
        self.source = "# Captured, not the current checkout\n"
        self.report = dict(version=1, finished_at="2026-10-26T00:00:00Z", base_revision="a"*40, tested_revision=None,
            documents={"owner.md": dict(source=self.source, source_hash=export.digest(self.source))},
            definition_hashes={"owner.md": export.digest(self.source)},
            checks=[dict(id="selected", owner="owner.md", status="failed", argv=["/host/python3", "-c", "exit(1)"], needs=[], assertion="exit-zero", exit_code=1, stdout="out.log"),
                    dict(id="private", owner="private.md", status="passed")])
        (self.run / "out.log").write_text("failure evidence")
        self.selection = dict(version=1, checks=["selected"], include_logs=True)

    def bundle(self):
        (self.run / "report.json").write_text(json.dumps(self.report))
        return export.bundle(self.run, self.selection)

    def test_export_keeps_captured_failure_and_excludes_unselected_content(self):
        value = self.bundle()
        self.assertEqual(len(value["checks"]), 1)
        self.assertEqual(value["checks"][0]["source"], self.source)
        self.assertEqual(value["checks"][0]["status"], "failed")
        self.assertEqual(value["checks"][0]["logs"]["stdout"], "failure evidence")
        self.assertEqual(value["checks"][0]["argv"][0], "python3")
        self.assertIsNone(value["tested_revision"])
        self.assertNotIn("private.md", json.dumps(value))
        self.selection["include_logs"] = False
        self.assertEqual(self.bundle()["checks"][0]["logs"], {})

    def test_changed_or_missing_capture_is_refused(self):
        self.report["documents"]["owner.md"]["source"] = "changed"
        with self.assertRaises(ValueError): self.bundle()
        self.report["documents"] = {}
        with self.assertRaises(ValueError): self.bundle()

    def test_escape_and_duplicate_selection_are_refused(self):
        self.report["checks"][0]["stdout"] = "../outside.log"
        with self.assertRaises(ValueError): self.bundle()
        self.selection["checks"] *= 2
        with self.assertRaises(ValueError): self.bundle()


if __name__ == "__main__":
    unittest.main()
</book:copy>
<book:file path="scripts/test-export-ci-site.py"><book:paste select="#ci-export-tests"/></book:file>

The view must preserve failed outcomes, show actual log text, and label drafts separately from historical evidence. Mock only the editor to test host interactions without duplicating its existing tests.

<book:copy id="ci-view-tests">
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { CiPublicationView, decodeCiPublication, type CiPublication } from "./CiPublication";
vi.mock("./DocumentEmbed", () => ({ DocumentEmbed: ({ source, readOnly, onChange }: { source: string; readOnly: boolean; onChange: (change: { source: string }) => void }) => <textarea aria-label="Document" value={source} readOnly={readOnly} onChange={event => onChange({ source: event.target.value })} /> }));
vi.mock("../lib/reviews", () => ({ sha256: vi.fn(async (source: string) => source === "original" ? "a".repeat(64) : "b".repeat(64)) }));
afterEach(cleanup);
const publication: CiPublication = { format: "hickory-ci-publication", version: 1, title: "Local run", captured_at: "2026-10-26T00:00:00Z", base_revision: "a".repeat(40), tested_revision: null, scope: "Historical only", checks: [
  { id: "first", owner: "behavior/contract.md", source: "original", source_hash: "a".repeat(64), status: "failed", argv: ["python3", "test.py"], needs: [], assertion: "exit-zero", exit_code: 1, elapsed_seconds: 0.1, logs: { stderr: "actual failure" } },
  { id: "second", owner: "another/contract.md", source: "original", source_hash: "a".repeat(64), status: "passed", argv: ["python3"], needs: [], assertion: "exit-zero", exit_code: 0, elapsed_seconds: 0.1, logs: {} },
] };
it("rejects tampered source and duplicate check identities", async () => {
  await expect(decodeCiPublication(publication)).resolves.toBe(publication);
  await expect(decodeCiPublication({ ...publication, checks: [{ ...publication.checks[0], source: "changed" }] })).rejects.toThrow("hash");
  await expect(decodeCiPublication({ ...publication, checks: [publication.checks[0], publication.checks[0]] })).rejects.toThrow("Invalid");
});
it("keeps historical results separate from a disposable draft", () => {
  render(<CiPublicationView publication={publication} />);
  const document = screen.getByLabelText("Document") as HTMLTextAreaElement;
  expect(document.readOnly).toBe(true);
  expect(screen.getByText("actual failure")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Try editing a local draft" }));
  const draft = screen.getByLabelText("Document") as HTMLTextAreaElement;
  expect(draft.readOnly).toBe(false);
  fireEvent.change(draft, { target: { value: "my draft" } });
  expect(screen.getByRole("status").textContent).toContain("have not been tested");
  fireEvent.click(screen.getByRole("button", { name: /passed.*second/ }));
  expect((screen.getByLabelText("Document") as HTMLTextAreaElement).value).toBe("original");
  expect((screen.getByLabelText("Document") as HTMLTextAreaElement).readOnly).toBe(true);
});
</book:copy>
<book:file path="apps/web/src/embed/CiPublication.test.tsx"><book:paste select="#ci-view-tests"/></book:file>


## Exercise the exported site in a browser

This focused flow loads the production bundle and its real WASM parser, reads
logs, edits and downloads a draft, restores captured source, changes documents,
and checks the narrow layout. It also refuses accidental API requests.

<book:copy id="ci-static-browser-test">
import { test, expect } from "@playwright/test";

test("a static CI publication renders the real document and keeps edits local", async ({ page }) => {
  test.skip(!process.env.HICKORY_CI_PREVIEW, "Requires the exported local CI site");
  const apiRequests: string[] = [];
  page.on("request", request => { if (new URL(request.url()).pathname.startsWith("/api/")) apiRequests.push(request.url()); });
  await page.goto("/ci.html");
  await expect(page.getByRole("heading", { name: "Literate CI, run locally" })).toBeVisible();
  await expect(page.locator(".cm-editor")).toBeVisible();
  await expect(page.locator(".cm-content")).toHaveAttribute("contenteditable", "false");
  await page.getByText("Captured command logs", { exact: true }).click();
  await expect(page.getByText(/Ran 6 tests/)).toBeVisible();
  await page.getByRole("button", { name: "Try editing a local draft" }).click();
  await expect(page.locator(".cm-content")).toHaveAttribute("contenteditable", "true");
  await page.locator(".cm-content").press("Control+Home");
  await page.locator(".cm-content").pressSequentially("Local draft ");
  await expect(page.getByRole("status")).toContainText("have not been tested");
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Download draft" }).click();
  await expect((await download).suggestedFilename()).toBe("contract.md");
  await page.getByRole("button", { name: "Return to captured source" }).click();
  await expect(page.locator(".cm-content")).toHaveAttribute("contenteditable", "false");
  await page.getByRole("button", { name: /publication-input-safety/ }).click();
  await expect(page.getByRole("region", { name: "Check result" }).getByRole("heading", { level: 2 })).toHaveText("publication-input-safety");
  await expect(page.locator(".cm-editor")).toBeVisible();
  expect(apiRequests).toEqual([]);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByRole("heading", { name: "Literate CI, run locally" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});
</book:copy>
<book:file path="apps/web/e2e/ciPublication.spec.ts"><book:paste select="#ci-static-browser-test"/></book:file>


## Evidence from this implementation

Local validation on 2026-10-26 passed the six runner tests, the named publication
input-refusal test, three exporter boundary tests and two viewer tests. A focused
Chromium flow exercised the exported production bundle with the real WASM parser,
read-only viewing, logs, document switching, draft download and a 390px layout.
Frontend type checking and both build configurations were checked separately.
These observations do not certify all editor behavior or private hosting.

The [preserved public preview](preview.md) records its own original scope.
<book:include file="preview.md" />
</book:doc>
