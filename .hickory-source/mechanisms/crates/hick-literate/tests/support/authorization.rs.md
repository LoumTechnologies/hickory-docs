<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/support/authorization.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1b65404a58a9"></a>
## `crates/hick-literate/tests/support/authorization.rs`

<a id="f-1b65404a58a9-VULNERABLE"></a>
### `VULNERABLE`

Trusted host fixture shared by the focused witness and explicit safe export.

<storycode:copy id="f-1b65404a58a9-VULNERABLE">
//! Trusted host fixture shared by the focused witness and explicit safe export.
use hick_literate::prediction::{OutputEncoding, combined, output_target};
use hick_observe::prediction::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const VULNERABLE: &str =
    include_str!("../../../../examples/security-authorization/vulnerable.md");
</storycode:copy>
<a id="f-1b65404a58a9-PATCHED"></a>
### `PATCHED`

<storycode:copy id="f-1b65404a58a9-PATCHED">
pub const PATCHED: &str = include_str!("../../../../examples/security-authorization/patched.md");

</storycode:copy>
<a id="f-1b65404a58a9-actor"></a>
### `actor`

<storycode:copy id="f-1b65404a58a9-actor">
pub fn actor() -> Actor {
    Actor {
        kind: "agent".into(),
        id: "synthetic-authorization-predictor".into(),
    }
}

</storycode:copy>
<a id="f-1b65404a58a9-request"></a>
### `request`

<storycode:copy id="f-1b65404a58a9-request">
pub fn request(ledger: &Ledger, source: &str, key: &str, prediction: i64) -> combined::Request {
    request_with_environment(
        ledger,
        source,
        key,
        prediction,
        BTreeMap::from([(
            "fixture-runtime".into(),
            digest(&"local-python-and-configured-debugpy").unwrap(),
        )]),
    )
}

</storycode:copy>
<a id="f-1b65404a58a9-request_with_environment"></a>
### `request_with_environment`

<storycode:copy id="f-1b65404a58a9-request_with_environment">
pub fn request_with_environment(
    ledger: &Ledger,
    source: &str,
    key: &str,
    prediction: i64,
    environment: BTreeMap<String, String>,
) -> combined::Request {
    let inputs = json!({"tenant":"tenant-a", "record":"record-b",
        "records":{"record-a":"tenant-a", "record-b":"tenant-b"}});
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&inputs).unwrap(),
        dependencies: BTreeMap::new(),
        environment,
    };
    let document = hick_lang::parse(source).unwrap();
    let line = document
        .all_tags()
        .into_iter()
        .find(|tag| tag.name == "exec")
        .unwrap()
        .source_line;
    let start = source.find("    return allowed\n").unwrap();
    let target = Target::Combined {
        output: Box::new(output_target("c", line, OutputEncoding::Json)),
        debugger: Box::new(Target::Debugger {
            source: "root".into(),
            source_revision: identity.source.clone(),
            start_byte: start as u64,
            end_byte: (start + "    return allowed\n".len()) as u64,
            expression: "allowed".into(),
            hit: 1,
            frame: Some("authorize".into()),
        }),
    };
    let fields = |value| {
        Predicate::Fields(BTreeMap::from([
            ("output".into(), Predicate::Equals(json!(value))),
            ("debugger".into(), Predicate::Equals(json!(value))),
        ]))
    };
    let receipt = ledger
        .seal(
            ledger.history(None).unwrap().generation,
            Seal {
                identity: identity.clone(),
                target: target.clone(),
                predicted: fields(prediction),
                rationale:
                    "Cross-tenant access must be denied while legitimate access stays available"
                        .into(),
                scenario: Scenario {
                    family: "synthetic-cross-tenant-authorization".into(),
                    inputs,
                    requirement_reference: "R-AUTH-1; R-LEGIT-1".into(),
                    requirement: fields(0),
                    identity_known: true,
                },
            },
            actor(),
        )
        .unwrap();
    combined::Request {
        seal: receipt.id,
        request_id: key.into(),
        actor: actor(),
        identity,
        target,
    }
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/support/authorization.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/support/authorization.rs"><storycode:paste select="#f-1b65404a58a9-VULNERABLE"/><storycode:paste select="#f-1b65404a58a9-PATCHED"/><storycode:paste select="#f-1b65404a58a9-actor"/><storycode:paste select="#f-1b65404a58a9-request"/><storycode:paste select="#f-1b65404a58a9-request_with_environment"/></storycode:file>

</storycode:doc>
