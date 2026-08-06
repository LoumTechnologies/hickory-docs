//! E3 — caching regression tests with teeth
//! (docs/specs/freeform/token-economics.md).
//!
//! Offline tests exercise the REAL request-building path
//! ([`AnthropicClient::request_body_bytes`]) and the usage plumbing through
//! the ReAct loop with a scripted client. The live test (network,
//! `#[ignore]`, gated on `HICKORY_AGENT_LIVE=1` + `ANTHROPIC_API_KEY`)
//! asserts `cache_read_input_tokens > 0` on turn ≥ 2 against the real API.

use hickory_agent::{
    AgentConfig, AgentEvent, AnthropicClient, LlmClient, Message, Role, ScriptedLlmClient, Usage,
    run_agent,
};
use hickory_executor::LocalExecutor;

fn conversation() -> Vec<Message> {
    vec![
        Message::new(Role::System, hickory_agent::SYSTEM_PROMPT),
        Message::new(Role::System, "## Document under discussion\n\nA doc."),
        Message::new(Role::User, "please rename the function"),
        Message::new(Role::Assistant, "<hick:next>done</hick:next>\nRenamed."),
        Message::new(Role::User, "thanks, now the constant"),
    ]
}

/// Same tools + system + history across two separately constructed
/// sessions must serialize to identical bytes, or the prefix cache is
/// silently dead. This runs the REAL request builder.
#[test]
fn prefix_is_byte_stable_across_sessions() {
    let session_a = AnthropicClient::new().with_api_key("k").with_max_tokens(64);
    let session_b = AnthropicClient::new().with_api_key("k").with_max_tokens(64);
    let bytes_a = session_a.request_body_bytes(&conversation(), true);
    let bytes_b = session_b.request_body_bytes(&conversation(), true);
    assert_eq!(bytes_a, bytes_b, "request bytes differ across sessions");
}

/// A timestamp (or any per-session interpolation) injected into the system
/// prompt MUST show up as a byte difference — this is the tripwire the
/// stability assertion relies on.
#[test]
fn injected_timestamp_is_caught() {
    let client = AnthropicClient::new().with_api_key("k");
    let clean = client.request_body_bytes(&conversation(), true);

    let mut poisoned = conversation();
    poisoned[0] = Message::new(
        Role::System,
        format!(
            "{}\nGenerated at: {}",
            hickory_agent::SYSTEM_PROMPT,
            chrono::Utc::now().to_rfc3339()
        ),
    );
    let dirty = client.request_body_bytes(&poisoned, true);
    assert_ne!(
        clean, dirty,
        "a timestamp in the system prompt went undetected"
    );
}

/// Growing the conversation must not rewrite earlier bytes: the previous
/// request's serialized prefix (minus its rolling cache_control marker)
/// must be a literal prefix-by-content of the longer request.
#[test]
fn earlier_turns_are_immutable_as_history_grows() {
    let client = AnthropicClient::new().with_api_key("k");
    let shorter = conversation();
    let mut longer = conversation();
    longer.push(Message::new(
        Role::Assistant,
        "<hick:next>done</hick:next>\nOk.",
    ));
    longer.push(Message::new(Role::User, "one more thing"));

    let a: serde_json::Value =
        serde_json::from_slice(&client.request_body_bytes(&shorter, true)).unwrap();
    let b: serde_json::Value =
        serde_json::from_slice(&client.request_body_bytes(&longer, true)).unwrap();

    // System is identical.
    assert_eq!(a["system"], b["system"]);
    // Every message of the shorter request appears unchanged in the longer
    // one, ignoring cache_control (the rolling breakpoint moves by design).
    let strip = |m: &serde_json::Value| {
        let mut m = m.clone();
        if let Some(content) = m["content"].as_array_mut() {
            for block in content {
                block.as_object_mut().unwrap().remove("cache_control");
            }
        }
        m
    };
    let msgs_a: Vec<_> = a["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(strip)
        .collect();
    let msgs_b: Vec<_> = b["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(strip)
        .collect();
    assert_eq!(&msgs_b[..msgs_a.len()], &msgs_a[..]);
}

/// The request never carries params that 400 on claude-sonnet-5, and the
/// breakpoint budget (max 4) holds even on long histories.
#[test]
fn request_respects_sonnet5_constraints_and_breakpoint_budget() {
    let client = AnthropicClient::new().with_api_key("k");
    let mut messages = conversation();
    for i in 0..40 {
        let role = if i % 2 == 0 {
            Role::Assistant
        } else {
            Role::User
        };
        messages.push(Message::new(role, format!("turn {i}")));
    }
    let v: serde_json::Value =
        serde_json::from_slice(&client.request_body_bytes(&messages, true)).unwrap();

    for forbidden in ["temperature", "top_p", "top_k", "thinking", "budget_tokens"] {
        assert!(v.get(forbidden).is_none(), "{forbidden} must never be sent");
    }

    let mut breakpoints = 0;
    for block in v["system"].as_array().unwrap() {
        if block.get("cache_control").is_some() {
            breakpoints += 1;
        }
    }
    for msg in v["messages"].as_array().unwrap() {
        for block in msg["content"].as_array().unwrap() {
            if block.get("cache_control").is_some() {
                breakpoints += 1;
            }
        }
    }
    assert!(breakpoints >= 2, "prefix + rolling breakpoints missing");
    assert!(
        breakpoints <= 4,
        "over the 4-breakpoint maximum: {breakpoints}"
    );

    // The rolling breakpoint sits on the last content block.
    let last = v["messages"].as_array().unwrap().last().unwrap();
    assert!(
        last["content"].as_array().unwrap().last().unwrap()["cache_control"].is_object(),
        "rolling breakpoint missing from the latest turn"
    );
}

/// Offline plumbing test: cache_read tokens reported by the LLM client on
/// turn ≥ 2 must reach [`AgentEvent::TurnUsage`], the session totals, and
/// the session log — if any link drops them, E3's live assertion could
/// never be trusted.
#[tokio::test(flavor = "multi_thread")]
async fn cache_read_usage_propagates_through_the_loop() {
    let turn1 = Usage {
        input_tokens: 40,
        cache_creation_input_tokens: 1500,
        cache_read_input_tokens: 0,
        output_tokens: 30,
    };
    let turn2 = Usage {
        input_tokens: 25,
        cache_creation_input_tokens: 60,
        cache_read_input_tokens: 1500,
        output_tokens: 12,
    };
    let llm = ScriptedLlmClient::with_usages([
        (
            "<hick:next>code</hick:next>\n```sh\necho hi\n```".to_string(),
            turn1,
        ),
        ("<hick:next>done</hick:next>\nAll done.".to_string(), turn2),
    ])
    .with_model_name("claude-sonnet-5");

    let dir = tempfile::tempdir().unwrap();
    let executor: std::sync::Arc<dyn hickory_executor::Executor> =
        std::sync::Arc::new(LocalExecutor::new().unwrap());
    let config = AgentConfig::new("say hi", dir.path());

    let mut turn_usages: Vec<(usize, Usage, Option<f64>)> = Vec::new();
    let outcome = run_agent(&llm, executor, &config, &mut |e: AgentEvent| {
        if let AgentEvent::TurnUsage {
            turn,
            usage,
            cost_usd,
            ..
        } = e
        {
            turn_usages.push((turn, usage, cost_usd));
        }
    })
    .await
    .expect("scripted run failed");

    assert_eq!(turn_usages.len(), 2);
    assert_eq!(turn_usages[0].1, turn1);
    assert_eq!(turn_usages[1].1, turn2);
    assert!(
        turn_usages[1].1.cache_read_input_tokens > 0,
        "turn 2 must report a cache read"
    );
    // Cost is computed from the sonnet-5 price table with multipliers.
    assert!(turn_usages[1].2.unwrap() > 0.0);

    let mut expected_total = turn1;
    expected_total.add(&turn2);
    assert_eq!(outcome.total_usage, expected_total);
    let expected_cost = hickory_agent::cost_usd("claude-sonnet-5", &expected_total).unwrap();
    assert!((outcome.total_cost_usd.unwrap() - expected_cost).abs() < 1e-12);

    // The session log carries per-turn usage and the session total.
    let source = std::fs::read_to_string(&outcome.session_path).unwrap();
    assert!(source.contains(r#"<hick:usage turn="0""#), "{source}");
    assert!(source.contains(r#"<hick:usage turn="1""#), "{source}");
    assert!(source.contains(r#"cache-read="1500""#), "{source}");
    assert!(
        source.contains(r#"<hick:usage scope="session""#),
        "{source}"
    );
    hick_lang::parse_session(&source).expect("session with usage must parse");
}

/// LIVE E3: turn ≥ 2 of a real multi-turn session must report
/// `usage.cache_read_input_tokens > 0` — if it is 0, something is
/// invalidating the prefix and this test fails the run.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "live network test — set HICKORY_AGENT_LIVE=1 and ANTHROPIC_API_KEY to run"]
async fn live_cache_read_on_second_turn() {
    if std::env::var("HICKORY_AGENT_LIVE").as_deref() != Ok("1") {
        eprintln!("HICKORY_AGENT_LIVE != 1 — skipping");
        return;
    }
    let client = AnthropicClient::new().with_max_tokens(256);

    // The prefix must clear the model's minimum or caching silently no-ops.
    let history = vec![
        Message::new(Role::System, hickory_agent::SYSTEM_PROMPT),
        Message::new(
            Role::System,
            // Padding doc context pushes the prefix past the 1024-token
            // minimum on sonnet-5 (measured below via count_tokens, the
            // only valid instrument).
            format!(
                "## Document under discussion\n\n{}",
                "lorem ipsum dolor sit amet, ".repeat(200)
            ),
        ),
        Message::new(Role::User, "Reply with the single word: one"),
    ];
    let check = client
        .verify_cacheable_prefix(&history)
        .await
        .expect("count_tokens failed");
    assert!(
        check.cacheable,
        "system prefix is {} tokens, below the {}-token minimum — fix the prefix, \
         the cache test cannot pass",
        check.tokens, check.minimum
    );

    let (text1, usage1) = client.complete_with_usage(history.clone()).await.unwrap();
    let mut history2 = history;
    history2.push(Message::new(Role::Assistant, text1));
    history2.push(Message::new(
        Role::User,
        "Now reply with the single word: two",
    ));
    let (_text2, usage2) = client.complete_with_usage(history2).await.unwrap();

    eprintln!("turn1 usage: {usage1:?}\nturn2 usage: {usage2:?}");
    assert!(
        usage2.cache_read_input_tokens > 0,
        "turn 2 read no cache — the prefix is being invalidated: {usage2:?}"
    );
}
