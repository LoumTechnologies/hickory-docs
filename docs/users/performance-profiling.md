# Inspect performance hotspots in literate source

For developers already using Hickory's code-coverage view who want to measure a
local app or test workload, including interactions performed by a human or AI.

Open **Workspace tools → Performance profiles**. Collect a workload, select its
recorded run and measured scope, then click a function to open its source. When
exact generated-file mappings exist, Hickory opens the literate document and
shows an expandable hotspot beside the function. Opening the hotspot selects
that evidence for agent context. Collapsing it removes that selection.

The hotspot shows self cost, inclusive cost, stack observations and available call
counts. These are function aggregates anchored at source locations. They are not
per-line timings. Inclusive costs overlap; sampling observations are not calls.

## Profile a browser interaction

1. Start Chrome with a separate profile and a loopback debugging port:
   `google-chrome --remote-debugging-port=9222 --user-data-dir=/tmp/hickory-profile-chrome`.
   Open your local app in that browser.
2. In **Performance profiles**, enter the project directory and any labels, such
   as `adversarial` or `manual`. Enter `http://127.0.0.1:9222` and enough of the
   page URL to identify exactly one tab.
3. Choose **Start browser CPU profile**. Exercise the app yourself, or have an
   AI automate that same tab. Choose **Stop and collect profile**.
4. Refresh and select the completed run. Inspect the function table, then click
   a source anchor to open its literate document. Expand a hotspot to inspect
   the exact scoped evidence or include it in agent context.

Browser profiling uses V8 sampling; it does not require Istanbul instrumentation.
Its scope is one page's V8 isolate. It does not measure backend processes, workers,
GPU work, all browser threads or end-to-end workflow latency. Idle and runtime
frames remain in the report. Sampling introduces noise and overhead.

JavaScript anchors require the executed script bytes to match the saved file.
TypeScript/transformed-source anchors require a source map with matching original
source contents. Inline base64 source maps and maps on the profiled loopback HTTP
origin are supported. Missing maps, native frames and unmatched sources stay
visible as unresolved; their costs are not assigned to guessed document lines.
A page reload invalidates the session; collect another segment after navigation.
Browser sessions are capped at 15 minutes.

The built-in and external agents use the same `performance_profile` tool:

> Start a browser profile for localhost:3000 through CDP at
> http://127.0.0.1:9222. Label it manual-search. Automate the search interaction in
> that tab, stop collection, inspect the hottest functions with the locations
> action, and open their mapped literate source. Report performance evidence
> separately from whether the interaction behaved correctly.

The agent needs its own browser-automation tool to interact with the application.
`performance_profile` collects and reads the profiler evidence.

## Profile saved Python

Enter a saved `.py` file or literate document. Under **Saved Python and native
apps or test workloads**, select **Profile saved Python confined**. For a
document with multiple generated programs, enter its generated Python entry.

This reuses Hickory's existing cProfile provider and cell sandbox, with a bounded
10-second capture in this UI. It retains total/primitive calls, self seconds,
cumulative seconds, environment and the original runtime-run identity. It does
not grant unrestricted execution to the Python agent. Native and unmapped
functions remain explicit. Function times include profiling overhead.

For Python apps requiring their own project environment, use an explicit native
collector profile and an exporter producing `cprofile-json` instead.

## Profile an app or automated test workload

Put `.hick-profile.json` in the selected project directory. It maps names to
collector profiles. For example, a Node workload can use its built-in CPU profiler:

```json
{
  "node-workload": {
    "argv": ["node", "--cpu-prof", "--cpu-prof-dir={root}/.hick-cache/profiles/{run}", "--cpu-prof-name=run.cpuprofile", "app.js"],
    "format": "chrome-cpu",
    "report": ".hick-cache/profiles/{run}/run.cpuprofile",
    "collector": "node-v8",
    "version_argv": ["node", "--version"]
  }
}
```

Choose **Read project profiles**, enter `node-workload`, and choose **Run profiler
profile**. Replace the application arguments with the test runner's entry point
to profile an automated workload. Long-running apps can be exercised manually and
stopped with **Stop and collect profile**. Their collector must support flushing
on interruption or through its configured `finish` command. An incomplete export
is unavailable or partial; it is not a successful empty profile.

Each argument is passed directly, without shell expansion. `{root}` names the
workspace, `{project}` the selected project, `{run}` the unique profile-run ID,
and `{report}` the absolute report destination. Keep runtime artifacts under
`.hick-cache` and include `{run}` to separate simultaneous runs. `finish` is an
optional argument array run after the collector exits. `version_argv` observes the
installed version; a literal `version` is a project declaration. Missing versions
remain unknown and prevent compatible-cost comparisons.

Native profiles run with your normal local permissions. They do not change the
existing confined Python provider's permissions.

## C#, Rust and existing profiler reports

| Workload | Collector/report route | Source availability |
| --- | --- | --- |
| C# | `dotnet-trace collect --format Speedscope` through a native profile | Frames without file/line metadata remain unresolved |
| Rust | Symbolicated processed Firefox/samply export, or a Speedscope export with source locations | Debug symbols and supplied file/line metadata are required for inline anchors |
| Python | Confined cProfile or a project exporter producing `cprofile-json` | Exact saved source and collector locations |
| TypeScript/JavaScript | Browser V8 CPU profile or Node `.cpuprofile` | Matching JavaScript or verified source maps |

For a built .NET application, a collector profile can launch:

```json
{
  "dotnet-workload": {
    "argv": ["dotnet-trace", "collect", "--format", "Speedscope", "--output", "{root}/.hick-cache/profiles/{run}/trace.nettrace", "--", "dotnet", "bin/Release/net10.0/App.dll"],
    "format": "speedscope",
    "report": ".hick-cache/profiles/{run}/trace.speedscope.json",
    "collector": "dotnet-trace",
    "version_argv": ["dotnet-trace", "--version"]
  }
}
```

Use your application's actual framework/output path. Speedscope scopes stay
separate; Hickory does not add multiple threads' inclusive durations together.
For Rust, import a processed/symbolicated export: raw addresses or unconverted
Gecko tables cannot establish source locations. Native sampling permissions and
symbol availability belong to the installed collector. Hickory does not install
collectors or alter kernel settings.

**Import a profile** accepts `chrome-cpu`, `speedscope`, `firefox` (processed) and
`cprofile-json`. To attach imported evidence to source, supply its original
`collector`, `version`, `configuration`, `scope` and `source_hashes` identity JSON.
Without it, evidence remains unknown and does not decorate current source. Source
paths must resolve within the workspace.

The `cprofile-json` format contains `elapsed_seconds`, an optional `scope`, and
`functions` with `name`, `file`, one-based `line`, `total_calls`, `primitive_calls`,
`self_seconds` and `cumulative_seconds`. Preserve the actual measured elapsed
interval; do not substitute a sum of inclusive function times.

## Compare and retain evidence

Select two runs collected with compatible versions, configuration, scope and
metric semantics. **Compare profile costs** retains both source identities and
shows self-cost differences for corresponding functions. Source revisions may
differ for before/after optimization. A missing function stays unavailable rather
than becoming a zero-cost speedup. Inputs, environment, sampling noise and profiler
overhead can affect the result. Rerun correctness independently.

Editing source removes current inline annotations while preserving the old run.
Records, source snapshots, mappings and original artifacts are retained under
`.hick-cache/profiles`. Project-owned commands use the live working tree; changes
detected during collection are flagged. The UI distinguishes historical and
unknown evidence.

Agents should use `performance_profile` with `action: "locations"`, `id`,
`thread`, `offset` and `limit` for bounded reads. `get` retrieves the full artifact.
The UI and agent service share `POST /api/performance`; expanded inline evidence
also uses the existing shared source-anchored observation/context tools.
