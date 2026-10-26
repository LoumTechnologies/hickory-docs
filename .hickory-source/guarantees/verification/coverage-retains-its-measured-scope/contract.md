<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Coverage retains its measured scope

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given collector evidence from automated, human, or AI-driven testing, importing
or comparing it preserves the measured executable denominator, collector data,
source identity and historical metadata. Missing data is unknown rather than
zero. Test success, assertion quality and workflow verification are separate.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/coverage-retains-its-measured-scope.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-814b32909c24"></a>
## `apps/web/src/coverage/CoverageTools.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-814b32909c24-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-814b32909c24-expression_statement">
import { cleanup, render, screen, fireEvent, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { CoveragePanel } from "./CoverageTools";
import { coverageCall } from "./client";
vi.mock("./client", () => ({ coverageCall: vi.fn() }));
</storycode:copy>
<a id="f-814b32909c24-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-814b32909c24-expression_statement-2">
vi.mock("../lib/revealLine", () => ({ openLocation: vi.fn() }));
</storycode:copy>
<a id="f-814b32909c24-run"></a>
### `run`

<storycode:copy id="f-814b32909c24-run">
const run = { id: "a".repeat(32), mode: "manual", status: "completed", source_state: "historical", started_ms: 1, labels: ["adversarial"], project: ".", language: "typescript", identity: { collector: "istanbul-browser", version: "v1", source_hashes: {} }, tests: [], output: "", evidence: null, metadata: { groups: {}, manual: {}, tests: {} }, mappings: {}, notes: "", manual_verdict: null, argv: [], checkpoints: [] };
</storycode:copy>
<a id="f-814b32909c24-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-814b32909c24-expression_statement-3">
afterEach(cleanup);
</storycode:copy>
<a id="f-814b32909c24-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-814b32909c24-expression_statement-4">
beforeEach(() => { vi.mocked(coverageCall).mockReset(); vi.mocked(coverageCall).mockImplementation(async body => (body.action === "list" ? { runs: [run] } : run) as never); });
</storycode:copy>
<a id="f-814b32909c24-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-814b32909c24-expression_statement-5">
// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
it("keeps missing coverage and historical source distinct from a failed manual verdict", async () => {
  render(<CoveragePanel close={() => {}} />);
  fireEvent.click(await screen.findByRole("button", { name: /manual · completed/ }));
  expect(await screen.findByText("Coverage unavailable. This is not zero coverage.")).toBeTruthy();
  expect(screen.getByText("manual · completed · historical source")).toBeTruthy();
  expect(screen.getByText("Recorded verdict: none")).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Manual workflow verdict"), { target: { value: "failed" } });
  fireEvent.change(screen.getByLabelText("Observation / prerequisites"), { target: { value: "Wrong rejection message" } });
  fireEvent.click(screen.getByRole("button", { name: "Record manual verdict" }));
  await waitFor(() => expect(coverageCall).toHaveBeenCalledWith(expect.objectContaining({ action: "annotate", verdict: "failed", notes: "Wrong rejection message" })));
});
</storycode:copy>
<a id="f-814b32909c24-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-814b32909c24-expression_statement-6">
it("offers human and AI browser collection through the same service", async () => {
  render(<CoveragePanel close={() => {}} />);
  fireEvent.click(screen.getByText("Human or AI manual testing"));
  fireEvent.change(screen.getByLabelText("Page URL contains"), { target: { value: "localhost:3000" } });
  fireEvent.click(screen.getByRole("button", { name: "Start browser coverage" }));
  await waitFor(() => expect(coverageCall).toHaveBeenCalledWith(expect.objectContaining({ action: "manual-start", url: "http://127.0.0.1:9222", selector: "localhost:3000" })));
});
</storycode:copy>

### Assemble `apps/web/src/coverage/CoverageTools.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/coverage/CoverageTools.test.tsx"><storycode:paste select="#f-814b32909c24-expression_statement"/><storycode:paste select="#f-814b32909c24-expression_statement-2"/><storycode:paste select="#f-814b32909c24-run"/><storycode:paste select="#f-814b32909c24-expression_statement-3"/><storycode:paste select="#f-814b32909c24-expression_statement-4"/><storycode:paste select="#f-814b32909c24-expression_statement-5"/><storycode:paste select="#f-814b32909c24-expression_statement-6"/></storycode:file>

<a id="output-043842eaed53"></a>
## `apps/web/src/coverage/gutter.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-043842eaed53-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-043842eaed53-expression_statement">
import { afterEach, expect, it, vi } from "vitest";
import { waitFor } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { coverageGutter } from "./gutter";
import { selectCoverage } from "./selection";
import type { CoverageRun } from "./types";
vi.mock("../lib/reviews", () => ({ sha256: async (text: string) => text }));
</storycode:copy>
<a id="f-043842eaed53-view"></a>
### `view`

<storycode:copy id="f-043842eaed53-view">
let view: EditorView | undefined;
</storycode:copy>
<a id="f-043842eaed53-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-043842eaed53-expression_statement-2">
afterEach(() => { view?.destroy(); view = undefined; selectCoverage(null); document.body.replaceChildren(); });
</storycode:copy>
<a id="f-043842eaed53-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-043842eaed53-expression_statement-3">
// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
it("draws measured code only for matching bytes and removes marks on buffer edits", async () => {
  const source = "accept()\nreject()\n";
  const run = { source_changed_during_run: false, source_state: "current", id: "run", labels: ["happy-path"], identity: { source_hashes: { "app.ts": source } }, evidence: { attribution: "group", report: { files: { "app.ts": { lines: { 1: 1, 2: 0 }, line_counts: true } } } }, mappings: {} } as unknown as CoverageRun;
  const host = document.createElement("div"); document.body.append(host);
  view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [coverageGutter(() => "app.ts")] }) });
  selectCoverage(run);
  await waitFor(() => expect(host.querySelectorAll(".cm-coverage-hit")).toHaveLength(1));
  expect(host.querySelectorAll(".cm-coverage-miss")).toHaveLength(1);
  expect(host.querySelector(".cm-coverage-gutter")?.textContent).toContain("○");
  view.dispatch({ changes: { from: 0, to: 6, insert: "changed" } });
  await waitFor(() => expect(host.querySelectorAll(".cm-coverage-hit, .cm-coverage-miss")).toHaveLength(0));
});
</storycode:copy>
<a id="f-043842eaed53-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-043842eaed53-expression_statement-4">
it("allows buffer edits when no coverage is selected without nested editor updates", async () => {
  selectCoverage(null);
  const host = document.createElement("div"); document.body.append(host);
  view = new EditorView({ parent: host, state: EditorState.create({ doc: "original", extensions: [coverageGutter(() => "app.ts")] }) });
  view.dispatch({ changes: { from: 0, to: 8, insert: "edited" } });
  await new Promise(resolve => setTimeout(resolve, 0));
  expect(view.state.doc.toString()).toBe("edited");
  expect(host.querySelectorAll(".cm-coverage-hit, .cm-coverage-miss")).toHaveLength(0);
});
</storycode:copy>

### Assemble `apps/web/src/coverage/gutter.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/coverage/gutter.test.ts"><storycode:paste select="#f-043842eaed53-expression_statement"/><storycode:paste select="#f-043842eaed53-view"/><storycode:paste select="#f-043842eaed53-expression_statement-2"/><storycode:paste select="#f-043842eaed53-expression_statement-3"/><storycode:paste select="#f-043842eaed53-expression_statement-4"/></storycode:file>

<a id="output-7ccf384862a0"></a>
## `apps/web/src/coverage/gutter.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-7ccf384862a0-setCoverage"></a>
### `setCoverage`

<storycode:copy id="f-7ccf384862a0-setCoverage">
import { RangeSetBuilder, StateEffect, StateField, type Extension } from "@codemirror/state";
import { Decoration, EditorView, GutterMarker, ViewPlugin, gutter, type DecorationSet, type ViewUpdate } from "@codemirror/view";
import { onCoverageSelection, selectedCoverage, coveredLocations } from "./selection";
import { sha256 } from "../lib/reviews";

const setCoverage = StateEffect.define<{ line: number; count: number; label: string }[]>();
</storycode:copy>
<a id="f-7ccf384862a0-rows"></a>
### `rows`

<storycode:copy id="f-7ccf384862a0-rows">
const rows = StateField.define<{ line: number; count: number; label: string }[]>({
  create: () => [], update(value, tr) { if (tr.docChanged) value = []; for (const e of tr.effects) if (e.is(setCoverage)) value = e.value; return value; },
});
</storycode:copy>
<a id="f-7ccf384862a0-decorations"></a>
### `decorations`

<storycode:copy id="f-7ccf384862a0-decorations">
const decorations = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    if (tr.docChanged || tr.effects.some(e => e.is(setCoverage))) {
      const ranges = new RangeSetBuilder<Decoration>();
      for (const row of tr.state.field(rows)) if (row.line <= tr.state.doc.lines && row.line > 0) {
        const at = tr.state.doc.line(row.line).from;
        ranges.add(at, at, Decoration.line({ class: row.count > 0 ? "cm-coverage-hit" : "cm-coverage-miss", attributes: { title: row.label } }));
      }
      value = ranges.finish();
    }
    return value;
  }, provide: field => EditorView.decorations.from(field),
});
</storycode:copy>
<a id="f-7ccf384862a0-Marker"></a>
### `Marker`

<storycode:copy id="f-7ccf384862a0-Marker">
class Marker extends GutterMarker {
  constructor(readonly count: number, readonly label: string) { super(); }
  eq(other: Marker) { return this.count === other.count && this.label === other.label; }
  toDOM() { const el = document.createElement("span"); el.textContent = this.count ? "●" : "○"; el.title = this.label; el.setAttribute("aria-label", this.label); return el; }
}
</storycode:copy>
<a id="f-7ccf384862a0-coverageGutter"></a>
### `coverageGutter`

Source-checked decorations shared by plain, generated and literate editors.

<storycode:copy id="f-7ccf384862a0-coverageGutter">
/** Source-checked decorations shared by plain, generated and literate editors. */
export function coverageGutter(path: () => string | undefined, serializedSource?: () => string | undefined): Extension {
  return [rows, decorations, ViewPlugin.fromClass(class {
    disposed = false; ticket = 0; off: () => void;
    constructor(readonly view: EditorView) { this.off = onCoverageSelection(() => { void this.refresh(); }); queueMicrotask(() => { void this.refresh(); }); }
    update(update: ViewUpdate) { if (update.docChanged) void this.refresh(); }
    async refresh() {
      const ticket = ++this.ticket, source = this.view.state.doc, run = selectedCoverage(), file = path();
      await Promise.resolve();
      let value: { line: number; count: number; label: string }[] = [];
      if (run && file && !run.source_changed_during_run && run.identity.source_hashes[file]) {
        const candidate = serializedSource?.();
        const text = candidate !== undefined && candidate.replace(/\r\n/g, "\n") === source.toString() ? candidate : source.toString();
        try {
          if (await sha256(text) === run.identity.source_hashes[file]) {
            // A literate line can contain multiple measured locations. A miss stays visible.
            const locations = new Map<number, { count: number; labels: string[] }>();
            for (const row of coveredLocations(run, file)) {
              const item = locations.get(row.line);
              const label = `${row.count > 0 ? "Covered" : "Uncovered executable"}: ${row.originalPath}:${row.originalLine}; count ${row.countsAvailable ? row.count : "unavailable"}; ${run.evidence?.attribution ?? "unavailable"} attribution; ${run.labels.join(", ")}; run ${run.id}; ${run.source_state}`;
              if (item) { item.count = Math.min(item.count, row.count); item.labels.push(label); } else locations.set(row.line, { count: row.count, labels: [label] });
            }
            value = [...locations].sort(([a], [b]) => a - b).map(([line, item]) => ({ line, count: item.count, label: item.labels.join("\n") }));
          }
        } catch { /* Without a source fingerprint, no current-source decoration. */ }
      }
      if (!this.disposed && ticket === this.ticket && this.view.state.doc === source) this.view.dispatch({ effects: setCoverage.of(value) });
    }
    destroy() { this.disposed = true; this.off(); }
  }), gutter({ class: "cm-coverage-gutter", markers(view) {
    const marks = new RangeSetBuilder<GutterMarker>();
    for (const row of view.state.field(rows)) if (row.line <= view.state.doc.lines && row.line > 0) { const at = view.state.doc.line(row.line).from; marks.add(at, at, new Marker(row.count, row.label)); }
    return marks.finish();
  } }), EditorView.theme({ ".cm-coverage-gutter": { color: "var(--muted)", fontSize: "0.7em" }, ".cm-coverage-hit": { backgroundColor: "color-mix(in srgb, var(--ok, green) 9%, transparent)" }, ".cm-coverage-miss": { backgroundColor: "color-mix(in srgb, var(--error, red) 13%, transparent)" } })];
}
</storycode:copy>

### Assemble `apps/web/src/coverage/gutter.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/coverage/gutter.ts"><storycode:paste select="#f-7ccf384862a0-setCoverage"/><storycode:paste select="#f-7ccf384862a0-rows"/><storycode:paste select="#f-7ccf384862a0-decorations"/><storycode:paste select="#f-7ccf384862a0-Marker"/><storycode:paste select="#f-7ccf384862a0-coverageGutter"/></storycode:file>

<a id="output-49aaee6bbbd9"></a>
## `apps/web/src/coverage/selection.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-49aaee6bbbd9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-49aaee6bbbd9-expression_statement">
import { describe, it, expect } from "vitest";
import { coveredLocations } from "./selection";
import type { CoverageRun } from "./types";
// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
describe("coverage source navigation", () => {
  it("maps only executable lines and preserves hit counts and original coordinates", () => {
    const run = { evidence: { report: { files: { "app.py": { lines: { 1: 1, 3: 0 }, line_counts: false } } } }, mappings: { "app.py": { 1: { path: "app.md", line: 5 }, 3: { path: "app.md", line: 7 } } } } as unknown as CoverageRun;
    expect(coveredLocations(run, "app.md")).toEqual([
      { line: 5, count: 1, countsAvailable: false, originalPath: "app.py", originalLine: 1 },
      { line: 7, count: 0, countsAvailable: false, originalPath: "app.py", originalLine: 3 },
    ]);
    expect(coveredLocations(run, "other.py")).toEqual([]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/coverage/selection.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/coverage/selection.test.ts"><storycode:paste select="#f-49aaee6bbbd9-expression_statement"/></storycode:file>

<a id="output-692cdd9dae7c"></a>
## `crates/hick-coverage/src/compare.rs`

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-692cdd9dae7c-Comparison"></a>
### `Comparison`

<storycode:copy id="f-692cdd9dae7c-Comparison">
use crate::Evidence;
use anyhow::{Result, ensure};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Serialize)]
pub struct Comparison {
    pub shared: BTreeSet<String>,
    pub left_only: BTreeSet<String>,
    pub right_only: BTreeSet<String>,
    pub missed: BTreeSet<String>,
    pub denominator: usize,
}

</storycode:copy>
<a id="f-692cdd9dae7c-compare"></a>
### `compare`

Compare only evidence with identical provenance and executable denominators.
Missing reports or unsupported metrics are an error, never an empty set.

<storycode:copy id="f-692cdd9dae7c-compare">
/// Compare only evidence with identical provenance and executable denominators.
/// Missing reports or unsupported metrics are an error, never an empty set.
pub fn compare(left: &Evidence, right: &Evidence, metric: &str) -> Result<Comparison> {
    ensure!(
        left.identity == right.identity,
        "Source, scope, collector/version or instrumentation differ"
    );
    ensure!(
        !left.identity.source_hashes.is_empty(),
        "Source revision is unknown"
    );
    ensure!(
        left.report.format == right.report.format,
        "Report semantics differ"
    );
    let locations =
        |e: &Evidence| -> Result<BTreeMap<String, bool>> {
            let mut out = BTreeMap::new();
            for (path, file) in &e.report.files {
                match metric {
                    "line" => {
                        for (n, count) in &file.lines {
                            out.insert(format!("{path}:{n}"), *count > 0);
                        }
                    }
                    "branch" => {
                        for b in file.branches.as_ref().ok_or_else(|| {
                            anyhow::anyhow!("Branch metric unavailable for {path}")
                        })? {
                            out.insert(format!("{path}:{}", b.id), b.covered);
                        }
                    }
                    _ => anyhow::bail!("Unsupported metric"),
                }
            }
            Ok(out)
        };
    let a = locations(left)?;
    let b = locations(right)?;
    ensure!(
        a.keys().eq(b.keys()),
        "Measured executable denominators differ"
    );
    let mut result = Comparison {
        denominator: a.len(),
        ..Default::default()
    };
    for (key, covered) in a {
        match (covered, b[&key]) {
            (true, true) => &mut result.shared,
            (true, false) => &mut result.left_only,
            (false, true) => &mut result.right_only,
            (false, false) => &mut result.missed,
        }
        .insert(key);
    }
    Ok(result)
}
</storycode:copy>

### Assemble `crates/hick-coverage/src/compare.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-coverage/src/compare.rs"><storycode:paste select="#f-692cdd9dae7c-Comparison"/><storycode:paste select="#f-692cdd9dae7c-compare"/></storycode:file>

<a id="output-4d440b3acbf1"></a>
## `crates/hick-coverage/src/import.rs`

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-4d440b3acbf1-parse"></a>
### `parse`

<storycode:copy id="f-4d440b3acbf1-parse">
use crate::{Branch, FileCoverage, Report};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::collections::BTreeMap;

pub fn parse(format: &str, raw: &str) -> Result<Report> {
    ensure!(
        raw.len() <= 16 * 1024 * 1024,
        "Coverage report exceeds 16 MiB"
    );
    let files = match format {
        "lcov" => lcov(raw)?,
        "coverage-py" => python(raw)?,
        "cobertura" => cobertura(raw)?,
        "istanbul" => istanbul(raw)?,
        _ => bail!("Unsupported coverage format: {format}"),
    };
    ensure!(!files.is_empty(), "Report contains no measured files");
    Ok(Report {
        format: format.into(),
        files,
        raw: raw.into(),
    })
}

</storycode:copy>
<a id="f-4d440b3acbf1-line"></a>
### `line`

<storycode:copy id="f-4d440b3acbf1-line">
fn line(value: u64) -> Result<u32> {
    ensure!(
        value > 0 && value <= u32::MAX as u64,
        "Invalid source line {value}"
    );
    Ok(value as u32)
}

</storycode:copy>
<a id="f-4d440b3acbf1-lcov"></a>
### `lcov`

<storycode:copy id="f-4d440b3acbf1-lcov">
fn lcov(raw: &str) -> Result<BTreeMap<String, FileCoverage>> {
    let mut files = BTreeMap::<String, FileCoverage>::new();
    let mut current = None;
    for row in raw.lines() {
        if let Some(path) = row.strip_prefix("SF:") {
            ensure!(current.is_none(), "Unterminated LCOV record");
            current = Some(path.to_string());
            files.entry(path.into()).or_default();
        } else if row == "end_of_record" {
            current = None;
        } else if let Some(path) = &current {
            let file = files.get_mut(path).unwrap();
            file.line_counts = true;
            if let Some(data) = row.strip_prefix("DA:") {
                let parts: Vec<_> = data.split(',').collect();
                ensure!(parts.len() >= 2, "Invalid LCOV line");
                let number = line(parts[0].parse()?)?;
                let count: u64 = parts[1].parse()?;
                let value = file.lines.entry(number).or_default();
                *value = value.saturating_add(count);
            } else if let Some(data) = row.strip_prefix("BRDA:") {
                let p: Vec<_> = data.split(',').collect();
                ensure!(p.len() == 4, "Invalid LCOV branch");
                let count = if p[3] == "-" {
                    None
                } else {
                    Some(p[3].parse::<u64>()?)
                };
                let branches = file.branches.get_or_insert_default();
                let id = format!("{}:{}:{}", p[0], p[1], p[2]);
                if let Some(previous) = branches.iter_mut().find(|b| b.id == id) {
                    previous.covered |= count.is_some_and(|n| n > 0);
                    previous.count = match (previous.count, count) {
                        (Some(a), Some(b)) => Some(a.saturating_add(b)),
                        _ => None,
                    };
                } else {
                    branches.push(Branch {
                        id,
                        line: line(p[0].parse()?)?,
                        covered: count.is_some_and(|n| n > 0),
                        count,
                    });
                }
            } else if row == "BRF:0" {
                file.branches.get_or_insert_default();
            }
        }
    }
    ensure!(current.is_none(), "Unterminated LCOV record");
    Ok(files)
}

</storycode:copy>
<a id="f-4d440b3acbf1-python"></a>
### `python`

<storycode:copy id="f-4d440b3acbf1-python">
fn python(raw: &str) -> Result<BTreeMap<String, FileCoverage>> {
    let value: Value = serde_json::from_str(raw)?;
    let files = value["files"]
        .as_object()
        .context("Missing coverage.py files")?;
    let mut output = BTreeMap::new();
    for (path, data) in files {
        let mut file = FileCoverage::default();
        for (key, count) in [("executed_lines", 1), ("missing_lines", 0)] {
            for number in data[key]
                .as_array()
                .context("Missing coverage.py executable lines")?
            {
                file.lines.insert(
                    line(number.as_u64().context("Invalid executable line")?)?,
                    count,
                );
            }
        }
        if let Some(excluded) = data["excluded_lines"].as_array() {
            for n in excluded {
                let n = line(n.as_u64().context("Invalid excluded line")?)?;
                file.lines.remove(&n);
                file.excluded.push(n);
            }
        }
        if value["meta"]["branch_coverage"].as_bool() == Some(true) {
            let mut branches = Vec::new();
            // coverage.py arcs can end at a negative synthetic function-exit line.
            for (key, covered) in [("executed_branches", true), ("missing_branches", false)] {
                for arc in data[key].as_array().context("Missing branch arcs")? {
                    let from = arc[0].as_i64().context("Invalid arc")?;
                    let to = arc[1].as_i64().context("Invalid arc")?;
                    if from > 0 {
                        branches.push(Branch {
                            id: format!("{from}:{to}"),
                            line: line(from as u64)?,
                            count: None,
                            covered,
                        });
                    }
                }
            }
            file.branches = Some(branches);
        }
        output.insert(path.clone(), file);
    }
    Ok(output)
}

</storycode:copy>
<a id="f-4d440b3acbf1-cobertura"></a>
### `cobertura`

<storycode:copy id="f-4d440b3acbf1-cobertura">
fn cobertura(raw: &str) -> Result<BTreeMap<String, FileCoverage>> {
    let doc = roxmltree::Document::parse(raw)?;
    let mut files = BTreeMap::<String, FileCoverage>::new();
    let sources: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("source"))
        .filter_map(|n| n.text())
        .filter(|s| !s.is_empty())
        .collect();
    for class in doc.descendants().filter(|n| n.has_tag_name("class")) {
        let path = class
            .attribute("filename")
            .context("Class has no source filename")?;
        let path = if !std::path::Path::new(path).is_absolute() && sources.len() == 1 {
            std::path::Path::new(sources[0])
                .join(path)
                .to_string_lossy()
                .into_owned()
        } else {
            path.to_string()
        };
        let file = files.entry(path).or_default();
        file.line_counts = true;
        // Method line tables repeat class line tables. Read only the class's direct lines.
        for node in class
            .children()
            .filter(|n| n.has_tag_name("lines"))
            .flat_map(|n| n.children())
            .filter(|n| n.has_tag_name("line"))
        {
            let n = line(
                node.attribute("number")
                    .context("Missing line number")?
                    .parse()?,
            )?;
            let count: u64 = node
                .attribute("hits")
                .context("Missing hit count")?
                .parse()?;
            file.lines
                .entry(n)
                .and_modify(|c| *c = (*c).max(count))
                .or_insert(count);
            if let Some(fraction) = node
                .attribute("condition-coverage")
                .and_then(|s| s.split('(').nth(1))
                .and_then(|s| s.strip_suffix(')'))
            {
                let (covered, total) = fraction
                    .split_once('/')
                    .context("Invalid branch fraction")?;
                let covered: u64 = covered.parse()?;
                let total: u64 = total.parse()?;
                ensure!(covered <= total, "Invalid branch denominator");
                file.branch_totals
                    .entry(n)
                    .and_modify(|old| {
                        old.0 = old.0.max(covered);
                        old.1 = old.1.max(total);
                    })
                    .or_insert((covered, total));
            }
            // Cobertura gives aggregate branch fractions, not branch identities. Raw data
            // retains those fractions; never manufacture identifiable branches from them.
        }
    }
    Ok(files)
}

</storycode:copy>
<a id="f-4d440b3acbf1-istanbul"></a>
### `istanbul`

<storycode:copy id="f-4d440b3acbf1-istanbul">
fn istanbul(raw: &str) -> Result<BTreeMap<String, FileCoverage>> {
    let value: Value = serde_json::from_str(raw)?;
    let mut files = BTreeMap::new();
    for (path, data) in value
        .as_object()
        .context("Expected Istanbul coverage map")?
    {
        let mut file = FileCoverage {
            line_counts: true,
            ..Default::default()
        };
        let statements = data["statementMap"]
            .as_object()
            .context("Missing statement map")?;
        for (id, location) in statements {
            let count = data["s"][id].as_u64().context("Missing statement count")?;
            let n = line(
                location["start"]["line"]
                    .as_u64()
                    .context("Invalid statement line")?,
            )?;
            // Istanbul line coverage uses the maximum count of statements starting there.
            file.lines
                .entry(n)
                .and_modify(|c| *c = (*c).max(count))
                .or_insert(count);
        }
        if let Some(map) = data["branchMap"].as_object() {
            let mut branches = Vec::new();
            for (id, location) in map {
                for (index, count) in data["b"][id]
                    .as_array()
                    .context("Missing branch counters")?
                    .iter()
                    .enumerate()
                {
                    let count = count.as_u64().context("Invalid branch count")?;
                    let loc = &location["locations"][index];
                    branches.push(Branch {
                        id: format!("{id}:{index}"),
                        line: line(
                            loc["start"]["line"]
                                .as_u64()
                                .or_else(|| location["line"].as_u64())
                                .context("Missing branch location")?,
                        )?,
                        count: Some(count),
                        covered: count > 0,
                    });
                }
            }
            file.branches = Some(branches);
        }
        files.insert(data["path"].as_str().unwrap_or(path).into(), file);
    }
    Ok(files)
}
</storycode:copy>

### Assemble `crates/hick-coverage/src/import.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-coverage/src/import.rs"><storycode:paste select="#f-4d440b3acbf1-parse"/><storycode:paste select="#f-4d440b3acbf1-line"/><storycode:paste select="#f-4d440b3acbf1-lcov"/><storycode:paste select="#f-4d440b3acbf1-python"/><storycode:paste select="#f-4d440b3acbf1-cobertura"/><storycode:paste select="#f-4d440b3acbf1-istanbul"/></storycode:file>

<a id="output-441045ed4fa4"></a>
## `crates/hick-coverage/src/lib.rs`

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-441045ed4fa4-compare"></a>
### `compare`

Collector evidence, independent of a debugger and of test success.

<storycode:copy id="f-441045ed4fa4-compare">
//! Collector evidence, independent of a debugger and of test success.
mod compare;
</storycode:copy>
<a id="f-441045ed4fa4-import"></a>
### `import`

<storycode:copy id="f-441045ed4fa4-import">
mod import;
</storycode:copy>
<a id="f-441045ed4fa4-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-441045ed4fa4-use_declaration">
pub use compare::{Comparison, compare};
</storycode:copy>
<a id="f-441045ed4fa4-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-441045ed4fa4-use_declaration-2">
pub use import::parse;

</storycode:copy>
<a id="f-441045ed4fa4-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-441045ed4fa4-use_declaration-3">
use serde::{Deserialize, Serialize};
</storycode:copy>
<a id="f-441045ed4fa4-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-441045ed4fa4-use_declaration-4">
use std::collections::BTreeMap;

</storycode:copy>
<a id="f-441045ed4fa4-Branch"></a>
### `Branch`

<storycode:copy id="f-441045ed4fa4-Branch">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Branch {
    pub id: String,
    pub line: u32,
    /// None means the collector did not supply a count.
    pub count: Option<u64>,
    pub covered: bool,
}

</storycode:copy>
<a id="f-441045ed4fa4-FileCoverage"></a>
### `FileCoverage`

<storycode:copy id="f-441045ed4fa4-FileCoverage">
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileCoverage {
    /// Only executable lines. Missing entries are unknown/non-executable, not zero.
    pub lines: BTreeMap<u32, u64>,
    /// False means line values are reachability flags (0/1), not execution counts.
    pub line_counts: bool,
    /// Provider aggregate fractions, without inferred branch identities.
    pub branch_totals: BTreeMap<u32, (u64, u64)>,
    /// None is an unsupported metric; Some([]) is a measured empty denominator.
    pub branches: Option<Vec<Branch>>,
    pub excluded: Vec<u32>,
}

</storycode:copy>
<a id="f-441045ed4fa4-Report"></a>
### `Report`

<storycode:copy id="f-441045ed4fa4-Report">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub format: String,
    pub files: BTreeMap<String, FileCoverage>,
    /// The original report retains regions, contexts and other provider detail.
    pub raw: String,
}

</storycode:copy>
<a id="f-441045ed4fa4-Identity"></a>
### `Identity`

<storycode:copy id="f-441045ed4fa4-Identity">
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub collector: String,
    pub version: String,
    pub configuration: String,
    pub scope: String,
    pub source_hashes: BTreeMap<String, String>,
}

</storycode:copy>
<a id="f-441045ed4fa4-Evidence"></a>
### `Evidence`

<storycode:copy id="f-441045ed4fa4-Evidence">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub identity: Identity,
    pub report: Report,
    /// Snapshot of metadata used at run time; never re-read for historical runs.
    pub labels: Vec<String>,
    pub tests: Vec<String>,
    /// group, individual, or unavailable. Aggregate imports default to unavailable.
    pub attribution: String,
}
</storycode:copy>

### Assemble `crates/hick-coverage/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-coverage/src/lib.rs"><storycode:paste select="#f-441045ed4fa4-compare"/><storycode:paste select="#f-441045ed4fa4-import"/><storycode:paste select="#f-441045ed4fa4-use_declaration"/><storycode:paste select="#f-441045ed4fa4-use_declaration-2"/><storycode:paste select="#f-441045ed4fa4-use_declaration-3"/><storycode:paste select="#f-441045ed4fa4-use_declaration-4"/><storycode:paste select="#f-441045ed4fa4-Branch"/><storycode:paste select="#f-441045ed4fa4-FileCoverage"/><storycode:paste select="#f-441045ed4fa4-Report"/><storycode:paste select="#f-441045ed4fa4-Identity"/><storycode:paste select="#f-441045ed4fa4-Evidence"/></storycode:file>

<a id="output-eeca04287370"></a>
## `crates/hick-coverage/tests/evidence.rs`

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-eeca04287370-evidence"></a>
### `evidence`

<storycode:copy id="f-eeca04287370-evidence">
// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
use hick_coverage::*;
use std::collections::BTreeMap;
fn evidence(raw: &str) -> Evidence {
    Evidence {
        identity: Identity {
            collector: "test".into(),
            version: "1".into(),
            configuration: "branch".into(),
            scope: "app".into(),
            source_hashes: BTreeMap::from([("a.rs".into(), "abc".into())]),
        },
        report: parse("lcov", raw).unwrap(),
        labels: vec!["happy-path".into(), "boundary".into()],
        tests: vec![],
        attribution: "group".into(),
    }
}
</storycode:copy>
<a id="f-eeca04287370-comparison_uses_sets_and_keeps_missed_executable_lines"></a>
### `comparison_uses_sets_and_keeps_missed_executable_lines`

<storycode:copy id="f-eeca04287370-comparison_uses_sets_and_keeps_missed_executable_lines">
#[test]
fn comparison_uses_sets_and_keeps_missed_executable_lines() {
    let a = evidence("SF:a.rs\nDA:1,3\nDA:2,1\nDA:3,0\nDA:4,0\nend_of_record\n");
    let b = evidence("SF:a.rs\nDA:1,2\nDA:2,0\nDA:3,1\nDA:4,0\nend_of_record\n");
    let c = compare(&a, &b, "line").unwrap();
    assert_eq!(c.denominator, 4);
    assert_eq!(c.shared.into_iter().collect::<Vec<_>>(), ["a.rs:1"]);
    assert_eq!(c.left_only.into_iter().collect::<Vec<_>>(), ["a.rs:2"]);
    assert_eq!(c.right_only.into_iter().collect::<Vec<_>>(), ["a.rs:3"]);
    assert_eq!(c.missed.into_iter().collect::<Vec<_>>(), ["a.rs:4"]);
}
</storycode:copy>
<a id="f-eeca04287370-incompatible_or_unknown_sources_and_denominators_cannot_compare"></a>
### `incompatible_or_unknown_sources_and_denominators_cannot_compare`

<storycode:copy id="f-eeca04287370-incompatible_or_unknown_sources_and_denominators_cannot_compare">
#[test]
fn incompatible_or_unknown_sources_and_denominators_cannot_compare() {
    let a = evidence("SF:a.rs\nDA:1,1\nend_of_record\n");
    let mut b = a.clone();
    b.identity.version = "2".into();
    assert!(compare(&a, &b, "line").is_err());
    b = a.clone();
    b.identity.source_hashes.clear();
    assert!(compare(&b, &b, "line").is_err());
    b = a.clone();
    b.report.files.get_mut("a.rs").unwrap().lines.insert(2, 0);
    assert!(compare(&a, &b, "line").is_err());
    assert!(compare(&a, &a, "branch").is_err());
}
</storycode:copy>
<a id="f-eeca04287370-lcov_retains_unknown_branch_counts_and_merges_duplicate_records"></a>
### `lcov_retains_unknown_branch_counts_and_merges_duplicate_records`

<storycode:copy id="f-eeca04287370-lcov_retains_unknown_branch_counts_and_merges_duplicate_records">
#[test]
fn lcov_retains_unknown_branch_counts_and_merges_duplicate_records() {
    let r = parse("lcov", "SF:a.ts\nDA:1,2\nBRDA:1,0,0,-\nend_of_record\nSF:a.ts\nDA:1,3\nBRDA:1,0,0,1\nend_of_record\n").unwrap();
    assert_eq!(r.files["a.ts"].lines[&1], 5);
    let b = &r.files["a.ts"].branches.as_ref().unwrap()[0];
    assert!(b.covered);
    assert_eq!(b.count, None);
}
</storycode:copy>
<a id="f-eeca04287370-python_exclusions_and_arcs_remain_distinct"></a>
### `python_exclusions_and_arcs_remain_distinct`

<storycode:copy id="f-eeca04287370-python_exclusions_and_arcs_remain_distinct">
#[test]
fn python_exclusions_and_arcs_remain_distinct() {
    let raw = r#"{"meta":{"branch_coverage":true},"files":{"app.py":{"executed_lines":[1],"missing_lines":[2,3],"excluded_lines":[3],"executed_branches":[[1,2]],"missing_branches":[[1,-1]]}}}"#;
    let r = parse("coverage-py", raw).unwrap();
    let f = &r.files["app.py"];
    assert_eq!(f.lines.len(), 2);
    assert_eq!(f.excluded, [3]);
    let branches = f.branches.as_ref().unwrap();
    assert_eq!(branches.len(), 2);
    assert_eq!(branches[1].id, "1:-1");
    assert!(!branches[1].covered);
    assert_eq!(r.raw, raw);
}
</storycode:copy>
<a id="f-eeca04287370-cobertura_does_not_double_count_method_tables_or_invent_branches"></a>
### `cobertura_does_not_double_count_method_tables_or_invent_branches`

<storycode:copy id="f-eeca04287370-cobertura_does_not_double_count_method_tables_or_invent_branches">
#[test]
fn cobertura_does_not_double_count_method_tables_or_invent_branches() {
    let r = parse("cobertura", r#"<coverage><packages><package><classes><class filename="App.cs"><methods><method><lines><line number="1" hits="80"/></lines></method></methods><lines><line number="1" hits="2" branch="true" condition-coverage="50% (1/2)"/><line number="2" hits="0"/></lines></class></classes></package></packages></coverage>"#).unwrap();
    assert_eq!(r.files["App.cs"].lines[&1], 2);
    assert_eq!(r.files["App.cs"].branches, None);
    assert!(r.raw.contains("50% (1/2)"));
}
</storycode:copy>
<a id="f-eeca04287370-browser_istanbul_statement_and_branch_counters_are_preserved"></a>
### `browser_istanbul_statement_and_branch_counters_are_preserved`

<storycode:copy id="f-eeca04287370-browser_istanbul_statement_and_branch_counters_are_preserved">
#[test]
fn browser_istanbul_statement_and_branch_counters_are_preserved() {
    let r = parse("istanbul", r#"{"app.ts":{"path":"app.ts","statementMap":{"0":{"start":{"line":1}},"1":{"start":{"line":1}}},"s":{"0":0,"1":4},"branchMap":{"0":{"line":1,"locations":[{"start":{"line":1}},{"start":{"line":2}}]}},"b":{"0":[4,0]}}}"#).unwrap();
    let f = &r.files["app.ts"];
    assert_eq!(f.lines[&1], 4);
    assert_eq!(f.branches.as_ref().unwrap()[1].count, Some(0));
}
</storycode:copy>
<a id="f-eeca04287370-malformed_reports_are_not_zero_coverage"></a>
### `malformed_reports_are_not_zero_coverage`

<storycode:copy id="f-eeca04287370-malformed_reports_are_not_zero_coverage">
#[test]
fn malformed_reports_are_not_zero_coverage() {
    for (format, raw) in [
        ("lcov", "SF:a\nDA:0,1\nend_of_record"),
        ("lcov", "SF:a\nDA:1,1"),
        ("coverage-py", "{}"),
        ("cobertura", "<coverage/>"),
        ("istanbul", "{}"),
        ("other", ""),
    ] {
        assert!(parse(format, raw).is_err(), "{format}: {raw}");
    }
}
</storycode:copy>
<a id="f-eeca04287370-coverlet_source_roots_resolve_relative_class_paths"></a>
### `coverlet_source_roots_resolve_relative_class_paths`

<storycode:copy id="f-eeca04287370-coverlet_source_roots_resolve_relative_class_paths">
#[test]
fn coverlet_source_roots_resolve_relative_class_paths() {
    let report=parse("cobertura",r#"<coverage><sources><source>/</source></sources><packages><package><classes><class filename="tmp/project/Code.cs"><lines><line number="7" hits="1" branch="true" condition-coverage="50% (1/2)"/></lines></class></classes></package></packages></coverage>"#).unwrap();
    assert_eq!(
        report.files["/tmp/project/Code.cs"].branch_totals[&7],
        (1, 2)
    );
}
</storycode:copy>

### Assemble `crates/hick-coverage/tests/evidence.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-coverage/tests/evidence.rs"><storycode:paste select="#f-eeca04287370-evidence"/><storycode:paste select="#f-eeca04287370-comparison_uses_sets_and_keeps_missed_executable_lines"/><storycode:paste select="#f-eeca04287370-incompatible_or_unknown_sources_and_denominators_cannot_compare"/><storycode:paste select="#f-eeca04287370-lcov_retains_unknown_branch_counts_and_merges_duplicate_records"/><storycode:paste select="#f-eeca04287370-python_exclusions_and_arcs_remain_distinct"/><storycode:paste select="#f-eeca04287370-cobertura_does_not_double_count_method_tables_or_invent_branches"/><storycode:paste select="#f-eeca04287370-browser_istanbul_statement_and_branch_counters_are_preserved"/><storycode:paste select="#f-eeca04287370-malformed_reports_are_not_zero_coverage"/><storycode:paste select="#f-eeca04287370-coverlet_source_roots_resolve_relative_class_paths"/></storycode:file>

<a id="output-41d8c4218d5b"></a>
## `crates/hickory-cli/src/coverage/tests.rs`

Related contracts: [Coverage retains its measured scope](contract.md).

<a id="f-41d8c4218d5b-project"></a>
### `project`

<storycode:copy id="f-41d8c4218d5b-project">
// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
use super::*;
use serde_json::json;
fn project() -> (tempfile::TempDir, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap();
    std::fs::write(root.join("app.py"), "print('hello')\n").unwrap();
    (t, root)
}
</storycode:copy>
<a id="f-41d8c4218d5b-req"></a>
### `req`

<storycode:copy id="f-41d8c4218d5b-req">
fn req(value: Value) -> Request {
    serde_json::from_value(value).unwrap()
}
</storycode:copy>
<a id="f-41d8c4218d5b-aggregate_import_is_unknown_and_history_is_durable"></a>
### `aggregate_import_is_unknown_and_history_is_durable`

<storycode:copy id="f-41d8c4218d5b-aggregate_import_is_unknown_and_history_is_durable">
#[tokio::test]
async fn aggregate_import_is_unknown_and_history_is_durable() {
    let (_temp, root) = project();
    let result=dispatch(&root,req(json!({"action":"import","path":".","format":"lcov","raw":"SF:app.py\nDA:1,1\nend_of_record\n","labels":["happy-path"]}))).await.unwrap();
    assert_eq!(result["source_state"], "unknown");
    assert_eq!(result["evidence"]["attribution"], "unavailable");
    assert!(
        dispatch(
            &root,
            req(json!({"action":"compare","id":result["id"],"other":result["id"]}))
        )
        .await
        .is_err()
    );
    let all = dispatch(&root, req(json!({"action":"list"})))
        .await
        .unwrap();
    assert_eq!(all["total"], 1);
}
</storycode:copy>
<a id="f-41d8c4218d5b-browser_manual_baseline_checkpoint_stop_and_verdict_are_separate"></a>
### `browser_manual_baseline_checkpoint_stop_and_verdict_are_separate`

<storycode:copy id="f-41d8c4218d5b-browser_manual_baseline_checkpoint_stop_and_verdict_are_separate">
#[tokio::test]
async fn browser_manual_baseline_checkpoint_stop_and_verdict_are_separate() {
    let (_temp, root) = project();
    let coverage = |count| {
        json!({"app.py":{"path":"app.py","statementMap":{"0":{"start":{"line":1}}},"s":{"0":count},"f":{},"branchMap":{},"b":{}}}).to_string()
    };
    let session=dispatch(&root,req(json!({"action":"manual-start","path":".","raw":coverage(8),"labels":["adversarial","boundary"]}))).await.unwrap();
    let checkpoint = dispatch(
        &root,
        req(
            json!({"action":"checkpoint","id":session["id"],"format":"istanbul","raw":coverage(9)}),
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        checkpoint["evidence"]["report"]["files"]["app.py"]["lines"]["1"],
        1
    );
    assert_eq!(checkpoint["evidence"]["attribution"], "group");
    assert_eq!(
        checkpoint["evidence"]["labels"],
        json!(["adversarial", "boundary"])
    );
    assert!(checkpoint["manual_verdict"].is_null());
    let stopped = dispatch(&root, req(json!({"action":"stop","id":session["id"]})))
        .await
        .unwrap();
    assert_eq!(stopped["status"], "completed");
    let verdict=dispatch(&root,req(json!({"action":"annotate","id":session["id"],"verdict":"failed","notes":"Rejected input displayed the wrong error"}))).await.unwrap();
    assert_eq!(verdict["manual_verdict"], "failed");
    assert_eq!(
        verdict["evidence"]["report"]["files"]["app.py"]["lines"]["1"],
        1
    );
    std::fs::write(root.join("app.py"), "print('changed')\n").unwrap();
    let stale = dispatch(&root, req(json!({"action":"get","id":session["id"]})))
        .await
        .unwrap();
    assert_eq!(stale["source_state"], "historical");
}
</storycode:copy>
<a id="f-41d8c4218d5b-browser_counter_resets_are_not_coverage"></a>
### `browser_counter_resets_are_not_coverage`

<storycode:copy id="f-41d8c4218d5b-browser_counter_resets_are_not_coverage">
#[test]
fn browser_counter_resets_are_not_coverage() {
    let map =
        |n| json!({"a":{"statementMap":{},"branchMap":{},"s":{"0":n},"f":{},"b":{}}}).to_string();
    assert!(runner::browser_delta(&map(2), &map(1)).is_err());
    assert!(runner::browser_delta(&map(2), "{}").is_err());
}
</storycode:copy>
<a id="f-41d8c4218d5b-selectors_are_argv_and_cannot_inject_runner_options"></a>
### `selectors_are_argv_and_cannot_inject_runner_options`

<storycode:copy id="f-41d8c4218d5b-selectors_are_argv_and_cannot_inject_runner_options">
#[test]
fn selectors_are_argv_and_cannot_inject_runner_options() {
    let (_temp, root) = project();
    let mut r = store::new_record(
        &root,
        &root,
        &req(json!({"action":"run","language":"python"})),
    )
    .unwrap();
    assert!(commands::automated(&root, &root, &mut r, &["--collect-only".into()]).is_err());
    let p = commands::automated(
        &root,
        &root,
        &mut r,
        &["test_app.py::test_rejection".into()],
    )
    .unwrap();
    assert_eq!(p.argv.last().unwrap(), "test_app.py::test_rejection");
    assert!(p.argv.iter().any(|a| a == "--cov-branch"));
}
</storycode:copy>
<a id="f-41d8c4218d5b-python_junit_retains_failure_identity_and_source"></a>
### `python_junit_retains_failure_identity_and_source`

<storycode:copy id="f-41d8c4218d5b-python_junit_retains_failure_identity_and_source">
#[test]
fn python_junit_retains_failure_identity_and_source() {
    let (_temp, root) = project();
    let raw = r#"<testsuite><testcase classname="app.TestApp" name="test_bad[x]" file="app.py" line="4"><failure>expected rejection</failure></testcase><testcase classname="app" name="test_skip"><skipped>no account</skipped></testcase></testsuite>"#;
    let tests = results::parse("python", raw, "", &Config::default(), &root).unwrap();
    assert_eq!(tests[0].id, "app.py::TestApp::test_bad[x]");
    assert_eq!(tests[0].status, "failed");
    assert_eq!(tests[0].line, Some(5));
    assert!(tests[0].output.contains("expected rejection"));
    assert_eq!(tests[1].status, "skipped");
}
</storycode:copy>
<a id="f-41d8c4218d5b-csharp_trx_and_rust_results_keep_runner_test_identities"></a>
### `csharp_trx_and_rust_results_keep_runner_test_identities`

<storycode:copy id="f-41d8c4218d5b-csharp_trx_and_rust_results_keep_runner_test_identities">
#[test]
fn csharp_trx_and_rust_results_keep_runner_test_identities() {
    let raw = r#"<TestRun xmlns="http://microsoft.com/schemas/VisualStudio/TeamTest/2010"><TestDefinitions><UnitTest id="abc"><TestMethod className="App.Tests, App" name="Rejects"/></UnitTest></TestDefinitions><Results><UnitTestResult testId="abc" outcome="Failed"><Output><ErrorInfo><Message>wrong error</Message></ErrorInfo></Output></UnitTestResult></Results></TestRun>"#;
    let tests = results::parse("csharp", raw, "", &Config::default(), Path::new(".")).unwrap();
    assert_eq!(tests[0].id, "App.Tests.Rejects");
    assert_eq!(tests[0].status, "failed");
    assert!(tests[0].output.contains("wrong error"));
    let tests=results::parse("rust","","test tests::rejects ... FAILED\ntest tests::accepts ... ok\ntest tests::needs_account ... ignored",&Config::default(),Path::new(".")).unwrap();
    assert_eq!(tests.len(), 3);
    assert_eq!(tests[2].status, "skipped");
}
</storycode:copy>
<a id="f-41d8c4218d5b-javascript_results_and_discovery_preserve_full_names"></a>
### `javascript_results_and_discovery_preserve_full_names`

<storycode:copy id="f-41d8c4218d5b-javascript_results_and_discovery_preserve_full_names">
#[test]
fn javascript_results_and_discovery_preserve_full_names() {
    let raw=json!({"testResults":[{"name":"app.test.ts","assertionResults":[{"fullName":"validation rejects invalid","status":"failed","failureMessages":["expected error"],"location":{"line":12}}]}]}).to_string();
    let tests = results::parse("typescript", &raw, "", &Config::default(), Path::new(".")).unwrap();
    assert_eq!(tests[0].id, "validation rejects invalid");
    assert_eq!(tests[0].line, Some(12));
    let tests = results::discover(
        "typescript",
        r#"[{"name":"rejects invalid","fullName":"validation rejects invalid","file":"app.test.ts"}]"#,
        &Config::default(),
    );
    assert_eq!(tests[0].id, "validation rejects invalid");
}
</storycode:copy>
<a id="f-41d8c4218d5b-paths_and_storage_refuse_escape_and_symlinks"></a>
### `paths_and_storage_refuse_escape_and_symlinks`

<storycode:copy id="f-41d8c4218d5b-paths_and_storage_refuse_escape_and_symlinks">
#[test]
fn paths_and_storage_refuse_escape_and_symlinks() {
    let (_temp, root) = project();
    assert!(safe_path(&root, "../other").is_err());
    assert!(safe_path(&root, "/etc/passwd").is_err());
    assert!(store::load(&root, "../record").is_err());
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("outside")).unwrap();
        assert!(safe_path(&root, "outside").is_err());
        std::os::unix::fs::symlink(outside.path(), root.join(".hick-cache")).unwrap();
        assert!(store::list(&root).is_err());
    }
}
</storycode:copy>
<a id="f-41d8c4218d5b-imported_sources_cannot_escape_the_workspace"></a>
### `imported_sources_cannot_escape_the_workspace`

<storycode:copy id="f-41d8c4218d5b-imported_sources_cannot_escape_the_workspace">
#[tokio::test]
async fn imported_sources_cannot_escape_the_workspace() {
    let (_temp, root) = project();
    let outside = tempfile::NamedTempFile::new().unwrap();
    let raw = format!("SF:{}\nDA:1,1\nend_of_record\n", outside.path().display());
    assert!(
        dispatch(
            &root,
            req(json!({"action":"import","path":".","format":"lcov","raw":raw}))
        )
        .await
        .is_err()
    );
}
</storycode:copy>
<a id="f-41d8c4218d5b-generated_coverage_maps_only_matching_bytes_to_document"></a>
### `generated_coverage_maps_only_matching_bytes_to_document`

<storycode:copy id="f-41d8c4218d5b-generated_coverage_maps_only_matching_bytes_to_document">
#[test]
fn generated_coverage_maps_only_matching_bytes_to_document() {
    let (_temp, root) = project();
    let source = "# App\n\n<hick:file path=\"app.py\">\nprint('hello')\n</hick:file>\n";
    std::fs::write(root.join("app.md"), source).unwrap();
    let mut record = store::new_record(
        &root,
        &root,
        &req(json!({"action":"run","language":"python"})),
    )
    .unwrap();
    record.evidence = Some(
        evidence(
            &root,
            &record,
            "lcov",
            "SF:app.py\nDA:1,1\nend_of_record\n",
            "group",
            vec![],
        )
        .unwrap(),
    );
    let map = mapping::capture(&root, &record).unwrap();
    assert_eq!(map["app.py"][&1]["path"], "app.md");
    assert_eq!(map["app.py"][&1]["line"], 4);
    std::fs::write(root.join("app.py"), "different\n").unwrap();
    assert!(mapping::capture(&root, &record).unwrap().is_empty());
}

</storycode:copy>
<a id="f-41d8c4218d5b-agent_location_pages_distinguish_reachability_from_counts"></a>
### `agent_location_pages_distinguish_reachability_from_counts`

<storycode:copy id="f-41d8c4218d5b-agent_location_pages_distinguish_reachability_from_counts">
#[tokio::test]
async fn agent_location_pages_distinguish_reachability_from_counts() {
    let (_temp, root) = project();
    let identity = Identity {
        collector: "coverage.py".into(),
        version: "7".into(),
        configuration: "branches".into(),
        scope: ".".into(),
        source_hashes: snapshot(&root).unwrap(),
    };
    let raw=json!({"meta":{"branch_coverage":false},"files":{"app.py":{"executed_lines":[1],"missing_lines":[2],"excluded_lines":[]}}}).to_string();
    let imported=dispatch(&root,req(json!({"action":"import","path":".","format":"coverage-py","identity":identity,"raw":raw}))).await.unwrap();
    let page = dispatch(
        &root,
        req(json!({"action":"locations","id":imported["id"],"path":"app.py","limit":1})),
    )
    .await
    .unwrap();
    assert_eq!(page["denominator"], 2);
    assert_eq!(page["next"], 1);
    assert_eq!(page["locations"][0]["covered"], true);
    assert!(page["locations"][0]["count"].is_null());
    assert!(page["tests"].is_null());
    assert!(
        dispatch(
            &root,
            req(json!({"action":"locations","id":imported["id"],"metric":"branch"}))
        )
        .await
        .is_err()
    );
}
</storycode:copy>
<a id="f-41d8c4218d5b-a_browser_reload_invalidates_even_larger_counters"></a>
### `a_browser_reload_invalidates_even_larger_counters`

<storycode:copy id="f-41d8c4218d5b-a_browser_reload_invalidates_even_larger_counters">
#[test]
fn a_browser_reload_invalidates_even_larger_counters() {
    let map = |epoch, n| {
        json!({"app.js":{"__hickoryTimeOrigin":epoch,"statementMap":{},"branchMap":{},"s":{"0":n},"f":{},"b":{}}}).to_string()
    };
    assert!(runner::browser_delta(&map(1, 2), &map(2, 3)).is_err());
}
</storycode:copy>
<a id="f-41d8c4218d5b-metadata_labels_are_snapshots_not_reinterpreted_from_current_config"></a>
### `metadata_labels_are_snapshots_not_reinterpreted_from_current_config`

<storycode:copy id="f-41d8c4218d5b-metadata_labels_are_snapshots_not_reinterpreted_from_current_config">
#[tokio::test]
async fn metadata_labels_are_snapshots_not_reinterpreted_from_current_config() {
    let (_temp, root) = project();
    std::fs::write(
        root.join(".hick-tests.json"),
        r#"{"tests":{"app.py::test_x":["happy-path"]}}"#,
    )
    .unwrap();
    let record = store::new_record(
        &root,
        &root,
        &req(json!({"action":"run","language":"python"})),
    )
    .unwrap();
    store::save(&root, &record).unwrap();
    std::fs::write(
        root.join(".hick-tests.json"),
        r#"{"tests":{"app.py::test_x":["adversarial"]}}"#,
    )
    .unwrap();
    let old = store::load(&root, &record.id).unwrap();
    assert_eq!(old.metadata.tests["app.py::test_x"], ["happy-path"]);
    assert_eq!(
        store::view(&root, &record.id).unwrap()["source_state"],
        "historical"
    );
}
</storycode:copy>
<a id="f-41d8c4218d5b-malformed_runner_output_does_not_panic_discovery"></a>
### `malformed_runner_output_does_not_panic_discovery`

<storycode:copy id="f-41d8c4218d5b-malformed_runner_output_does_not_panic_discovery">
#[test]
fn malformed_runner_output_does_not_panic_discovery() {
    assert!(results::discover("typescript", "}\n{\n]\n[", &Config::default()).is_empty());
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/coverage/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/coverage/tests.rs"><storycode:paste select="#f-41d8c4218d5b-project"/><storycode:paste select="#f-41d8c4218d5b-req"/><storycode:paste select="#f-41d8c4218d5b-aggregate_import_is_unknown_and_history_is_durable"/><storycode:paste select="#f-41d8c4218d5b-browser_manual_baseline_checkpoint_stop_and_verdict_are_separate"/><storycode:paste select="#f-41d8c4218d5b-browser_counter_resets_are_not_coverage"/><storycode:paste select="#f-41d8c4218d5b-selectors_are_argv_and_cannot_inject_runner_options"/><storycode:paste select="#f-41d8c4218d5b-python_junit_retains_failure_identity_and_source"/><storycode:paste select="#f-41d8c4218d5b-csharp_trx_and_rust_results_keep_runner_test_identities"/><storycode:paste select="#f-41d8c4218d5b-javascript_results_and_discovery_preserve_full_names"/><storycode:paste select="#f-41d8c4218d5b-paths_and_storage_refuse_escape_and_symlinks"/><storycode:paste select="#f-41d8c4218d5b-imported_sources_cannot_escape_the_workspace"/><storycode:paste select="#f-41d8c4218d5b-generated_coverage_maps_only_matching_bytes_to_document"/><storycode:paste select="#f-41d8c4218d5b-agent_location_pages_distinguish_reachability_from_counts"/><storycode:paste select="#f-41d8c4218d5b-a_browser_reload_invalidates_even_larger_counters"/><storycode:paste select="#f-41d8c4218d5b-metadata_labels_are_snapshots_not_reinterpreted_from_current_config"/><storycode:paste select="#f-41d8c4218d5b-malformed_runner_output_does_not_panic_discovery"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/coverage/gutter.ts](contract.md#output-7ccf384862a0)
- [crates/hick-coverage/src/compare.rs](contract.md#output-692cdd9dae7c)
- [crates/hick-coverage/src/import.rs](contract.md#output-4d440b3acbf1)
- [crates/hick-coverage/src/lib.rs](contract.md#output-441045ed4fa4)
- [crates/hickory-cli/src/coverage/mapping.rs](../../../mechanisms/crates/hickory-cli/src/coverage/mapping.rs.md#output-9c1a7ccb215d)
- [crates/hickory-cli/src/coverage/tests.rs](contract.md#output-41d8c4218d5b)

### Tests and fixtures

- [apps/web/src/coverage/CoverageTools.test.tsx](contract.md#output-814b32909c24)
- [apps/web/src/coverage/gutter.test.ts](contract.md#output-043842eaed53)
- [apps/web/src/coverage/selection.test.ts](contract.md#output-49aaee6bbbd9)
- [crates/hick-coverage/tests/evidence.rs](contract.md#output-eeca04287370)
- [crates/hickory-cli/src/serve/test_coverage.rs](../../../mechanisms/crates/hickory-cli/src/serve/test_coverage.rs.md#output-cdbb6e20bcf7)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
