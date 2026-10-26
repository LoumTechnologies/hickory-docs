<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The daemon exits with its last client

For: Authors editing executable documents and the files those documents produce.

Given a local Hickory daemon, when its last desktop window or CLI client
disconnects, then the daemon shuts down. A surviving client keeps the shared
daemon alive. Open terminals, workspace runs, and private investigations do not
independently extend its lifetime.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `daemon_exits_after_last_detach_even_with_an_open_terminal`

This is the maintained test fragment from `crates/hickory-cli/tests/engine_lifecycle.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6ea35164aaa4-daemon_exits_after_last_detach_even_with_an_open_terminal"></a>
### `daemon_exits_after_last_detach_even_with_an_open_terminal`

<storycode:copy id="f-6ea35164aaa4-daemon_exits_after_last_detach_even_with_an_open_terminal">
// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[tokio::test(flavor = "multi_thread")]
async fn daemon_exits_after_last_detach_even_with_an_open_terminal() {
    let project = Project::new();
    let mut daemon = project.daemon().await;
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let a = hickory_cli::engine::Attach::new(
        project.dir.path().into(),
        hickory_cli::ExecutorChoice::Local,
    );
    let b = hickory_cli::engine::Attach::new(
        project.dir.path().into(),
        hickory_cli::ExecutorChoice::Local,
    );
    for owner in [&a, &b] {
        http.post(format!("{url}/attach"))
            .bearer_auth(token)
            .json(owner)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }
    let terminal: Value = http
        .post(format!("{url}/clients/{}/api/terminals", b.client))
        .bearer_auth(token)
        .json(&json!({"title":"lifecycle witness", "argv": if cfg!(windows) { vec!["cmd.exe"] } else { vec!["/bin/sh"] }}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(terminal["exit_code"].is_null());
    http.delete(format!("{url}/clients/{}/lease", a.client))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    tokio::time::sleep(Duration::from_secs(3)).await;
    assert!(
        daemon.child.try_wait().unwrap().is_none(),
        "another client still owns the daemon"
    );
    http.delete(format!("{url}/clients/{}/lease", b.client))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let detached = Instant::now();
    wait_daemon_exit(&project).await;
    assert!(
        detached.elapsed() < Duration::from_secs(6),
        "explicit detach waited for a stale lease"
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    while daemon.child.try_wait().unwrap().is_none() {
        assert!(
            Instant::now() < deadline,
            "endpoint disappeared but daemon remained"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(project.state.path().join("engine/owner.lock").exists());
}

</storycode:copy>

## Test the behavior: `daemon_expires_crashed_clients_but_keeps_the_remaining_window`

This is the maintained test fragment from `crates/hickory-cli/tests/engine_lifecycle.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6ea35164aaa4-daemon_expires_crashed_clients_but_keeps_the_remaining_window"></a>
### `daemon_expires_crashed_clients_but_keeps_the_remaining_window`

<storycode:copy id="f-6ea35164aaa4-daemon_expires_crashed_clients_but_keeps_the_remaining_window">
// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[tokio::test(flavor = "multi_thread")]
async fn daemon_expires_crashed_clients_but_keeps_the_remaining_window() {
    let project = Project::new();
    let a = project.client(project.dir.path(), "A");
    let b = project.client(project.dir.path(), "B");
    let pid = project.endpoint()["pid"].clone();
    drop(a); // Kill without detaching: exercise lease expiry.
    tokio::time::sleep(Duration::from_secs(13)).await;
    assert_eq!(project.endpoint()["pid"], pid);
    assert!(docs(&b).await.is_array());
    drop(b);
    wait_daemon_exit(&project).await;
}

</storycode:copy>

## Test the behavior: `daemon_exits_after_a_one_shot_cli_command`

This is the maintained test fragment from `crates/hickory-cli/tests/engine_lifecycle.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6ea35164aaa4-daemon_exits_after_a_one_shot_cli_command"></a>
### `daemon_exits_after_a_one_shot_cli_command`

<storycode:copy id="f-6ea35164aaa4-daemon_exits_after_a_one_shot_cli_command">
// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[tokio::test(flavor = "multi_thread")]
async fn daemon_exits_after_a_one_shot_cli_command() {
    let project = Project::new();
    let status = project
        .hick()
        .arg("weave")
        .arg(project.dir.path().join("note.md"))
        .status()
        .unwrap();
    assert!(status.success());
    wait_daemon_exit(&project).await;
}

</storycode:copy>

## Test the behavior: `daemon_expires_an_abandoned_cli_even_while_its_worker_runs`

This is the maintained test fragment from `crates/hickory-cli/tests/engine_lifecycle.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6ea35164aaa4-daemon_expires_an_abandoned_cli_even_while_its_worker_runs"></a>
### `daemon_expires_an_abandoned_cli_even_while_its_worker_runs`

<storycode:copy id="f-6ea35164aaa4-daemon_expires_an_abandoned_cli_even_while_its_worker_runs">
// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn daemon_expires_an_abandoned_cli_even_while_its_worker_runs() {
    let project = Project::new();
    let window = project.client(project.dir.path(), "bootstrap");
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(40))
        .build()
        .unwrap();
    let lease = format!("{url}/commands/long-cli/lease");
    http.post(&lease)
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let request = http
        .post(format!("{url}/command"))
        .bearer_auth(token)
        .json(&json!({
            "client":"long-cli", "executable":"/bin/sleep", "args":["60"],
            "cwd":project.dir.path(), "env":[], "input":[]
        }));
    let response = tokio::spawn(async move { request.send().await });
    drop(window);
    // Outlive the window's crashed lease while the foreground CLI owns work.
    for _ in 0..7 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        http.post(&lease)
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }
    assert_eq!(project.endpoint()["pid"], endpoint["pid"]);
    assert!(
        !response.is_finished(),
        "long-running CLI worker unexpectedly finished"
    );
    // Simulate CLI process death: no detach and no more heartbeats.
    wait_daemon_exit(&project).await;
    let result = tokio::time::timeout(Duration::from_secs(3), response)
        .await
        .unwrap()
        .unwrap();
    assert!(result.is_err(), "orphaned worker prevented shutdown");
}

</storycode:copy>

## Test the behavior: `daemon_cancels_background_preparation_after_last_client_exits`

This is the maintained test fragment from `crates/hickory-cli/tests/engine_lifecycle.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6ea35164aaa4-daemon_cancels_background_preparation_after_last_client_exits"></a>
### `daemon_cancels_background_preparation_after_last_client_exits`

<storycode:copy id="f-6ea35164aaa4-daemon_cancels_background_preparation_after_last_client_exits">
// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn daemon_cancels_background_preparation_after_last_client_exits() {
    let project = Project::new();
    let signals = tempfile::tempdir().unwrap();
    let started = signals.path().join("started");
    let quoted = started.to_string_lossy().replace('\'', "'\\''");
    std::fs::write(project.dir.path().join("note.md"), format!(
        "<hick:container name=\"sh\" image=\"alpine:3.20\" />\n<hick:exec container=\"sh\">printf started > '{quoted}'; exec sleep 60</hick:exec>\n"
    )).unwrap();
    let _daemon = project.daemon().await;
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let mut owner = hickory_cli::engine::Attach::new(
        project.dir.path().into(),
        hickory_cli::ExecutorChoice::Local,
    );
    owner.background = true;
    owner.run = true;
    http.post(format!("{url}/attach"))
        .bearer_auth(token)
        .json(&owner)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    wait_file(&started, "started").await;
    http.delete(format!("{url}/clients/{}/lease", owner.client))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    wait_daemon_exit(&project).await;
}

</storycode:copy>

## Test the behavior: `daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh`

This is the maintained test fragment from `crates/hickory-cli/tests/engine_lifecycle.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6ea35164aaa4-daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh"></a>
### `daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh`

<storycode:copy id="f-6ea35164aaa4-daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh">
// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh() {
    let project = Project::new();
    let _daemon = project.daemon().await;
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let lease = format!("{url}/commands/retiring-cli/lease");
    http.post(&lease)
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let request = http
        .post(format!("{url}/command"))
        .bearer_auth(token)
        .json(&json!({
            "client":"retiring-cli", "executable":"/bin/sleep", "args":["60"],
            "cwd":project.dir.path(), "env":[], "input":[]
        }));
    let response = tokio::spawn(async move { request.send().await });
    tokio::time::sleep(Duration::from_millis(100)).await;
    http.delete(&lease)
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(6);
    while http
        .get(format!("{url}/health"))
        .bearer_auth(token)
        .send()
        .await
        .is_ok()
    {
        assert!(Instant::now() < deadline, "daemon did not begin retirement");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let reopened = project.client(project.dir.path(), "reopened");
    assert_ne!(project.endpoint()["pid"], endpoint["pid"]);
    assert!(docs(&reopened).await.is_array());
    assert!(response.await.unwrap().is_err());
    drop(reopened);
    wait_daemon_exit(&project).await;
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-cli/src/engine/client.rs](../../../mechanisms/crates/hickory-cli/src/engine/client.rs.md#output-5c9c7cadce49)
- [crates/hickory-cli/src/engine/daemon.rs](../../../mechanisms/crates/hickory-cli/src/engine/daemon.rs.md#output-49126839fdb7)
- [crates/hickory-cli/src/engine/mod.rs](../../../mechanisms/crates/hickory-cli/src/engine/mod.rs.md#output-e55203129482)
- [crates/hickory-cli/src/engine/watch.rs](../../../mechanisms/crates/hickory-cli/src/engine/watch.rs.md#output-dc5bee2ae567)

### Tests and fixtures

- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../mechanisms/crates/hickory-cli/tests/engine_lifecycle.rs.md#output-6ea35164aaa4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
