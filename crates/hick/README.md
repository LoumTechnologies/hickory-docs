# hick

Top-level pipeline orchestration crate and main `hick` CLI binary.

Wires together every subsystem — `hick-lang` for parsing, `hick-exec` for the
reactive node graph and DAG, `hick-handlers` for tag dispatch, `hick-live` for
streaming operators, `hick-feature` for feature flags, and `hick-condition` for
`when` evaluation — into cohesive pipeline execution.

## Entry points

- `run_pipeline` — dry-run (no containers spawned)
- `run_pipeline_live` — real execution with WASM containers
- `run_pipeline_multi_stage` — multi-stage pipeline execution
- `pipeline_session_replay` — replay a recorded AI-agent session

The `run_pipeline_cmd` helper is shared between the `hick` and `hick-agent`
binaries.

## Additional binaries

- `hick-equiv` — equivalence checking between two pipeline outputs
- `hick-compact` — compact a pipeline definition
- `hick-promote` — promote a feature branch
