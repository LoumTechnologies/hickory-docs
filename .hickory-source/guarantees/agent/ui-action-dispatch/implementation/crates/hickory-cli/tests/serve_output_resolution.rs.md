<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/serve_output_resolution.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-074b9e71856c"></a>
## `crates/hickory-cli/tests/serve_output_resolution.rs`

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../contract.md), [Keep divergent output until an explicit decision names the write](../../../../../../authoring/an-output-that-cannot-be-carried-back-is-held/contract.md).

[Read the test `output_resolution_http_waits_for_the_applied_or_refused_loop_reply` beside its requirement](../../../../contract.md#f-074b9e71856c-output_resolution_http_waits_for_the_applied_or_refused_loop_reply).

### Assemble `crates/hickory-cli/tests/serve_output_resolution.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_output_resolution.rs"><storycode:paste select="#f-074b9e71856c-output_resolution_http_waits_for_the_applied_or_refused_loop_reply"/></storycode:file>

</storycode:doc>
