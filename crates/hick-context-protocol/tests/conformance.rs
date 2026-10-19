use hick_context_protocol::*;
use serde_json::{Value, json};

fn run(case: &Value) -> Result<Value> {
    let p = &case["payload"];
    Ok(match case["operation"].as_str().unwrap() {
        "bundle" => {
            let bundle = validate_bundle(p)?;
            let serialized = serde_json::to_value(bundle).unwrap();
            assert_eq!(serialized, *p, "generated wire types must round-trip");
            serialized
        }
        "negotiate" => serde_json::to_value(negotiate(&p[0], &p[1])?).unwrap(),
        "offset" => json!(utf16_to_utf8(
            p["text"].as_str().unwrap(),
            p["offset"].as_u64().unwrap()
        )?),
        "page" => serde_json::to_value(validate_page(
            &p["page"],
            p["epoch"].as_str().unwrap(),
            p["branch"].as_str().unwrap(),
        )?)
        .unwrap(),
        "frame" => validate_frame(
            p["raw"].as_str().unwrap().as_bytes(),
            p["limit"].as_u64().unwrap() as usize,
            if p["expected_id"].is_null() {
                None
            } else {
                Some(&p["expected_id"])
            },
        )?,
        "resolve" => serde_json::to_value(resolve_handle(&p["handle"], &p["request"])?).unwrap(),
        "replay" => {
            let mut keys = Idempotency::default();
            let mut results = Vec::new();
            for pair in p.as_array().unwrap() {
                results.push(keys.register(pair[0].as_str().unwrap(), pair[1].as_str().unwrap())?);
            }
            json!(results)
        }
        op => panic!("Unknown fixture operation {op}"),
    })
}
// Guarantee: docs/guarantees/agent/context-protocol-validates-lineage.md
#[test]
fn shared_conformance() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../../../protocols/context-v1/fixtures.json")).unwrap();
    for case in cases {
        let result = run(&case);
        if case["expected"] == "ok" {
            let result = result.unwrap_or_else(|e| panic!("{}: {e}", case["name"]));
            if let Some(expected) = case.get("result") {
                assert_eq!(&result, expected, "{}", case["name"]);
            }
        } else {
            let error = result.expect_err(&format!("{} should fail", case["name"]));
            assert_eq!(
                serde_json::to_value(error.code).unwrap(),
                case["expected"],
                "{}: {error}",
                case["name"]
            );
        }
    }
}
