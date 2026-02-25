use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

use wrist_flick::llm::MockLlmClient;

async fn app() -> axum::Router {
    let llm = Arc::new(MockLlmClient::new("Hello! I'm your assistant."));
    let state = wrist_flick::build_app_state(llm).await.unwrap();
    wrist_flick::build_router(state)
}

#[tokio::test]
async fn test_health() {
    let app = app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_conversation_crud() {
    let app = app().await;

    // Create
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/conversations")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Test Conv"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let conv: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let conv_id = conv["id"].as_str().unwrap().to_string();
    assert_eq!(conv["title"].as_str().unwrap(), "Test Conv");

    // List
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/conversations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.len(), 1);

    // Get
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/conversations/{}", conv_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Delete
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("/api/v1/conversations/{}", conv_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_memory_crud() {
    let app = app().await;

    // Create
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/memories")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"key":"test-key","content":"test content"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let mem: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let mem_id = mem["id"].as_str().unwrap().to_string();

    // List
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/memories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Search
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/memories/search?q=test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!results.is_empty());

    // Delete
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("/api/v1/memories/{}", mem_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_task_crud() {
    let app = app().await;

    // Create
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/tasks")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"description":"Buy milk"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = task["id"].as_str().unwrap().to_string();
    assert_eq!(task["status"].as_str().unwrap(), "open");

    // List
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Update
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(&format!("/api/v1/tasks/{}", task_id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"done"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"].as_str().unwrap(), "done");

    // Delete
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("/api/v1/tasks/{}", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_state_vector() {
    let app = app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/sync/state-vector")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let sv: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(sv["state_vector"].as_str().is_some());
}
