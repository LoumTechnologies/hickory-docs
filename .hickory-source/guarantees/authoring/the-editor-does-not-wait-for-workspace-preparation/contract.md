<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Editor Does Not Wait For Workspace Preparation

For: Authors editing executable documents and the files those documents produce.

Opening a file or folder observes its documents without generating outputs,
executing cells, draining an inbox or establishing generation baselines. Watcher
events and source saves remain observational. Generation requires a separate
explicit action with a root, entry and captured source revision. Included source
pages never become independent generation roots; missing or ambiguous declarations
fail without a standalone fallback.

Requirement change (2026-11-01, user-directed): this replaces the old startup weave
with read-only opening and explicit project generation. The retained historical
startup details below describe the earlier policy, not authorization to publish
outputs while discovering documents. Explicit standalone cell execution is still
a separate Run action; shared project constructions use the project action.

Requirement clarification (2026-11-15, user-directed): available source text must
remain readable while parsing, ownership and language capabilities initialize or
fail. Read access does not authorize a write or report a save. The direct file and
document-source routes and named non-writing source capabilities bypass preparation;
rendering, execution and source writes do not.

The earlier startup requirement is retained below as history. The user-directed read-only policy above supersedes its implicit generation behavior.

Focused routing evidence (2026-11-15): `source_reads_bypass_pending_and_failed_preparation_without_unlocking_writes` passed for pending and failed preparation, direct raw reads, encoded document identifiers, mutations and capability subroutes. The two `engine::writes::observation_tests` passed for save-gate independence. This is routing/concurrency evidence, not a blocked-startup end-to-end or native-device acceptance claim.

Independent capability routing evidence (2026-11-15): all four focused startup cases and the two write-dispatch observation cases pass. The shared named POST classifier admits source facts, construction reading, resource planning/acquisition/consequences and cleanup during pending or failed preparation, while mutation/execution paths and render/run subroutes remain gated. These are dispatch/concurrency tests, not a forced-pending full-engine or native-device acceptance claim.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

Startup repair evidence (2026-11-01): the two focused `engine_lifecycle` tests
`background_attach_serves_editor_storage_before_outputs_are_ready` and
`file_and_folder_windows_keep_independent_files_visibility` passed in the development
profile. They check the policy probe during blocked preparation, retained read/write
gating, selected-file discovery, sibling saves and explicit folder expansion. The
nine frontend tests in `src/main.test.tsx` and `src/api/workspaceStartup.test.tsx`
also passed. Seven changed output constructions were generated and byte-checked
with the native adapter in an isolated project using their maintained fragments.
At that run, full repository `generate` and `check` were blocked by missing
included source pages and `audit` reported a missing constructor source. The later
cleanup restored those pages from hash-verified tracked outputs; full generation,
source/output checking and ownership audit then passed. This evidence does not claim
a packaged desktop walkthrough or broad regression coverage.



## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-4049bfca1169"></a>
## `crates/hickory-cli/src/engine/startup.rs`

Related contracts: [The Editor Does Not Wait For Workspace Preparation](contract.md).

<a id="f-4049bfca1169-Startup"></a>
### `Startup`

An attached editor can boot while output ownership is being established.

<storycode:copy id="f-4049bfca1169-Startup">
//! An attached editor can boot while output ownership is being established.
use anyhow::{Result, anyhow};
use tokio::sync::watch;

#[derive(Clone)]
pub(super) struct Startup {
    receiver: watch::Receiver<Option<Result<(), String>>>,
    progress: std::sync::Arc<std::sync::Mutex<Progress>>,
    started: std::time::Instant,
}
</storycode:copy>
<a id="f-4049bfca1169-Progress"></a>
### `Progress`

<storycode:copy id="f-4049bfca1169-Progress">
#[derive(Clone)]
struct Progress {
    phase: String,
    completed: usize,
    total: usize,
    document: Option<String>,
    changed: std::time::Instant,
}

</storycode:copy>
<a id="f-4049bfca1169-impl-Startup"></a>
### `impl-Startup`

<storycode:copy id="f-4049bfca1169-impl-Startup">
impl Startup {
    pub fn complete() -> Self {
        let (sender, startup) = Self::channel();
        let _ = sender.send(Some(Ok(())));
        startup
    }
    pub fn result(&self) -> Option<Result<(), String>> {
        self.receiver.borrow().clone().or_else(|| {
            self.receiver
                .has_changed()
                .is_err()
                .then(|| Err("Workspace preparation stopped. Reopen the folder.".into()))
        })
    }

    pub fn channel() -> (watch::Sender<Option<Result<(), String>>>, Self) {
        let (tx, rx) = watch::channel(None);
        let now = std::time::Instant::now();
        (
            tx,
            Self {
                receiver: rx,
                started: now,
                progress: std::sync::Arc::new(std::sync::Mutex::new(Progress {
                    phase: "queued".into(),
                    completed: 0,
                    total: 0,
                    document: None,
                    changed: now,
                })),
            },
        )
    }

    pub fn progress(&self, phase: &str, completed: usize, total: usize, document: Option<String>) {
        *self.progress.lock().unwrap() = Progress {
            phase: phase.into(),
            completed,
            total,
            document,
            changed: std::time::Instant::now(),
        };
    }
    pub fn status(&self) -> serde_json::Value {
        let progress = self.progress.lock().unwrap();
        serde_json::json!({ "ready": false, "phase": progress.phase, "completed": progress.completed,
            "total": progress.total, "document": progress.document,
            "elapsed_ms": self.started.elapsed().as_millis() as u64,
            "stage_elapsed_ms": progress.changed.elapsed().as_millis() as u64 })
    }

    pub async fn wait(&self) -> Result<()> {
        let mut receiver = self.receiver.clone();
        loop {
            if let Some(result) = receiver.borrow_and_update().clone() {
                return result.map_err(|error| anyhow!(error));
            }
            receiver
                .changed()
                .await
                .map_err(|_| anyhow!("Workspace preparation stopped. Reopen the folder."))?;
        }
    }
}

</storycode:copy>
<a id="f-4049bfca1169-editor_bootstrap"></a>
### `editor_bootstrap`

The read-only prediction status decides which editor may mount. It must bypass
weaving just like settings: otherwise the ordinary editor remains a blank opening
screen until every document finishes. The probe reads already-bound policy; it
neither enables private mode nor unlocks execution or file writes.

Raw file reads, direct document-source reads and named non-writing source
capabilities join editor bootstrap/storage before initialization completes. Rendering, execution, writes and collaboration sockets
retain their readiness checks. Matching the direct document route rather than its
prefix keeps capability subroutes behind those checks.

<storycode:copy id="f-4049bfca1169-editor_bootstrap">
/// Raw source and named read-only capabilities remain independent of preparation.
/// Writes, rendering, execution and collaboration retain readiness checks.
pub(super) fn editor_bootstrap(path: &str, method: &axum::http::Method) -> bool {
    use axum::http::Method;
    let direct_document = path
        .strip_prefix("/api/docs/")
        .is_some_and(|id| !id.is_empty() && !id.contains('/'));
    super::writes::post_observation(method, path)
        || (*method == Method::GET && (matches!(path, "/api/file" | "/api/startup-source") || direct_document))
        || matches!(
            (method, path),
            (
                &Method::GET,
                "/api/health"
                    | "/api/settings/ui"
                    | "/api/files"
                    | "/api/projects"
                    | "/api/prediction/state"
                    | "/api/source-history"
                    | "/api/behavior/verification"
                    | "/api/git/conflict-resolution/status"
                    | "/api/changes/resource-recovery"
                    | "/api/changes/resource-recoveries"
            ) | (
                &Method::GET | &Method::PUT | &Method::DELETE,
                "/api/workspace/ui" | "/api/workspace/drafts"
            ) | (
                &Method::POST,
                "/api/window/close" | "/api/save-file-dialog" | "/api/pick-folder"
            )
        )
}
</storycode:copy>
<a id="f-4049bfca1169-tests"></a>
### `tests`

<storycode:copy id="f-4049bfca1169-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn progress_is_shared_without_unlocking_preparation() {
        let (sender, startup) = Startup::channel();
        let observer = startup.clone();
        startup.progress("weaving", 2, 5, Some("sessions/large.md".into()));
        let status = observer.status();
        assert_eq!(status["ready"], false);
        assert_eq!(status["completed"], 2);
        assert_eq!(status["total"], 5);
        assert_eq!(status["document"], "sessions/large.md");
        assert!(status["elapsed_ms"].is_number());
        assert!(observer.result().is_none());
        let waiting = tokio::spawn(async move { observer.wait().await });
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());
        sender.send(Some(Ok(()))).unwrap();
        waiting.await.unwrap().unwrap();
    }

    // Guarantee: docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md
    #[tokio::test]
    async fn a_stopped_preparation_reports_failure_instead_of_waiting_forever() {
        let (sender, startup) = Startup::channel();
        drop(sender);
        assert!(
            startup
                .result()
                .unwrap()
                .unwrap_err()
                .contains("Reopen the folder")
        );
        assert!(startup.wait().await.is_err());
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/engine/startup.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

## Read source without granting capability authority

The policy tests cover pending and failed preparation. Literal direct routes and encoded document identifiers remain readable, while subroutes, mutations and unknown paths stay gated. This tests dispatch classification; it does not claim native device acceptance.

<storycode:copy id="raw-source-startup-tests">
#[cfg(test)]
mod raw_source_tests {
    use super::*;
    use axum::http::Method;

    // Contract: .hickory-source/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md
    #[test]
    fn source_reads_bypass_pending_and_failed_preparation_without_unlocking_writes() {
        let (sender, startup) = Startup::channel();
        for failed in [false, true] {
            if failed {
                sender.send(Some(Err("capability failed".into()))).unwrap();
            }
            assert!(startup.result().is_none_or(|result| result.is_err()));
            for path in [
                "/api/file",
                "/api/startup-source",
                "/api/source-history",
                "/api/behavior/verification",
                "/api/git/conflict-resolution/status",
                "/api/changes/resource-recovery",
                "/api/changes/resource-recoveries",
                "/api/docs/source.md",
                "/api/docs/dir%2Fsource.md",
            ] {
                assert!(editor_bootstrap(path, &Method::GET), "{path}");
                for method in [Method::PUT, Method::POST, Method::DELETE] {
                    assert!(!editor_bootstrap(path, &method), "{method} {path}");
                }
            }
            for path in [
                "/api/docs/",
                "/api/docs/source.md/render",
                "/api/docs/source.md/run",
                "/api/file/extra",
                "/api/startup-source/extra",
                "/api/room/source.md",
            ] {
                assert!(!editor_bootstrap(path, &Method::GET), "{path}");
            }
        }
    }
}
</storycode:copy>

## Observe source independently of preparation readiness

Named non-writing POST capabilities share one classifier with request dispatch. They remain available during pending or failed preparation, while actual writes, test execution and render/run subroutes remain gated. This is focused routing evidence; it does not claim a forced-pending end-to-end startup or native device acceptance.

<storycode:copy id="independent-readonly-capability-tests">
#[cfg(test)]
mod independent_capability_tests {
    use super::*;
    use axum::http::Method;
    // Contract: .hickory-source/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md
    #[test]
    fn readonly_capabilities_do_not_inherit_a_failed_or_slow_preparation() {
        let (sender, startup) = Startup::channel();
        for failed in [false, true] {
            if failed {
                sender
                    .send(Some(Err("unrelated capability failed".into())))
                    .unwrap();
            }
            assert!(startup.result().is_none_or(|result| result.is_err()));
            for path in [
                "/api/tests/source-candidates",
                "/api/source-history/restore-draft",
                "/api/behavior/reading",
                "/api/behavior/source",
                "/api/behavior/profiles",
                "/api/git/conflict-resolution/read",
                "/api/git/conflict-resolution/prepare",
                "/api/git/conflict-resolution/validate",
                "/api/tests/debug-plan",
                "/api/construction/reading",
                "/api/changes/plan-resources",
                "/api/changes/resource-targets",
                "/api/changes/resource-inputs",
                "/api/changes/cleanup",
                "/api/changes/resource-consequences",
                "/api/changes/prepare-restoration",
            ] {
                assert!(editor_bootstrap(path, &Method::POST), "{path}");
                assert!(!editor_bootstrap(path, &Method::PUT), "{path}");
            }
            for path in [
                "/api/tests/run-source",
                "/api/changes/publish-resources",
                "/api/changes/resource-save",
                "/api/changes/restore-resources",
                "/api/file",
                "/api/files/op",
                "/api/docs/source/run",
                "/api/construction/reading/extra",
            ] {
                assert!(!editor_bootstrap(path, &Method::POST), "{path}");
            }
            for path in ["/api/docs/source/render", "/api/docs/source/run"] {
                assert!(!editor_bootstrap(path, &Method::GET), "{path}");
            }
        }
    }
}
</storycode:copy>

<storycode:file path="crates/hickory-cli/src/engine/startup.rs"><storycode:paste select="#f-4049bfca1169-Startup"/><storycode:paste select="#f-4049bfca1169-Progress"/><storycode:paste select="#f-4049bfca1169-impl-Startup"/><storycode:paste select="#f-4049bfca1169-editor_bootstrap"/><storycode:paste select="#f-4049bfca1169-tests"/><storycode:paste select="#raw-source-startup-tests"/><storycode:paste select="#independent-readonly-capability-tests"/></storycode:file>

## Test the behavior: `background_attach_serves_editor_storage_before_outputs_are_ready`

This is the maintained test fragment from `crates/hickory-cli/tests/engine_lifecycle.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6ea35164aaa4-background_attach_serves_editor_storage_before_outputs_are_ready"></a>
### `background_attach_serves_editor_storage_before_outputs_are_ready`

A real running cell holds initialization open until the test releases it.
This tests ordering, rather than a launch-time threshold sensitive to CI load.

<storycode:copy id="f-6ea35164aaa4-background_attach_serves_editor_storage_before_outputs_are_ready">
// Opening a folder may discover executable documents, but grants no publication.
#[tokio::test(flavor = "multi_thread")]
async fn background_attach_serves_editor_storage_before_outputs_are_ready() {
    let project = Project::new();
    let workspace = tempfile::tempdir().unwrap();
    let root = workspace.path();
    let sources = root.join(".hickory-source");
    std::fs::create_dir(&sources).unwrap();
    let part = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:copy id=\"body\">one</h:copy><h:file path=\"app.txt\"><h:paste select=\"#body\" /></h:file></h:doc>";
    std::fs::write(sources.join("part.md"), part).unwrap();
    std::fs::write(root.join("project.md"), "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\".hickory-source/part.md\" /></h:doc>").unwrap();
    std::fs::write(sources.join("unselected.md"), DOC).unwrap();
    let window = project.client(root, "observation-window");
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap();
    let api = format!("{}/api", window.url);
    assert!(!root.join("app.txt").exists());
    assert!(!sources.join("app.txt").exists());
    assert!(!sources.join("hello.py").exists());
    assert!(!root.join(".hick-journal").exists());
    let policy = http
        .get(format!("{api}/prediction/state"))
        .send()
        .await
        .unwrap();
    assert_eq!(policy.status(), 422);
    let response = http
        .put(format!("{api}/workspace/drafts"))
        .json(
            &json!({"path":"untitled:observation", "contents":"Writing while browsing", "base":""}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let target = json!({"root":".", "entry":"project.md"});
    // A preview is read-only; a raw included page has no generation context.
    let plan: Value = http
        .post(format!("{api}/generation/preview"))
        .json(&target)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(plan["outputs"], json!(["app.txt"]));
    assert!(!root.join("app.txt").exists());
    let refusal = http
        .post(format!("{api}/generation/preview"))
        .json(&json!({"root":".hickory-source", "entry":"part.md"}))
        .send()
        .await
        .unwrap();
    assert_eq!(refusal.status(), 422);
    assert!(!sources.join("app.txt").exists());
    // The legacy Run action must not publish an included project construction.
    let documents = docs(&window).await;
    let part_id = documents
        .as_array()
        .unwrap()
        .iter()
        .find(|doc| doc["path"] == ".hickory-source/part.md")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    assert_eq!(
        http.post(format!("{api}/docs/{part_id}/run"))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    // A source save is observed, not woven. The previous plan becomes stale.
    std::fs::write(sources.join("part.md"), part.replace("one", "two")).unwrap();
    tokio::time::sleep(Duration::from_millis(450)).await;
    assert!(!root.join("app.txt").exists());
    let response = http
        .post(format!("{api}/generation/generate"))
        .json(&json!({"root":".", "entry":"project.md", "revision":plan["revision"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 409);
    let plan: Value = http
        .post(format!("{api}/generation/preview"))
        .json(&target)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let generation = json!({"root":".", "entry":"project.md", "revision":plan["revision"]});
    let response = http
        .post(format!("{api}/generation/generate"))
        .json(&generation)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert_eq!(
        std::fs::read_to_string(root.join("app.txt")).unwrap(),
        "two"
    );
    assert!(!sources.join("app.txt").exists());
    assert!(!sources.join("hello.py").exists());
    // Output edits survive observation and cannot be erased by generating again.
    std::fs::write(root.join("app.txt"), "external edit").unwrap();
    tokio::time::sleep(Duration::from_millis(450)).await;
    assert_eq!(
        std::fs::read_to_string(root.join("app.txt")).unwrap(),
        "external edit"
    );
    let response = http
        .post(format!("{api}/generation/generate"))
        .json(&generation)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 409);
    assert_eq!(
        std::fs::read_to_string(root.join("app.txt")).unwrap(),
        "external edit"
    );
    assert_eq!(
        std::fs::read_to_string(sources.join("part.md")).unwrap(),
        part.replace("one", "two")
    );
}
</storycode:copy>

## Implementation review

### Implementation

- apps/desktop/src-tauri/src/launch_path.rs
- apps/web/src/App.tsx
- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/api/workspaceStartup.ts](../../../mechanisms/apps/web/src/api/workspaceStartup.ts.md#output-073d5b2b2a12)
- apps/web/src/components/StartupNotice.tsx
- apps/web/src/lib/focusWhenShown.ts
- apps/web/src/main.tsx
- apps/web/src/views/deferredAgent.tsx
- [crates/hickory-cli/src/engine/daemon.rs](../../../mechanisms/crates/hickory-cli/src/engine/daemon.rs.md#output-49126839fdb7)
- [crates/hickory-cli/src/engine/startup.rs](contract.md#output-4049bfca1169)
- [crates/hickory-cli/src/engine/watch.rs](../../../mechanisms/crates/hickory-cli/src/engine/watch.rs.md#output-dc5bee2ae567)

### Tests and fixtures

- apps/web/e2e/startup.spec.ts
- apps/web/src/api/workspaceStartup.test.tsx
- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../mechanisms/crates/hickory-cli/tests/engine_lifecycle.rs.md#output-6ea35164aaa4)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.


## Focused verification of the read-only opening policy

Executed on 2026-11-01: the two generation-context unit tests, two generation-dialog
tests and eleven document-toolbar tests passed. The real-engine opening/generation
flow and multi-note/folder flow passed, as did all four native in-process observer
tests. The headless generation/detach witness passed with a per-process polling
backend; native notification delivery in the detached engine was unreliable on
this host, so that witness does not certify its native backend. Frontend type
checking passed. These are focused behavior/safety checks, not broad regression
or installed-desktop acceptance. The source/output check and ownership audit
cover construction agreement separately.

## An observer cannot inherit a departed generator's authority

An explicit headless subscription may publish a declared project. Detaching it
must leave a remaining app window observational, including after another source
save. The witness checks both the authorized update and the post-detach refusal
to publish, so it does not merely assert that generation is disabled everywhere.

<storycode:copy id="generation-authority-lifetime-test">
/// A headless subscription may generate, but its open observer cannot inherit
/// that authority when the explicitly generating client detaches.
#[tokio::test(flavor = "multi_thread")]
async fn detaching_generation_leaves_observers_read_only() {
    let project = Project::new();
    // Use the same canonical identity as the captured generation context.
    let canonical = project.dir.path().canonicalize().unwrap();
    let root = canonical.as_path();
    std::fs::remove_file(root.join("note.md")).unwrap();
    std::fs::create_dir(root.join(".hickory-source")).unwrap();
    let part = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"app.txt\">one</h:file></h:doc>";
    std::fs::write(root.join(".hickory-source/part.md"), part).unwrap();
    std::fs::write(root.join("project.md"), "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\".hickory-source/part.md\" /></h:doc>").unwrap();
    // This is a publication-authority test, not a native event-delivery test.
    // A per-process polling backend keeps it deterministic without changing
    // process-global environment for the other tests.
    let child = project.hick().arg("__engine")
        .env("HICKORY_WATCH_MODE", "poll").spawn().unwrap();
    let _daemon = Client { child, url: String::new() };
    let deadline = Instant::now() + Duration::from_secs(10);
    while !project.state.path().join("engine/endpoint.json").exists() {
        assert!(Instant::now() < deadline, "polling engine did not start");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let observer = project.client(root, "observer");
    assert!(!root.join("app.txt").exists());
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let mut options =
        hickory_cli::engine::Attach::new(root.to_path_buf(), hickory_cli::ExecutorChoice::Local);
    options.generation_entry = Some("project.md".into());
    let response = http
        .post(format!("{url}/attach"))
        .bearer_auth(token)
        .json(&options)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    wait_file(&root.join("app.txt"), "one").await;
    std::fs::write(
        root.join(".hickory-source/part.md"),
        part.replace("one", "two"),
    )
    .unwrap();
    wait_file(&root.join("app.txt"), "two").await;
    let response = http
        .delete(format!("{url}/clients/{}/lease", options.client))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 204);
    std::fs::write(
        root.join(".hickory-source/part.md"),
        part.replace("one", "three"),
    )
    .unwrap();
    std::fs::write(root.join("after-detach.md"), "Still observing").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let listed = docs(&observer).await;
        if listed.as_array().unwrap().iter().any(|doc| doc["path"] == "after-detach.md") { break; }
        assert!(Instant::now() < deadline, "remaining observer stopped discovering files");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    // Preview shares the write act, so it also waits for that event batch to
    // finish. It must observe the new source without publishing its outputs.
    let response = http.post(format!("{}/api/generation/preview", observer.url))
        .json(&json!({"root":".", "entry":"project.md"})).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(std::fs::read_to_string(root.join("app.txt")).unwrap(), "two");
    assert!(!root.join(".hickory-source/app.txt").exists());
}
</storycode:copy>
</storycode:doc>
