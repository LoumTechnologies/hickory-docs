<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `6d120ac10af40e241f28b9fd16b4717bd5da784ba5952850008528090218730d`.

<book:entry path="crates/hick-literate/README.md">
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
</book:entry>

</book:doc>
