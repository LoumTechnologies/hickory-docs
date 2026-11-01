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

The earlier startup requirement is retained below as history. The user-directed read-only policy above supersedes its implicit generation behavior.

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

<a id="output-9743c7ba02f8"></a>
## `apps/web/src/api/workspaceStartup.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Editor Does Not Wait For Workspace Preparation](contract.md).

<a id="f-9743c7ba02f8-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9743c7ba02f8-expression_statement">
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler, request } from "./client";
import { StartupNotice } from "../components/StartupNotice";

afterEach(() => { cleanup(); vi.unstubAllGlobals(); vi.useRealTimers(); });

</storycode:copy>
<a id="f-9743c7ba02f8-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9743c7ba02f8-expression_statement-2">
// Guarantee: docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md
it("shares one preparation wait and submits refused writes only after readiness", async () => {
  installMockHandler(null as never);
  let ready!: (response: Response) => void;
  const gate = new Promise<Response>((resolve) => { ready = resolve; });
  let dispatched = false;
  const fetch = vi.fn(async (path: string, _options?: RequestInit) => {
    if (path === "/api/startup") return gate;
    return dispatched ? new Response('{"ok":true}') : new Response('{"error":"preparing"}', {
      status: 503, headers: { "x-hickory-startup": "pending" },
    });
  });
  vi.stubGlobal("fetch", fetch);
  const app = render(<StartupNotice />);
  const payload = { content: "one" };
  const edits = [request("PUT", "/api/file", payload), request("POST", "/api/projects/local/docs", { path: "two.md" })];
  await app.findByText(/You can keep writing your note/);
  expect(fetch.mock.calls.filter(([path]) => path === "/api/startup")).toHaveLength(1);
  expect(fetch.mock.calls.filter(([path]) => path !== "/api/startup")).toHaveLength(2);
  payload.content = "later form edit";
  await act(async () => { dispatched = true; ready(new Response('{"ready":true}')); await Promise.all(edits); });
  expect(fetch.mock.calls.filter(([path]) => path !== "/api/startup")).toHaveLength(4);
  expect(fetch.mock.calls.filter(([path]) => path === "/api/file").map(([, options]) => options?.body)).toEqual(['{"content":"one"}', '{"content":"one"}']);
  expect(app.queryByRole("status")).toBeNull();
});

</storycode:copy>
<a id="f-9743c7ba02f8-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-9743c7ba02f8-expression_statement-3">
it("does not replay a failed mutation without an explicit pre-dispatch marker", async () => {
  installMockHandler(null as never);
  const fetch = vi.fn(async () => new Response('{"error":"save failed"}', { status: 503 }));
  vi.stubGlobal("fetch", fetch);
  await expect(request("PUT", "/api/file", { content: "unsaved" })).rejects.toThrow("save failed");
  expect(fetch).toHaveBeenCalledTimes(1);
});

</storycode:copy>
<a id="f-9743c7ba02f8-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-9743c7ba02f8-expression_statement-4">
it("keeps a preparation failure visible and never submits the blocked write", async () => {
  installMockHandler(null as never);
  const fetch = vi.fn(async (path: string) => path === "/api/startup"
    ? new Response('{"error":"Workspace preparation stopped. Reopen the folder."}', { status: 503 })
    : new Response('{}', { status: 503, headers: { "x-hickory-startup": "pending" } }));
  vi.stubGlobal("fetch", fetch);
  const app = render(<StartupNotice />);
  await act(async () => { await expect(request("PUT", "/api/file", {})).rejects.toThrow("Reopen the folder"); });
  await waitFor(() => expect(app.getByRole("status").textContent).toContain("Reopen the folder"));
  expect(fetch.mock.calls.filter(([path]) => path === "/api/file")).toHaveLength(1);
});

</storycode:copy>
<a id="f-9743c7ba02f8-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-9743c7ba02f8-expression_statement-5">
it("recovers from a hung status probe after an engine restart and replays only pre-dispatch writes", async () => {
  vi.useFakeTimers();
  installMockHandler(null as never);
  let probes = 0, writes = 0;
  let oldSignal: AbortSignal | undefined;
  const fetch = vi.fn((path: string, options?: RequestInit): Promise<Response> => {
    if (path === "/api/startup") {
      if (++probes === 1) {
        oldSignal = options?.signal as AbortSignal;
        // Simulate a stuck old-generation fetch, even if it ignores abort.
        return new Promise(() => {});
      }
      return Promise.resolve(new Response('{"ready":true}'));
    }
    writes++;
    return Promise.resolve(writes === 1
      ? new Response('{}', { status: 503, headers: { "x-hickory-startup": "pending" } })
      : new Response('{"ok":true}'));
  });
  vi.stubGlobal("fetch", fetch);
  const { startupNotice } = await import("./workspaceStartup");
  const saving = request("PUT", "/api/file", { content: "retained bytes" });
  await vi.advanceTimersByTimeAsync(0);
  expect(probes).toBe(1);
  expect(writes).toBe(1);
  expect(startupNotice()).toContain("Preparing generated files");
  await vi.advanceTimersByTimeAsync(5100);
  await expect(saving).resolves.toEqual({ ok: true });
  expect(oldSignal?.aborted).toBe(true);
  expect(probes).toBe(2);
  expect(writes).toBe(2);
  expect(startupNotice()).toBeNull();
  expect(fetch.mock.calls.filter(([path]) => path === "/api/file").map(([, options]) => options?.body))
    .toEqual(['{"content":"retained bytes"}', '{"content":"retained bytes"}']);
});

</storycode:copy>
<a id="f-9743c7ba02f8-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-9743c7ba02f8-expression_statement-6">
it("bounds unreachable status probes, retains an actionable notice, and permits a fresh readiness attempt", async () => {
  vi.useFakeTimers();
  const fetch = vi.fn((): Promise<Response> => new Promise(() => {}));
  vi.stubGlobal("fetch", fetch);
  const { waitForWorkspace, startupNotice } = await import("./workspaceStartup");
  const waiting = waitForWorkspace();
  const rejected = expect(waiting).rejects.toThrow("Reopen the folder to reconnect");
  await vi.advanceTimersByTimeAsync(15200);
  await rejected;
  expect(fetch).toHaveBeenCalledTimes(3);
  expect(startupNotice()).toContain("Cannot reach workspace preparation status");
  fetch.mockImplementation(() => Promise.resolve(new Response('{"ready":true}')));
  await waitForWorkspace();
  expect(startupNotice()).toBeNull();
});

</storycode:copy>
<a id="f-9743c7ba02f8-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-9743c7ba02f8-expression_statement-7">
it("shows source-free preparation progress and an honest long-stage notice until ready", async () => {
  vi.useFakeTimers();
  let ready = false;
  vi.stubGlobal("fetch", vi.fn(async () => new Response(JSON.stringify(ready ? {ready:true} : {
    ready:false,phase:"weaving",completed:12,total:17,document:"sessions/large.md",elapsed_ms:62000,stage_elapsed_ms:31000,
  }))));
  const { waitForWorkspace, startupNotice } = await import("./workspaceStartup");
  const waiting=waitForWorkspace();
  await vi.advanceTimersByTimeAsync(0);
  expect(startupNotice()).toContain("12 of 17 documents finished");
  expect(startupNotice()).toContain("sessions/large.md");
  expect(startupNotice()).toContain("Elapsed 62s");
  expect(startupNotice()).toContain("31s; preparation is still pending, not complete");
  ready=true;
  await vi.advanceTimersByTimeAsync(100);
  await waiting;
  expect(startupNotice()).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/api/workspaceStartup.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/workspaceStartup.test.tsx"><storycode:paste select="#f-9743c7ba02f8-expression_statement"/><storycode:paste select="#f-9743c7ba02f8-expression_statement-2"/><storycode:paste select="#f-9743c7ba02f8-expression_statement-3"/><storycode:paste select="#f-9743c7ba02f8-expression_statement-4"/><storycode:paste select="#f-9743c7ba02f8-expression_statement-5"/><storycode:paste select="#f-9743c7ba02f8-expression_statement-6"/><storycode:paste select="#f-9743c7ba02f8-expression_statement-7"/></storycode:file>

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

Only editor bootstrap/storage may bypass initialization. In particular,
file reads, writes, execution, and collaboration sockets must wait until
generated-file ownership and the reverse-edit baselines are established.

<storycode:copy id="f-4049bfca1169-editor_bootstrap">
/// Only editor bootstrap/storage may bypass initialization. In particular,
/// file reads, writes, execution, and collaboration sockets must wait until
/// generated-file ownership and the reverse-edit baselines are established.
pub(super) fn editor_bootstrap(path: &str, method: &axum::http::Method) -> bool {
    use axum::http::Method;
    matches!(
        (method, path),
        (
            &Method::GET,
            "/api/health" | "/api/settings/ui" | "/api/files" | "/api/projects" | "/api/prediction/state"
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

<storycode:file path="crates/hickory-cli/src/engine/startup.rs"><storycode:paste select="#f-4049bfca1169-Startup"/><storycode:paste select="#f-4049bfca1169-Progress"/><storycode:paste select="#f-4049bfca1169-impl-Startup"/><storycode:paste select="#f-4049bfca1169-editor_bootstrap"/><storycode:paste select="#f-4049bfca1169-tests"/></storycode:file>

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

- [apps/desktop/src-tauri/src/launch_path.rs](../../../mechanisms/apps/desktop/src-tauri/src/launch_path.rs.md#output-9697ca6e6f7a)
- [apps/web/src/App.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-25cc6f821b46)
- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/api/workspaceStartup.ts](../../../mechanisms/apps/web/src/api/workspaceStartup.ts.md#output-073d5b2b2a12)
- [apps/web/src/components/StartupNotice.tsx](../../../mechanisms/apps/web/src/components/StartupNotice.tsx.md#output-9a5486901220)
- [apps/web/src/lib/focusWhenShown.ts](../../../mechanisms/apps/web/src/lib/focusWhenShown.ts.md#output-e9524c90facf)
- [apps/web/src/main.tsx](../../verification/predictions-precede-managed-observation/implementation/apps/web/src/main.tsx.md#output-44decff65943)
- [apps/web/src/views/deferredAgent.tsx](../../../mechanisms/apps/web/src/views/deferredAgent.tsx.md#output-47b9f1579b62)
- [crates/hickory-cli/src/engine/daemon.rs](../../../mechanisms/crates/hickory-cli/src/engine/daemon.rs.md#output-49126839fdb7)
- [crates/hickory-cli/src/engine/startup.rs](contract.md#output-4049bfca1169)
- [crates/hickory-cli/src/engine/watch.rs](../../../mechanisms/crates/hickory-cli/src/engine/watch.rs.md#output-dc5bee2ae567)

### Tests and fixtures

- [apps/web/e2e/startup.spec.ts](../../../mechanisms/apps/web/e2e/startup.spec.ts.md#output-4e63365d81de)
- [apps/web/src/api/workspaceStartup.test.tsx](contract.md#output-9743c7ba02f8)
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
