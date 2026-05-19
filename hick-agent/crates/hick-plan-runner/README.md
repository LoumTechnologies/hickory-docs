# hick-plan-runner

Sidecar binary (`hick-plan-runner` executable) for executing hick XML plans in
batch mode.

Parses a plan XML (from a file or stdin), sets up a `TagRegistry` with
`hick-live` handlers, builds a reactive Node DAG via `hick-flow`, and executes
it while collecting results and API call records.

## Features

- Mock and live API backends
- Optional Python policy script for filtering/gating API calls
- Outputs results to CSV/JSON

Intended as a non-interactive execution harness rather than an interactive agent
— suitable for CI pipelines and scheduled batch runs.
