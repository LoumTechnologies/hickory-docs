import type { Block, Doc, Plan, Project, TranscriptEvent } from "../api/types";

// ---------------------------------------------------------------------------
// Mock document 1: runnable CLI-tool documentation (the exedocs use case).
// ---------------------------------------------------------------------------

export const CLI_SOURCE = `# hickory quickstart

Hickory Docs turns documentation into a verified pipeline. Every example
below executes on \`hickory check\`; drift between docs and binary is a
build failure.

Install the CLI, then confirm the version:

<hick:exec container="shell" image="debian:12">
hickory --version
<hick:expect match="regex-lines">
hickory \\d+\\.\\d+\\.\\d+
</hick:expect>
</hick:exec>

Initialise a project. This installs the pre-commit verification hook:

<hick:exec container="shell">
hickory init demo && ls demo/.hick
</hick:exec>

Write a document and run it. Output below is captured as a timed
transcript, replayable in the web UI:

<hick:exec container="shell">
hickory run demo/hello.hick
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
  { t: 0, kind: "cmd", data: "hickory --version" },
  { t: 350, kind: "out", data: "hickory 0.4.2\n" },
  { t: 380, kind: "exit", code: 0 },
];

const initTranscript: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "hickory init demo && ls demo/.hick" },
  { t: 420, kind: "out", data: "initialised demo/\n" },
  { t: 700, kind: "out", data: "installed pre-commit hook (sentinel-delimited)\n" },
  { t: 1100, kind: "out", data: "cache  git  hooks\n" },
  { t: 1150, kind: "exit", code: 0 },
];

export const CLI_BLOCKS: Block[] = [
  {
    kind: "prose",
    html: "<h1>hickory quickstart</h1><p>Hickory Docs turns documentation into a verified pipeline. Every example below executes on <code>hickory check</code>; drift between docs and binary is a build failure.</p><p>Install the CLI, then confirm the version:</p>",
    span: [0, CLI_SOURCE.indexOf("<hick:exec")],
  },
  {
    kind: "exec",
    id: "cli-version",
    container: "shell",
    image: "debian:12",
    command: "hickory --version",
    span: span(CLI_SOURCE, '<hick:exec container="shell" image="debian:12">\nhickory --version\n<hick:expect match="regex-lines">\nhickory \\d+\\.\\d+\\.\\d+\n</hick:expect>\n</hick:exec>'),
    transcript: versionTranscript,
    expect: { match: "regex-lines", body: "hickory \\d+\\.\\d+\\.\\d+" },
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
    command: "hickory init demo && ls demo/.hick",
    span: span(CLI_SOURCE, '<hick:exec container="shell">\nhickory init demo && ls demo/.hick\n</hick:exec>'),
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
    command: "hickory run demo/hello.hick",
    span: span(CLI_SOURCE, '<hick:exec container="shell">\nhickory run demo/hello.hick\n<hick:expect match="exact">\nconverged: 3 nodes, 0 stale\n</hick:expect>\n</hick:exec>'),
    expect: { match: "exact", body: "converged: 3 nodes, 0 stale" },
    status: "never-run",
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
// Projects, docs, plans
// ---------------------------------------------------------------------------

export const MOCK_PROJECTS: Project[] = [
  { id: "p1", name: "hickory", visibility: "public", created_at: "2026-07-02T10:00:00Z" },
  { id: "p2", name: "latency-paper", visibility: "private", created_at: "2026-07-20T09:30:00Z" },
];

export const MOCK_DOCS: (Doc & { project_id: string })[] = [
  {
    id: "d1",
    project_id: "p1",
    path: "docs/quickstart.hick",
    source: CLI_SOURCE,
    updated_at: "2026-08-01T16:12:00Z",
  },
  {
    id: "d2",
    project_id: "p2",
    path: "paper/convergence.hick",
    source: PAPER_SOURCE,
    updated_at: "2026-08-03T11:45:00Z",
  },
];

export const MOCK_BLOCKS: Record<string, Block[]> = {
  d1: CLI_BLOCKS,
  d2: PAPER_BLOCKS,
};

export const MOCK_PLANS: Plan[] = [
  {
    key: "free",
    name: "Free",
    description: "Public projects, local execution, community support.",
    price_key: "free",
    amount_cents: 0,
    currency: "usd",
    interval: "month",
    features: ["Unlimited public projects", "Local executor", "Transcript playback", "Community support"],
  },
  {
    key: "pro",
    name: "Pro",
    description: "Private projects and Canopy microVM execution.",
    price_key: "price_pro_monthly",
    amount_cents: 1800,
    currency: "usd",
    interval: "month",
    features: [
      "Everything in Free",
      "Private projects",
      "Canopy microVM execution (500 min/mo)",
      "AI agent sessions",
      "Email support",
    ],
    highlight: true,
  },
  {
    key: "team",
    name: "Team",
    description: "Shared workspaces, verification in CI, priority execution.",
    price_key: "price_team_monthly",
    amount_cents: 6000,
    currency: "usd",
    interval: "month",
    features: [
      "Everything in Pro",
      "Shared workspaces & live collaboration",
      "CI verification tokens",
      "Priority microVM pool (2,000 min/mo)",
      "Priority support",
    ],
  },
];
