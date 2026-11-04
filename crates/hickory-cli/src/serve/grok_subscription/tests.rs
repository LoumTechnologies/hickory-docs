use super::*;
fn report() -> Value {
    json!({"loginPolicy":{"apiKeyAuthDisabled":true},"projectInstructions":[],"hooks":[],"skills":[],"agents":[],"plugins":[],"mcpServers":[],"lspServers":[]})
}
#[test]
fn configuration_requires_subscription_policy_and_no_native_extensions() {
    let good = report();
    inspect(&good).unwrap();
    for field in [
        "projectInstructions",
        "hooks",
        "skills",
        "agents",
        "plugins",
        "mcpServers",
        "lspServers",
    ] {
        let mut bad = good.clone();
        bad[field] = json!([{"name":"fixture"}]);
        assert!(inspect(&bad).is_err(), "{field}");
        bad[field] = json!([{"disabled":true}]);
        inspect(&bad).unwrap();
        bad.as_object_mut().unwrap().remove(field);
        assert!(inspect(&bad).is_err());
    }
    let mut builtin = good.clone();
    builtin["agents"] = json!([{"name":"fixture","source":{"type":"builtin"}}]);
    inspect(&builtin).unwrap();
    builtin["agents"][0]["source"]["type"] = json!("user");
    assert!(inspect(&builtin).is_err());
    let mut bad = good.clone();
    bad["loginPolicy"]["apiKeyAuthDisabled"] = json!(false);
    assert!(inspect(&bad).is_err());
    let mut bad = good;
    bad["configWarnings"] = json!(["unsupported pin"]);
    assert!(inspect(&bad).is_err());
}
#[test]
fn reports_disjoint_tokens_without_interpreting_native_api_cost() {
    let mut data = json!({"text":"answer","stopReason":"end_turn","num_turns":1,"total_cost_usd":1234.0,"usage":{"input_tokens":10,"output_tokens":4,"cache_read_input_tokens":20,"cache_creation_input_tokens":3}});
    let chunk = response(&serde_json::to_vec(&data).unwrap()).unwrap();
    let usage = chunk.usage.unwrap();
    assert_eq!(
        (
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_read_input_tokens,
            usage.cache_creation_input_tokens
        ),
        (10, 4, 20, 3)
    );
    data["usage_is_incomplete"] = json!(true);
    assert!(
        response(&serde_json::to_vec(&data).unwrap())
            .unwrap()
            .usage
            .is_none()
    );
    data.as_object_mut().unwrap().remove("num_turns");
    data.as_object_mut().unwrap().remove("usage");
    assert!(
        response(&serde_json::to_vec(&data).unwrap())
            .unwrap()
            .usage
            .is_none()
    );
    data["stopReason"] = json!("max_turns");
    assert!(response(&serde_json::to_vec(&data).unwrap()).is_err());
    assert!(
        response(br#"{"type":"error","message":"secret credential"}"#)
            .unwrap_err()
            .to_string()
            .contains("no API billing fallback")
    );
}
#[test]
fn selection_restores_only_latest_subscription_and_never_prices_it() {
    let turn = super::super::agent::TurnRecord {
        backend: "python".into(),
        id: "fixture".into(),
        parent_id: None,
        prompt: String::new(),
        answer: None,
        status: "ok".into(),
        error: None,
        created_at: "now".into(),
        provider: SELECTOR.into(),
        model: "claude-sonnet-5".into(),
        usage: Some(Usage {
            input_tokens: 1000000,
            ..Default::default()
        }),
        session: String::new(),
    };
    let turns = vec![turn];
    assert_eq!(
        super::super::subscription::retained_choice(&turns)
            .unwrap()
            .provider
            .as_deref(),
        Some(SELECTOR)
    );
    assert_eq!(super::super::agent::totals_of(&turns).1, None);
    assert_eq!(
        super::super::subscription::default_model(SELECTOR),
        Some(DEFAULT_MODEL)
    );
}
fn fixture(script: &str) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new("python3");
    cmd.args(["-c", script]).kill_on_drop(true);
    cmd
}
#[tokio::test]
async fn cached_auth_never_selects_api_keys_or_starts_model_sessions() {
    for (methods, failure, ready) in [
        (json!([{"id":"cached_token"}]), false, Some(true)),
        (json!([{"id":"grok.com"}]), false, Some(false)),
        (
            json!([{"id":"cached_token"},{"id":"xai.api_key"}]),
            false,
            None,
        ),
        (json!([{"id":"cached_token"}]), true, None),
    ] {
        let script = format!(
            r#"import sys,json
for line in sys.stdin:
 m=json.loads(line)
 if m['method']=='initialize':
  result={{'authMethods':json.loads({methods:?})}}
 elif m['method']=='authenticate':
  assert m['params']=={{'methodId':'cached_token','_meta':{{'headless':True}}}}
  if {failure}:
   print(json.dumps({{'id':m['id'],'error':{{'message':'fixture auth/network error'}}}}),flush=True)
   continue
  result={{}}
 else: raise Exception('Unexpected native agent, purchase or API billing operation')
 print(json.dumps({{'id':m['id'],'result':result}}),flush=True)
"#,
            methods = methods.to_string(),
            failure = if failure { "True" } else { "False" }
        );
        let (rpc, _events) = Rpc::spawn(fixture(&script)).unwrap();
        let result = cached_auth(&rpc).await;
        rpc.kill().await;
        match ready {
            Some(ready) => assert_eq!(result.unwrap(), ready),
            None => assert!(result.is_err()),
        }
    }
}
#[cfg(target_os = "macos")]
#[test]
fn native_profile_pins_auth_and_drops_all_inherited_credentials() {
    let root = tempfile::tempdir().unwrap();
    let profile = Profile::at(
        DEFAULT_MODEL,
        root.path().into(),
        root.path().join("auth.json"),
    )
    .unwrap();
    let config = std::fs::read_to_string(profile.scratch.path().join("requirements.toml")).unwrap();
    assert!(config.contains("disable_api_key_auth = true"));
    assert!(config.contains("preferred_method = \"oidc\""));
    assert!(config.contains("https://api.x.ai/v1"));
    assert!(config.contains("ignore = [\"/\"]"));
    let cmd = inference_command(&profile, Path::new("/usr/bin/true"), DEFAULT_MODEL).unwrap();
    let cmd = cmd.as_std();
    let env: std::collections::BTreeMap<_, _> = cmd.get_envs().collect();
    assert_eq!(
        env.get(std::ffi::OsStr::new("GROK_DISABLE_API_KEY_AUTH")),
        Some(&Some(std::ffi::OsStr::new("true")))
    );
    assert!(
        !env.keys()
            .any(|key| key.to_string_lossy().ends_with("_API_KEY"))
    );
    let args: Vec<_> = cmd.get_args().map(|v| v.to_string_lossy()).collect();
    for required in [
        ["--tools", "WebSearch"],
        [
            "--disallowed-tools",
            "web_search,web_fetch,search_tool,use_tool",
        ],
        ["--permission-mode", "dontAsk"],
        ["--max-turns", "1"],
    ] {
        assert!(args.windows(2).any(|pair| pair == required));
    }
    for required in [
        "--disable-web-search",
        "--no-plan",
        "--no-subagents",
        "--no-memory",
        "--no-auto-update",
    ] {
        assert!(args.iter().any(|arg| arg == required));
    }
    assert!(args[1].contains("(deny process-exec"));
    assert!(args[1].contains("(deny file-read-data"));
}
#[tokio::test]
async fn failed_exit_and_oversized_output_are_not_admitted() {
    assert!(
        process::capture(fixture(
            "print('valid but unsuccessful'); raise SystemExit(1)"
        ))
        .await
        .is_err()
    );
    assert!(
        process::capture(fixture("print('x'*1100000)"))
            .await
            .is_err()
    );
    assert_eq!(
        process::capture(fixture("print('fixture')")).await.unwrap(),
        b"fixture\n"
    );
}
#[cfg(unix)]
#[tokio::test]
async fn cancellation_kills_native_process() {
    let root = tempfile::tempdir().unwrap();
    let pidfile = root.path().join("pid");
    let script = format!(
        "import os,time,pathlib; pathlib.Path({:?}).write_text(str(os.getpid())); time.sleep(60)",
        pidfile.to_str().unwrap()
    );
    let task = tokio::spawn(process::capture(fixture(&script)));
    tokio::time::timeout(Duration::from_secs(5), async {
        while !pidfile.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let pid: i32 = std::fs::read_to_string(pidfile).unwrap().parse().unwrap();
    task.abort();
    let _ = task.await;
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            // SAFETY: signal zero only probes fixture process existence.
            if unsafe { libc::kill(pid, 0) } != 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("cancelled Grok process remained alive");
}
