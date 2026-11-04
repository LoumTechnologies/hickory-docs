//! Experiment host adapter: use the production private-output runner and ledger.
use hick_literate::prediction::{OutputEncoding, OutputRequest, output_target, run_output};
use hick_observe::prediction::*;
use hickory_executor::LocalExecutor;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{io::{self, Read}, sync::Arc};

fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let v: Value = serde_json::from_str(&input)?;
    let ledger = Ledger::open(v["ledger"].as_str().unwrap())?;
    let source = v["source"].as_str().unwrap();
    let identity = Identity {
        source: hash(source.as_bytes()), inputs: digest(&v["inputs"] )?,
        dependencies: serde_json::from_value(v["dependencies"].clone())?,
        environment: serde_json::from_value(v["environment"].clone())?,
    };
    let actor = Actor { kind: "agent".into(), id: "benchmark-predictor".into() };
    if v["phase"] == "seal" {
        let receipt = ledger.seal(ledger.history(None)?.generation, Seal {
            scenario: Scenario {
                family: v["family"].as_str().unwrap().into(), inputs: v["inputs"].clone(),
                requirement_reference: "Frozen benchmark public cases; exact JSON results".into(),
                requirement: Predicate::Equals(v["expected"].clone()), identity_known: true,
            }, identity: identity.clone(), target: output_target("c", 2, OutputEncoding::Json),
            predicted: Predicate::Equals(v["prediction"].clone()),
            rationale: v["rationale"].as_str().unwrap_or("").into(),
        }, actor)?;
        println!("{}", json!({"seal":receipt.id,"identity":identity}));
    } else {
        let result = run_output(&ledger, &OutputRequest {
            seal: v["seal"].as_str().unwrap().into(),
            request_id: v["request_id"].as_str().unwrap().into(), actor,
            identity: identity.clone(), encoding: OutputEncoding::Json,
        }, source, Arc::new(LocalExecutor::new()?)).await?;
        let promotion = ledger.promote(&result.summary.attempt, &identity,
            Actor {kind:"host".into(),id:"benchmark-host".into()}).ok();
        println!("{}", json!({"result":result,"promotion":promotion}));
    }
    Ok(())
}
