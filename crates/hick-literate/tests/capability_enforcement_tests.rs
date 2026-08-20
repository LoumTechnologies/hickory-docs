//! What a document declares is what a container gets.
//!
//! These protect docs/guarantees/execution/declared-capabilities-are-enforced.md.
//!
//! They run against `LocalExecutor`, which is deliberately unsandboxed — and
//! that is the point for volumes: volume access is mediated by the *pipeline*
//! (it tars data into and out of a container), so it is enforced identically
//! on every backend. Network confinement is the executor's job, so the
//! network test here checks that the declaration reaches the executor, and
//! `hickory-executor-docker` checks what a backend does with it.

use std::sync::Arc;

use hickory_executor::{Executor, LocalExecutor};

// Contents are compared TRIMMED. What these guarantee is which bytes land in
// which file and which writes are refused; the line ending is the shell's, not
// the document's, and `echo` on cmd adds CRLF where `echo` on sh adds LF. The
// alternative in cmd -- `<nul set /p=` -- exits 1 and leaves a trailing space,
// so it breaks the `&&` chains AND the byte count. See tests/common/mod.rs.

mod common;
use common::{and, make_dir, require_absent, show, write};

fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
    )
}

async fn run(src: &str) -> anyhow::Result<hick_literate::PipelineResult> {
    let executor = Arc::new(LocalExecutor::new().unwrap());
    hick_literate::run_pipeline_live(
        &[("test.hick", src)],
        &hick_literate::PipelineConfig::default(),
        &[],
        None,
        executor,
    )
    .await
}

// ---------------------------------------------------------------------------
// Volumes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_container_the_volume_does_not_name_gets_nothing() {
    // The denied direction. `intruder` mounts a volume whose access rules
    // never mention it; the run stops rather than handing over the data.
    let src = hick_doc(&format!(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="intruder" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">{writer}</hick:exec>
<hick:exec container="intruder" mount="shared:/in">{reader}</hick:exec>"#,
        writer = write("secret", "out/file.txt"),
        reader = show("in/file.txt"),
    ));

    let Err(err) = run(&src).await else {
        panic!("the intruder was handed the volume");
    };
    let msg = err.to_string();
    assert!(
        msg.contains("intruder") && msg.contains("grants it no access"),
        "expected a refusal naming the container, got: {msg}"
    );
    // And it must say how to fix it — this surfaces on the CLI.
    assert!(msg.contains("hick:allow"), "no next step offered: {msg}");
}

#[tokio::test]
async fn a_named_container_gets_what_it_was_granted() {
    // The granted direction, same document shape.
    let src = hick_doc(&format!(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">{writer}</hick:exec>
<hick:exec container="reader" mount="shared:/in">{reader}</hick:exec>"#,
        writer = write("secret", "out/file.txt"),
        reader = show("in/file.txt"),
    ));

    let result = run(&src).await.expect("the reader was granted read access");
    assert_eq!(
        result.files.get("result/file.txt").unwrap().trim(),
        "secret"
    );
}

#[tokio::test]
async fn a_partial_read_grant_hands_over_only_that_part() {
    // `read="public/**"` must mean the container never sees the rest, not
    // that it is trusted to ignore it. The reader asserts the absence
    // itself, so a leak fails the exec.
    let src = hick_doc(&format!(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="public/**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">{writer}</hick:exec>
<hick:exec container="reader" mount="shared:/in">{reader}</hick:exec>"#,
        writer = and(&[
            make_dir("out/public"),
            write("open", "out/public/ok.txt"),
            write("closed", "out/private.txt"),
        ]),
        reader = and(&[show("in/public/ok.txt"), require_absent("in/private.txt")]),
    ));

    let result = run(&src)
        .await
        .expect("the reader saw a file outside its read grant");
    // The volume itself still has both: the grant narrowed the copy handed
    // to the reader, it did not delete anything.
    assert_eq!(
        result.files.get("result/private.txt").unwrap().trim(),
        "closed"
    );
    assert_eq!(
        result.files.get("result/public/ok.txt").unwrap().trim(),
        "open"
    );
}

#[tokio::test]
async fn a_read_only_container_cannot_change_the_volume() {
    // Write access is not "whoever mounted it last wins".
    let src = hick_doc(&format!(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">{writer}</hick:exec>
<hick:exec container="reader" mount="shared:/in">{reader}</hick:exec>"#,
        writer = write("original", "out/file.txt"),
        reader = write("tampered", "in/file.txt"),
    ));

    let result = run(&src).await.expect("pipeline should complete");
    assert_eq!(
        result.files.get("result/file.txt").unwrap().trim(),
        "original",
        "a read-only container's writes escaped its container"
    );
}

#[tokio::test]
async fn a_partial_writer_changes_only_the_paths_it_was_granted() {
    // The case that used to be enforced as all-or-nothing: `patcher` may
    // rewrite Controllers/ and nothing else. Its other write must not land,
    // and its permitted write must.
    let src = hick_doc(&format!(
        r#"<hick:container name="scaffolder" image="alpine" />
<hick:container name="patcher" image="alpine" />
<hick:volume name="project" output="result">
  <hick:allow container="scaffolder" write="**" />
  <hick:allow container="patcher" read="**" />
  <hick:allow container="patcher" write="Controllers/**" />
</hick:volume>
<hick:exec container="scaffolder" mount="project:/p">{scaffolder}</hick:exec>
<hick:exec container="patcher" mount="project:/p">{patcher}</hick:exec>"#,
        scaffolder = and(&[
            make_dir("p/Controllers"),
            write("base", "p/Models.cs"),
            write("scaffolded", "p/Controllers/Home.cs"),
        ]),
        patcher = and(&[
            write("patched", "p/Controllers/Home.cs"),
            write("hacked", "p/Models.cs"),
        ]),
    ));

    let result = run(&src).await.expect("pipeline should complete");
    assert_eq!(
        result
            .files
            .get("result/Controllers/Home.cs")
            .unwrap()
            .trim(),
        "patched",
        "the granted write did not land"
    );
    assert_eq!(
        result.files.get("result/Models.cs").unwrap().trim(),
        "base",
        "the ungranted write landed anyway"
    );
}

#[tokio::test]
async fn a_volume_with_no_allow_children_stays_unrestricted() {
    // Documents written before access rules existed must keep working:
    // rules apply to everyone only once someone is named.
    let src = hick_doc(&format!(
        r#"<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:volume name="shared" output="result" />
<hick:exec container="a" mount="shared:/out">{a}</hick:exec>
<hick:exec container="b" mount="shared:/in">{b}</hick:exec>"#,
        a = write("from-a", "out/file.txt"),
        b = and(&[show("in/file.txt"), write("from-b", "in/file.txt")]),
    ));

    let result = run(&src).await.expect("unrestricted volume should work");
    assert_eq!(
        result.files.get("result/file.txt").unwrap().trim(),
        "from-b"
    );
}

// ---------------------------------------------------------------------------
// Network
// ---------------------------------------------------------------------------

#[tokio::test]
async fn the_declared_network_capability_reaches_the_executor() {
    // The plumbing this whole change is about: a container's declaration is
    // handed to the executor before it starts, so a backend that can confine
    // it has something to confine it with. `LocalExecutor` records and does
    // not impose — see the crate docs there.
    let src = hick_doc(
        r#"<hick:container name="online" image="alpine">
  <hick:allow network="github.com:443" />
</hick:container>
<hick:container name="offline" image="alpine" />
<hick:exec container="online">echo hi</hick:exec>
<hick:exec container="offline">echo hi</hick:exec>"#,
    );

    let executor = Arc::new(LocalExecutor::new().unwrap());
    let recorder: Arc<dyn Executor> = executor.clone();
    hick_literate::run_pipeline_live(
        &[("test.hick", &src)],
        &hick_literate::PipelineConfig::default(),
        &[],
        None,
        recorder,
    )
    .await
    .unwrap();

    assert!(
        executor
            .declared_capabilities("online")
            .expect("no capabilities declared for 'online'")
            .allows_network(),
        "the document's network grant never reached the executor"
    );
    assert!(
        !executor
            .declared_capabilities("offline")
            .expect("no capabilities declared for 'offline'")
            .allows_network(),
        "a container that declared nothing was given network access"
    );
}
