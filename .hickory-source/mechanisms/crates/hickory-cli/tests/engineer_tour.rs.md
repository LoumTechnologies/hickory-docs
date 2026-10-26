<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/engineer_tour.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ae34e687590d"></a>
## `crates/hickory-cli/tests/engineer_tour.rs`

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../guarantees/agent/ui-action-dispatch/contract.md), [The engineer tour distinguishes replay from execution](../../../../guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md).

[Read the test `expected_root_source_rejects_changed_bytes_before_parsing_or_execution` beside its requirement](../../../../guarantees/agent/ui-action-dispatch/contract.md#f-ae34e687590d-expected_root_source_rejects_changed_bytes_before_parsing_or_execution).
[Read the test `deadline_witness_passes_and_exposes_the_original_bug` beside its requirement](../../../../guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md#f-ae34e687590d-deadline_witness_passes_and_exposes_the_original_bug).
[Read the test `native_verification_receipt_names_the_exact_source_that_ran` beside its requirement](../../../../guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md#f-ae34e687590d-native_verification_receipt_names_the_exact_source_that_ran).

### Assemble `crates/hickory-cli/tests/engineer_tour.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/engineer_tour.rs"><storycode:paste select="#f-ae34e687590d-expected_root_source_rejects_changed_bytes_before_parsing_or_execution"/><storycode:paste select="#f-ae34e687590d-deadline_witness_passes_and_exposes_the_original_bug"/><storycode:paste select="#f-ae34e687590d-native_verification_receipt_names_the_exact_source_that_ran"/></storycode:file>

</storycode:doc>
