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
    let src = hick_doc(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="intruder" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">printf secret > out/file.txt</hick:exec>
<hick:exec container="intruder" mount="shared:/in">cat in/file.txt</hick:exec>"#,
    );

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
    let src = hick_doc(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">printf secret > out/file.txt</hick:exec>
<hick:exec container="reader" mount="shared:/in">cat in/file.txt</hick:exec>"#,
    );

    let result = run(&src).await.expect("the reader was granted read access");
    assert_eq!(result.files.get("result/file.txt").unwrap(), "secret");
}

#[tokio::test]
async fn a_partial_read_grant_hands_over_only_that_part() {
    // `read="public/**"` must mean the container never sees the rest, not
    // that it is trusted to ignore it. The reader asserts the absence
    // itself, so a leak fails the exec.
    let src = hick_doc(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="public/**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">mkdir -p out/public && printf open > out/public/ok.txt && printf closed > out/private.txt</hick:exec>
<hick:exec container="reader" mount="shared:/in">cat in/public/ok.txt && test ! -f in/private.txt</hick:exec>"#,
    );

    let result = run(&src)
        .await
        .expect("the reader saw a file outside its read grant");
    // The volume itself still has both: the grant narrowed the copy handed
    // to the reader, it did not delete anything.
    assert_eq!(result.files.get("result/private.txt").unwrap(), "closed");
    assert_eq!(result.files.get("result/public/ok.txt").unwrap(), "open");
}

#[tokio::test]
async fn a_read_only_container_cannot_change_the_volume() {
    // Write access is not "whoever mounted it last wins".
    let src = hick_doc(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" output="result">
  <hick:allow container="writer" write="**" />
  <hick:allow container="reader" read="**" />
</hick:volume>
<hick:exec container="writer" mount="shared:/out">printf original > out/file.txt</hick:exec>
<hick:exec container="reader" mount="shared:/in">printf tampered > in/file.txt</hick:exec>"#,
    );

    let result = run(&src).await.expect("pipeline should complete");
    assert_eq!(
        result.files.get("result/file.txt").unwrap(),
        "original",
        "a read-only container's writes escaped its container"
    );
}

#[tokio::test]
async fn a_partial_writer_changes_only_the_paths_it_was_granted() {
    // The case that used to be enforced as all-or-nothing: `patcher` may
    // rewrite Controllers/ and nothing else. Its other write must not land,
    // and its permitted write must.
    let src = hick_doc(
        r#"<hick:container name="scaffolder" image="alpine" />
<hick:container name="patcher" image="alpine" />
<hick:volume name="project" output="result">
  <hick:allow container="scaffolder" write="**" />
  <hick:allow container="patcher" read="**" />
  <hick:allow container="patcher" write="Controllers/**" />
</hick:volume>
<hick:exec container="scaffolder" mount="project:/p">mkdir -p p/Controllers && printf base > p/Models.cs && printf scaffolded > p/Controllers/Home.cs</hick:exec>
<hick:exec container="patcher" mount="project:/p">printf patched > p/Controllers/Home.cs && printf hacked > p/Models.cs</hick:exec>"#,
    );

    let result = run(&src).await.expect("pipeline should complete");
    assert_eq!(
        result.files.get("result/Controllers/Home.cs").unwrap(),
        "patched",
        "the granted write did not land"
    );
    assert_eq!(
        result.files.get("result/Models.cs").unwrap(),
        "base",
        "the ungranted write landed anyway"
    );
}

#[tokio::test]
async fn a_volume_with_no_allow_children_stays_unrestricted() {
    // Documents written before access rules existed must keep working:
    // rules apply to everyone only once someone is named.
    let src = hick_doc(
        r#"<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:volume name="shared" output="result" />
<hick:exec container="a" mount="shared:/out">printf from-a > out/file.txt</hick:exec>
<hick:exec container="b" mount="shared:/in">cat in/file.txt && printf from-b > in/file.txt</hick:exec>"#,
    );

    let result = run(&src).await.expect("unrestricted volume should work");
    assert_eq!(result.files.get("result/file.txt").unwrap(), "from-b");
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
<hick:exec container="online">printf hi</hick:exec>
<hick:exec container="offline">printf hi</hick:exec>"#,
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
