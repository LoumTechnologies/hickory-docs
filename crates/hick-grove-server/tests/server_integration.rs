use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use serde_json::{Value, json};
use tower::ServiceExt;

use hick_grove::plugin::NamespaceRegistry;
use hick_grove::plugins::tasks::{TasksHandler, TasksSqliteStore};
use hick_grove::{GroveConfig, GroveEngine};

fn build_app() -> (Router, Arc<TasksSqliteStore>) {
    let tasks_store = Arc::new(TasksSqliteStore::new_in_memory().unwrap());
    let tasks_handler = Arc::new(TasksHandler::new(tasks_store.clone()));

    let mut registry = NamespaceRegistry::new();
    registry.register(tasks_handler);

    let engine = Arc::new(GroveEngine::new(GroveConfig { registry }));

    // We can't import from the binary's private modules directly,
    // so we duplicate the router setup here.
    // In a real setup, the routes would be in a lib crate.
    let app = Router::new()
        .route(
            "/documents",
            post(hick_grove_server_routes::create_document),
        )
        .route(
            "/documents/{id}/messages",
            post(hick_grove_server_routes::process_message),
        )
        .route(
            "/documents/{id}/state-vector",
            get(hick_grove_server_routes::get_state_vector),
        )
        .with_state(engine);

    (app, tasks_store)
}

// Inline route handlers (since we can't import from the binary).
mod hick_grove_server_routes {
    use std::sync::Arc;

    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::response::Json;
    use serde::{Deserialize, Serialize};

    use hick_grove::GroveEngine;
    use hick_grove::command::GroveCommand;
    use hick_grove::namespace::{DocId, NamespaceUri};

    pub type AppState = Arc<GroveEngine>;

    #[derive(Deserialize)]
    pub struct CreateDocRequest {
        pub namespaces: std::collections::HashMap<String, String>,
    }

    #[derive(Serialize)]
    pub struct CreateDocResponse {
        pub id: String,
    }

    pub async fn create_document(
        State(engine): State<AppState>,
        Json(req): Json<CreateDocRequest>,
    ) -> Result<Json<CreateDocResponse>, StatusCode> {
        let namespaces: Vec<(String, NamespaceUri)> = req
            .namespaces
            .into_iter()
            .map(|(prefix, uri)| (prefix, NamespaceUri::new(uri)))
            .collect();

        let doc_id = engine.create_document(namespaces).await;

        Ok(Json(CreateDocResponse {
            id: doc_id.as_str().to_string(),
        }))
    }

    #[derive(Deserialize)]
    pub struct MessageRequest {
        pub xml: String,
    }

    #[derive(Serialize)]
    pub struct MessageResponse {
        pub events_dispatched: usize,
    }

    pub async fn process_message(
        State(engine): State<AppState>,
        Path(id): Path<String>,
        Json(req): Json<MessageRequest>,
    ) -> Result<Json<MessageResponse>, StatusCode> {
        let doc_id = DocId::from_string(id);
        let command = GroveCommand::StructuredMessage {
            doc_id,
            xml: req.xml,
        };

        let result = engine
            .process_command(command)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        Ok(Json(MessageResponse {
            events_dispatched: result.events_dispatched,
        }))
    }

    #[derive(Serialize)]
    pub struct StateVectorResponse {
        pub state_vector: String,
    }

    pub async fn get_state_vector(
        State(engine): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<Json<StateVectorResponse>, StatusCode> {
        use base64::Engine as _;
        let doc_id = DocId::from_string(id);
        let sv = engine
            .state_vector(&doc_id)
            .map_err(|_| StatusCode::NOT_FOUND)?;

        let encoded = base64::engine::general_purpose::STANDARD.encode(&sv);

        Ok(Json(StateVectorResponse {
            state_vector: encoded,
        }))
    }
}

#[tokio::test]
async fn test_create_document_and_send_message() {
    let (app, tasks_store) = build_app();

    // 1. Create a document
    let create_body = json!({
        "namespaces": {
            "task": "https://example.com/vocab/task#"
        }
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/documents")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_resp: Value = serde_json::from_slice(&body).unwrap();
    let doc_id = create_resp["id"].as_str().unwrap().to_string();
    assert!(doc_id.starts_with("doc-"));

    // 2. Send a structured message
    let xml = format!(
        r#"<message xmlns:task="https://example.com/vocab/task#">
            <task:create id="task-1" description="Change oil">created a task</task:create>
        </message>"#
    );
    let msg_body = json!({ "xml": xml });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/documents/{}/messages", doc_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&msg_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let msg_resp: Value = serde_json::from_slice(&body).unwrap();
    assert!(msg_resp["events_dispatched"].as_u64().unwrap() > 0);

    // 3. Verify task was created in SQLite
    let task = tasks_store.get_task("task-1").unwrap();
    assert!(task.is_some());
    let task = task.unwrap();
    assert_eq!(task.description, "Change oil");

    // 4. Get state vector
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/documents/{}/state-vector", doc_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_nonexistent_document_returns_error() {
    let (app, _) = build_app();

    let msg_body = json!({ "xml": "<root/>" });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/documents/nonexistent/messages")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&msg_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
