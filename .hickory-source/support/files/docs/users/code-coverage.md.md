<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/users/code-coverage.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `b4c597379049eca71d60196eeca5f8fe73e2f1c4081d89a0fc2a0d4bf3b52362`.

<book:entry path="docs/users/code-coverage.md">
# Check which code a test or manual session reached

For developers who run Python, C#, TypeScript or Rust apps locally and want to
inspect coverage from automated tests, their own app interactions, or an AI's
browser automation.

Open **Workspace tools → Tests & coverage**. Enter a project directory or one
of its source files, select the language, and choose **Discover tests** followed
by **Run with coverage**. This runs the selected project's test suite. To run a
smaller group, configure it below and choose **Read project groups and profiles**.

Install the collector in the project before collecting:

| Language | Test runner and collector | Report |
| --- | --- | --- |
| Python | pytest, pytest-cov and coverage.py in the project's `.venv` or selected uv environment | coverage.py JSON with branch arcs |
| TypeScript/JavaScript | Project-installed Vitest with its matching coverage plugin, or Jest with the project's transforms | LCOV |
| C# | `dotnet test` using VSTest and `coverlet.collector` | Cobertura plus TRX results |
| Rust | `cargo-llvm-cov` and Rust's `llvm-tools-preview` component | LCOV |

Collection never installs dependencies automatically. Unsupported runners or
missing plugins report unavailable coverage. Microsoft.Testing.Platform reports
can be imported as Cobertura; its runner switches differ from VSTest's switches.

Select a recorded run to see executable lines, branch data, results and failure
output. **Rerun failed tests** uses the recorded runner identities against the
current saved project. It creates a new run. Coverage remains useful when a test
fails; the test result is recorded separately.

## Separate happy-path and adversarial coverage

Put `.hick-tests.json` in the test project's manifest directory:

```json
{
  "groups": {
    "happy-path": {
      "selectors": ["tests/test_validation.py::test_accepts"],
      "labels": ["boundary"]
    },
    "adversarial": {
      "selectors": ["tests/test_validation.py::test_rejects"],
      "labels": ["boundary"]
    }
  },
  "tests": {
    "tests/test_validation.py::test_accepts": ["happy-path", "boundary"],
    "tests/test_validation.py::test_rejects": ["adversarial", "boundary"]
  }
}
```

Selectors use pytest node IDs, Rust test-name filters, C# fully qualified method
names, or Vitest/Jest full test names. Each selected group runs separately and
keeps its own report. Labels overlap; they do not add percentages.

Choose one run, another run, and **Compare compatible groups**. The comparison
shows shared locations, locations unique to either run, and executable locations
missed by both. Source hashes, collector/version, configuration, measured scope
and executable denominators must match. Missing or incompatible evidence produces
an explanation. Compatible older evidence is labeled historical.

## Test an app yourself, or ask an AI to drive it

For browser apps, enable Istanbul instrumentation in the app's development build
so its tested page exposes `globalThis.__coverage__`. Instrument the original
source with source maps configured by your build tool. Reported paths must resolve
to files in the open workspace. Production builds normally omit instrumentation.

1. Start Chrome with remote debugging on a loopback port and a separate profile:
   `google-chrome --remote-debugging-port=9222 --user-data-dir=/tmp/hickory-coverage-chrome`.
   Open your instrumented app in that browser.
2. Open **Human or AI manual testing**, enter `http://127.0.0.1:9222`, and enter
   enough of the page URL to identify exactly one tab.
3. Add labels such as `adversarial` and choose **Start browser coverage**.
4. Exercise the app's UI yourself or have an AI automate that same tab.
5. Choose **Capture browser checkpoint** to inspect reached and missed paths.
   Checkpoints are cumulative since the session started, after subtracting its
   initial counters. Start separate sessions to compare separate workflows.
6. Choose **Stop collection**, then record a workflow verdict and notes.

A page reload invalidates the baseline; start a new session. Code already reached
before the session does not count as a new hit. A missed branch is only a measured
branch in instrumented code; uninstrumented files remain unknown. Coverage does
not establish that a displayed error or workflow outcome was correct.

The built-in agent and external agents connected through `hick mcp` use the same
`code_coverage` tool. For example, ask the AI:

> Start manual coverage for my instrumented app at localhost:3000, using Chrome's
> CDP endpoint http://127.0.0.1:9222. Tag it adversarial. Automate the invalid-login
> flow in that tab, capture coverage, and inspect the rejection paths with the
> locations action. Stop collection and separately record the observed outcome.

The tool supplies collection evidence; the agent needs its own browser-automation
capability to interact with the app.

## Instrument a native app or backend

A manual profile runs an already-instrumented application command, then exports
coverage when it stops. Optional checkpoint commands can flush a collector while
the application is still running. Add a profile to `.hick-tests.json`:

```json
{
  "manual": {
    "python-app": {
      "argv": [".venv/bin/python", "-m", "coverage", "run", "--branch", "--data-file", "{root}/.hick-cache/coverage/{run}/data", "app.py"],
      "collector": "coverage.py",
      "version": "7.17.0",
      "format": "coverage-py",
      "report": ".hick-cache/coverage/{run}/manual.json",
      "finish": [".venv/bin/python", "-m", "coverage", "json", "--data-file", "{root}/.hick-cache/coverage/{run}/data", "-o", "{report}"]
    }
  }
}
```

Use your installed collector version. Choose **Read project groups and profiles**,
enter `python-app`, and start the instrumented app. Exercise its UI or API. Stop
collection to interrupt the app and export coverage; record the manual outcome
separately. An export failure leaves the session unavailable with its output.

Profiles run argument arrays in the selected project directory. `{root}` is the
workspace, `{project}` the selected project, `{run}` the unique run ID, and
`{report}` the absolute report destination. Put runtime artifacts in
`.hick-cache`; use `{run}` to avoid mixing concurrent sessions. Shell syntax is
not interpreted. The application runs with your normal local permissions.

Rust applications can use `cargo llvm-cov run --lcov --output-path {report}`.
C# applications can use `dotnet-coverage` collection with Cobertura export.
Configure a collector's `checkpoint` command only when it can flush a running
session; otherwise collect on exit or import an externally generated report.
These profiles collect backend code exercised through a frontend as well.

## Import existing reports and read source

Choose **Import a standard report** for LCOV, coverage.py JSON, Cobertura, or an
Istanbul coverage map. An aggregate import has unavailable test attribution.
Supply the original `collector`, `version`, `configuration`, `scope` and
`source_hashes` as identity JSON if you retained them at collection time. Without
that identity, the report remains unknown and cannot be compared as current code.

Selecting evidence adds source-checked coverage markers to plain files, generated
files and literate documents. Click a source row to open its document when an
exact generated-file mapping exists. Changed buffers lose their coverage marks;
the original report remains in history. Unmapped source stays navigable as a file.

Cobertura branch fractions remain aggregates. Identified branch comparisons are
available for reports with actual branch identities. Coverage.py gives line
reachability, not line execution counts. Original reports retain provider details.
Generated-file mapping reuses the debugger/LSP coordinate mapping without running
a debugger or executing document cells; mismatched generated bytes are not mapped.

Records and reports persist under `.hick-cache/coverage`. Source hashes and
metadata are captured at admission; the app runs from the working tree, which is
not frozen. Changes during collection are flagged and prevent direct comparisons.

For agents, prefer `code_coverage` with `action: "locations"`, a run `id`, a source
`path` and a bounded `limit`. It returns code-path evidence and document coordinates.
`get` retrieves the complete report and output. `inspect`, `discover`, `run`,
`manual-start`, `browser-checkpoint`, `checkpoint`, `stop`, `compare`,
`rerun-failed`, and `annotate` share the UI service at `POST /api/tests/coverage`.
</book:entry>

</book:doc>
