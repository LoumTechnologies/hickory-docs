//! Background LLM-as-judge task: scores agent responses after each `UiEvent::Done` (TASK-48).

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use hick_agent_sdk::{AnthropicClient, IdGen, LlmClient, Message, OpenAiCompatClient, Role, UiEvent};
use hick_eval_store::{BacktrackingEvent, EvalRecord, GitBackend, JudgeScoreRecord, MachineId, StorageBackend};
use tokio::sync::broadcast;

use crate::config::{JudgeDimension, JudgeSection, Provider};

/// Configuration for a judge task bound to one agent tab.
pub struct JudgeTaskConfig {
    pub judge_section: JudgeSection,
    /// Display label of the config being judged (e.g. `"claude-opus / hick-edit [config-b]"`).
    pub config_name: String,
}

/// Run the judge as a long-lived background task.
///
/// Subscribes to `sub` (a per-tab broadcast receiver), accumulates query/response
/// across each turn, and fires a judge call after every `UiEvent::Done`.
/// Sends `JudgeScoreReady` or `JudgeError` back to `ui_tx` (non-fatal).
pub async fn run_judge_task(
    mut sub: broadcast::Receiver<UiEvent>,
    task_cfg: JudgeTaskConfig,
    ui_tx: broadcast::Sender<UiEvent>,
) {
    let ids = Arc::new(IdGen::new());
    let mut current_query: Option<String> = None;
    let mut current_response: Option<String> = None;
    let mut turn_start: Option<std::time::Instant> = None;
    let mut rework_count: u32 = 0;

    loop {
        let ev = match sub.recv().await {
            Ok(ev) => ev,
            Err(broadcast::error::RecvError::Closed) => break,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
        };

        match ev {
            UiEvent::UserTurn { text, .. } => {
                current_query = Some(text);
                turn_start = Some(std::time::Instant::now());
                rework_count = 0;
                current_response = None;
            }
            UiEvent::AssistantDone { text, .. } => {
                current_response = Some(text);
            }
            UiEvent::ActionEnd { exit_code, .. } => {
                if exit_code.map_or(false, |c| c != 0) {
                    rework_count += 1;
                }
            }
            UiEvent::Done { .. } => {
                let (query, response) = match (current_query.take(), current_response.take()) {
                    (Some(q), Some(r)) => (q, r),
                    _ => continue,
                };
                let wall_secs = turn_start.take().map(|t| t.elapsed().as_secs_f64());
                let rework = rework_count;
                rework_count = 0;

                let judge_section = task_cfg.judge_section.clone();
                let config_name = task_cfg.config_name.clone();
                let ids_clone = Arc::clone(&ids);
                let ui_tx_clone = ui_tx.clone();

                tokio::spawn(async move {
                    call_judge_and_emit(
                        judge_section,
                        config_name,
                        query,
                        response,
                        wall_secs,
                        rework,
                        ids_clone,
                        ui_tx_clone,
                    )
                    .await;
                });
            }
            _ => {}
        }
    }
}

async fn call_judge_and_emit(
    judge_section: JudgeSection,
    config_name: String,
    query: String,
    response: String,
    wall_secs: Option<f64>,
    rework_count: u32,
    ids: Arc<IdGen>,
    ui_tx: broadcast::Sender<UiEvent>,
) {
    let llm = match build_judge_llm(&judge_section) {
        Ok(l) => l,
        Err(e) => {
            emit_judge_error(&ui_tx, &ids, &config_name, &format!("failed to build judge LLM: {e}"));
            return;
        }
    };

    let dimensions = judge_section.effective_dimensions();
    let dim_text = dimensions
        .iter()
        .map(|d| format!("- {} (weight {:.2})", d.name, d.weight))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = judge_section
        .effective_prompt_template()
        .replace("{query}", &query)
        .replace("{response}", &response)
        .replace("{dimensions}", &dim_text);

    let prompt_hash = compute_prompt_hash(&prompt);
    let judge_model = judge_section.model.clone();
    let dim_names: Vec<String> = dimensions.iter().map(|d| d.name.clone()).collect();

    let messages = vec![Message { role: Role::User, content: prompt }];

    let judge_response_text = match llm.complete(messages).await {
        Ok(r) => r,
        Err(e) => {
            emit_judge_error(&ui_tx, &ids, &config_name, &format!("judge LLM call failed: {e}"));
            return;
        }
    };

    match parse_judge_response(&judge_response_text, &dimensions) {
        Err(e) => {
            emit_judge_error(
                &ui_tx,
                &ids,
                &config_name,
                &format!("judge response parse failed: {e}"),
            );
        }
        Ok((scores, backtracking_events)) => {
            let timestamp = Utc::now();

            let machine_id_str = MachineId::load()
                .map(|m| m.uuid().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let record = JudgeScoreRecord {
                machine_id: machine_id_str,
                timestamp,
                config_name: config_name.clone(),
                judge_model: judge_model.clone(),
                prompt_hash,
                query: query.clone(),
                scores: scores.clone(),
                backtracking_events: backtracking_events.clone(),
                token_cost: None,
                wall_secs,
                rework_count: Some(rework_count),
            };

            if let Ok(backend) = GitBackend::new() {
                if let Err(e) = backend.append(&EvalRecord::JudgeScore(record)) {
                    log::warn!("eval store: failed to append judge score for {config_name}: {e}");
                } else {
                    backend.sync_session();
                }
            }

            let backtracking_rationale: Vec<String> = backtracking_events
                .iter()
                .map(|e| {
                    if e.avoidable {
                        format!("(avoidable) {}", e.description)
                    } else {
                        format!("(unavoidable) {}", e.description)
                    }
                })
                .collect();

            let node_id = ids.next();
            let _ = ui_tx.send(UiEvent::JudgeScoreReady {
                node_id,
                config_name,
                query,
                judge_model,
                scores,
                dimensions: dim_names,
                backtracking_rationale,
                wall_secs,
                rework_count: Some(rework_count),
                judge_timestamp: timestamp.to_rfc3339(),
            });
        }
    }
}

fn emit_judge_error(
    ui_tx: &broadcast::Sender<UiEvent>,
    ids: &IdGen,
    config_name: &str,
    error: &str,
) {
    let node_id = ids.next();
    let _ = ui_tx.send(UiEvent::JudgeError {
        node_id,
        config_name: config_name.to_string(),
        error: error.to_string(),
    });
}

fn build_judge_llm(section: &JudgeSection) -> anyhow::Result<Box<dyn LlmClient>> {
    let api_key = section
        .api_key_env
        .as_deref()
        .and_then(|env| std::env::var(env).ok())
        .or_else(|| match section.provider {
            Provider::Anthropic  => std::env::var("ANTHROPIC_API_KEY").ok(),
            Provider::OpenAI     => std::env::var("OPENAI_API_KEY").ok(),
            Provider::XAI        => std::env::var("XAI_API_KEY").ok(),
            Provider::OpenRouter => std::env::var("OPENROUTER_API_KEY").ok(),
        })
        .unwrap_or_default();

    match section.provider {
        Provider::Anthropic => {
            Ok(Box::new(AnthropicClient::with_config(section.model.clone(), api_key)))
        }
        Provider::OpenAI => Ok(Box::new(OpenAiCompatClient::custom(
            "https://api.openai.com/v1",
            api_key,
            section.model.clone(),
            "openai",
        ))),
        Provider::XAI => Ok(Box::new(OpenAiCompatClient::custom(
            "https://api.x.ai/v1",
            api_key,
            section.model.clone(),
            "xai",
        ))),
        Provider::OpenRouter => Ok(Box::new(OpenAiCompatClient::custom(
            "https://openrouter.ai/api/v1",
            api_key,
            section.model.clone(),
            "openrouter",
        ))),
    }
}

fn parse_judge_response(
    content: &str,
    dimensions: &[JudgeDimension],
) -> anyhow::Result<(HashMap<String, f64>, Vec<BacktrackingEvent>)> {
    let json_start = content
        .find('{')
        .ok_or_else(|| anyhow::anyhow!("no JSON object in response"))?;
    let json_end = content
        .rfind('}')
        .ok_or_else(|| anyhow::anyhow!("no closing brace in response"))?;
    let json_str = &content[json_start..=json_end];

    let val: serde_json::Value = serde_json::from_str(json_str)?;
    let obj = val
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("response is not a JSON object"))?;

    let mut scores = HashMap::new();
    for dim in dimensions {
        if let Some(score) = obj.get(&dim.name).and_then(|v| v.as_f64()) {
            scores.insert(dim.name.clone(), score.clamp(1.0, 10.0));
        }
    }

    let backtracking_events = obj
        .get("backtracking_events")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let description = item.get("description")?.as_str()?.to_string();
                    let avoidable = item.get("avoidable")?.as_bool()?;
                    Some(BacktrackingEvent { description, avoidable })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok((scores, backtracking_events))
}

/// FNV-1a 64-bit hash — deterministic across Rust versions (no per-process randomness).
fn compute_prompt_hash(prompt: &str) -> String {
    let mut hash: u64 = 14695981039346656037;
    for byte in prompt.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_hash_is_deterministic() {
        let a = compute_prompt_hash("hello world");
        let b = compute_prompt_hash("hello world");
        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn prompt_hash_differs_for_different_inputs() {
        assert_ne!(compute_prompt_hash("a"), compute_prompt_hash("b"));
    }

    #[test]
    fn parse_judge_response_extracts_scores_and_backtracking() {
        let dims = vec![
            JudgeDimension { name: "task_completion".into(), weight: 0.3 },
            JudgeDimension { name: "backtracking".into(), weight: 0.25 },
        ];
        let content = r#"
Some preamble...
{
  "task_completion": 8.5,
  "backtracking": 9.0,
  "backtracking_events": [
    {"description": "rewrote main.rs after test failure", "avoidable": true}
  ]
}
"#;
        let (scores, events) = parse_judge_response(content, &dims).unwrap();
        assert_eq!(scores["task_completion"], 8.5);
        assert_eq!(scores["backtracking"], 9.0);
        assert_eq!(events.len(), 1);
        assert!(events[0].avoidable);
    }

    #[test]
    fn parse_judge_response_clamps_out_of_range_scores() {
        let dims = vec![JudgeDimension { name: "task_completion".into(), weight: 1.0 }];
        let content = r#"{"task_completion": 15.0}"#;
        let (scores, _) = parse_judge_response(content, &dims).unwrap();
        assert_eq!(scores["task_completion"], 10.0);
    }

    #[test]
    fn parse_judge_response_empty_backtracking_events() {
        let dims = vec![JudgeDimension { name: "task_completion".into(), weight: 1.0 }];
        let content = r#"{"task_completion": 7.0, "backtracking_events": []}"#;
        let (scores, events) = parse_judge_response(content, &dims).unwrap();
        assert_eq!(scores["task_completion"], 7.0);
        assert!(events.is_empty());
    }
}
