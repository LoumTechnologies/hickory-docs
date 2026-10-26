<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/serve_runtime_runs.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b659fc307bcd"></a>
## `crates/hickory-cli/tests/serve_runtime_runs.rs`

Related contracts: [Python observations stay confined and source bound](../../../../guarantees/execution/python-observations-stay-confined/contract.md), [Runtime history keeps runs and invocation occurrences separate](../../../../guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate/contract.md).

[Read the test `runtime_history_http_keeps_evidence_immutable_and_pins_explicit` beside its requirement](../../../../guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate/contract.md#f-b659fc307bcd-runtime_history_http_keeps_evidence_immutable_and_pins_explicit).
[Read the test `user_python_capture_is_retained_and_refuses_workspace_escape` beside its requirement](../../../../guarantees/execution/python-observations-stay-confined/contract.md#f-b659fc307bcd-user_python_capture_is_retained_and_refuses_workspace_escape).

### Assemble `crates/hickory-cli/tests/serve_runtime_runs.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_runtime_runs.rs"><storycode:paste select="#f-b659fc307bcd-runtime_history_http_keeps_evidence_immutable_and_pins_explicit"/><storycode:paste select="#f-b659fc307bcd-user_python_capture_is_retained_and_refuses_workspace_escape"/></storycode:file>

</storycode:doc>
