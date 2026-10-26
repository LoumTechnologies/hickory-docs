<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/prediction_combined.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-736f0187c52d"></a>
## `crates/hick-literate/tests/prediction_combined.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-736f0187c52d-privacy"></a>
### `privacy`

<storycode:copy id="f-736f0187c52d-privacy">
#![cfg(unix)]
//! Same-revision ordinary exec and private DAP capture under one ledger gate.
#[path = "support/prediction_privacy.rs"]
mod privacy;
</storycode:copy>
<a id="f-736f0187c52d-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-736f0187c52d-use_declaration">
use hick_literate::prediction::{
    OutputEncoding,
    combined::{self, Request},
    output_target,
};
</storycode:copy>
<a id="f-736f0187c52d-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-736f0187c52d-use_declaration-2">
use hick_observe::prediction::*;
</storycode:copy>
<a id="f-736f0187c52d-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-736f0187c52d-use_declaration-3">
use hickory_executor::{Executor, LocalExecutor};
</storycode:copy>
<a id="f-736f0187c52d-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-736f0187c52d-use_declaration-4">
use serde_json::json;
</storycode:copy>
<a id="f-736f0187c52d-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-736f0187c52d-use_declaration-5">
use sha2::{Digest, Sha256};
</storycode:copy>
<a id="f-736f0187c52d-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-736f0187c52d-use_declaration-6">
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

</storycode:copy>
<a id="f-736f0187c52d-actor"></a>
### `actor`

<storycode:copy id="f-736f0187c52d-actor">
fn actor() -> Actor {
    Actor {
        kind: "agent".into(),
        id: "predictor".into(),
    }
}
</storycode:copy>
<a id="f-736f0187c52d-source"></a>
### `source`

<storycode:copy id="f-736f0187c52d-source">
fn source(corrected: bool, cap_output: bool) -> String {
    let allowance = if corrected {
        "budget - elapsed"
    } else {
        "budget"
    };
    let final_value = if cap_output {
        "min(100, retry([80, 80], 100))"
    } else {
        "retry([80, 80], 100)"
    };
    format!(
        r#"<hick:container name="c" image="python" />
<hick:volume name="code" input="." />
<hick:file path="deadline.py">
def retry(attempts, budget):
    elapsed = 0
    for duration in attempts:
        allowance = {allowance}
        elapsed += min(duration, allowance)
    return elapsed
print({final_value})
</hick:file>
<hick:exec container="c" mount="code:/work">
python3 work/deadline.py
<hick:capture at="deadline.py:5" of="allowance" max="2" />
<hick:expect>100
</hick:expect>
</hick:exec>"#
    )
}
</storycode:copy>
<a id="f-736f0187c52d-request"></a>
### `request`

<storycode:copy id="f-736f0187c52d-request">
fn request(ledger: &Ledger, source: &str, key: &str) -> Request {
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&json!([80, 80])).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([(
            "fixture-runtime".into(),
            digest(&"local-python-and-configured-debugpy").unwrap(),
        )]),
    };
    let document = hick_lang::parse(source).unwrap();
    let line = document
        .all_tags()
        .into_iter()
        .find(|tag| tag.name == "exec")
        .unwrap()
        .source_line;
    let start = source.find("        elapsed +=").unwrap();
    let end = start + source[start..].find('\n').unwrap() + 1;
    let target = Target::Combined {
        output: Box::new(output_target("c", line, OutputEncoding::Json)),
        debugger: Box::new(Target::Debugger {
            source: "root".into(),
            source_revision: identity.source.clone(),
            start_byte: start as u64,
            end_byte: end as u64,
            expression: "allowance".into(),
            hit: 2,
            frame: Some("retry".into()),
        }),
    };
    let seal = Seal {
        identity: identity.clone(),
        target: target.clone(),
        rationale: "One total budget leaves twenty before the second attempt".into(),
        predicted: Predicate::Fields(BTreeMap::from([
            ("output".into(), Predicate::Equals(json!(100))),
            ("debugger".into(), Predicate::Equals(json!(20))),
        ])),
        scenario: Scenario {
            family: "retry-budget/combined".into(),
            inputs: json!([80, 80]),
            requirement_reference: "one total 100ms allowance".into(),
            identity_known: true,
            requirement: Predicate::Fields(BTreeMap::from([
                ("output".into(), Predicate::NumberAtMost(100.0)),
                ("debugger".into(), Predicate::Equals(json!(20))),
            ])),
        },
    };
    let receipt = ledger
        .seal(ledger.history(None).unwrap().generation, seal, actor())
        .unwrap();
    Request {
        seal: receipt.id,
        request_id: key.into(),
        actor: actor(),
        identity,
        target,
    }
}

</storycode:copy>
[Read the test `same_revision_output_and_debugger_must_both_pass_before_promotion` beside its requirement](../../../../contract.md#f-736f0187c52d-same_revision_output_and_debugger_must_both_pass_before_promotion).
[Read the test `supported_combination_reaches_the_shared_seal_gate_without_an_adapter` beside its requirement](../../../../contract.md#f-736f0187c52d-supported_combination_reaches_the_shared_seal_gate_without_an_adapter).
[Read the test `unrelated_stdout_program_and_external_input_volume_refuse_before_execution` beside its requirement](../../../../contract.md#f-736f0187c52d-unrelated_stdout_program_and_external_input_volume_refuse_before_execution).

### Assemble `crates/hick-literate/tests/prediction_combined.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/prediction_combined.rs"><storycode:paste select="#f-736f0187c52d-privacy"/><storycode:paste select="#f-736f0187c52d-use_declaration"/><storycode:paste select="#f-736f0187c52d-use_declaration-2"/><storycode:paste select="#f-736f0187c52d-use_declaration-3"/><storycode:paste select="#f-736f0187c52d-use_declaration-4"/><storycode:paste select="#f-736f0187c52d-use_declaration-5"/><storycode:paste select="#f-736f0187c52d-use_declaration-6"/><storycode:paste select="#f-736f0187c52d-actor"/><storycode:paste select="#f-736f0187c52d-source"/><storycode:paste select="#f-736f0187c52d-request"/><storycode:paste select="#f-736f0187c52d-same_revision_output_and_debugger_must_both_pass_before_promotion"/><storycode:paste select="#f-736f0187c52d-supported_combination_reaches_the_shared_seal_gate_without_an_adapter"/><storycode:paste select="#f-736f0187c52d-unrelated_stdout_program_and_external_input_volume_refuse_before_execution"/></storycode:file>

</storycode:doc>
