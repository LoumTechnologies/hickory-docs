import type { Block, Doc, Project, TranscriptEvent } from "../api/types";

// ---------------------------------------------------------------------------
// Mock document 1: runnable CLI-tool documentation (the exedocs use case).
// ---------------------------------------------------------------------------

export const CLI_SOURCE = `# hick quickstart

Hickory Docs turns documentation into a verified pipeline. Every example
below executes on \`hick test\`; drift between docs and binary is a
build failure.

Everything below runs in one declared environment:

<hick:container name="shell" image="debian:12" />

Install the CLI, then confirm the version:

<hick:exec container="shell" image="debian:12">
hick --version
<hick:expect match="regex-lines">
hick \\d+\\.\\d+\\.\\d+
</hick:expect>
</hick:exec>

Initialise a project. This installs the pre-commit verification hook:

<hick:exec container="shell">
hick init demo && ls demo/.hick
</hick:exec>

Write a document and run it. Output below is captured as a timed
transcript, replayable in the web UI:

<hick:exec container="shell">
hick run demo/hello.md
<hick:expect match="exact">
converged: 3 nodes, 0 stale
</hick:expect>
</hick:exec>
`;

function span(source: string, needle: string): [number, number] {
  const start = source.indexOf(needle);
  if (start < 0) throw new Error(`mock span not found: ${needle.slice(0, 30)}`);
  return [start, start + needle.length];
}

const versionTranscript: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "hick --version" },
  { t: 350, kind: "out", data: "hick 0.4.2\n" },
  { t: 380, kind: "exit", code: 0 },
];

const initTranscript: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "hick init demo && ls demo/.hick" },
  { t: 420, kind: "out", data: "initialised demo/\n" },
  { t: 700, kind: "out", data: "installed pre-commit hook (sentinel-delimited)\n" },
  { t: 1100, kind: "out", data: "cache  git  hooks\n" },
  { t: 1150, kind: "exit", code: 0 },
];

export const CLI_BLOCKS: Block[] = [
  {
    kind: "prose",
    html: "<h1>hick quickstart</h1><p>Hickory Docs turns documentation into a verified pipeline. Every example below executes on <code>hick test</code>; drift between docs and binary is a build failure.</p><p>Install the CLI, then confirm the version:</p>",
    span: [0, CLI_SOURCE.indexOf("<hick:exec")],
  },
  {
    kind: "exec",
    id: "cli-version",
    container: "shell",
    image: "debian:12",
    command: "hick --version",
    span: span(CLI_SOURCE, '<hick:exec container="shell" image="debian:12">\nhick --version\n<hick:expect match="regex-lines">\nhick \\d+\\.\\d+\\.\\d+\n</hick:expect>\n</hick:exec>'),
    transcript: versionTranscript,
    expect: { match: "regex-lines", body: "hick \\d+\\.\\d+\\.\\d+" },
    status: "ok",
  },
  {
    kind: "prose",
    html: "<p>Initialise a project. This installs the pre-commit verification hook:</p>",
    span: span(CLI_SOURCE, "Initialise a project. This installs the pre-commit verification hook:"),
  },
  {
    kind: "exec",
    id: "cli-init",
    container: "shell",
    command: "hick init demo && ls demo/.hick",
    span: span(CLI_SOURCE, '<hick:exec container="shell">\nhick init demo && ls demo/.hick\n</hick:exec>'),
    transcript: initTranscript,
    status: "ok",
  },
  {
    kind: "prose",
    html: "<p>Write a document and run it. Output below is captured as a timed transcript, replayable in the web UI:</p>",
    span: span(CLI_SOURCE, "Write a document and run it."),
  },
  {
    kind: "exec",
    id: "cli-run",
    container: "shell",
    command: "hick run demo/hello.md",
    span: span(CLI_SOURCE, '<hick:exec container="shell">\nhick run demo/hello.md\n<hick:expect match="exact">\nconverged: 3 nodes, 0 stale\n</hick:expect>\n</hick:exec>'),
    expect: { match: "exact", body: "converged: 3 nodes, 0 stale" },
    status: "unrecorded",
  },
];

// ---------------------------------------------------------------------------
// Mock document 2: a statistical-paper-style document whose figure is
// reproduced from source data by an executed cell (stdout is inline SVG).
// ---------------------------------------------------------------------------

export const PAPER_SOURCE = `# Convergence latency under load

We measure wall-clock convergence latency of the reactive graph as the
number of dirty nodes grows. Each point is the median of 50 runs on a
single Canopy microVM (2 vCPU).

<hick:file path="analysis/latency.py" language="python">
import json, statistics
runs = json.load(open("data/runs.json"))
medians = {n: statistics.median(r) for n, r in runs.items()}
print(render_svg(medians))
</hick:file>

Reproduce the figure from the raw data:

<hick:exec container="py" image="python:3.12">
python analysis/latency.py
</hick:exec>

Latency grows sub-linearly to 256 dirty nodes: the converge pass batches
sibling recomputation, so fan-out is amortised. Beyond 256 nodes the
cache-miss cliff dominates (see appendix).
`;

const CHART_SVG = `<svg viewBox="0 0 560 300" role="img" aria-label="Median convergence latency by dirty-node count" xmlns="http://www.w3.org/2000/svg" font-family="inherit">
<g stroke="currentColor" stroke-opacity="0.15">
<line x1="60" y1="40" x2="60" y2="250"/><line x1="60" y1="250" x2="530" y2="250"/>
<line x1="60" y1="180" x2="530" y2="180" stroke-dasharray="3 4"/>
<line x1="60" y1="110" x2="530" y2="110" stroke-dasharray="3 4"/>
<line x1="60" y1="40" x2="530" y2="40" stroke-dasharray="3 4"/>
</g>
<g fill="currentColor" fill-opacity="0.65" font-size="11">
<text x="52" y="254" text-anchor="end">0</text>
<text x="52" y="184" text-anchor="end">40</text>
<text x="52" y="114" text-anchor="end">80</text>
<text x="52" y="44" text-anchor="end">120 ms</text>
<text x="60" y="270" text-anchor="middle">1</text>
<text x="154" y="270" text-anchor="middle">4</text>
<text x="248" y="270" text-anchor="middle">16</text>
<text x="342" y="270" text-anchor="middle">64</text>
<text x="436" y="270" text-anchor="middle">256</text>
<text x="530" y="270" text-anchor="middle">1024</text>
<text x="295" y="292" text-anchor="middle">dirty nodes (log scale)</text>
</g>
<polyline points="60,244 154,238 248,224 342,196 436,141 530,52" fill="none" stroke="#4a7d5f" stroke-width="2.5" stroke-linejoin="round"/>
<g fill="#4a7d5f">
<circle cx="60" cy="244" r="4"/><circle cx="154" cy="238" r="4"/><circle cx="248" cy="224" r="4"/>
<circle cx="342" cy="196" r="4"/><circle cx="436" cy="141" r="4"/><circle cx="530" cy="52" r="4"/>
</g>
</svg>`;

export const PAPER_CHART_SVG = CHART_SVG;

const paperTranscript: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "python analysis/latency.py" },
  { t: 600, kind: "out", data: CHART_SVG },
  { t: 640, kind: "exit", code: 0 },
];

export const PAPER_BLOCKS: Block[] = [
  {
    kind: "prose",
    html: "<h1>Convergence latency under load</h1><p>We measure wall-clock convergence latency of the reactive graph as the number of dirty nodes grows. Each point is the median of 50 runs on a single Canopy microVM (2 vCPU).</p>",
    span: [0, PAPER_SOURCE.indexOf("<hick:file")],
  },
  {
    kind: "file",
    path: "analysis/latency.py",
    language: "python",
    body: 'import json, statistics\nruns = json.load(open("data/runs.json"))\nmedians = {n: statistics.median(r) for n, r in runs.items()}\nprint(render_svg(medians))',
    span: span(PAPER_SOURCE, '<hick:file path="analysis/latency.py" language="python">'),
  },
  {
    kind: "prose",
    html: "<p>Reproduce the figure from the raw data:</p>",
    span: span(PAPER_SOURCE, "Reproduce the figure from the raw data:"),
  },
  {
    kind: "exec",
    id: "paper-figure",
    container: "py",
    image: "python:3.12",
    command: "python analysis/latency.py",
    span: span(PAPER_SOURCE, '<hick:exec container="py" image="python:3.12">\npython analysis/latency.py\n</hick:exec>'),
    transcript: paperTranscript,
    status: "stale",
  },
  {
    kind: "prose",
    html: "<p>Latency grows sub-linearly to 256 dirty nodes: the converge pass batches sibling recomputation, so fan-out is amortised. Beyond 256 nodes the cache-miss cliff dominates (see appendix).</p>",
    span: span(PAPER_SOURCE, "Latency grows sub-linearly"),
  },
];

// ---------------------------------------------------------------------------
// Mock document 3: literate weaving — two hick:copy slots woven into one
// generated Python file, demonstrating /outputs lineage and edit-back.
// ---------------------------------------------------------------------------

export const WEAVE_SOURCE = `# Woven module demo

This document **weaves** one Python module from two named slots. Edit the
document *or* the generated output — hick maps output edits back to the
source spans they came from (note: raw < and > need no escaping here).

The loader slot:

<hick:copy id="load">
def load_runs(path):
    import json
    with open(path) as f:
        return json.load(f)
</hick:copy>

The summary slot:

<hick:copy id="summarise">
def summarise(runs):
    import statistics
    return {n: statistics.median(r) for n, r in runs.items()}
</hick:copy>

Weave both slots plus a literal entry point into \`src/latency.py\`:

<hick:file path="src/latency.py" language="python">
<hick:paste select="#load" />

<hick:paste select="#summarise" />

if __name__ == "__main__":
    print(summarise(load_runs("data/runs.json")))
</hick:file>

Run the woven module:

<hick:exec container="py" image="python:3.12">
python src/latency.py
</hick:exec>
`;

export const WEAVE_BLOCKS: Block[] = [
  {
    kind: "prose",
    html: "<h1>Woven module demo</h1><p>This document <strong>weaves</strong> one Python module from two named slots. Edit the document <em>or</em> the generated output — hick maps output edits back to the source spans they came from.</p>",
    span: [0, WEAVE_SOURCE.indexOf("<hick:copy")],
  },
  {
    kind: "file",
    path: "src/latency.py",
    language: "python",
    body: '<hick:paste select="#load" />\n\n<hick:paste select="#summarise" />\n\nif __name__ == "__main__":\n    print(summarise(load_runs("data/runs.json")))',
    span: span(WEAVE_SOURCE, '<hick:file path="src/latency.py" language="python">'),
  },
  {
    kind: "exec",
    id: "weave-run",
    container: "py",
    image: "python:3.12",
    command: "python src/latency.py",
    span: span(
      WEAVE_SOURCE,
      '<hick:exec container="py" image="python:3.12">\npython src/latency.py\n</hick:exec>',
    ),
    status: "unrecorded",
  },
];

// ---------------------------------------------------------------------------
// Projects, docs
// ---------------------------------------------------------------------------

export const MOCK_PROJECTS: Project[] = [
  { id: "p1", name: "hick", visibility: "public", created_at: "2026-07-02T10:00:00Z" },
  { id: "p2", name: "latency-paper", visibility: "private", created_at: "2026-07-20T09:30:00Z" },
];

export const MOCK_DOCS: (Doc & { project_id: string })[] = [
  {
    id: "d1",
    project_id: "p1",
    path: "docs/quickstart.md",
    source: CLI_SOURCE,
    updated_at: "2026-08-01T16:12:00Z",
  },
  {
    id: "d2",
    project_id: "p2",
    path: "paper/convergence.md",
    source: PAPER_SOURCE,
    updated_at: "2026-08-03T11:45:00Z",
  },
  {
    id: "d3",
    project_id: "p1",
    path: "docs/weave-demo.md",
    source: WEAVE_SOURCE,
    updated_at: "2026-08-04T09:05:00Z",
  },
];

export const MOCK_BLOCKS: Record<string, Block[]> = {
  d1: CLI_BLOCKS,
  d2: PAPER_BLOCKS,
  d3: WEAVE_BLOCKS,
};

