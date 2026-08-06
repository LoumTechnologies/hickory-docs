//! Integration tests against a real Postgres (docker compose `db` service).
//! Run with `--test-threads=1` (the workspace convention).

use std::sync::Arc;

use futures::StreamExt as _;
use hickory_server::config::{AppEnv, Config, ExecutorKind, StripeConfig};
use hickory_server::{build_router, build_state};
use hmac::{Hmac, Mac as _};
use serde_json::{Value, json};
use sha2::Sha256;
use tokio_tungstenite::tungstenite::Message as TtMessage;

const WEBHOOK_SECRET: &str = "whsec_test_secret";

struct TestApp {
    base: String,
    port: u16,
    db: sqlx::PgPool,
    client: reqwest::Client,
    _git_dir: tempfile::TempDir,
    git: hickory_server::gitstore::GitStore,
}

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

async fn setup() -> TestApp {
    let _ = env_logger::builder().is_test(true).try_init();

    let admin_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://hickory:hickory@localhost:5433/hickory".to_string());

    // Bring up Postgres via docker compose when it is not already reachable.
    if sqlx::PgPool::connect(&admin_url).await.is_err() {
        let status = std::process::Command::new("docker")
            .args(["compose", "up", "-d", "--wait", "db"])
            .current_dir(repo_root())
            .status()
            .expect("docker compose must be available for integration tests");
        assert!(status.success(), "docker compose up db failed");
    }
    let admin = sqlx::PgPool::connect(&admin_url)
        .await
        .expect("postgres reachable");

    // Fresh database per test.
    let dbname = format!("hickory_test_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {dbname}"))
        .execute(&admin)
        .await
        .unwrap();
    let test_url = {
        let (base, _) = admin_url.rsplit_once('/').unwrap();
        format!("{base}/{dbname}")
    };

    let git_dir = tempfile::tempdir().unwrap();
    let config = Config {
        app_env: AppEnv::Dev,
        port: 0,
        database_url: test_url.clone(),
        jwt_secret: "test-secret".to_string(),
        git_data_dir: git_dir.path().to_path_buf(),
        executor: ExecutorKind::Local,
        app_base_url: "http://localhost:0".to_string(),
        stripe: Some(StripeConfig {
            secret_key: "sk_test_dummy".to_string(),
            webhook_secret: Some(WEBHOOK_SECRET.to_string()),
        }),
        posthog: None,
        web_dist_dir: None,
        plan_set: None,
        anthropic_api_key: None,
    };

    let db = hickory_server::init_db(&test_url).await.unwrap();
    let state = build_state(config, db.clone()).unwrap();
    let git = state.git.clone();
    let router = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    TestApp {
        base: format!("http://127.0.0.1:{port}"),
        port,
        db,
        client: reqwest::Client::new(),
        _git_dir: git_dir,
        git,
    }
}

impl TestApp {
    async fn post(&self, path: &str, token: Option<&str>, body: Value) -> (u16, Value) {
        let mut req = self.client.post(format!("{}{path}", self.base)).json(&body);
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let resp = req.send().await.unwrap();
        let status = resp.status().as_u16();
        let v = resp.json().await.unwrap_or(Value::Null);
        (status, v)
    }

    async fn get(&self, path: &str, token: Option<&str>) -> (u16, Value) {
        let mut req = self.client.get(format!("{}{path}", self.base));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let resp = req.send().await.unwrap();
        let status = resp.status().as_u16();
        let v = resp.json().await.unwrap_or(Value::Null);
        (status, v)
    }

    async fn put(&self, path: &str, token: Option<&str>, body: Value) -> (u16, Value) {
        let mut req = self.client.put(format!("{}{path}", self.base)).json(&body);
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let resp = req.send().await.unwrap();
        let status = resp.status().as_u16();
        let v = resp.json().await.unwrap_or(Value::Null);
        (status, v)
    }

    async fn signup(&self, email: &str) -> (String, String) {
        let (status, v) = self
            .post(
                "/api/auth/signup",
                None,
                json!({ "email": email, "password": "password123" }),
            )
            .await;
        assert_eq!(status, 200, "signup failed: {v}");
        (
            v["token"].as_str().unwrap().to_string(),
            v["user"]["id"].as_str().unwrap().to_string(),
        )
    }
}

const TEST_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Streaming test

<hick:container name="shell" image="alpine:3.20" />

<hick:exec container="shell">
echo hello
<hick:expect match="exact">hello
</hick:expect>
</hick:exec>
</hick:doc>
"#;

// ---------------------------------------------------------------------------
// Auth + doc CRUD → git
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn auth_projects_docs_and_git() {
    let app = setup().await;
    let (token, user_id) = app.signup("alice@example.com").await;

    // me
    let (status, me) = app.get("/api/me", Some(&token)).await;
    assert_eq!(status, 200);
    assert_eq!(me["id"].as_str().unwrap(), user_id);
    assert_eq!(me["plan"], "open");

    // login
    let (status, v) = app
        .post(
            "/api/auth/login",
            None,
            json!({ "email": "alice@example.com", "password": "password123" }),
        )
        .await;
    assert_eq!(status, 200);
    assert!(v["token"].is_string());

    // bad login
    let (status, _) = app
        .post(
            "/api/auth/login",
            None,
            json!({ "email": "alice@example.com", "password": "wrong-pass" }),
        )
        .await;
    assert_eq!(status, 401);

    // Private project (open plan allows exactly one).
    let (status, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "p1", "visibility": "private" }),
        )
        .await;
    assert_eq!(status, 201, "{project}");
    let project_id = project["id"].as_str().unwrap().to_string();

    let (status, err) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "p2", "visibility": "private" }),
        )
        .await;
    assert_eq!(
        status, 403,
        "second private project must hit the plan limit"
    );
    assert!(err["error"].as_str().unwrap().contains("upgrade"));

    // Doc create + save → git commits.
    let (status, doc) = app
        .post(
            &format!("/api/projects/{project_id}/docs"),
            Some(&token),
            json!({ "path": "guide.hick", "source": "# v1\n" }),
        )
        .await;
    assert_eq!(status, 201, "{doc}");
    let doc_id = doc["id"].as_str().unwrap().to_string();

    let (status, saved) = app
        .put(
            &format!("/api/docs/{doc_id}"),
            Some(&token),
            json!({ "source": "# v2\n" }),
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(saved["source"], "# v2\n");

    let project_uuid: uuid::Uuid = project_id.parse().unwrap();
    let commits = app.git.commit_count(project_uuid).await.unwrap();
    assert_eq!(commits, 2, "one commit per save");
    let on_disk =
        std::fs::read_to_string(app.git.project_dir(project_uuid).join("guide.hick")).unwrap();
    assert_eq!(on_disk, "# v2\n");

    // Doc list + fetch.
    let (status, docs) = app
        .get(&format!("/api/projects/{project_id}/docs"), Some(&token))
        .await;
    assert_eq!(status, 200);
    assert_eq!(docs.as_array().unwrap().len(), 1);

    // Private doc is invisible to strangers and anonymous readers.
    let (status, _) = app.get(&format!("/api/docs/{doc_id}"), None).await;
    assert_eq!(status, 403);
    let (bob_token, _) = app.signup("bob@example.com").await;
    let (status, _) = app
        .get(&format!("/api/docs/{doc_id}"), Some(&bob_token))
        .await;
    assert_eq!(status, 403);

    // Public project docs are readable anonymously.
    let (_, public_project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "pub", "visibility": "public" }),
        )
        .await;
    let pub_id = public_project["id"].as_str().unwrap();
    let (_, pub_doc) = app
        .post(
            &format!("/api/projects/{pub_id}/docs"),
            Some(&token),
            json!({ "path": "open.hick", "source": "# public\n" }),
        )
        .await;
    let (status, fetched) = app
        .get(
            &format!("/api/docs/{}", pub_doc["id"].as_str().unwrap()),
            None,
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(fetched["source"], "# public\n");
}

// ---------------------------------------------------------------------------
// Run streaming over the WS run channel
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn run_streams_transcript_events() {
    let app = setup().await;
    let (token, _) = app.signup("runner@example.com").await;
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "runs", "visibility": "public" }),
        )
        .await;
    let project_id = project["id"].as_str().unwrap();
    let (_, doc) = app
        .post(
            &format!("/api/projects/{project_id}/docs"),
            Some(&token),
            json!({ "path": "stream.hick", "source": TEST_DOC }),
        )
        .await;
    let doc_id = doc["id"].as_str().unwrap().to_string();

    // Connect the realtime socket first so run events reach us.
    let ws_url = format!(
        "ws://127.0.0.1:{}/api/ws?doc=doc:{doc_id}&token={token}",
        app.port
    );
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url).await.unwrap();

    // Start the run.
    let (status, v) = app
        .post(&format!("/api/docs/{doc_id}/run"), Some(&token), json!({}))
        .await;
    assert_eq!(status, 202, "{v}");
    let run_id = v["run_id"].as_str().unwrap().to_string();

    // Collect run-channel messages until the terminal one.
    let mut events: Vec<Value> = Vec::new();
    let mut terminal: Option<Value> = None;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    while terminal.is_none() {
        let frame = tokio::time::timeout_at(deadline, ws.next())
            .await
            .expect("run did not finish in time")
            .expect("socket closed early")
            .unwrap();
        let data = match frame {
            TtMessage::Binary(b) => b,
            _ => continue,
        };
        if data.is_empty() || data[0] != 0x01 {
            continue; // Yjs channel
        }
        let msg: Value = serde_json::from_slice(&data[1..]).unwrap();
        assert_eq!(msg["run_id"].as_str().unwrap(), run_id);
        if msg.get("status").is_some() {
            terminal = Some(msg);
        } else {
            events.push(msg);
        }
    }
    ws.close(None).await.ok();

    assert_eq!(terminal.unwrap()["status"], "ok");
    assert!(!events.is_empty(), "expected streamed transcript events");
    // Events are keyed exactly like block ids and carry ms offsets.
    let kinds: Vec<&str> = events
        .iter()
        .map(|e| e["event"]["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"cmd"), "kinds: {kinds:?}");
    assert!(kinds.contains(&"out"), "kinds: {kinds:?}");
    assert!(kinds.contains(&"exit"), "kinds: {kinds:?}");
    for e in &events {
        let exec_id = e["exec_id"].as_str().unwrap();
        assert!(exec_id.starts_with("shell:"), "exec_id {exec_id}");
        assert!(
            e["event"]["t"].is_u64(),
            "t must be ms since run start: {e}"
        );
    }
    let out = events.iter().find(|e| e["event"]["kind"] == "out").unwrap();
    assert!(out["event"]["data"].as_str().unwrap().contains("hello"));

    // Stored run.
    let (status, run) = app.get(&format!("/api/runs/{run_id}"), Some(&token)).await;
    assert_eq!(status, 200);
    assert_eq!(run["status"], "ok");
    assert!(run["started_at"].is_string());
    let blocks = run["blocks"].as_array().unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0]["status"], "ok");
    assert!(blocks[0]["exec_id"].as_str().unwrap().starts_with("shell:"));
    assert!(!blocks[0]["transcript"].as_array().unwrap().is_empty());

    // Render shows the last-run status + transcript.
    let (status, render) = app
        .get(&format!("/api/docs/{doc_id}/render"), Some(&token))
        .await;
    assert_eq!(status, 200);
    let exec = render["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["kind"] == "exec")
        .expect("exec block in render");
    assert_eq!(exec["status"], "ok");
    assert!(!exec["transcript"].as_array().unwrap().is_empty());

    // /check also works end to end (expectation passes).
    let (status, v) = app
        .post(
            &format!("/api/docs/{doc_id}/check"),
            Some(&token),
            json!({}),
        )
        .await;
    assert_eq!(status, 202);
    let check_id = v["run_id"].as_str().unwrap().to_string();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let (_, run) = app
            .get(&format!("/api/runs/{check_id}"), Some(&token))
            .await;
        match run["status"].as_str().unwrap() {
            "ok" => break,
            "failed" => panic!("check failed: {run}"),
            _ => {
                assert!(tokio::time::Instant::now() < deadline, "check timed out");
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plans endpoint shape (mirror of apps/web/src/api/types.ts)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn plans_endpoint_matches_pinned_shape() {
    let app = setup().await;
    let (status, v) = app.get("/api/billing/plans", None).await;
    assert_eq!(status, 200);

    let plans = v["plans"].as_array().expect("plans array");
    let keys: Vec<&str> = plans.iter().map(|p| p["key"].as_str().unwrap()).collect();
    assert_eq!(
        keys,
        vec!["open", "pro", "team", "business"],
        "plan-set order"
    );

    for plan in plans {
        for field in ["key", "name", "description"] {
            assert!(plan[field].is_string(), "plan.{field} missing: {plan}");
        }
        let features = plan["features"].as_array().unwrap();
        assert!(!features.is_empty(), "features derived from entitlements");
        assert!(features.iter().all(Value::is_string));
        for price in plan["prices"].as_array().unwrap() {
            assert!(price["key"].is_string());
            let interval = price["interval"].as_str().unwrap();
            assert!(interval == "month" || interval == "year");
            assert!(price["amount_cents"].is_i64() || price["amount_cents"].is_u64());
            assert!(price["currency"].is_string());
            // Base prices only: per-seat add-ons never appear here.
            assert!(price.get("per_seat").is_none_or(|v| v != true));
        }
    }
    // Free tier has no prices; paid tiers have month+year.
    assert!(plans[0]["prices"].as_array().unwrap().is_empty());
    assert_eq!(plans[2]["prices"].as_array().unwrap().len(), 2);
    assert_eq!(plans[2]["highlight"], true);
    assert_eq!(plans[1]["trial_days"], 14);
    assert_eq!(v["enterprise"]["contact"], true);
}

// ---------------------------------------------------------------------------
// Webhook idempotency, raced concurrently
// ---------------------------------------------------------------------------

fn sign(payload: &str) -> String {
    let t = chrono::Utc::now().timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(WEBHOOK_SECRET.as_bytes()).unwrap();
    mac.update(format!("{t}.").as_bytes());
    mac.update(payload.as_bytes());
    format!("t={t},v1={}", hex::encode(mac.finalize().into_bytes()))
}

fn checkout_event(event_id: &str, user_id: &str, subscription_id: &str) -> String {
    json!({
        "id": event_id,
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "id": "cs_test_1",
                "subscription": subscription_id,
                "customer": "cus_test_1",
                "client_reference_id": user_id,
                "amount_total": 14900,
                "currency": "usd",
                "metadata": {
                    "user_id": user_id,
                    "price_key": "team-monthly-v1",
                    "plan_key": "team"
                }
            }
        }
    })
    .to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn webhook_fulfillment_races_to_exactly_one() {
    let app = setup().await;
    let (_token, user_id) = app.signup("buyer@example.com").await;

    // Distinct Stripe event ids for the SAME subscription object: the event
    // log cannot short-circuit these; only the unique fulfillment claim on
    // the subscription id may collapse them. Genuinely spawned tasks on a
    // multi-thread runtime, released together by a barrier.
    const N: usize = 6;
    let barrier = Arc::new(tokio::sync::Barrier::new(N));
    let mut handles = Vec::new();
    for i in 0..N {
        let barrier = barrier.clone();
        let base = app.base.clone();
        let payload = checkout_event(&format!("evt_race_{i}"), &user_id, "sub_race_1");
        handles.push(tokio::spawn(async move {
            let client = reqwest::Client::new();
            let sig = sign(&payload);
            barrier.wait().await;
            let resp = client
                .post(format!("{base}/api/billing/webhook"))
                .header("stripe-signature", sig)
                .header("content-type", "application/json")
                .body(payload)
                .send()
                .await
                .unwrap();
            resp.status().as_u16()
        }));
    }
    for h in handles {
        assert_eq!(h.await.unwrap(), 200);
    }

    // Exactly one fulfillment.
    let subs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM subscriptions WHERE stripe_subscription_id = 'sub_race_1'",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(
        subs, 1,
        "exactly one subscription row despite {N} racing events"
    );

    let (plan, price): (String, Option<String>) =
        sqlx::query_as("SELECT plan_key, price_key FROM users WHERE email = 'buyer@example.com'")
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(plan, "team");
    assert_eq!(price.as_deref(), Some("team-monthly-v1"));

    // All distinct event ids were logged.
    let events: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM stripe_events WHERE id LIKE 'evt_race_%'")
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(events, N as i64);

    // Redelivery of an already-seen event id short-circuits on the log.
    let payload = checkout_event("evt_race_0", &user_id, "sub_race_1");
    let resp = app
        .client
        .post(format!("{}/api/billing/webhook", app.base))
        .header("stripe-signature", sign(&payload))
        .header("content-type", "application/json")
        .body(payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let v: Value = resp.json().await.unwrap();
    assert_eq!(v["duplicate"], true);

    // Bad signature is rejected.
    let payload = checkout_event("evt_bad_sig", &user_id, "sub_race_1");
    let resp = app
        .client
        .post(format!("{}/api/billing/webhook", app.base))
        .header("stripe-signature", "t=1,v1=deadbeef")
        .header("content-type", "application/json")
        .body(payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 400);

    // Dunning restricts to free-tier entitlements (never deletes).
    let payload = json!({
        "id": "evt_dunning_1",
        "type": "invoice.payment_failed",
        "data": { "object": { "subscription": "sub_race_1" } }
    })
    .to_string();
    let resp = app
        .client
        .post(format!("{}/api/billing/webhook", app.base))
        .header("stripe-signature", sign(&payload))
        .header("content-type", "application/json")
        .body(payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let status: String =
        sqlx::query_scalar("SELECT billing_status FROM users WHERE email = 'buyer@example.com'")
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(status, "past_due");

    // Subscription deletion downgrades to open, keeping the account.
    let payload = json!({
        "id": "evt_deleted_1",
        "type": "customer.subscription.deleted",
        "data": { "object": { "id": "sub_race_1" } }
    })
    .to_string();
    let resp = app
        .client
        .post(format!("{}/api/billing/webhook", app.base))
        .header("stripe-signature", sign(&payload))
        .header("content-type", "application/json")
        .body(payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let (plan, status): (String, String) = sqlx::query_as(
        "SELECT plan_key, billing_status FROM users WHERE email = 'buyer@example.com'",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(plan, "open");
    assert_eq!(status, "active");
}

// ---------------------------------------------------------------------------
// Misc contract checks
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn health_and_agent_stub() {
    let app = setup().await;
    let (status, v) = app.get("/api/health", None).await;
    assert_eq!(status, 200);
    assert_eq!(v["ok"], true);
    assert_eq!(v["executor"], "local");
    assert_eq!(v["db"], true);

    // GET /api/executor: local executor reports no image map (api.md).
    let (status, v) = app.get("/api/executor", None).await;
    assert_eq!(status, 200);
    assert_eq!(v["kind"], "local");
    assert_eq!(v["images"], serde_json::Value::Null);

    let (token, _) = app.signup("agent@example.com").await;
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "a", "visibility": "public" }),
        )
        .await;
    let (_, doc) = app
        .post(
            &format!("/api/projects/{}/docs", project["id"].as_str().unwrap()),
            Some(&token),
            json!({ "path": "a.hick", "source": "# hi\n" }),
        )
        .await;
    let (status, v) = app
        .post(
            &format!("/api/docs/{}/agent", doc["id"].as_str().unwrap()),
            Some(&token),
            json!({ "prompt": "write docs" }),
        )
        .await;
    // hickory-agent is wired, but this test config has no ANTHROPIC_API_KEY:
    // the endpoint no-ops with a clear 503 (graceful degradation).
    assert_eq!(status, 503);
    assert_eq!(v["error"], "agent not configured (ANTHROPIC_API_KEY unset)");
}

// ---------------------------------------------------------------------------
// Run commits a woven baseline; check passes against it and fails on drift
// (regression for the "output file missing on disk" first-check failure).
// ---------------------------------------------------------------------------

const WEAVE_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="weave.md">
# Baseline test

<hick:container name="shell" image="alpine:3.20" />

<hick:exec container="shell">
echo hello
<hick:expect match="exact">hello
</hick:expect>
</hick:exec>
</hick:doc>
"#;

#[tokio::test]
async fn run_commits_baseline_then_check_passes_and_drift_fails() {
    let app = setup().await;
    let (token, _) = app.signup("baseline@example.com").await;
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "base", "visibility": "private" }),
        )
        .await;
    let project_id = project["id"].as_str().unwrap();
    let (_, doc) = app
        .post(
            &format!("/api/projects/{project_id}/docs"),
            Some(&token),
            json!({ "path": "weave.hick", "source": WEAVE_DOC }),
        )
        .await;
    let doc_id = doc["id"].as_str().unwrap().to_string();

    let wait = |run_id: String| {
        let app = &app;
        let token = token.clone();
        async move {
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                let (_, r) = app.get(&format!("/api/runs/{run_id}"), Some(&token)).await;
                let status = r["status"].as_str().unwrap_or("").to_string();
                if status == "ok" || status == "failed" {
                    return r;
                }
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "run never finished: {r}"
                );
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    };

    // Run: succeeds and commits the woven baseline into the project repo.
    let (status, v) = app
        .post(&format!("/api/docs/{doc_id}/run"), Some(&token), json!({}))
        .await;
    assert_eq!(status, 202, "{v}");
    let r = wait(v["run_id"].as_str().unwrap().to_string()).await;
    assert_eq!(r["status"], "ok", "{r}");

    // Check now passes against that baseline.
    let (status, v) = app
        .post(
            &format!("/api/docs/{doc_id}/check"),
            Some(&token),
            json!({}),
        )
        .await;
    assert_eq!(status, 202, "{v}");
    let c = wait(v["run_id"].as_str().unwrap().to_string()).await;
    assert_eq!(c["status"], "ok", "{c}");

    // Drift the expectation: check fails and the reason is in the response.
    let drifted = WEAVE_DOC.replace(">hello\n", ">goodbye\n");
    app.put(
        &format!("/api/docs/{doc_id}"),
        Some(&token),
        json!({ "source": drifted }),
    )
    .await;
    let (_, v) = app
        .post(
            &format!("/api/docs/{doc_id}/check"),
            Some(&token),
            json!({}),
        )
        .await;
    let c = wait(v["run_id"].as_str().unwrap().to_string()).await;
    assert_eq!(c["status"], "failed", "{c}");
    let err = c["error"].as_str().unwrap_or("");
    assert!(
        err.contains("expectation failed"),
        "error not surfaced: {c}"
    );
}

// ---------------------------------------------------------------------------
// Generated outputs & lineage (api.md v0.2)
// ---------------------------------------------------------------------------

const LINEAGE_DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Lineage test

<hick:copy id="alpha">fn alpha() {}
</hick:copy>
<hick:copy id="beta">fn beta() {}
</hick:copy>
<hick:file path="gen.rs"><hick:paste select="#alpha" /><hick:paste select="#beta" /></hick:file>
</hick:doc>
"##;

const SEPARATOR_DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Separator lineage test

<hick:copy id="a" class="fns">fn a() {}
</hick:copy>
<hick:copy id="b" class="fns">fn b() {}
</hick:copy>
<hick:file path="all.rs"><hick:paste select=".fns" separator=" // SEP " /></hick:file>
</hick:doc>
"##;

impl TestApp {
    /// Create a project + doc, run it, and wait for the run to finish ok.
    async fn seed_and_run(&self, token: &str, path: &str, source: &str) -> (String, String) {
        let (_, project) = self
            .post(
                "/api/projects",
                Some(token),
                json!({ "name": "lineage", "visibility": "private" }),
            )
            .await;
        let project_id = project["id"].as_str().unwrap().to_string();
        let (_, doc) = self
            .post(
                &format!("/api/projects/{project_id}/docs"),
                Some(token),
                json!({ "path": path, "source": source }),
            )
            .await;
        let doc_id = doc["id"].as_str().unwrap().to_string();
        self.run_and_wait(token, &doc_id).await;
        (project_id, doc_id)
    }

    async fn run_and_wait(&self, token: &str, doc_id: &str) {
        let (status, v) = self
            .post(&format!("/api/docs/{doc_id}/run"), Some(token), json!({}))
            .await;
        assert_eq!(status, 202, "{v}");
        let run_id = v["run_id"].as_str().unwrap().to_string();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let (_, r) = self.get(&format!("/api/runs/{run_id}"), Some(token)).await;
            match r["status"].as_str().unwrap_or("") {
                "ok" => return,
                "failed" => panic!("run failed: {r}"),
                _ => {
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "run never finished: {r}"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            }
        }
    }
}

/// Two hick:copy slots pasted into one hick:file: GET /outputs lists the
/// file, provenance covers the whole content gap-free, and every
/// span-carrying range is byte-identical to the source span it names.
#[tokio::test(flavor = "multi_thread")]
async fn outputs_listing_and_byte_precise_provenance() {
    let app = setup().await;
    let (token, _) = app.signup("lineage@example.com").await;
    let (_, doc_id) = app.seed_and_run(&token, "lineage.hick", LINEAGE_DOC).await;

    // Listing shows the generated file with its detected language.
    let (status, v) = app
        .get(&format!("/api/docs/{doc_id}/outputs"), Some(&token))
        .await;
    assert_eq!(status, 200, "{v}");
    let files = v["files"].as_array().unwrap();
    let gen_file = files
        .iter()
        .find(|f| f["path"] == "gen.rs")
        .expect("gen.rs in outputs listing");
    assert_eq!(gen_file["language"], "rust");

    // File content + provenance.
    let (status, v) = app
        .get(
            &format!("/api/docs/{doc_id}/outputs/file?path=gen.rs"),
            Some(&token),
        )
        .await;
    assert_eq!(status, 200, "{v}");
    assert_eq!(v["path"], "gen.rs");
    assert_eq!(v["language"], "rust");
    let content = v["content"].as_str().unwrap();
    assert_eq!(content, "fn alpha() {}\nfn beta() {}\n");

    let prov = v["provenance"].as_array().unwrap();
    assert!(!prov.is_empty());
    // Gap-free coverage of the whole content.
    let mut covered = 0usize;
    let mut sourced = 0usize;
    for p in prov {
        let start = p["start"].as_u64().unwrap() as usize;
        let end = p["end"].as_u64().unwrap() as usize;
        assert_eq!(start, covered, "provenance must be gap-free: {p}");
        covered = end;
        let origin = &p["origin"];
        if origin["kind"] != "synthetic" {
            sourced += 1;
            assert_eq!(origin["doc_path"], "lineage.hick");
            let s = origin["span"][0].as_u64().unwrap() as usize;
            let e = origin["span"][1].as_u64().unwrap() as usize;
            assert_eq!(
                &LINEAGE_DOC[s..e],
                &content[start..end],
                "provenance range must map to the exact source bytes"
            );
        }
    }
    assert_eq!(covered, content.len());
    assert_eq!(sourced, 2, "both pasted copy blocks carry source spans");

    // Read follows doc visibility: the project is private, anonymous is 403.
    let (status, _) = app.get(&format!("/api/docs/{doc_id}/outputs"), None).await;
    assert_eq!(status, 403);
    let (status, _) = app
        .get(
            &format!("/api/docs/{doc_id}/outputs/file?path=gen.rs"),
            None,
        )
        .await;
    assert_eq!(status, 403);
}

/// The round trip: edit an output range that came from a hick:copy block via
/// POST /outputs/edit → the source doc is updated (DB + git commit) → re-run
/// → the new output equals the edited output byte-for-byte.
#[tokio::test(flavor = "multi_thread")]
async fn output_edit_round_trips_byte_for_byte() {
    let app = setup().await;
    let (token, _) = app.signup("roundtrip@example.com").await;
    let (project_id, doc_id) = app.seed_and_run(&token, "rt.hick", LINEAGE_DOC).await;

    let (_, v) = app
        .get(
            &format!("/api/docs/{doc_id}/outputs/file?path=gen.rs"),
            Some(&token),
        )
        .await;
    let content = v["content"].as_str().unwrap().to_string();
    let start = content.find("alpha").unwrap();
    let end = start + "alpha".len();
    let expected = format!("{}gamma{}", &content[..start], &content[end..]);

    // Owner-only: another signed-in user is rejected.
    let (other_token, _) = app.signup("intruder@example.com").await;
    let (status, _) = app
        .post(
            &format!("/api/docs/{doc_id}/outputs/edit"),
            Some(&other_token),
            json!({ "path": "gen.rs", "edits": [{ "start": start, "end": end, "text": "gamma" }] }),
        )
        .await;
    assert_eq!(status, 403);

    let (status, v) = app
        .post(
            &format!("/api/docs/{doc_id}/outputs/edit"),
            Some(&token),
            json!({ "path": "gen.rs", "edits": [{ "start": start, "end": end, "text": "gamma" }] }),
        )
        .await;
    assert_eq!(status, 200, "{v}");
    assert_eq!(v["applied"], true);
    let source_edits = v["source_edits"].as_array().unwrap();
    assert_eq!(source_edits.len(), 1);
    assert_eq!(source_edits[0]["doc_path"], "rt.hick");
    assert_eq!(source_edits[0]["text"], "gamma");
    let s = source_edits[0]["span"][0].as_u64().unwrap() as usize;
    let e = source_edits[0]["span"][1].as_u64().unwrap() as usize;
    assert_eq!(
        &LINEAGE_DOC[s..e],
        "alpha",
        "source edit targets the copy block bytes"
    );

    // The doc source was updated in the DB…
    let (_, doc) = app.get(&format!("/api/docs/{doc_id}"), Some(&token)).await;
    let new_source = doc["source"].as_str().unwrap().to_string();
    assert!(new_source.contains("fn gamma() {}"), "{new_source}");
    assert!(!new_source.contains("fn alpha() {}"));

    // …and committed to git with the pinned message.
    let project_uuid: uuid::Uuid = project_id.parse().unwrap();
    let repo = app.git.project_dir(project_uuid);
    let log = std::process::Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "log",
            "-1",
            "--format=%s",
            "--",
            "rt.hick",
        ])
        .output()
        .unwrap();
    let subject = String::from_utf8_lossy(&log.stdout);
    assert_eq!(subject.trim(), "lineage edit via gen.rs");

    // Re-run: the next run reproduces the edited output byte-for-byte.
    app.run_and_wait(&token, &doc_id).await;
    let (status, v) = app
        .get(
            &format!("/api/docs/{doc_id}/outputs/file?path=gen.rs"),
            Some(&token),
        )
        .await;
    assert_eq!(status, 200, "{v}");
    assert_eq!(
        v["content"].as_str().unwrap(),
        expected,
        "round trip must be byte-for-byte"
    );
}

/// Edits overlapping a synthetic separator are rejected with 422 and the
/// offending output range.
#[tokio::test(flavor = "multi_thread")]
async fn output_edit_overlapping_synthetic_separator_is_422() {
    let app = setup().await;
    let (token, _) = app.signup("synthetic@example.com").await;
    let (_, doc_id) = app.seed_and_run(&token, "sep.hick", SEPARATOR_DOC).await;

    let (_, v) = app
        .get(
            &format!("/api/docs/{doc_id}/outputs/file?path=all.rs"),
            Some(&token),
        )
        .await;
    let content = v["content"].as_str().unwrap().to_string();
    let sep_start = content.find(" // SEP ").expect("separator in output");
    let sep_end = sep_start + " // SEP ".len();
    // Sanity: the separator range is reported synthetic.
    let prov = v["provenance"].as_array().unwrap();
    assert!(
        prov.iter().any(|p| {
            p["origin"]["kind"] == "synthetic"
                && (p["start"].as_u64().unwrap() as usize) < sep_end
                && (p["end"].as_u64().unwrap() as usize) > sep_start
        }),
        "separator must be synthetic: {prov:?}"
    );

    // An edit reaching into the separator fails with the offending range.
    let (status, v) = app
        .post(
            &format!("/api/docs/{doc_id}/outputs/edit"),
            Some(&token),
            json!({ "path": "all.rs", "edits": [{ "start": sep_start - 2, "end": sep_start + 3, "text": "X" }] }),
        )
        .await;
    assert_eq!(status, 422, "{v}");
    // Pinned 422 body: {error, range: {start, end}} (api.md).
    let range = v["range"]
        .as_object()
        .expect("422 body carries the offending range");
    let r0 = range["start"].as_u64().unwrap() as usize;
    let r1 = range["end"].as_u64().unwrap() as usize;
    assert!(
        r0 >= sep_start && r1 <= sep_end,
        "offending range {r0}..{r1} within separator"
    );
    assert!(v["error"].as_str().unwrap().contains("synthetic"));

    // The doc source was not touched.
    let (_, doc) = app.get(&format!("/api/docs/{doc_id}"), Some(&token)).await;
    assert_eq!(doc["source"].as_str().unwrap(), SEPARATOR_DOC);
}
