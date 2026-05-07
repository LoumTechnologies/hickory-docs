use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;

use hick_grove::GroveEngine;
use hick_grove::namespace::DocId;

use crate::routes::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(engine): State<AppState>,
) -> Response {
    let doc_id = DocId::from_string(id);
    ws.on_upgrade(move |socket| handle_ws(socket, doc_id, engine))
}

async fn handle_ws(mut socket: WebSocket, doc_id: DocId, engine: Arc<GroveEngine>) {
    // Send initial state vector
    if let Ok(sv) = engine.state_vector(&doc_id) {
        let _ = socket.send(Message::Binary(sv.into())).await;
    }

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Binary(data) => {
                // Treat binary messages as Yrs updates
                let command = hick_grove::command::GroveCommand::BinaryUpdate {
                    doc_id: doc_id.clone(),
                    update: data.to_vec(),
                };
                if let Ok(result) = engine.process_command(command).await {
                    // Send back the new state vector
                    if let Ok(sv) = engine.state_vector(&doc_id) {
                        let _ = socket.send(Message::Binary(sv.into())).await;
                    }
                    log::debug!(
                        "WS update applied, {} events dispatched",
                        result.events_dispatched
                    );
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
