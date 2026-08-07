//! Integration tests against a real Postgres (docker compose `db` service).
//! Run with `--test-threads=1` (the workspace convention).

use std::sync::Arc;

use futures::{SinkExt as _, StreamExt as _};
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
    renders: Arc<hickory_server::render_cache::RenderCache>,
}

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

async fn setup() -> TestApp {
    setup_with_agent(None).await
}

/// `setup`, with the agent enabled by an API key. Pair with
/// `ANTHROPIC_BASE_URL` to point the real client at a local endpoint.
async fn setup_with_agent(agent_key: Option<String>) -> TestApp {
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
        agent_llm: agent_key.map(|api_key| hickory_server::AgentLlmConfig {
            provider: "anthropic".to_string(),
            api_key,
            model: None,
        }),
        agent_provider: "anthropic".to_string(),
        signup_allowlist: Vec::new(),
    };

    let db = hickory_server::init_db(&test_url).await.unwrap();
    let state = build_state(config, db.clone()).unwrap();
    let git = state.git.clone();
    let renders = state.renders.clone();
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
        renders,
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
    assert_eq!(
        v["error"],
        "agent not configured (ANTHROPIC_API_KEY unset for provider anthropic)"
    );
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

// ---------------------------------------------------------------------------
// Editor intelligence — LSP bridge (api.md v0.3)
// ---------------------------------------------------------------------------

const LSP_PY_DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# LSP bridge test

<hick:file path="app.py">
def greet():
    return "hi"

greet()
</hick:file>
</hick:doc>
"##;

/// 0-based (line, UTF-16 character) of `needle` + `delta` bytes in an ASCII
/// source.
fn pos_of(source: &str, needle: &str, delta: usize) -> (u32, u32) {
    let offset = source.find(needle).expect("needle present") + delta;
    let before = &source[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32;
    let col = (offset - before.rfind('\n').map(|i| i + 1).unwrap_or(0)) as u32;
    (line, col)
}

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn lsp_send(ws: &mut Ws, msg: &Value) {
    let json = serde_json::to_vec(msg).unwrap();
    let mut frame = Vec::with_capacity(json.len() + 1);
    frame.push(0x02);
    frame.extend_from_slice(&json);
    ws.send(TtMessage::Binary(frame)).await.unwrap();
}

/// Read frames (skipping other channels and unrelated LSP messages) until the
/// response with `id` arrives.
async fn lsp_response(ws: &mut Ws, id: i64, deadline: tokio::time::Instant) -> Value {
    loop {
        let frame = tokio::time::timeout_at(deadline, ws.next())
            .await
            .expect("timed out waiting for LSP response")
            .expect("socket closed early")
            .unwrap();
        let data = match frame {
            TtMessage::Binary(b) => b,
            _ => continue,
        };
        if data.is_empty() || data[0] != 0x02 {
            continue;
        }
        let msg: Value = serde_json::from_slice(&data[1..]).unwrap();
        if msg.get("id").and_then(|i| i.as_i64()) == Some(id) {
            return msg;
        }
    }
}

async fn lsp_ws(app: &TestApp, token: &str, doc_id: &str) -> Ws {
    let ws_url = format!(
        "ws://127.0.0.1:{}/api/ws?doc=doc:{doc_id}&token={token}",
        app.port
    );
    let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await.unwrap();
    ws
}

fn lsp_did_open(path: &str, source: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": { "textDocument": {
            "uri": format!("hick:///{path}"),
            "languageId": "hick",
            "version": 1,
            "text": source,
        }},
    })
}

/// Hover/definition inside a `hick:file` python block: with pyright absent the
/// bridge degrades to structural/no-result answers — a `result` (possibly
/// null), NEVER an error frame or a dead channel.
#[tokio::test(flavor = "multi_thread")]
async fn ws_lsp_channel_degrades_gracefully_without_child_servers() {
    let app = setup().await;
    let (token, _) = app.signup("lsp@example.com").await;
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "lsp", "visibility": "public" }),
        )
        .await;
    let project_id = project["id"].as_str().unwrap();
    let (_, doc) = app
        .post(
            &format!("/api/projects/{project_id}/docs"),
            Some(&token),
            json!({ "path": "lsp.hick", "source": LSP_PY_DOC }),
        )
        .await;
    let doc_id = doc["id"].as_str().unwrap().to_string();

    let mut ws = lsp_ws(&app, &token, &doc_id).await;
    lsp_send(&mut ws, &lsp_did_open("lsp.hick", LSP_PY_DOC)).await;

    let (line, character) = pos_of(LSP_PY_DOC, "def greet", 4);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    for (id, method) in [(1i64, "textDocument/hover"), (2, "textDocument/definition")] {
        lsp_send(
            &mut ws,
            &json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": {
                    "textDocument": { "uri": "hick:///lsp.hick" },
                    "position": { "line": line, "character": character },
                },
            }),
        )
        .await;
        let resp = lsp_response(&mut ws, id, deadline).await;
        assert!(
            resp.get("error").is_none(),
            "{method} must not error the channel: {resp}"
        );
        assert!(
            resp.as_object().unwrap().contains_key("result"),
            "{method} must answer with a result: {resp}"
        );
    }
    ws.close(None).await.ok();
}

/// References round-trip where the definition target lives in the same
/// virtual file: at the `alpha` copy block, references resolve to both its
/// paste sites (structural answers — no child server required), translated to
/// `hick:///` URIs with .hick coordinates.
#[tokio::test(flavor = "multi_thread")]
async fn ws_lsp_references_round_trip_in_same_virtual_file() {
    let app = setup().await;
    let (token, _) = app.signup("lsp-refs@example.com").await;
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "lsp-refs", "visibility": "public" }),
        )
        .await;
    let project_id = project["id"].as_str().unwrap();
    let (_, doc) = app
        .post(
            &format!("/api/projects/{project_id}/docs"),
            Some(&token),
            json!({ "path": "lineage.hick", "source": LINEAGE_DOC }),
        )
        .await;
    let doc_id = doc["id"].as_str().unwrap().to_string();

    let mut ws = lsp_ws(&app, &token, &doc_id).await;
    lsp_send(&mut ws, &lsp_did_open("lineage.hick", LINEAGE_DOC)).await;

    // Cursor on the copy definition tag. didOpen is a notification, so allow
    // the request to land before processing finishes: retry until non-empty.
    let (line, character) = pos_of(LINEAGE_DOC, r#"<hick:copy id="alpha">"#, 12);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut refs: Vec<Value> = Vec::new();
    for id in 10i64.. {
        lsp_send(
            &mut ws,
            &json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "textDocument/references",
                "params": {
                    "textDocument": { "uri": "hick:///lineage.hick" },
                    "position": { "line": line, "character": character },
                    "context": { "includeDeclaration": true },
                },
            }),
        )
        .await;
        let resp = lsp_response(&mut ws, id, deadline).await;
        assert!(resp.get("error").is_none(), "references errored: {resp}");
        if let Some(items) = resp["result"].as_array()
            && !items.is_empty()
        {
            refs = items.clone();
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert!(
            tokio::time::Instant::now() < deadline,
            "references never returned results"
        );
    }
    ws.close(None).await.ok();

    // Declaration + the alpha paste site, all in .hick coordinates.
    assert!(
        refs.len() >= 2,
        "expected declaration + paste site: {refs:?}"
    );
    for r in &refs {
        assert_eq!(r["uri"], "hick:///lineage.hick", "{r}");
    }
    let (paste_line, paste_col) = pos_of(LINEAGE_DOC, r##"<hick:paste select="#alpha" />"##, 0);
    assert!(
        refs.iter().any(|r| {
            r["range"]["start"]["line"].as_u64() == Some(paste_line as u64)
                && r["range"]["start"]["character"].as_u64() == Some(paste_col as u64)
        }),
        "expected a reference at the paste site {paste_line}:{paste_col}: {refs:?}"
    );
}

/// Output-view navigation: an offset inside the woven `alpha` bytes of gen.rs
/// maps through provenance to the source doc, and definition lands back on
/// the `<hick:copy id="alpha">` block as a `hick:///` byte range.
#[tokio::test(flavor = "multi_thread")]
async fn outputs_nav_definition_maps_back_to_copy_block() {
    let app = setup().await;
    let (token, _) = app.signup("lsp-nav@example.com").await;
    let (_, doc_id) = app.seed_and_run(&token, "lineage.hick", LINEAGE_DOC).await;

    // Byte 3 of "fn alpha() {}\n…" is inside the pasted alpha bytes.
    let (status, v) = app
        .post(
            &format!("/api/docs/{doc_id}/outputs/nav"),
            Some(&token),
            json!({ "path": "gen.rs", "offset": 3, "kind": "definition" }),
        )
        .await;
    assert_eq!(status, 200, "{v}");
    let targets = v["targets"].as_array().unwrap();
    assert!(!targets.is_empty(), "expected a definition target: {v}");
    let t = &targets[0];
    assert_eq!(t["uri"], "hick:///lineage.hick", "{t}");
    let start = t["range"]["start"].as_u64().unwrap() as usize;
    let end = t["range"]["end"].as_u64().unwrap() as usize;
    assert!(
        LINEAGE_DOC[start..end].starts_with(r#"<hick:copy id="alpha""#),
        "definition should land on the copy block, got: {:?}",
        &LINEAGE_DOC[start..end]
    );

    // Same owner/visibility rules as other doc reads: the project is private,
    // anonymous access is rejected.
    let (status, _) = app
        .post(
            &format!("/api/docs/{doc_id}/outputs/nav"),
            None,
            json!({ "path": "gen.rs", "offset": 3, "kind": "definition" }),
        )
        .await;
    assert_ne!(status, 200);

    // Unknown kinds are a 400, not a crash.
    let (status, _) = app
        .post(
            &format!("/api/docs/{doc_id}/outputs/nav"),
            Some(&token),
            json!({ "path": "gen.rs", "offset": 3, "kind": "rename" }),
        )
        .await;
    assert_eq!(status, 400);
}

// ---------------------------------------------------------------------------
// Reconnecting must not duplicate the document (migration 0003).
// ---------------------------------------------------------------------------

/// One client-side Yjs sync round trip over the framed socket: reply to the
/// server's sync-step-1, ask for its state, and apply what comes back.
async fn yjs_sync(ws: &mut Ws, ydoc: &yrs::Doc) {
    use yrs::sync::{Message, SyncMessage};
    use yrs::updates::decoder::Decode as _;
    use yrs::updates::encoder::Encode as _;
    use yrs::{ReadTxn as _, Transact as _};

    let frame = |m: &Message| {
        let mut f = vec![0x00u8];
        f.extend_from_slice(&m.encode_v1());
        TtMessage::Binary(f)
    };

    // Announce our state so the server sends only what we are missing.
    let sv = ydoc.transact().state_vector();
    ws.send(frame(&Message::Sync(SyncMessage::SyncStep1(sv))))
        .await
        .unwrap();

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut got_step2 = false;
    while !got_step2 {
        let Ok(Some(Ok(msg))) = tokio::time::timeout_at(deadline, ws.next()).await else {
            panic!("timed out waiting for the server's sync step 2");
        };
        let TtMessage::Binary(bytes) = msg else {
            continue;
        };
        if bytes.first() != Some(&0x00) {
            continue;
        }
        let Ok(message) = Message::decode_v1(&bytes[1..]) else {
            continue;
        };
        match message {
            Message::Sync(SyncMessage::SyncStep1(their_sv)) => {
                // Send everything they are missing from us.
                let update = ydoc.transact().encode_state_as_update_v1(&their_sv);
                ws.send(frame(&Message::Sync(SyncMessage::SyncStep2(update))))
                    .await
                    .unwrap();
            }
            Message::Sync(SyncMessage::SyncStep2(update))
            | Message::Sync(SyncMessage::Update(update)) => {
                let update = yrs::Update::decode_v1(&update).unwrap();
                ydoc.transact_mut().apply_update(update).unwrap();
                got_step2 = true;
            }
            _ => {}
        }
    }
}

/// Reconnecting a client whose Y.Doc already holds the document must not
/// append a second copy of it.
///
/// Rooms are created on the first socket and dropped when the last one
/// leaves. Re-creating a room used to mint a *new* insert of `docs.source`
/// under a fresh client id, which the CRDT merged with the copy the client
/// still held — concatenating them. In the dev stack this ran away to a
/// 49 MB document (3268 copies of a 15 KB file), which is what made
/// `GET /api/docs/:id/render` take 7.8 s.
#[tokio::test(flavor = "multi_thread")]
async fn reconnecting_does_not_duplicate_the_document() {
    use yrs::{GetString as _, Transact as _};

    let app = setup().await;
    let (token, _) = app.signup("crdt@example.com").await;
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "crdt", "visibility": "private" }),
        )
        .await;
    let (_, doc) = app
        .post(
            &format!("/api/projects/{}/docs", project["id"].as_str().unwrap()),
            Some(&token),
            json!({ "path": "collab.hick", "source": TEST_DOC }),
        )
        .await;
    let doc_id = doc["id"].as_str().unwrap().to_string();

    // One long-lived client Y.Doc across several connections, like a browser
    // tab that reconnects (or a page that keeps its doc across a WS drop).
    let ydoc = yrs::Doc::new();
    let text = ydoc.get_or_insert_text("source");

    for round in 0..4 {
        let mut ws = lsp_ws(&app, &token, &doc_id).await;
        yjs_sync(&mut ws, &ydoc).await;
        let content = text.get_string(&ydoc.transact());
        assert_eq!(
            content,
            TEST_DOC,
            "round {round}: the document gained a duplicate copy \
             ({} bytes vs {} expected)",
            content.len(),
            TEST_DOC.len()
        );
        ws.close(None).await.unwrap();
        // Let the socket close, the room drain and the debounced persist run.
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    // Whatever was persisted must still be exactly one copy.
    let stored: String = sqlx::query_scalar("SELECT source FROM docs WHERE id = $1")
        .bind(doc_id.parse::<uuid::Uuid>().unwrap())
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(stored, TEST_DOC, "persisted source picked up a duplicate");
}

// ---------------------------------------------------------------------------
// Render performance: the weave is cached, and it never blocks the runtime.
// ---------------------------------------------------------------------------

/// A document big enough that weaving it is measurably expensive.
fn big_doc(tag: &str, paragraphs: usize) -> String {
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n",
    );
    for i in 0..paragraphs {
        s.push_str(&format!(
            "Paragraph {i} of the {tag} document, long enough to cost real work to weave.\n"
        ));
    }
    s.push_str("</hick:doc>\n");
    s
}

/// Renders are served from the in-process cache when nothing they depend on
/// changed, and recomputed as soon as the source moves. Regression guard for
/// re-weaving the same document on every save / run event / page load.
#[tokio::test(flavor = "multi_thread")]
async fn render_is_cached_and_recomputed_when_the_source_changes() {
    let app = setup().await;
    let (token, _) = app.signup("rendercache@example.com").await;
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "cache", "visibility": "private" }),
        )
        .await;
    let (_, doc) = app
        .post(
            &format!("/api/projects/{}/docs", project["id"].as_str().unwrap()),
            Some(&token),
            json!({ "path": "cached.hick", "source": big_doc("first", 20_000) }),
        )
        .await;
    let doc_id = doc["id"].as_str().unwrap().to_string();

    let cold = std::time::Instant::now();
    let (status, first) = app
        .get(&format!("/api/docs/{doc_id}/render"), Some(&token))
        .await;
    let cold = cold.elapsed();
    assert_eq!(status, 200, "{first}");
    assert_eq!(
        app.renders.stats(),
        (0, 1),
        "the first render must miss the cache"
    );

    let warm = std::time::Instant::now();
    let (status, second) = app
        .get(&format!("/api/docs/{doc_id}/render"), Some(&token))
        .await;
    let warm = warm.elapsed();
    assert_eq!(status, 200);
    assert_eq!(second, first, "a cache hit must be byte-identical");
    assert_eq!(app.renders.stats().0, 1, "the second render must hit");
    // The guarantee is "the second render does not re-weave", which the hit
    // count above already proves. Wall-clock is a weak signal here: runs now
    // weave once up front to stage their files, so by the time this test asks
    // for a render the process is warm and the "cold" path can itself be
    // sub-millisecond. Compare times only when the cold render was slow enough
    // for the comparison to mean anything.
    if cold > std::time::Duration::from_millis(20) {
        assert!(
            warm < cold,
            "warm render ({warm:?}) should be cheaper than cold ({cold:?})"
        );
    }

    // A save changes the source hash: the cache must not serve the old weave.
    let (status, _) = app
        .put(
            &format!("/api/docs/{doc_id}"),
            Some(&token),
            json!({ "source": big_doc("second", 20_000) }),
        )
        .await;
    assert_eq!(status, 200);
    let (status, third) = app
        .get(&format!("/api/docs/{doc_id}/render"), Some(&token))
        .await;
    assert_eq!(status, 200);
    assert_ne!(third, first, "the render must follow the edited source");
    assert!(
        serde_json::to_string(&third)
            .unwrap()
            .contains("second document"),
        "the render should reflect the new source"
    );
    assert_eq!(app.renders.stats().1, 2, "the edited source must miss");
}

/// The weave is CPU-bound and the checkout seed is synchronous filesystem
/// work; both must run on the blocking pool. Pinned to a single runtime
/// worker so that doing either on the runtime would visibly stall the
/// server — which is exactly what happened in the dev stack: under render
/// load `GET /api/health` stopped answering at all.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn health_stays_responsive_while_renders_are_in_flight() {
    let app = setup().await;
    let (token, _) = app.signup("nonblocking@example.com").await;
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "load", "visibility": "private" }),
        )
        .await;
    let project_id = project["id"].as_str().unwrap().to_string();

    // Distinct documents so every render is a genuine cache miss. The docs
    // themselves are small — the expensive part under test is the per-render
    // checkout seed, which copies the whole project tree.
    let mut doc_ids = Vec::new();
    for i in 0..8 {
        let (_, doc) = app
            .post(
                &format!("/api/projects/{project_id}/docs"),
                Some(&token),
                json!({
                    "path": format!("load-{i}.hick"),
                    "source": big_doc(&format!("load {i}"), 200),
                }),
            )
            .await;
        doc_ids.push(doc["id"].as_str().unwrap().to_string());
    }

    // A heavy (but not declared-output) file in the project checkout, as an
    // exec's incidental artifact would be: every render copies it.
    let project_dir = app
        .git
        .project_dir(project_id.parse::<uuid::Uuid>().unwrap());
    std::fs::write(
        project_dir.join("artifact.bin"),
        vec![7u8; 48 * 1024 * 1024],
    )
    .unwrap();

    let base = app.base.clone();
    let renders: Vec<_> = doc_ids
        .into_iter()
        .map(|id| {
            let client = app.client.clone();
            let url = format!("{base}/api/docs/{id}/render");
            let token = token.clone();
            tokio::spawn(async move { client.get(url).bearer_auth(token).send().await })
        })
        .collect();

    // Let the renders reach the server before probing.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // While those are in flight, health must keep answering promptly.
    let mut worst = std::time::Duration::ZERO;
    for _ in 0..10 {
        let t = std::time::Instant::now();
        let (status, v) = app.get("/api/health", None).await;
        worst = worst.max(t.elapsed());
        assert_eq!(status, 200, "{v}");
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(
        worst < std::time::Duration::from_millis(500),
        "GET /api/health took {worst:?} while renders were in flight — \
         CPU-bound work is back on the async runtime"
    );

    for r in renders {
        assert_eq!(r.await.unwrap().unwrap().status().as_u16(), 200);
    }
}

/// The agent conversation is a TREE: rewinding to an earlier turn and sending
/// again must fork a branch, leaving what followed that turn intact on its own
/// branch. This drives the storage and the read model directly (running a real
/// agent turn needs an API key, which CI does not have).
#[tokio::test(flavor = "multi_thread")]
async fn agent_turns_form_a_branching_tree() {
    let app = setup().await;
    let (token, user_id) = app.signup("branches@example.com").await;
    let user_id: uuid::Uuid = user_id.parse().unwrap();
    let (_, project) = app
        .post(
            "/api/projects",
            Some(&token),
            json!({ "name": "chat", "visibility": "private" }),
        )
        .await;
    let project_id = project["id"].as_str().unwrap();
    let (_, doc) = app
        .post(
            &format!("/api/projects/{project_id}/docs"),
            Some(&token),
            json!({ "path": "chat.hick", "source": "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">hi</hick:doc>" }),
        )
        .await;
    let doc_id: uuid::Uuid = doc["id"].as_str().unwrap().parse().unwrap();

    let mut ids = Vec::new();
    // root → a → a2, and root → b (the fork produced by rewinding to root).
    for (prompt, parent) in [
        ("root", None),
        ("a", Some(0usize)),
        ("a2", Some(1usize)),
        ("b", Some(0usize)),
    ] {
        let id = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO agent_turns (id, doc_id, user_id, parent_id, prompt, answer, status)
             VALUES ($1, $2, $3, $4, $5, $6, 'ok')",
        )
        .bind(id)
        .bind(doc_id)
        .bind(user_id)
        .bind(parent.map(|i| ids[i]))
        .bind(prompt)
        .bind(format!("answer to {prompt}"))
        .execute(&app.db)
        .await
        .unwrap();
        ids.push(id);
    }

    let (status, v) = app
        .get(&format!("/api/docs/{doc_id}/agent/turns"), Some(&token))
        .await;
    assert_eq!(status, 200, "{v}");
    let turns = v["turns"].as_array().unwrap();
    assert_eq!(turns.len(), 4, "every branch is kept, none overwritten");

    // The fork: two distinct children of root, so the rewind did not destroy
    // the turns that followed it.
    let root = ids[0].to_string();
    let children: Vec<&str> = turns
        .iter()
        .filter(|t| t["parent_id"].as_str() == Some(root.as_str()))
        .map(|t| t["prompt"].as_str().unwrap())
        .collect();
    assert_eq!(children, vec!["a", "b"]);

    // Ordering is chronological, which is what the client walks.
    let prompts: Vec<&str> = turns
        .iter()
        .map(|t| t["prompt"].as_str().unwrap())
        .collect();
    assert_eq!(prompts, vec!["root", "a", "a2", "b"]);

    // Another user cannot read this conversation.
    let (other, _) = app.signup("nosy@example.com").await;
    let (status, _) = app
        .get(&format!("/api/docs/{doc_id}/agent/turns"), Some(&other))
        .await;
    assert_eq!(status, 403);
}

// ---------------------------------------------------------------------------
// Agent conversations, driven through the REAL Anthropic client against a
// local endpoint that speaks the Messages API and records what it receives.
//
// Nothing about the agent path is mocked: the same `AnthropicClient`, the same
// SSE parsing, the same ReAct loop and tool execution. Only the far side of the
// socket is ours — which is the point, because the questions worth answering
// are "what conversation history does the server actually send?" and "does the
// agent's work survive the temp workspace?".
// ---------------------------------------------------------------------------

/// A local stand-in for `POST /v1/messages`: replays scripted assistant turns
/// as SSE and records every request body.
struct FakeAnthropic {
    url: String,
    requests: Arc<std::sync::Mutex<Vec<Value>>>,
    replies: Arc<std::sync::Mutex<std::collections::VecDeque<String>>>,
}

async fn fake_anthropic(replies: Vec<String>) -> FakeAnthropic {
    use axum::extract::State as AxState;
    use axum::routing::post;

    let requests: Arc<std::sync::Mutex<Vec<Value>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let queue: Arc<std::sync::Mutex<std::collections::VecDeque<String>>> =
        Arc::new(std::sync::Mutex::new(replies.into_iter().collect()));

    type St = (
        Arc<std::sync::Mutex<Vec<Value>>>,
        Arc<std::sync::Mutex<std::collections::VecDeque<String>>>,
    );

    async fn handle(
        AxState((reqs, queue)): AxState<St>,
        axum::Json(body): axum::Json<Value>,
    ) -> String {
        reqs.lock().unwrap().push(body);
        let reply = queue
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| "<hick:next>done</hick:next>\nout of script".to_string());
        // Minimal but real SSE: the client needs message_start (usage),
        // content_block_delta (text), message_delta (stop reason) and stop.
        format!(
            "event: message_start\ndata: {}\n\n\
             event: content_block_delta\ndata: {}\n\n\
             event: message_delta\ndata: {}\n\n\
             event: message_stop\ndata: {{}}\n\n",
            json!({"message": {"usage": {"input_tokens": 10, "output_tokens": 0}}}),
            json!({"delta": {"text": reply}}),
            json!({"delta": {"stop_reason": "end_turn"}, "usage": {"output_tokens": 5}}),
        )
    }

    let app = axum::Router::new()
        .route("/v1/messages", post(handle))
        .with_state((requests.clone(), queue.clone()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });
    FakeAnthropic {
        url: format!("http://127.0.0.1:{port}/v1/messages"),
        requests,
        replies: queue,
    }
}

/// Flatten a recorded request's messages into `role: text` pairs.
fn message_texts(request: &Value) -> Vec<(String, String)> {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            let role = m["role"].as_str().unwrap_or("").to_string();
            let text = match &m["content"] {
                Value::String(s) => s.clone(),
                Value::Array(parts) => parts
                    .iter()
                    .filter_map(|p| p["text"].as_str())
                    .collect::<Vec<_>>()
                    .join(""),
                _ => String::new(),
            };
            (role, text)
        })
        .collect()
}

const CONV_DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Conversation

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

#[tokio::test(flavor = "multi_thread")]
async fn agent_replays_branch_history_and_writes_its_edits_back() {
    let done = |answer: &str| format!("<hick:next>done</hick:next>\n{answer}");
    // The last turn edits the document through the generated output, anchored
    // by the same 4-hex content hash the tools themselves report.
    let anchor = hickory_agent::hashline::line_hash("fn greet() { println!(\"hello\"); }");
    let fake = fake_anthropic(vec![
        done("greet.rs prints hello."),
        done("Renamed it."),
        done("Added a comment instead."),
        format!(
            "<hick:next>tool</hick:next>\n\
             <hick:tool name=\"edit_output\">\n\
             <hick:arg name=\"path\">greet.rs</hick:arg>\n\
             <hick:arg name=\"run\">{anchor}</hick:arg>\n\
             <hick:input>fn greet() {{ println!(\"hello, world\"); }}</hick:input>\n\
             </hick:tool>"
        ),
        done("Changed the greeting through the output."),
    ])
    .await;

    // The real client reads its endpoint from the environment. The suite runs
    // single-threaded (see the justfile), so this cannot race another test.
    unsafe { std::env::set_var("ANTHROPIC_BASE_URL", &fake.url) };

    let app = setup_with_agent(Some("local-harness-key".into())).await;
    let (token, _) = app.signup("conversation@example.com").await;
    let (_, doc_id) = app.seed_and_run(&token, "conv.hick", CONV_DOC).await;

    let say = async |prompt: &str, parent: Option<&str>| -> Value {
        let (status, v) = app
            .post(
                &format!("/api/docs/{doc_id}/agent"),
                Some(&token),
                json!({ "prompt": prompt, "parent_id": parent }),
            )
            .await;
        assert_eq!(status, 202, "{v}");
        let id = v["session_id"].as_str().unwrap().to_string();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(60);
        loop {
            let (_, t) = app
                .get(&format!("/api/docs/{doc_id}/agent/turns"), Some(&token))
                .await;
            let turn = t["turns"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id.as_str())
                .cloned();
            if let Some(turn) = turn
                && turn["status"] != "running"
            {
                return turn;
            }
            assert!(tokio::time::Instant::now() < deadline, "agent turn hung");
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
    };

    // Turn 1 (root) — the answer is persisted, not just streamed.
    let t1 = say("What does greet.rs contain?", None).await;
    assert_eq!(t1["status"], "ok", "{t1}");
    assert_eq!(t1["answer"], "greet.rs prints hello.");
    let root = t1["id"].as_str().unwrap().to_string();

    // Turn 2 continues turn 1: the model must SEE that exchange as history.
    let t2 = say("Now rename it to hello().", Some(&root)).await;
    assert_eq!(t2["status"], "ok", "{t2}");
    let second = message_texts(fake.requests.lock().unwrap().last().unwrap());
    assert_eq!(
        second.iter().map(|(r, _)| r.as_str()).collect::<Vec<_>>(),
        vec!["user", "assistant", "user"],
        "turn 2 must replay the earlier exchange before the new prompt"
    );
    assert!(second[0].1.contains("What does greet.rs contain?"));
    assert!(second[1].1.contains("greet.rs prints hello."));
    assert!(second[2].1.contains("Now rename it to hello()."));

    // Turn 3 REWINDS to turn 1 and forks. It must carry turn 1's exchange and
    // NOT turn 2's — that is the whole point of branching rather than editing.
    let t3 = say("Actually, add a doc comment instead.", Some(&root)).await;
    assert_eq!(t3["status"], "ok", "{t3}");
    let third = message_texts(fake.requests.lock().unwrap().last().unwrap());
    let third_text = third
        .iter()
        .map(|(_, t)| t.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(third_text.contains("What does greet.rs contain?"));
    assert!(
        !third_text.contains("Now rename it to hello()."),
        "the forked branch leaked the turn it was rewound past:\n{third_text}"
    );

    // Both branches survive.
    let (_, turns) = app
        .get(&format!("/api/docs/{doc_id}/agent/turns"), Some(&token))
        .await;
    let children: Vec<&str> = turns["turns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| t["parent_id"] == root.as_str())
        .map(|t| t["prompt"].as_str().unwrap())
        .collect();
    assert_eq!(
        children.len(),
        2,
        "the rewind destroyed a branch: {children:?}"
    );

    // Finally: an edit the agent makes must OUTLIVE its temp workspace. The
    // agent works in a throwaway checkout, so without a write-back the tool
    // truthfully reports success and the user sees nothing change.
    let t4 = say("Make the greeting friendlier.", None).await;
    assert_eq!(t4["status"], "ok", "{t4}");
    let (_, doc) = app.get(&format!("/api/docs/{doc_id}"), Some(&token)).await;
    assert!(
        doc["source"].as_str().unwrap().contains("hello, world"),
        "the agent's document edit was dropped with the temp workspace"
    );

    assert!(
        fake.replies.lock().unwrap().is_empty(),
        "not every scripted reply was consumed"
    );
    unsafe { std::env::remove_var("ANTHROPIC_BASE_URL") };
}

/// Signup is gated when SIGNUP_ALLOWLIST is set.
///
/// Guarantee: docs/guarantees/collaboration/signups-can-be-closed.md
///
/// A public deployment with open signup, unverified emails, and container
/// execution is a standing invitation to anyone scanning for one. Until email
/// verification exists this is the gate.
#[tokio::test]
async fn the_signup_allowlist_admits_exactly_who_it_should() {
    use hickory_server::parse_allowlist;

    let list = parse_allowlist(Some(" Nate@Example.com , @loum.dev "));
    assert_eq!(list, vec!["nate@example.com", "@loum.dev"]);

    // Exact address, case-insensitively (the handler lowercases first).
    assert!(hickory_server::signup_allowed("nate@example.com", &list));
    // Whole domain.
    assert!(hickory_server::signup_allowed("anyone@loum.dev", &list));
    // A near miss on the domain must not pass — the check is on the part
    // after the LAST '@', so a lookalike address cannot smuggle it in.
    assert!(!hickory_server::signup_allowed(
        "nate@example.com.evil.dev",
        &list
    ));
    assert!(!hickory_server::signup_allowed(
        "someone@example.com",
        &list
    ));
    assert!(!hickory_server::signup_allowed("nate@loum.dev.evil", &list));

    // Unset means open, so existing deployments are not locked out.
    let open = parse_allowlist(None);
    assert!(open.is_empty());
    assert!(hickory_server::signup_allowed(
        "anybody@anywhere.test",
        &open
    ));
    assert!(hickory_server::signup_allowed(
        "x@y.z",
        &parse_allowlist(Some("  "))
    ));
}
