//! Offline end-to-end test of the token-economics harness: a scripted
//! client drives `run_experiment`, JSONL rows land on disk, and
//! `generate_report` renders them — no network, no API key.

use std::sync::Arc;

use hickory_agent::harness::{ArmSpec, ExperimentSpec, RunRecord, generate_report, run_experiment};
use hickory_agent::{LlmClient, ScriptedLlmClient, Usage};
use hickory_executor::LocalExecutor;

#[tokio::test(flavor = "multi_thread")]
async fn harness_records_runs_and_reports_offline() {
    let spec: ExperimentSpec = serde_json::from_str(
        r#"{
            "experiment": "E0-offline-smoke",
            "description": "harness plumbing test",
            "arms": [
                {"name": "baseline", "baseline": true},
                {"name": "challenger", "effort": "low"}
            ],
            "tasks": [
                {"id": "t1", "prompt": "finish immediately", "check_cmd": "true"},
                {"id": "t2", "prompt": "finish immediately again", "check_cmd": "true"}
            ]
        }"#,
    )
    .unwrap();

    let dir = tempfile::tempdir().unwrap();
    let runs_dir = dir.path().join("runs");
    let out_path = runs_dir.join("e0.jsonl");
    let executor: Arc<dyn hickory_executor::Executor> = Arc::new(LocalExecutor::new().unwrap());

    let factory = |_arm: &ArmSpec| -> anyhow::Result<Arc<dyn LlmClient>> {
        Ok(Arc::new(
            ScriptedLlmClient::with_usages([
                (
                    "<hick:next>done</hick:next>\nDone one.".to_string(),
                    Usage {
                        input_tokens: 100,
                        cache_read_input_tokens: 50,
                        output_tokens: 10,
                        ..Default::default()
                    },
                ),
                (
                    "<hick:next>done</hick:next>\nDone two.".to_string(),
                    Usage {
                        input_tokens: 90,
                        output_tokens: 9,
                        ..Default::default()
                    },
                ),
            ])
            .with_model_name("claude-sonnet-5"),
        ))
    };

    run_experiment(&spec, dir.path(), dir.path(), &factory, executor, &out_path)
        .await
        .expect("offline experiment run failed");

    let raw = std::fs::read_to_string(&out_path).unwrap();
    let records: Vec<RunRecord> = raw
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    // 2 arms x 2 tasks x (1 turn row + 1 summary row).
    assert_eq!(records.len(), 8);
    let summaries: Vec<&RunRecord> = records.iter().filter(|r| r.turn.is_none()).collect();
    assert_eq!(summaries.len(), 4);
    assert!(summaries.iter().all(|r| r.completed == Some(true)));
    assert!(summaries.iter().all(|r| r.check_pass == Some(true)));
    assert!(summaries.iter().all(|r| r.cost_usd.unwrap() > 0.0));
    // The four-way split survives the JSONL round trip.
    let t1 = summaries
        .iter()
        .find(|r| r.arm == "baseline" && r.task_id == "t1")
        .unwrap();
    assert_eq!(t1.usage.cache_read_input_tokens, 50);

    let report = generate_report(&runs_dir).unwrap();
    assert!(report.contains("E0-offline-smoke"));
    assert!(report.contains("baseline (baseline)"));
    assert!(
        report.contains("100%"),
        "check-pass rate missing:\n{report}"
    );
}

/// The committed experiment specs must always parse, reference an existing
/// corpus, and keep exactly one baseline arm where arms are compared.
#[test]
fn committed_specs_parse_and_reference_real_files() {
    let tasks_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/token-economics/tasks");
    let mut seen = 0;
    for entry in std::fs::read_dir(&tasks_dir).expect("tasks dir missing") {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        seen += 1;
        let spec = ExperimentSpec::load(&path)
            .unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()));
        assert!(!spec.tasks.is_empty(), "{}: no tasks", spec.experiment);
        let baselines = spec.arms.iter().filter(|a| a.baseline).count();
        assert!(baselines <= 1, "{}: multiple baselines", spec.experiment);
        if let Some(corpus) = &spec.corpus {
            let corpus_dir = tasks_dir.join(corpus);
            assert!(corpus_dir.is_dir(), "{}: corpus missing", spec.experiment);
            for task in &spec.tasks {
                if let Some(doc) = &task.doc {
                    assert!(
                        corpus_dir.join(doc).is_file(),
                        "{}: task {} doc missing from corpus",
                        spec.experiment,
                        task.id
                    );
                }
                for out in &task.outputs {
                    assert!(
                        corpus_dir.join(out).is_file(),
                        "{}: task {} output {} missing from corpus",
                        spec.experiment,
                        task.id,
                        out.display()
                    );
                }
            }
        }
    }
    assert!(
        seen >= 3,
        "expected the committed E1/E2/E4 specs, found {seen}"
    );
}
