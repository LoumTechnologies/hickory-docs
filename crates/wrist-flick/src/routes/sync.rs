use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{Json, Response};

use hick_grove::GroveEngine;
use hick_grove::command::GroveCommand;
use hick_grove::namespace::DocId;

use crate::app_state::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let doc_id = state.doc_id.clone();
    let engine = state.engine.clone();
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
                let command = GroveCommand::BinaryUpdate {
                    doc_id: doc_id.clone(),
                    update: data.to_vec(),
                };
                if let Ok(_result) = engine.process_command(command).await {
                    if let Ok(sv) = engine.state_vector(&doc_id) {
                        let _ = socket.send(Message::Binary(sv.into())).await;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

#[derive(serde::Serialize)]
pub struct StateVectorResponse {
    pub state_vector: String,
}

pub async fn state_vector(
    State(state): State<AppState>,
) -> Result<Json<StateVectorResponse>, StatusCode> {
    let sv = state
        .engine
        .state_vector(&state.doc_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&sv);

    Ok(Json(StateVectorResponse {
        state_vector: encoded,
    }))
}
