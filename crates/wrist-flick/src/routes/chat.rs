use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::{Stream, StreamExt};
use serde::Deserialize;
use tokio::sync::Mutex;

use drift_signal_core::AppDriftSignal;

use crate::app_state::AppState;
use crate::llm::{ChatChunk, Message, Role};
use crate::routes::conversations::MessageResponse;
use crate::sse::ChatEvent;

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

pub async fn send_message(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    // 1. Persist user message
    {
        let mut core = state.core.lock().await;
        core.send_message(&conversation_id, "user", &req.content)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // 2. Search memories for context + build message history (read lock)
    let (memories, recent) = {
        let core = state.core.lock().await;

        let keywords = extract_keywords(&req.content);
        let memories = if !keywords.is_empty() {
            core.search_memories(&keywords).unwrap_or_default()
        } else {
            vec![]
        };

        let recent = core
            .list_recent_messages(&conversation_id, 20)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        (memories, recent)
    };

    // 3. Build LLM messages
    let mut llm_messages = Vec::new();

    let mut system = String::from(
        "You are a helpful personal AI assistant called Wrist-Flick. \
         You help the user manage tasks, remember important facts, and have conversations.\n\n\
         When you want to save a fact for later, include an XML block in your response:\n\
         <mem:fact key=\"short-key\">The fact content</mem:fact>\n\n\
         When you want to create a task, include:\n\
         <task:create description=\"Task description\"/>\n",
    );

    if !memories.is_empty() {
        system.push_str("\nRelevant memories:\n");
        for mem in &memories {
            system.push_str(&format!("- [{}]: {}\n", mem.key, mem.content));
        }
    }

    llm_messages.push(Message {
        role: Role::System,
        content: system,
    });

    for msg in &recent {
        let role = match msg.role.as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => continue,
        };
        llm_messages.push(Message {
            role,
            content: msg.content.clone(),
        });
    }

    // 4. Stream LLM response
    let assistant_msg_id = format!("msg-{}", uuid::Uuid::now_v7());
    let stream = state
        .llm
        .complete_stream(llm_messages)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event_stream = make_event_stream(
        stream,
        assistant_msg_id,
        conversation_id,
        state.core.clone(),
    );

    let sse_stream = event_stream.map(|chat_event| {
        let event_name = chat_event.event_name().to_string();
        let data = serde_json::to_string(&chat_event).unwrap_or_default();
        Ok(Event::default().event(event_name).data(data))
    });

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

pub async fn list_messages(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let core = state.core.lock().await;
    let messages = core
        .list_messages(&conversation_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        messages
            .into_iter()
            .map(|m| MessageResponse {
                id: m.id,
                role: m.role,
                content: m.content,
                created_at: m.created_at,
            })
            .collect(),
    ))
}

fn make_event_stream(
    stream: std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>,
    msg_id: String,
    conv_id: String,
    core: Arc<Mutex<AppDriftSignal>>,
) -> impl Stream<Item = ChatEvent> + Send {
    async_stream::stream! {
        use futures::StreamExt as _;

        yield ChatEvent::MessageStart {
            message_id: msg_id.clone(),
            role: "assistant".into(),
        };

        let mut full_response = String::new();
        let mut finish_reason = "stop".to_string();

        let mut stream = stream;
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    if !chunk.delta.is_empty() {
                        full_response.push_str(&chunk.delta);
                        yield ChatEvent::ContentDelta {
                            delta: chunk.delta,
                        };
                    }
                    if let Some(reason) = chunk.finish_reason {
                        finish_reason = reason;
                    }
                }
                Err(e) => {
                    yield ChatEvent::Error {
                        message: format!("{}", e),
                    };
                    return;
                }
            }
        }

        // Parse structured actions from the response
        let actions = parse_structured_actions(&full_response);

        // Persist assistant message + side effects via drift-signal-core
        {
            let mut core = core.lock().await;

            if let Err(e) = core.send_message(&conv_id, "assistant", &full_response) {
                log::error!("Failed to persist assistant message: {}", e);
            }

            for action in &actions {
                match action {
                    StructuredAction::Memory { key, content } => {
                        match core.create_fact(key, content, Some(&conv_id)) {
                            Ok(id) => {
                                yield ChatEvent::MemoryCreated {
                                    id,
                                    key: key.clone(),
                                };
                            }
                            Err(e) => {
                                log::error!("Failed to create memory: {}", e);
                            }
                        }
                    }
                    StructuredAction::Task { description } => {
                        match core.create_task(description, Some(&conv_id)) {
                            Ok(id) => {
                                yield ChatEvent::TaskCreated {
                                    id,
                                    description: description.clone(),
                                };
                            }
                            Err(e) => {
                                log::error!("Failed to create task: {}", e);
                            }
                        }
                    }
                }
            }
        }

        yield ChatEvent::MessageEnd {
            message_id: msg_id,
            finish_reason,
        };
    }
}

#[derive(Debug)]
enum StructuredAction {
    Memory { key: String, content: String },
    Task { description: String },
}

fn parse_structured_actions(response: &str) -> Vec<StructuredAction> {
    let mut actions = Vec::new();

    // Parse <mem:fact key="...">content</mem:fact>
    let mut search = response;
    while let Some(start) = search.find("<mem:fact") {
        if let Some(end) = search[start..].find("</mem:fact>") {
            let tag_content = &search[start..start + end + 11];
            if let (Some(key), Some(content)) = (
                extract_attr(tag_content, "key"),
                extract_text_content(tag_content, "mem:fact"),
            ) {
                actions.push(StructuredAction::Memory { key, content });
            }
            search = &search[start + end + 11..];
        } else {
            break;
        }
    }

    // Parse <task:create description="..."/>
    search = response;
    while let Some(start) = search.find("<task:create") {
        if let Some(end) = search[start..].find("/>") {
            let tag_content = &search[start..start + end + 2];
            if let Some(desc) = extract_attr(tag_content, "description") {
                actions.push(StructuredAction::Task { description: desc });
            }
            search = &search[start + end + 2..];
        } else if let Some(end) = search[start..].find("</task:create>") {
            let tag_content = &search[start..start + end + 14];
            if let Some(desc) = extract_attr(tag_content, "description") {
                actions.push(StructuredAction::Task { description: desc });
            }
            search = &search[start + end + 14..];
        } else {
            break;
        }
    }

    actions
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}=""#, attr_name);
    let start = tag.find(&pattern)?;
    let after = &tag[start + pattern.len()..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn extract_text_content(tag: &str, tag_name: &str) -> Option<String> {
    let open_end = tag.find('>')?;
    let close_start = tag.rfind(&format!("</{}", tag_name))?;
    Some(tag[open_end + 1..close_start].trim().to_string())
}

fn extract_keywords(text: &str) -> String {
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "can", "shall",
        "to", "of", "in", "for", "on", "with", "at", "by", "from", "as", "into", "about", "like",
        "through", "after", "over", "between", "out", "that", "this", "these", "those", "it",
        "its", "i", "me", "my", "we", "our", "you", "your", "he", "she", "they", "what", "which",
        "who", "when", "where", "why", "how", "not", "no", "but", "and", "or", "if", "then", "so",
        "up", "just",
    ];

    text.split_whitespace()
        .filter(|w| w.len() > 2 && !stop_words.contains(&w.to_lowercase().as_str()))
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory_action() {
        let response =
            r#"Here is info. <mem:fact key="api-status">API is 60% done</mem:fact> More text."#;
        let actions = parse_structured_actions(response);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            StructuredAction::Memory { key, content } => {
                assert_eq!(key, "api-status");
                assert_eq!(content, "API is 60% done");
            }
            _ => panic!("Expected Memory action"),
        }
    }

    #[test]
    fn test_parse_task_action() {
        let response = r#"Done. <task:create description="Follow up on migration"/>"#;
        let actions = parse_structured_actions(response);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            StructuredAction::Task { description } => {
                assert_eq!(description, "Follow up on migration");
            }
            _ => panic!("Expected Task action"),
        }
    }

    #[test]
    fn test_parse_multiple_actions() {
        let response = r#"Info. <mem:fact key="k1">fact1</mem:fact> <task:create description="t1"/> <mem:fact key="k2">fact2</mem:fact>"#;
        let actions = parse_structured_actions(response);
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn test_extract_keywords() {
        let text = "What is the status of the API migration project?";
        let keywords = extract_keywords(text);
        assert!(keywords.contains("status"));
        assert!(keywords.contains("API"));
        assert!(keywords.contains("migration"));
        assert!(!keywords.contains("the"));
    }
}
