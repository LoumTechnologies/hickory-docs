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
