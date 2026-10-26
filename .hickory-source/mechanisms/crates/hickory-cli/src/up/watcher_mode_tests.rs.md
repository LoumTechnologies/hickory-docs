<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/up/watcher_mode_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c4acfdaf72fe"></a>
## `crates/hickory-cli/src/up/watcher_mode_tests.rs`

<a id="f-c4acfdaf72fe-explicit_polling_observes_a_real_file_change_and_invalid_modes_fail"></a>
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/up/watcher_mode_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/up/watcher_mode_tests.rs"><storycode:paste select="#f-c4acfdaf72fe-explicit_polling_observes_a_real_file_change_and_invalid_modes_fail"/></storycode:file>

</storycode:doc>
