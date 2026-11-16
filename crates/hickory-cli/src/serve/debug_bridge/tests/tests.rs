use super::*;
use hick_dap::Step;

// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
#[test]
fn agent_action_fixture_uses_opaque_handles_without_display_line_arithmetic() {
    let reading: hick_observe::reading::Reading = serde_json::from_str(include_str!(
        "../../../../../../apps/web/src/debug/fixtures/reading.json"
    ))
    .unwrap();
    let action = json!({"op":"located", "session":"debug-fixture", "location":reading.locations[0].id, "scope":reading.scope, "how":"jump"});
    assert!(action.get("line").is_none());
    let request: Request = serde_json::from_value(action).unwrap();
    match request {
        Request::Located {
            location, scope, ..
        } => {
            assert_eq!(
                reading
                    .resolve(&location, &scope, &reading.source.bytes, Some(7))
                    .unwrap(),
                1
            );
            let mut stale = scope;
            stale.stop = "old-stop".into();
            assert!(
                reading
                    .resolve(&location, &stale, &reading.source.bytes, Some(7))
                    .is_err()
            );
        }
        _ => panic!("expected located action"),
    }
}

// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn real_adapter_relocation_and_stale_actions_share_the_reading() {
    use hick_observe::reading::DebugAnnotation as A;
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if cache.join("adapters/python/bin/python3").exists() {
        std::os::unix::fs::symlink(cache, dir.path().join(".hick-cache")).unwrap();
    }
    if hick_dap::discover("python", dir.path()).is_none() {
        eprintln!("SKIPPED: install Python adapter with hick dap install python");
        return;
    }
    let source = "# Reading\n<hick:file path=\"main.py\">\nvalue = 1\n\n\ndef count(n):\n    return count(n - 1) if n else n\nprint(count(2))\n</hick:file>\n";
    let path = dir.path().join("demo.md");
    std::fs::write(&path, source).unwrap();
    let registry = Arc::new(Registry::new());
    let out = handle(
        &registry,
        dir.path(),
        Request::Start {
            request_id: None,
            doc: "hick:///demo.md".into(),
            breakpoints: vec![hick_dap::Breakpoint {
                line: 3,
                condition: None,
                hit_condition: None,
                log_message: None,
            }],
            program: Some("main.py".into()),
            project: None,
            profile: None,
            sources: Default::default(),
        },
    )
    .await;
    let (session, reading) = out
        .iter()
        .find_map(|r| match r {
            Response::Reading { session, reading } => Some((session.clone(), reading.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing reading: {out:?}"));
    let requested = reading
        .annotations
        .iter()
        .find(|a| matches!(a.payload, A::BreakpointRequested { .. }))
        .unwrap();
    let bound = reading
        .annotations
        .iter()
        .find(|a| matches!(a.payload, A::BreakpointBound { .. }))
        .unwrap();
    let paused = reading
        .annotations
        .iter()
        .find(|a| matches!(a.payload, A::Paused { .. }))
        .unwrap();
    assert_ne!(requested.location, bound.location);
    assert_eq!(bound.location, paused.location);
    assert_eq!(reading.source.bytes, source.as_bytes());
    let location = paused.location.clone().unwrap();
    let mut wrong_frame = reading.scope.clone();
    wrong_frame.frame = Some(-999);
    assert!(
        registry
            .get(&session)
            .await
            .unwrap()
            .resolve_location(&location, &wrong_frame)
            .await
            .is_err()
    );
    std::fs::write(&path, source.replace("value = 1", "value = 10")).unwrap();
    let changed = handle(
        &registry,
        dir.path(),
        Request::Located {
            session: session.clone(),
            location: location.clone(),
            scope: reading.scope.clone(),
            how: protocol::LocatedAction::Jump,
        },
    )
    .await;
    assert!(changed.iter().any(
        |r| matches!(r, Response::Failed { message, .. } if message.contains("source changed"))
    ));
    std::fs::write(&path, source).unwrap();
    let _ = handle(
        &registry,
        dir.path(),
        Request::Step {
            session: session.clone(),
            how: hick_dap::Step::Over,
            frame: None,
        },
    )
    .await;
    let stale = handle(
        &registry,
        dir.path(),
        Request::Located {
            session: session.clone(),
            location,
            scope: reading.scope,
            how: protocol::LocatedAction::RunTo,
        },
    )
    .await;
    assert!(stale.iter().any(|r| matches!(r, Response::Failed { .. })));
    registry.reap(&session).await;
}

/// A C# document whose build cannot succeed, and what the client is told.
///
/// The point is the ORDER and the SURVIVAL: the build's own output has to
/// reach the client on the failing path, before the failure, or a person
/// gets "building app.csproj failed" and none of the reasons.
#[tokio::test]
async fn a_failed_build_reaches_the_client_before_the_failure() {
    let dir = tempfile::tempdir().unwrap();
    // A document that generates C# and no project file — the refusal
    // `build` makes without running anything, so this test needs no
    // .NET SDK to be meaningful.
    std::fs::write(
        dir.path().join("a.md"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             <hick:file path=\"Program.cs\">\n\
             class Program { static void Main() {} }\n\
             </hick:file>\n\
             </hick:doc>\n",
    )
    .unwrap();

    let registry = Arc::new(Registry::new());
    let responses = handle(
        &registry,
        dir.path(),
        Request::Start {
            request_id: None,
            doc: "hick:///a.md".into(),
            breakpoints: vec![],
            program: None,
            project: None,
            profile: None,
            sources: Default::default(),
        },
    )
    .await;

    // Discovery runs before the build, so on a machine with no
    // netcoredbg the adapter is what is missing and the build never
    // gets a chance to speak. Either way the person is told something
    // they can act on, and that is what is asserted.
    let failure = responses
        .iter()
        .find_map(|r| match r {
            Response::Failed { message, .. } => Some(message.clone()),
            _ => None,
        })
        .expect("a C# document with no project file cannot start");
    assert!(
        failure.contains("no project file") || failure.contains("no debug adapter"),
        "the failure does not say what is missing: {failure}"
    );
    // And it says how to get out of it. Which sentence that is depends on
    // which of the two things was missing, and both are checked by
    // `dap_install`'s drift test — the point here is that a person is
    // never left with only the bad news.
    assert!(
        failure.contains("hick dap install csharp")
            || failure.contains("hick ingest")
            || failure.contains("hick:file"),
        "the failure says what is wrong but not what to do: {failure}"
    );
    // Whatever the build managed to say arrives BEFORE the failure.
    if let Some(build_at) = responses
        .iter()
        .position(|r| matches!(r, Response::Build { .. }))
    {
        let failed_at = responses
            .iter()
            .position(|r| matches!(r, Response::Failed { .. }))
            .unwrap();
        assert!(build_at < failed_at, "the build arrived after the failure");
    }
}

#[test]
fn a_build_line_is_shaped_like_a_transcript_event() {
    // Not because it IS a transcript — nothing here is recorded or
    // compared — but because the client already has a terminal that
    // renders exactly this shape.
    let since = std::time::Instant::now();
    let cmd = build_event(BuildOutput::Cmd("dotnet build".into()), since);
    assert_eq!(cmd["kind"], "cmd");
    assert_eq!(cmd["data"], "dotnet build");
    let out = build_event(BuildOutput::Out("Restored app.csproj".into()), since);
    assert_eq!(out["kind"], "out");
    // Lines arrive without their newline and the terminal needs one.
    assert_eq!(out["data"], "Restored app.csproj\n");
    // A note is this app talking, not the build tool, and it is indented
    // the way the spec writes it.
    let note = build_event(BuildOutput::Note("packages restore into x".into()), since);
    assert_eq!(note["data"], "  packages restore into x\n");
    let exit = build_event(BuildOutput::Exit(1), since);
    assert_eq!(exit["kind"], "exit");
    assert_eq!(exit["code"], 1);
}

#[test]
fn a_document_is_named_in_the_clients_scheme() {
    // The browser never learns where the file is on disk, exactly as on
    // the language channel.
    let root = std::path::Path::new("/home/u/project");
    assert_eq!(resolve(root, "hick:///docs/a.md"), root.join("docs/a.md"));
    assert_eq!(resolve(root, "/docs/a.md"), root.join("docs/a.md"));
    assert_eq!(resolve(root, "a.md"), root.join("a.md"));
}

#[test]
fn requests_parse_from_their_frame() {
    let mut frame = vec![CHANNEL_DEBUG];
    frame.extend_from_slice(br#"{"op":"start","doc":"hick:///a.md","breakpoints":[{"line":7}]}"#);
    let request = request_of(&frame).expect("parses");
    match request {
        Request::Start {
            doc,
            breakpoints,
            program,
            project,
            ..
        } => {
            assert!(project.is_none());
            assert_eq!(doc, "hick:///a.md");
            assert_eq!(breakpoints[0].line, 7);
            // Optional fields are absent, not zero.
            assert!(breakpoints[0].condition.is_none());
            // No program named: the session falls back to the first
            // debuggable file, which is what a one-file document wants.
            assert!(program.is_none());
        }
        other => panic!("wrong request: {other:?}"),
    }
}

#[test]
fn a_step_names_the_session_apis_own_verb() {
    let mut frame = vec![CHANNEL_DEBUG];
    frame.extend_from_slice(br#"{"op":"step","session":"dbg-0","how":"drop_frame"}"#);
    match request_of(&frame).expect("parses") {
        Request::Step { how, .. } => assert_eq!(how, Step::DropFrame),
        other => panic!("wrong request: {other:?}"),
    }
}

#[test]
fn a_response_frame_is_tagged_with_the_debug_channel() {
    let frame = frame_of(&Response::Ended {
        session: "dbg-0".into(),
    });
    assert_eq!(frame[0], CHANNEL_DEBUG);
    let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
    assert_eq!(value["event"], "ended");
    assert_eq!(value["session"], "dbg-0");
}

#[test]
fn a_failure_carries_words_rather_than_a_code() {
    // The UI shows this text; a code would need a second table nobody
    // would keep in step.
    let frame = frame_of(&Response::Failed {
        request_id: None,
        doc: None,
        session: Some("dbg-1".into()),
        message: "that line is prose, not code".into(),
        about: Some("breakpoints".into()),
        lines: vec![4],
        offer_install: None,
    });
    let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
    assert_eq!(value["event"], "failed");
    assert!(value["message"].as_str().unwrap().contains("prose"));
    // And says what it was about, so the app can show it on the
    // breakpoint rather than in a banner that outlives its cause.
    assert_eq!(value["about"], "breakpoints");
    assert_eq!(value["lines"][0], 4);
}

#[test]
fn a_failure_about_nothing_in_particular_carries_no_lines() {
    // `about`/`lines` are omitted rather than sent empty: a UI that keys
    // on their presence should not have to know two spellings of absent.
    let frame = frame_of(&Response::Failed {
        request_id: None,
        doc: None,
        session: None,
        message: "no such session".into(),
        about: None,
        lines: Vec::new(),
        offer_install: None,
    });
    let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
    assert!(value.get("about").is_none(), "{value}");
    assert!(value.get("lines").is_none(), "{value}");
}

#[test]
fn the_channel_carries_no_way_to_edit_anything() {
    // The isolation guarantee, checked against the protocol itself: if an
    // edit verb ever appears here, this fails.
    let described = describe();
    let requests = described["requests"].as_array().unwrap();
    for request in requests {
        let name = request.as_str().unwrap();
        assert!(
            !name.contains("edit") && !name.contains("write") && !name.contains("save"),
            "the debug channel grew a way to write: {name}"
        );
    }
}
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires debugpy in HICKORY_THREAD_TEST_VENV or target/thread-debug-env"]
async fn real_threads_select_current_stack_and_refuse_stale_scope_without_source_writes() {
    async fn ask(registry: &Arc<Registry>, root: &std::path::Path, value: Value) -> Vec<Value> {
        handle(registry, root, serde_json::from_value(value).unwrap())
            .await
            .into_iter()
            .map(|response| serde_json::to_value(response).unwrap())
            .collect()
    }
    fn event<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
        rows.iter()
            .find(|row| row["event"] == kind)
            .unwrap_or_else(|| panic!("Missing {kind}: {rows:?}"))
    }
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(".git")).unwrap();
    let venv = std::env::var_os("HICKORY_THREAD_TEST_VENV")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/thread-debug-env")
        });
    assert!(
        venv.join("bin/python").exists(),
        "Prepare the explicit debugpy test virtualenv first"
    );
    std::os::unix::fs::symlink(
        std::fs::canonicalize(venv).unwrap(),
        root.path().join(".venv"),
    )
    .unwrap();
    let source = "import threading\nready = threading.Event()\nrelease = threading.Event()\ndef worker():\n    marker = 41\n    ready.set()\n    while not release.is_set():\n        marker += 0\n    marker += 1\nthread = threading.Thread(target=worker, name='hickory-worker')\nthread.start()\nready.wait()\nvalue = 7\nrelease.set()\nthread.join()\nprint(value)\n";
    std::fs::write(root.path().join("main.py"), source).unwrap();
    let registry = Arc::new(Registry::new());
    let breakpoint = source
        .lines()
        .position(|line| line == "release.set()")
        .unwrap();
    let started = ask(
        &registry,
        root.path(),
        json!({"op":"start","doc":"hick:///main.py","request_id":"launch-threads","breakpoints":[{"line":breakpoint}]}),
    )
    .await;
    let session = event(&started, "started")["session"]
        .as_str()
        .unwrap()
        .to_owned();
    let stopped = event(&started, "stopped");
    let stop = stopped["stop"].as_u64().unwrap();
    let main = stopped["thread"].as_i64().unwrap();
    let main_frame = stopped["frames"][0]["id"].as_i64().unwrap();
    let live = registry.get(&session).await.unwrap();
    let mut delivered = false;
    let listed = handle_with(
        &registry,
        root.path(),
        serde_json::from_value(json!({"op":"threads","session":session,"request_id":"list-1"}))
            .unwrap(),
        |responses| {
            assert!(
                live.operation.try_lock().is_err(),
                "Response queueing lost the session operation guard"
            );
            assert!(!responses.is_empty());
            delivered = true;
        },
    )
    .await
    .into_iter()
    .map(|response| serde_json::to_value(response).unwrap())
    .collect::<Vec<_>>();
    assert!(delivered);
    let list = event(&listed, "threads");
    assert_eq!(list["stop"], stop);
    assert_eq!(list["selected"], main);
    assert_eq!(list["request_id"], "list-1");
    let worker = list["threads"]
        .as_array()
        .unwrap()
        .iter()
        .find(|thread| thread["name"] == "hickory-worker")
        .unwrap()["id"]
        .as_i64()
        .unwrap();
    assert_ne!(worker, main);
    for (requested, generation) in [(worker, stop - 1), (i64::MAX, stop)] {
        let refused = ask(&registry, root.path(), json!({"op":"select_thread","session":session,"thread":requested,"stop":generation,"request_id":"refused"})).await;
        assert_eq!(event(&refused, "failed")["request_id"], "refused");
        assert_eq!(
            *registry.get(&session).await.unwrap().thread_id.lock().await,
            Some(main)
        );
    }
    let selected = ask(&registry, root.path(), json!({"op":"select_thread","session":session,"thread":worker,"stop":stop,"request_id":"select-worker"})).await;
    let paused = event(&selected, "stopped");
    assert_eq!(paused["thread"], worker);
    let worker_stop = paused["stop"].as_u64().unwrap();
    assert!(worker_stop > stop);
    let worker_frame = paused["frames"]
        .as_array()
        .unwrap()
        .iter()
        .find(|frame| frame["name"] == "worker")
        .unwrap()["id"]
        .as_i64()
        .unwrap();
    assert_eq!(event(&selected, "threads")["request_id"], "select-worker");
    let locals = ask(&registry, root.path(), json!({"op":"state","session":session,"frame":worker_frame,"stop":worker_stop,"thread":worker,"request_id":"worker-locals"})).await;
    let locals = event(&locals, "frame_variables");
    assert_eq!(locals["request_id"], "worker-locals");
    assert_eq!(locals["frame"], worker_frame);
    assert_eq!(locals["thread"], worker);
    assert_eq!(locals["stop"], worker_stop);
    assert!(
        locals["variables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value["name"] == "marker" && value["value"] == "41")
    );
    let foreign = ask(&registry, root.path(), json!({"op":"eval","session":session,"expression":"value","frame":main_frame,"context":"watch","stop":worker_stop,"thread":worker,"request_id":"foreign-frame"})).await;
    assert_eq!(event(&foreign, "failed")["request_id"], "foreign-frame");
    let value = ask(&registry, root.path(), json!({"op":"eval","session":session,"expression":"marker","frame":worker_frame,"context":"watch","stop":worker_stop,"thread":worker,"request_id":"worker-value"})).await;
    let value = event(&value, "value");
    assert_eq!(value["value"], "41");
    assert_eq!(value["stop"], worker_stop);
    assert_eq!(value["thread"], worker);
    assert_eq!(value["frame"], worker_frame);
    assert_eq!(value["request_id"], "worker-value");
    let stale = ask(&registry, root.path(), json!({"op":"eval","session":session,"expression":"marker","frame":worker_frame,"context":"watch","stop":stop,"thread":worker,"request_id":"old-value"})).await;
    assert_eq!(event(&stale, "failed")["request_id"], "old-value");
    let selected = ask(&registry, root.path(), json!({"op":"select_thread","session":session,"thread":main,"stop":worker_stop,"request_id":"select-main"})).await;
    assert_eq!(event(&selected, "stopped")["thread"], main);
    let done = ask(
        &registry,
        root.path(),
        json!({"op":"step","session":session,"how":"continue"}),
    )
    .await;
    assert!(
        done.iter().any(|row| row["event"] == "finished"),
        "{done:?}"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.py")).unwrap(),
        source
    );
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires debugpy in HICKORY_THREAD_TEST_VENV or target/thread-debug-env"]
async fn real_running_launch_exposes_session_for_stop_and_live_breakpoints() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(".git")).unwrap();
    let venv = std::env::var_os("HICKORY_THREAD_TEST_VENV")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/thread-debug-env")
        });
    std::os::unix::fs::symlink(
        std::fs::canonicalize(venv).unwrap(),
        root.path().join(".venv"),
    )
    .unwrap();
    let source = "import time\nwhile True:\n    time.sleep(0.01)\n";
    std::fs::write(root.path().join("running.py"), source).unwrap();
    let registry = Arc::new(Registry::new());
    let (tx, mut receiver) = mpsc::unbounded_channel();
    let running = {
        let registry = registry.clone();
        let path = root.path().to_path_buf();
        tokio::spawn(async move {
            dispatch(
                &registry,
                &path,
                serde_json::from_value(
                    json!({"op":"start","doc":"hick:///running.py","request_id":"launch-running","breakpoints":[]}),
                )
                .unwrap(),
                &tx,
            )
            .await;
        })
    };
    let session = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let frame = receiver
                .recv()
                .await
                .expect("Debug launch closed before reporting its session");
            let response: Value = serde_json::from_slice(&frame[1..]).unwrap();
            assert_ne!(response["event"], "failed", "{response}");
            if response["event"] == "started" {
                assert_eq!(response["request_id"], "launch-running");
                break response["session"].as_str().unwrap().to_owned();
            }
        }
    })
    .await
    .expect("A running program hid its real session until its first stop");
    let breakpoints = tokio::time::timeout(Duration::from_secs(10), handle(&registry, root.path(), serde_json::from_value(json!({"op":"breakpoints","session":session,"breakpoints":[{"line":2,"condition":"False"}]})).unwrap())).await.expect("Live breakpoints waited for the program to stop");
    assert!(
        breakpoints
            .iter()
            .any(|response| matches!(response, Response::Breakpoints { .. })),
        "{breakpoints:?}"
    );
    let ended = tokio::time::timeout(
        Duration::from_secs(10),
        handle(
            &registry,
            root.path(),
            serde_json::from_value(json!({"op":"stop","session":session})).unwrap(),
        ),
    )
    .await
    .expect("Stop could not interrupt the running launch");
    assert!(
        ended
            .iter()
            .any(|response| matches!(response, Response::Ended { .. })),
        "{ended:?}"
    );
    tokio::time::timeout(Duration::from_secs(10), running)
        .await
        .expect("The initial stop waiter survived adapter shutdown")
        .unwrap();
    assert!(registry.get(&session).await.is_err());
    assert_eq!(
        std::fs::read_to_string(root.path().join("running.py")).unwrap(),
        source
    );
}
