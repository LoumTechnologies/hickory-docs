#![cfg(unix)]
//! Focused real-executor checks for provisional versus published verification.
use hickory_agent::{EditSession, ToolInvocation, execute_tool};
use hickory_executor::LocalExecutor;
use std::{path::Path, sync::Arc};

fn document(output: &str, value: &str, expected: &str) -> String {
    format!(
        r#"<hick:container name="c" image="alpine" />
<hick:file path="{output}"><hick:exec container="c" show="output">printf {value}<hick:expect>{expected}</hick:expect></hick:exec></hick:file>"#
    )
}

async fn verify(path: &Path, guarded: bool) -> hickory_agent::ToolOutcome {
    let mut session = EditSession::open(path, &[]).await.unwrap();
    if guarded {
        session = session.require_verified_publication();
    }
    execute_tool(
        &mut session,
        Arc::new(LocalExecutor::new().unwrap()),
        &ToolInvocation::synthetic("verify", vec![], None).unwrap(),
    )
    .await
}

// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
#[tokio::test]
async fn failed_outputs_stay_provisional_until_unchanged_expectation_passes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("doc.md");
    let output = dir.path().join("out.txt");
    std::fs::write(&output, "accepted earlier").unwrap();
    std::fs::write(&path, document("out.txt", "160", "100")).unwrap();
    let failed = verify(&path, true).await;
    assert!(!failed.ok, "{}", failed.text);
    assert!(failed.text.contains("no verified outputs published"));
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "accepted earlier"
    );
    std::fs::remove_file(&output).unwrap();
    assert!(!verify(&path, true).await.ok);
    assert!(
        !output.exists(),
        "a failed first run cannot create an accepted artifact"
    );
    std::fs::write(&path, document("out.txt", "100", "100")).unwrap();
    let passed = verify(&path, true).await;
    assert!(passed.ok, "{}", passed.text);
    assert!(std::fs::read_to_string(output).unwrap().contains("100"));
}

// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
#[tokio::test]
async fn upstream_failure_blocks_primary_and_upstream_publication() {
    let dir = tempfile::tempdir().unwrap();
    let upstream = dir.path().join("up");
    let primary = dir.path().join("primary");
    std::fs::create_dir_all(&upstream).unwrap();
    std::fs::create_dir_all(&primary).unwrap();
    std::fs::write(upstream.join("up.md"), document("up.txt", "160", "41")).unwrap();
    let path = primary.join("doc.md");
    std::fs::write(
        &path,
        format!(
            "<hick:upstream file=\"../up/up.md\" />\n{}",
            document("out.txt", "100", "100")
        ),
    )
    .unwrap();
    let failed = verify(&path, true).await;
    assert!(!failed.ok, "{}", failed.text);
    assert!(!upstream.join("up.txt").exists());
    assert!(!primary.join("out.txt").exists());
    std::fs::write(upstream.join("up.md"), document("up.txt", "41", "41")).unwrap();
    let passed = verify(&path, true).await;
    assert!(passed.ok, "{}", passed.text);
    assert!(upstream.join("up.txt").is_file());
    assert!(primary.join("out.txt").is_file());
    assert!(
        !std::fs::read_to_string(primary.join("out.txt"))
            .unwrap()
            .contains("41")
    );
}

#[tokio::test]
async fn ordinary_verification_still_publishes_observed_output_on_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("doc.md");
    std::fs::write(&path, document("out.txt", "160", "100")).unwrap();
    let failed = verify(&path, false).await;
    assert!(!failed.ok, "{}", failed.text);
    assert!(
        std::fs::read_to_string(dir.path().join("out.txt"))
            .unwrap()
            .contains("160")
    );
}

// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
#[cfg(unix)]
#[tokio::test]
async fn source_changes_during_execution_cannot_publish_a_stale_verification() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("doc.md");
    let quoted = path.to_string_lossy().replace('\'', "'\"'\"'");
    let text = document("out.txt", "100", "100").replace(
        "printf 100",
        &format!("printf changed > '{quoted}'; printf 100"),
    );
    std::fs::write(&path, text).unwrap();
    let refused = verify(&path, true).await;
    assert!(!refused.ok, "{}", refused.text);
    assert!(refused.text.contains("Source closure changed"));
    assert!(!dir.path().join("out.txt").exists());
}

// Guarantee: docs/guarantees/agent/verify-covers-the-whole-editable-set.md
#[cfg(unix)]
#[tokio::test]
async fn publication_refuses_existing_symlink_without_modifying_its_target() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("secret");
    std::fs::write(&target, "preserve").unwrap();
    std::os::unix::fs::symlink(&target, dir.path().join("out.txt")).unwrap();
    let path = dir.path().join("doc.md");
    std::fs::write(&path, document("out.txt", "100", "100")).unwrap();
    let refused = verify(&path, true).await;
    assert!(!refused.ok, "{}", refused.text);
    assert_eq!(std::fs::read_to_string(target).unwrap(), "preserve");
}
