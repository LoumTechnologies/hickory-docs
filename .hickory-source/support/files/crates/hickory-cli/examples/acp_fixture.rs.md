<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/examples/acp_fixture.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `68d177f2fa3cd33883815ad41c99480b4b4422760875e1d095bb5b4839fa18ff`.

<book:entry path="crates/hickory-cli/examples/acp_fixture.rs">
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
    let changesets = std::env::args().any(|a| a == "--changesets");
    let mut contract_turn = String::new();
    let mut never_finalize = false;
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    while let Some(line) = lines.next() {
        let message: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let id = message["id"].clone();
        let params = &message["params"];
        match message["method"].as_str() {
            Some("initialize") => {
                let mut capabilities = json!({"sessionCapabilities":{"fork":{},"resume":{}},"loadSession":true,"mcpCapabilities":{"http":true}});
                if changesets {
                    assert_eq!(
                        params["clientCapabilities"]["_meta"]["hickorydocs.com/changesets"]["version"],
                        1
                    );
                    capabilities["_meta"] = json!({"hickorydocs.com/changesets":{"version":1}});
                }
                result(
                    id,
                    json!({"protocolVersion":1,"agentInfo":{"name":"codex-acp-test-peer"},"agentCapabilities":capabilities,"authMethods":[{"id":"login","name":"Sign in to fixture"}]}),
                );
            }
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
                let prompt = params["prompt"][0]["text"]
                    .as_str()
                    .unwrap()
                    .split("\n\n")
                    .next()
                    .unwrap()
                    .lines()
                    .next()
                    .unwrap();
                if changesets
                    && (prompt.starts_with("contract-")
                        || prompt.starts_with("Hickory changeset not accepted"))
                {
                    let meta = &params["_meta"]["hickorydocs.com/changesets"];
                    assert_eq!(meta["required"], true);
                    let boundary = meta["boundary"].as_str().unwrap();
                    let first = contract_turn != boundary;
                    contract_turn = boundary.into();
                    if first && prompt == "contract-buffer" {
                        let client = reqwest::Client::new();
                        let read: Value = client.post(&bridge).json(&json!({"jsonrpc":"2.0","id":21,"method":"tools/call","params":{"name":"read_buffer","arguments":{}}})).send().await.unwrap().json().await.unwrap();
                        let text = read["result"]["content"][0]["text"].as_str().unwrap();
                        let hash = text
                            .lines()
                            .filter_map(|l| l.split_once('|'))
                            .map(|(hash, _)| hash)
                            .find(|hash| {
                                !hash.is_empty() && hash.bytes().all(|b| b.is_ascii_hexdigit())
                            })
                            .unwrap();
                        let edit: Value = client.post(&bridge).json(&json!({"jsonrpc":"2.0","id":22,"method":"tools/call","params":{"name":"edit_buffer","arguments":{"run":hash,"input":"Edited by the agent"}}})).send().await.unwrap().json().await.unwrap();
                        assert_ne!(edit["result"]["isError"], true, "{edit}");
                    }
                    if first {
                        std::fs::write(
                            std::path::Path::new(&cwd).join("app.ts"),
                            "abandoned draft\n",
                        )
                        .unwrap();
                        std::fs::write(std::path::Path::new(&cwd).join("app.ts"), "contract B\n")
                            .unwrap();
                    }
                    if prompt == "contract-never" {
                        never_finalize = true;
                    }
                    if never_finalize || prompt == "contract-cancel" {
                        // Dedicated failure/cancellation witnesses.
                        if prompt == "contract-cancel" {
                            continue;
                        }
                        result(id, json!({"stopReason":"end_turn"}));
                        continue;
                    }
                    // Exercise the negotiated ACP route, including stale-turn rejection.
                    send(
                        json!({"jsonrpc":"2.0","id":801,"method":"_hickorydocs.com/changeset","params":{"sessionId":"fixture-session","boundary":"expired","operation":"status"}}),
                    );
                    let stale: Value =
                        serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
                    assert!(stale.get("error").is_some());
                    send(
                        json!({"jsonrpc":"2.0","id":802,"method":"_hickorydocs.com/changeset","params":{"sessionId":"fixture-session","boundary":boundary,"operation":"status"}}),
                    );
                    let status: Value =
                        serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
                    let status = &status["result"];
                    assert_eq!(status["version"], 1);
                    let entry = status["entry"].as_str().unwrap();
                    let sources = json!({entry:status["template"]});
                    let arguments = json!({"boundary":boundary,"expected":status["expected"],"entry":entry,"sources":sources});
                    let bad_submission = first && prompt != "contract-late-write";
                    let body = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"submit_changeset","arguments":if bad_submission { let mut bad = arguments.clone(); bad["sources"] = json!({"response.md":"<hick:project/>"}); bad } else { arguments.clone() }}});
                    let response: Value = reqwest::Client::new()
                        .post(&bridge)
                        .json(&body)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    let value: Value = serde_json::from_str(
                        response["result"]["content"][0]["text"].as_str().unwrap(),
                    )
                    .unwrap();
                    assert_eq!(value["accepted"], !bad_submission);
                    if first && prompt == "contract-late-write" {
                        std::fs::write(std::path::Path::new(&cwd).join("app.ts"), "late C\n")
                            .unwrap();
                    }
                    if !first {
                        // Re-submit through ACP too: both routes share acceptance.
                        let mut args = arguments;
                        args["sessionId"] = json!("fixture-session");
                        args["operation"] = json!("submit");
                        send(
                            json!({"jsonrpc":"2.0","id":803,"method":"_hickorydocs.com/changeset","params":args}),
                        );
                        let accepted: Value =
                            serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
                        assert_eq!(accepted["result"]["accepted"], true);
                    }
                    update(
                        json!({"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"Clean successful explanation."}}),
                    );
                    result(id, json!({"stopReason":"end_turn"}));
                    continue;
                }
                if prompt == "hang" {
                    continue;
                }
                if prompt == "literate-native" {
                    let root = std::path::Path::new(&cwd);
                    std::fs::write(root.join("app.ts"), "abandoned draft\n").unwrap();
                    update(
                        json!({"sessionUpdate":"tool_call","toolCallId":"native-draft","title":"Native draft","status":"completed","rawOutput":"abandoned draft"}),
                    );
                    std::fs::write(root.join("app.ts"), "native completed\n").unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    update(
                        json!({"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"Kept the completed native change."}}),
                    );
                    result(id, json!({"stopReason":"end_turn"}));
                    continue;
                }
                if prompt == "untitled-paragraph" {
                    // Deliberately omit a citation: context must still accompany final speech.
                    update(
                        json!({"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"The second paragraph has the most characters."}}),
                    );
                    result(id, json!({"stopReason":"end_turn"}));
                    continue;
                }
                if prompt == "ribbon-file" {
                    update(
                        json!({"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"See [code.py](code.py:1)."}}),
                    );
                    result(id, json!({"stopReason":"end_turn"}));
                    continue;
                }
                update(
                    json!({"sessionUpdate":"agent_thought_chunk","content":{"type":"text","text":"Checking the document."}}),
                );
                if matches!(
                    prompt,
                    "file-read" | "file-read-range" | "file-generated" | "file-outside"
                ) {
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
                        json!({"jsonrpc":"2.0","id":"file-1","method":if prompt.starts_with("file-read") {"fs/read_text_file"} else {"fs/write_text_file"},
                        "params":{"sessionId":"fixture-session","path":path,"line":if prompt == "file-read-range" {2} else {1},"limit":1,"content":"must be refused"}}),
                    );
                    continue;
                }
                if prompt == "permission" {
                    prompt_waiting = Some(id);
                    send(
                        json!({"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","toolCall":{"toolCallId":"tool-1","title":"Edit the document","rawInput":{"path":"demo.md"}},"options":[{"optionId":"allow","name":"Allow once","kind":"allow_once"},{"optionId":"deny","name":"Reject","kind":"reject_once"}]}}),
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
                if prompt == "buffer-edit" {
                    let http = reqwest::Client::new();
                    let mut n = 400;
                    let mut call = async |name: &str, arguments: Value| {
                        n += 1;
                        http.post(&bridge).json(&json!({"jsonrpc":"2.0","id":n,"method":"tools/call","params":{"name":name,"arguments":arguments}})).send().await.unwrap().json::<Value>().await.unwrap()
                    };
                    let read = call("read_buffer", json!({})).await;
                    let text = read["result"]["content"][0]["text"].as_str().unwrap();
                    let hash = text
                        .lines()
                        .find_map(|l| {
                            l.split_once('|')
                                .filter(|(h, _)| h.len() == 64)
                                .map(|(h, _)| h.to_string())
                        })
                        .unwrap();
                    let result = call(
                        "edit_buffer",
                        json!({"run":hash,"input":"# Changed by ACP 📝"}),
                    )
                    .await;
                    update(
                        json!({"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":result["result"]["content"][0]["text"]}}),
                    );
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
                                .filter(|(hash, _)| hash.len() == 64)
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
</book:entry>

</book:doc>
