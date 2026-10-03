//! Deterministic ACP conformance peer for integration tests, never a product backend.
use serde_json::{Value, json};
use std::io::{BufRead, Write};

fn send(value: Value) {
    println!("{value}");
    std::io::stdout().flush().unwrap();
}
fn result(id: Value, value: Value) {
    send(json!({"jsonrpc":"2.0","id":id,"result":value}));
}
fn update(value: Value) {
    send(
        json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":value}}),
    );
}

#[tokio::main]
async fn main() {
    let mut bridge = String::new();
    let mut cwd = String::new();
    let mut prompt_waiting: Option<Value> = None;
    let mut auth = std::env::args().any(|a| a == "--auth");
    let mut inactive = false;
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let message: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let id = message["id"].clone();
        let params = &message["params"];
        match message["method"].as_str() {
            Some("initialize") => result(
                id,
                json!({"protocolVersion":1,"agentInfo":{"name":"codex-acp-test-peer"},"agentCapabilities":{"sessionCapabilities":{"fork":{},"resume":{}},"loadSession":true,"mcpCapabilities":{"http":true}},"authMethods":[{"id":"login","name":"Sign in to fixture"}]}),
            ),
            Some("authenticate") => {
                auth = false;
                result(id, json!({}));
            }
            Some("session/new" | "session/load" | "session/resume") => {
                inactive = false;
                cwd = params["cwd"].as_str().unwrap().into();
                if auth {
                    send(
                        json!({"id":id,"error":{"code":-32000,"message":"Sign in to the agent to continue"}}),
                    );
                    continue;
                }
                bridge = params["mcpServers"][0]["url"].as_str().unwrap().into();
                update(
                    json!({"sessionUpdate":"available_commands_update","availableCommands":[{"name":"fixture","description":"An adapter command"}]}),
                );
                result(
                    id,
                    json!({"sessionId":"fixture-session","configOptions":[{"id":"model","name":"Model","category":"model","type":"select","currentValue":"fixture-model","options":[{"value":"fixture-model","name":"Fixture"},{"value":"other","name":"Other"}]}]}),
                );
            }
            Some("session/fork") => {
                inactive = true;
                assert_eq!(
                    params["_meta"]["jetbrains"]["air"]["fork"]["messageId"],
                    "message-1"
                );
                result(id, json!({"sessionId":"fixture-session"}));
            }
            Some("session/set_config_option") => result(
                id,
                json!({"configOptions":[{"id":"model","name":"Model","category":"model","type":"select","currentValue":params["value"],"options":[{"value":"other","name":"Other"}]}]}),
            ),
            Some("session/prompt") => {
                assert!(!inactive, "A fork must be resumed before prompting");
                let prompt = params["prompt"][0]["text"].as_str().unwrap();
                if prompt == "hang" {
                    continue;
                }
                update(
                    json!({"sessionUpdate":"agent_thought_chunk","content":{"type":"text","text":"Checking the document."}}),
                );
                if matches!(prompt, "file-read" | "file-generated" | "file-outside") {
                    prompt_waiting = Some(id);
                    let path = if prompt == "file-outside" {
                        std::env::temp_dir().join("outside-acp-file.txt")
                    } else {
                        std::path::Path::new(&cwd).join(if prompt == "file-generated" {
                            "greet.rs"
                        } else {
                            "demo.md"
                        })
                    };
                    send(
                        json!({"jsonrpc":"2.0","id":"file-1","method":if prompt == "file-read" {"fs/read_text_file"} else {"fs/write_text_file"},
                        "params":{"sessionId":"fixture-session","path":path,"line":1,"limit":1,"content":"must be refused"}}),
                    );
                    continue;
                }
                if prompt == "permission" {
                    prompt_waiting = Some(id);
                    send(
                        json!({"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","toolCall":{"toolCallId":"tool-1","title":"Edit the document","rawInput":{"path":"demo.hick"}},"options":[{"optionId":"allow","name":"Allow once","kind":"allow_once"},{"optionId":"deny","name":"Reject","kind":"reject_once"}]}}),
                    );
                    continue;
                }
                if let Some(view_id) = prompt.strip_prefix("literate-edit:") {
                    let http = reqwest::Client::new();
                    let mut n = 100;
                    let mut call = async |name: &str, arguments: Value| {
                        n += 1;
                        let answer=http.post(&bridge).json(&json!({"jsonrpc":"2.0","id":n,"method":"tools/call","params":{"name":name,"arguments":arguments}})).send().await.unwrap().json::<Value>().await.unwrap();
                        assert!(answer.get("error").is_none(), "{answer}");
                        serde_json::from_str::<Value>(
                            answer["result"]["content"][0]["text"].as_str().unwrap(),
                        )
                        .unwrap()
                    };
                    let view = call("read_literate_view", json!({"id":view_id})).await;
                    let file = &view["files"][0];
                    let arranged=call("organize_literate_view",json!({"id":view_id,"revision":view["revision"],"sections":[{"path":file["path"],"from":0,"to":file["content"].as_str().unwrap().len(),"heading":"What this code does","explanation":"ACP reading of the source."}]})).await;
                    let source = arranged["source"]
                        .as_str()
                        .unwrap()
                        .replace("value = 1", "value = 2");
                    call(
                        "edit_literate_view",
                        json!({"id":view_id,"revision":arranged["revision"],"source":source}),
                    )
                    .await;
                }
                if prompt == "edit" {
                    let http = reqwest::Client::new();
                    let mut request_id = 1;
                    let mut call = async |name: &str, arguments: Value| {
                        request_id += 1;
                        http.post(&bridge).json(&json!({"jsonrpc":"2.0","id":request_id,"method":"tools/call","params":{"name":name,"arguments":arguments}})).send().await.unwrap().json::<Value>().await.unwrap()
                    };
                    let read = call(
                        "read_output",
                        json!({"path":"greet.rs","with_lineage":true}),
                    )
                    .await;
                    let text = read["result"]["content"][0]["text"].as_str().unwrap();
                    let hash = text
                        .lines()
                        .find_map(|line| {
                            line.split_once('|')
                                .filter(|(hash, _)| hash.len() == 4)
                                .map(|(hash, _)| hash.to_string())
                        })
                        .unwrap();
                    let edited = call("edit_output",json!({"path":"greet.rs","run":hash,"input":"fn greet() { println!(\"ACP\"); }"})).await;
                    assert_ne!(edited["result"]["isError"], true, "{edited}");
                    let verified = call("verify", json!({})).await;
                    assert_ne!(verified["result"]["isError"], true, "{verified}");
                }
                update(
                    json!({"sessionUpdate":"tool_call","toolCallId":"tool-1","title":"Read document","kind":"read","status":"in_progress"}),
                );
                update(
                    json!({"sessionUpdate":"tool_call_update","toolCallId":"tool-1","status":"completed","content":[{"type":"content","content":{"type":"text","text":"Read successfully"}}]}),
                );
                for text in ["Hello ", "from ACP. <hick:file> is literal here."] {
                    update(
                        json!({"sessionUpdate":"agent_message_chunk","messageId":"message-1","content":{"type":"text","text":text}}),
                    );
                }
                result(id, json!({"stopReason":"end_turn"}));
            }
            Some("session/cancel") => {} // Intentionally unresponsive: tests bounded kill.
            None if message["id"] == "file-1" => {
                let text = message["result"]["content"]
                    .as_str()
                    .or_else(|| message["error"]["message"].as_str())
                    .unwrap_or("missing reply");
                update(
                    json!({"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":text}}),
                );
                if let Some(id) = prompt_waiting.take() {
                    result(id, json!({"stopReason":"end_turn"}));
                }
            }
            None if message["id"] == "permission-1" => {
                update(
                    json!({"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":format!("Permission {}",message["result"]["outcome"]["optionId"].as_str().unwrap_or("cancelled"))}}),
                );
                if let Some(id) = prompt_waiting.take() {
                    result(id, json!({"stopReason":"end_turn"}));
                }
            }
            _ => result(id, json!({})),
        }
    }
}
