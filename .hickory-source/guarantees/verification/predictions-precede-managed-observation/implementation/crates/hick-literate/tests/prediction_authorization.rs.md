<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/prediction_authorization.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-91952cf1f2e5"></a>
## `crates/hick-literate/tests/prediction_authorization.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-91952cf1f2e5-fixture"></a>
### `fixture`

<storycode:copy id="f-91952cf1f2e5-fixture">
#![cfg(unix)]
//! Synthetic authorization uses the same private output/debugger gate as retry.
use hick_literate::prediction::combined;
use hick_observe::prediction::*;
use hickory_executor::LocalExecutor;
use std::{path::PathBuf, sync::Arc};

#[path = "support/authorization.rs"]
mod fixture;
</storycode:copy>
<a id="f-91952cf1f2e5-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-91952cf1f2e5-use_declaration">
use fixture::*;

</storycode:copy>
[Read the test `vulnerable_and_patched_authorization_keep_one_requirement_and_prediction` beside its requirement](../../../../contract.md#f-91952cf1f2e5-vulnerable_and_patched_authorization_keep_one_requirement_and_prediction).
[Read the test `correctly_predicting_the_vulnerability_does_not_accept_the_security_requirement` beside its requirement](../../../../contract.md#f-91952cf1f2e5-correctly_predicting_the_vulnerability_does_not_accept_the_security_requirement).
[Read the test `source_question_rebinds_a_repair_but_refuses_retargeted_capture_metadata_or_statement` beside its requirement](../../../../contract.md#f-91952cf1f2e5-source_question_rebinds_a_repair_but_refuses_retargeted_capture_metadata_or_statement).

### Assemble `crates/hick-literate/tests/prediction_authorization.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/prediction_authorization.rs"><storycode:paste select="#f-91952cf1f2e5-fixture"/><storycode:paste select="#f-91952cf1f2e5-use_declaration"/><storycode:paste select="#f-91952cf1f2e5-vulnerable_and_patched_authorization_keep_one_requirement_and_prediction"/><storycode:paste select="#f-91952cf1f2e5-correctly_predicting_the_vulnerability_does_not_accept_the_security_requirement"/><storycode:paste select="#f-91952cf1f2e5-source_question_rebinds_a_repair_but_refuses_retargeted_capture_metadata_or_statement"/></storycode:file>

</storycode:doc>
