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
