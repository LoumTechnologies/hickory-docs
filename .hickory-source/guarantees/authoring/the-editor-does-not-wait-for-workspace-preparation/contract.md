<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Editor Does Not Wait For Workspace Preparation

For: Authors editing executable documents and the files those documents produce.

Given a desktop window opens a workspace, when its watcher has been installed
and startup configuration validated, then the window can load before the initial
weave finishes. Settings, folder identity, layout and draft recovery remain
available. An unsaved note can be edited and its recovery draft persisted while
generated files are being prepared.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


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
            "/api/health" | "/api/settings/ui" | "/api/files" | "/api/projects"
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
// Guarantee: docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md
// A real running cell holds initialization open until the test releases it.
// This tests ordering, rather than a launch-time threshold sensitive to CI load.
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn background_attach_serves_editor_storage_before_outputs_are_ready() {
    let project = Project::new();
    let _client = project.client(project.dir.path(), "bootstrap");
    let workspace = tempfile::tempdir().unwrap();
    let signals = tempfile::tempdir().unwrap();
    let started = signals.path().join("started");
    let release = signals.path().join("release");
    let quote = |path: &Path| format!("'{}'", path.display().to_string().replace('\'', "'\\''"));
    let source = format!(
        r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
<hick:container name="sh" image="alpine:3.20" />
<hick:exec container="sh">
printf started > {}
i=0; while [ ! -f {} ]; do i=$((i+1)); [ "$i" -lt 300 ] || exit 1; sleep 0.1; done
printf finished
</hick:exec>
<hick:file path="owned.txt">owned by the document</hick:file>
</hick:doc>
"#,
        quote(&started),
        quote(&release)
    );
    std::fs::write(workspace.path().join("slow.md"), &source).unwrap();
    std::fs::write(workspace.path().join("plain.txt"), "before").unwrap();
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let mut opts = hickory_cli::engine::Attach::new(
        workspace.path().to_path_buf(),
        hickory_cli::ExecutorChoice::Local,
    );
    opts.background = true;
    opts.run = true;
    let response = http
        .post(format!("{url}/attach"))
        .bearer_auth(token)
        .json(&opts)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    wait_file(&started, "started").await;
    let api = format!("{url}/clients/{}/api", opts.client);
    let status: Value = http
        .get(format!("{api}/startup"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["ready"], false);
    let draft = json!({ "path": "untitled:startup", "contents": "Writing while the workspace loads", "base": "" });
    let response = http
        .put(format!("{api}/workspace/drafts"))
        .bearer_auth(token)
        .json(&draft)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let drafts: Value = http
        .get(format!("{api}/workspace/drafts"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(drafts["drafts"][0]["contents"], draft["contents"]);
    for path in ["settings/ui", "workspace/ui", "files"] {
        assert_eq!(
            http.get(format!("{api}/{path}"))
                .bearer_auth(token)
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    let edit = json!({ "path": "plain.txt", "content": "after" });
    let response = http
        .put(format!("{api}/file"))
        .bearer_auth(token)
        .json(&edit)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 503);
    assert_eq!(response.headers()["x-hickory-startup"], "pending");
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("plain.txt")).unwrap(),
        "before"
    );
    let response = http
        .get(format!("{api}/file?path=owned.txt"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        503,
        "output reads must wait for ownership too"
    );
    std::fs::write(release, "go").unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let status: Value = http
            .get(format!("{api}/startup"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if status["ready"] == true {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "workspace never finished: {status}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("owned.txt")).unwrap(),
        "owned by the document"
    );
    let response = http
        .put(format!("{api}/file"))
        .bearer_auth(token)
        .json(&edit)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("plain.txt")).unwrap(),
        "after"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("slow.md")).unwrap(),
        source
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

</storycode:doc>
