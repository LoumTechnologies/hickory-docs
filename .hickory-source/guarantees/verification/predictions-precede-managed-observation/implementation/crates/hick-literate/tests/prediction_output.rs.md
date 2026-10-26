<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/prediction_output.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1730eb6852af"></a>
## `crates/hick-literate/tests/prediction_output.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-1730eb6852af-privacy"></a>
### `privacy`

<storycode:copy id="f-1730eb6852af-privacy">
#![cfg(unix)]
//! Actual host-process witnesses for the bounded private output adapter.
//! LocalExecutor is not a sandbox/native/provider witness.
#[path = "support/prediction_privacy.rs"]
mod privacy;
</storycode:copy>
<a id="f-1730eb6852af-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-1730eb6852af-use_declaration">
use hick_literate::prediction::{OutputEncoding, OutputRequest, output_target, run_output};
</storycode:copy>
<a id="f-1730eb6852af-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-1730eb6852af-use_declaration-2">
use hick_observe::prediction::*;
</storycode:copy>
<a id="f-1730eb6852af-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-1730eb6852af-use_declaration-3">
use hickory_executor::{Executor, LocalExecutor};
</storycode:copy>
<a id="f-1730eb6852af-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-1730eb6852af-use_declaration-4">
use serde_json::json;
</storycode:copy>
<a id="f-1730eb6852af-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-1730eb6852af-use_declaration-5">
use sha2::{Digest, Sha256};
</storycode:copy>
<a id="f-1730eb6852af-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-1730eb6852af-use_declaration-6">
use std::{collections::BTreeMap, sync::Arc};

</storycode:copy>
<a id="f-1730eb6852af-actor"></a>
### `actor`

<storycode:copy id="f-1730eb6852af-actor">
fn actor() -> Actor {
    Actor {
        kind: "agent".into(),
        id: "predictor".into(),
    }
}
</storycode:copy>
<a id="f-1730eb6852af-source"></a>
### `source`

<storycode:copy id="f-1730eb6852af-source">
fn source(attempts: &[u64], corrected: bool, expected: u64) -> String {
    let allowance = if corrected { "100 - elapsed" } else { "100" };
    format!(
        "<hick:container name=\"c\" image=\"python\" />\n<hick:file path=\"answer.txt\"><hick:exec container=\"c\" show=\"output\">python3 -c 'attempts={attempts:?}; elapsed=0\nfor duration in attempts:\n allowance={allowance}\n elapsed += min(duration, allowance)\nprint(elapsed)'<hick:expect>{expected}\n</hick:expect></hick:exec></hick:file>"
    )
}
</storycode:copy>
<a id="f-1730eb6852af-seal_request"></a>
### `seal_request`

<storycode:copy id="f-1730eb6852af-seal_request">
fn seal_request(
    ledger: &Ledger,
    source: &str,
    inputs: &[u64],
    prediction: u64,
    key: &str,
) -> OutputRequest {
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&json!(inputs)).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([(
            "test-host-runtime".into(),
            digest(&"local-python3").unwrap(),
        )]),
    };
    let seal = Seal {
        scenario: Scenario {
            family: "retry-total-deadline".into(),
            inputs: json!(inputs),
            requirement_reference: "total allowance <= 100ms".into(),
            requirement: Predicate::NumberAtMost(100.0),
            identity_known: true,
        },
        identity: identity.clone(),
        target: output_target("c", 2, OutputEncoding::Json),
        predicted: Predicate::Equals(json!(prediction)),
        rationale: "The total budget bounds all attempts".into(),
    };
    let receipt = ledger
        .seal(ledger.history(None).unwrap().generation, seal, actor())
        .unwrap();
    OutputRequest {
        seal: receipt.id,
        request_id: key.into(),
        actor: actor(),
        identity,
        encoding: OutputEncoding::Json,
    }
}
</storycode:copy>
<a id="f-1730eb6852af-executor"></a>
### `executor`

<storycode:copy id="f-1730eb6852af-executor">
fn executor() -> Arc<LocalExecutor> {
    Arc::new(LocalExecutor::new().unwrap())
}

</storycode:copy>
[Read the test `real_fault_correction_and_novel_inputs_retain_private_evidence` beside its requirement](../../../../contract.md#f-1730eb6852af-real_fault_correction_and_novel_inputs_retain_private_evidence).
[Read the test `missing_stale_wrong_target_and_unsupported_paths_do_not_execute` beside its requirement](../../../../contract.md#f-1730eb6852af-missing_stale_wrong_target_and_unsupported_paths_do_not_execute).
[Read the test `runner_expectation_failure_cannot_be_overridden_by_a_matching_prediction` beside its requirement](../../../../contract.md#f-1730eb6852af-runner_expectation_failure_cannot_be_overridden_by_a_matching_prediction).
[Read the test `execution_errors_and_unparseable_answers_are_nonpassing_private_states` beside its requirement](../../../../contract.md#f-1730eb6852af-execution_errors_and_unparseable_answers_are_nonpassing_private_states).

### Assemble `crates/hick-literate/tests/prediction_output.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/prediction_output.rs"><storycode:paste select="#f-1730eb6852af-privacy"/><storycode:paste select="#f-1730eb6852af-use_declaration"/><storycode:paste select="#f-1730eb6852af-use_declaration-2"/><storycode:paste select="#f-1730eb6852af-use_declaration-3"/><storycode:paste select="#f-1730eb6852af-use_declaration-4"/><storycode:paste select="#f-1730eb6852af-use_declaration-5"/><storycode:paste select="#f-1730eb6852af-use_declaration-6"/><storycode:paste select="#f-1730eb6852af-actor"/><storycode:paste select="#f-1730eb6852af-source"/><storycode:paste select="#f-1730eb6852af-seal_request"/><storycode:paste select="#f-1730eb6852af-executor"/><storycode:paste select="#f-1730eb6852af-real_fault_correction_and_novel_inputs_retain_private_evidence"/><storycode:paste select="#f-1730eb6852af-missing_stale_wrong_target_and_unsupported_paths_do_not_execute"/><storycode:paste select="#f-1730eb6852af-runner_expectation_failure_cannot_be_overridden_by_a_matching_prediction"/><storycode:paste select="#f-1730eb6852af-execution_errors_and_unparseable_answers_are_nonpassing_private_states"/></storycode:file>

</storycode:doc>
