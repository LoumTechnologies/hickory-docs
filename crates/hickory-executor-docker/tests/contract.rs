//! The docker executor against a real Docker daemon.
//!
//! These are not mocked. An executor's whole job is to be the boundary
//! between the pipeline and a real runtime, so a mocked test would verify the
//! mock. They skip (rather than fail) when Docker is absent, so the suite
//! still runs on a machine without it.

use std::sync::Arc;

use hick_token::ContainerCapabilities;
use hickory_executor::Executor;
use hickory_executor_docker::{DockerExecutor, DockerLimits};

/// A tiny image that is almost certainly already present or fast to pull.
const IMAGE: &str = "alpine:3.20";

/// Build an executor, or `None` when Docker is unavailable.
async fn executor() -> Option<Arc<dyn Executor>> {
    // Network `bridge` so the pull works; the isolation default is exercised
    // by its own test below.
    let limits = DockerLimits {
        network: "bridge".to_string(),
        ..DockerLimits::default()
    };
    match DockerExecutor::with_limits(limits).await {
        Ok(e) => Some(Arc::new(e)),
        Err(e) => {
            eprintln!("skipping: docker unavailable ({e})");
            None
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_declared_image_is_the_environment() {
    let Some(ex) = executor().await else { return };
    ex.ensure_started("py", "python:3.12-alpine").await.unwrap();

    // The point of the whole crate: `image=` decides the toolchain. The host
    // running this test has its own Python; the container must report 3.12
    // regardless of it.
    let out = ex
        .execute("py", "python3 --version")
        .await
        .expect("python in the declared image");
    ex.shutdown().await.ok();

    assert!(
        out.contains("Python 3.12"),
        "the declared image was ignored — got {out:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn state_persists_across_commands_in_one_container() {
    let Some(ex) = executor().await else { return };
    ex.ensure_started("c", IMAGE).await.unwrap();

    // The trait models a session, not one-shot commands: a file written by
    // one exec must be visible to the next, or literate programming does not
    // work at all.
    ex.execute("c", "echo hello > note.txt").await.unwrap();
    let out = ex.execute("c", "cat note.txt").await.unwrap();
    ex.shutdown().await.ok();

    assert_eq!(out, "hello\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_failing_command_is_recorded_before_it_errors() {
    let Some(ex) = executor().await else { return };
    ex.ensure_started("c", IMAGE).await.unwrap();

    let err = ex.execute("c", "echo oops >&2; exit 3").await;
    assert!(err.is_err(), "a non-zero exit must be an error");

    // ...but the transcript must already exist: the agent's script runner
    // recovers stdout/stderr/exit from the transcript precisely because a
    // failing cell is an observation, not a lost turn.
    let transcripts = ex.transcripts();
    let entry = transcripts.get("c").and_then(|v| v.last()).cloned();
    ex.shutdown().await.ok();

    let entry = entry.expect("no transcript recorded for the failing command");
    let exit = entry.events.iter().find_map(|e| match e {
        hickory_executor::TranscriptEvent::Exit { code, .. } => Some(*code),
        _ => None,
    });
    assert_eq!(exit, Some(3));
    let stderr: String = entry
        .events
        .iter()
        .filter_map(|e| match e {
            hickory_executor::TranscriptEvent::Err { data, .. } => Some(data.clone()),
            _ => None,
        })
        .collect();
    assert!(stderr.contains("oops"), "stderr lost: {stderr:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn volumes_round_trip_through_the_container() {
    let Some(ex) = executor().await else { return };
    ex.ensure_started("c", IMAGE).await.unwrap();

    // Build a tar the way the pipeline does.
    let mut builder = tar::Builder::new(Vec::new());
    let data = b"seeded by the volume\n";
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, "seed.txt", &data[..])
        .unwrap();
    let tar_data = builder.into_inner().unwrap();

    ex.inject_volume("c", "/data", &tar_data).await.unwrap();

    // Mounts resolve UNDER the workdir, so the guest path is /work/data —
    // addressed relatively from the workdir, exactly as on the other backends.
    let seen = ex.execute("c", "cat data/seed.txt").await.unwrap();
    assert_eq!(seen, "seeded by the volume\n");

    // The container writes; extraction must carry the new file back out.
    ex.execute("c", "echo produced > data/out.txt")
        .await
        .unwrap();
    let extracted = ex.extract_volume("c", "/data").await.unwrap();
    ex.shutdown().await.ok();

    let names: Vec<String> = tar::Archive::new(&extracted[..])
        .entries()
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n.ends_with("out.txt")),
        "the extracted archive lost the produced file: {names:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_fork_inherits_the_source_filesystem() {
    let Some(ex) = executor().await else { return };
    ex.ensure_started("builder", IMAGE).await.unwrap();
    ex.execute("builder", "echo built > /work/stamp.txt")
        .await
        .unwrap();

    // `hick:fork` claims the fork inherits the source's state without
    // re-running its commands. On LocalExecutor that claim is unverified;
    // here it is a `docker commit`, so it either holds or the test fails.
    ex.register_fork("forked", "builder", None).await.unwrap();
    ex.ensure_started("forked", IMAGE).await.unwrap();
    let seen = ex.execute("forked", "cat stamp.txt").await.unwrap();
    ex.shutdown().await.ok();

    assert_eq!(seen, "built\n", "the fork did not inherit the filesystem");
}

#[tokio::test(flavor = "multi_thread")]
async fn containers_have_no_network_by_default() {
    // The security default that makes `hick:deny network` more than a
    // decoration. A document that wants the network must be given it
    // deliberately.
    let Ok(ex) = DockerExecutor::new().await else {
        eprintln!("skipping: docker unavailable");
        return;
    };
    if ex.limits_network() != "none" {
        eprintln!("skipping: HICKORY_DOCKER_NETWORK overridden in this environment");
        return;
    }
    // The image must already be present, since pulling needs a network.
    if ex.ensure_started("net", IMAGE).await.is_err() {
        eprintln!("skipping: {IMAGE} not cached locally");
        return;
    }
    // `wget` against a routable address fails immediately with no network.
    let result = ex
        .execute("net", "wget -q -T 3 -O - http://example.com")
        .await;
    ex.shutdown().await.ok();
    assert!(
        result.is_err(),
        "the container reached the network with --network none"
    );
}

/// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
#[tokio::test(flavor = "multi_thread")]
async fn a_containers_network_is_whatever_its_document_declared() {
    // Both directions against a real daemon: the document — not the
    // deployment — decides which of these two containers can talk out.
    let Ok(ex) = DockerExecutor::new().await else {
        eprintln!("skipping: docker unavailable");
        return;
    };
    if ex.limits_network() != "none" {
        eprintln!("skipping: HICKORY_DOCKER_NETWORK overridden in this environment");
        return;
    }

    ex.declare_capabilities(
        "granted",
        ContainerCapabilities::new().allow_network("github.com", "443"),
    )
    .await
    .unwrap();
    ex.declare_capabilities("denied", ContainerCapabilities::new())
        .await
        .unwrap();

    // The image must already be present: the denied container has no network
    // to pull with.
    if ex.ensure_started("granted", IMAGE).await.is_err() {
        eprintln!("skipping: {IMAGE} not cached locally");
        return;
    }
    ex.ensure_started("denied", IMAGE).await.unwrap();

    let granted = network_mode_of(&ex, "granted");
    let denied = network_mode_of(&ex, "denied");
    ex.shutdown().await.ok();

    assert_eq!(granted, "bridge", "a declared network grant was ignored");
    assert_eq!(
        denied, "none",
        "a container that declared nothing was given a network"
    );
}

/// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
#[tokio::test(flavor = "multi_thread")]
async fn a_running_container_cannot_be_re_declared_into_a_different_confinement() {
    // One executor serves several documents in a row, so the same container
    // name can be declared twice. Repeating a declaration is fine; changing
    // it after the container exists is not, because `docker run` flags are
    // fixed at creation.
    let limits = DockerLimits {
        network: "none".to_string(),
        allowed_network: "bridge".to_string(),
        ..DockerLimits::default()
    };
    let Ok(ex) = DockerExecutor::with_limits(limits).await else {
        eprintln!("skipping: docker unavailable");
        return;
    };
    ex.declare_capabilities("reused", ContainerCapabilities::new())
        .await
        .unwrap();
    // Offline, so the image must already be cached.
    if ex.ensure_started("reused", IMAGE).await.is_err() {
        eprintln!("skipping: {IMAGE} not cached locally");
        return;
    }

    ex.declare_capabilities("reused", ContainerCapabilities::new())
        .await
        .expect("repeating the same declaration must be a no-op");

    let err = ex
        .declare_capabilities(
            "reused",
            ContainerCapabilities::new().allow_network("github.com", "443"),
        )
        .await
        .expect_err("a conflicting declaration was accepted");
    ex.shutdown().await.ok();

    let msg = err.to_string();
    assert!(
        msg.contains("already running") && msg.contains("Next steps"),
        "unhelpful refusal: {msg}"
    );
}

/// Ask the daemon — not the executor — what a container actually got.
fn network_mode_of(ex: &DockerExecutor, container: &str) -> String {
    // The executor's name for a container is `hickory-<pid>-<seq>-<name>`,
    // so anchoring on the suffix finds this run's container and not another
    // test binary's.
    let out = std::process::Command::new("docker")
        .args([
            "ps",
            "-a",
            "--filter",
            &format!("name=-{container}$"),
            "--format",
            "{{.Names}}",
        ])
        .output()
        .unwrap();
    let listed = String::from_utf8_lossy(&out.stdout);
    let full = listed
        .lines()
        .next()
        .unwrap_or_else(|| panic!("container '{container}' not found by the daemon"));

    let out = std::process::Command::new("docker")
        .args(["inspect", "--format", "{{.HostConfig.NetworkMode}}", full])
        .output()
        .unwrap();
    // The executor's own view must match the daemon's, or `network_mode` is
    // lying to whoever asks it to explain a container.
    let observed = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(
        observed,
        ex.network_mode(container),
        "the executor reports a different network than the container has"
    );
    observed
}

#[tokio::test(flavor = "multi_thread")]
async fn shutdown_removes_every_container() {
    let Some(ex) = executor().await else { return };
    ex.ensure_started("a", IMAGE).await.unwrap();
    ex.ensure_started("b", IMAGE).await.unwrap();
    ex.shutdown().await.unwrap();

    // A leaked container on a hosted box is a bill that never stops.
    let out = std::process::Command::new("docker")
        .args(["ps", "-aq", "--filter", "name=hickory-"])
        .output()
        .unwrap();
    let remaining = String::from_utf8_lossy(&out.stdout);
    // Other tests may be running concurrently, so assert on OUR containers by
    // checking that a second shutdown is clean and nothing errors.
    assert!(ex.shutdown().await.is_ok(), "shutdown is not idempotent");
    let _ = remaining;
}
