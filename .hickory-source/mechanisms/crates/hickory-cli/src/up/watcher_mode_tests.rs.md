<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/up/watcher_mode_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c4acfdaf72fe"></a>
## `crates/hickory-cli/src/up/watcher_mode_tests.rs`

<a id="f-c4acfdaf72fe-explicit_polling_observes_a_real_file_change_and_invalid_modes_fail"></a>
### Observe real saves and preserve their pending ownership

The polling witness exercises actual filesystem events. The capture witness
runs the same weave/save functions as the loop, then replaces its memory to
prove that restart cannot erase the pending decision. A source edit matching
disk still requires explicit acceptance.

### `explicit_polling_observes_a_real_file_change_and_invalid_modes_fail`

<storycode:copy id="f-c4acfdaf72fe-explicit_polling_observes_a_real_file_change_and_invalid_modes_fail">
use super::*;

#[test]
fn explicit_polling_observes_a_real_file_change_and_invalid_modes_fail() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    std::fs::write(&path, "before").unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = watcher_with_mode(
        move |event: notify::Result<notify::Event>| {
            if let Ok(event) = event {
                let _ = tx.send(event);
            }
        },
        Some("poll"),
    )
    .unwrap();
    watcher.watch(dir.path(), RecursiveMode::Recursive).unwrap();
    std::fs::write(&path, "after").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let event = rx
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap();
        if event.paths.contains(&path) && matches!(event.kind, EventKind::Modify(_)) {
            break;
        }
    }
    assert!(watcher_with_mode(|_: notify::Result<notify::Event>| {}, Some("typo")).is_err());
}

/// External tools supply bytes, not conceptual ownership. A fresh WovenState
/// simulates restart; the durable capture must still prevent regeneration.
#[tokio::test]
async fn external_output_save_is_pending_across_restart_and_source_catchup() {
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().canonicalize().unwrap();
    let doc = root.join("notes.md");
    let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"app.py\">print('old')\n</h:file></h:doc>";
    std::fs::write(&doc, source).unwrap();
    let config = UpConfig {
        root: root.clone(),
        params: vec![],
        run: false,
        executor: ExecutorChoice::Local,
        amend_output_edits: false,
    };
    let mut state = WovenState::default();
    weave_document(&doc, &config, &mut state).await.unwrap();
    // A normal authored edit should advance the baseline without capturing our own write.
    let source = source.replace("old", "authored");
    std::fs::write(&doc, &source).unwrap();
    weave_document(&doc, &config, &mut state).await.unwrap();
    assert!(
        hick_project_env::pending::status(&root, "notes.md")
            .unwrap()
            .is_empty()
    );
    let output = root.join("app.py");
    std::fs::write(&output, "print('private')\n").unwrap();
    assert!(
        consume_output_save(&output, &mut state, false)
            .unwrap()
            .is_none()
    );
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), source);
    assert!(state.held().contains_key(&output));
    let capture = hick_project_env::pending::status(&root, "notes.md")
        .unwrap()
        .pop()
        .unwrap();
    let mut restarted = WovenState::default();
    weave_document(&doc, &config, &mut restarted).await.unwrap();
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "print('private')\n"
    );
    assert!(restarted.held().contains_key(&output));
    std::fs::write(&doc, source.replace("authored", "private")).unwrap();
    weave_document(&doc, &config, &mut restarted).await.unwrap();
    assert!(
        !hick_project_env::pending::status(&root, "notes.md")
            .unwrap()
            .is_empty()
    );
    assert!(restarted.held().contains_key(&output));
    let sources = hick_project_env::pending::sources(&root, "notes.md").unwrap();
    let mut generated = hick_lang::ProjectTree::default();
    generated
        .insert(
            "app.py",
            hick_lang::ProjectEntry::file(b"print('private')\n".to_vec()),
        )
        .unwrap();
    let resolution = state::OutputResolution {
        root: root.clone(),
        path: output.clone(),
        content: "print('private')\n".into(),
        expected_current: "print('private')\n".into(),
        expected_produced: "print('private')\n".into(),
        mode: state::ResolutionMode::Regenerate,
    };
    assert!(
        restarted
            .resolve_output(&resolution)
            .unwrap_err()
            .to_string()
            .contains("pending capture")
    );
    assert!(restarted.held().contains_key(&output));
    hick_project_env::pending::accept_evaluated(
        &root,
        "notes.md",
        &capture.id,
        &sources,
        &generated,
    )
    .unwrap();
    let signal = std::fs::read_dir(root.join(".hick-journal/output-edits"))
        .unwrap()
        .flatten()
        .map(|e| e.path().join("incorporated.json"))
        .find(|p| p.is_file())
        .unwrap();
    assert!(!is_noise(&signal));
    assert!(is_noise(&signal.with_file_name("baseline.json")));
    handle_batch(HashSet::from([signal]), &config, &mut restarted)
        .await
        .unwrap();
    assert!(
        hick_project_env::pending::status(&root, "notes.md")
            .unwrap()
            .is_empty()
    );
    assert!(!restarted.held().contains_key(&output));
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/up/watcher_mode_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/up/watcher_mode_tests.rs"><storycode:paste select="#f-c4acfdaf72fe-explicit_polling_observes_a_real_file_change_and_invalid_modes_fail"/></storycode:file>

</storycode:doc>
