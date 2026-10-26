<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Only `hickory-executor-canopy` Knows Cloud Canopy's API

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given that Cloud Canopy's API is under concurrent development, when any code
in this repository talks to canopy (gRPC, sandbox channel protocol, image
path mapping), then that code lives in `crates/hickory-executor-canopy`
and nowhere else; all other code depends only on the `Executor` trait in
`crates/hickory-executor`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/canopy-api-isolated-to-one-crate.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-5017d9438178"></a>
## `crates/hickory-executor-canopy/src/lib.rs`

Related contracts: [Only `hickory-executor-canopy` Knows Cloud Canopy's API](contract.md).

<a id="f-5017d9438178-config"></a>
### `config`

Cloud Canopy executor for Hickory Docs.
This crate is the ONLY code in the repository that knows Cloud Canopy's
API (see `docs/guarantees/execution/canopy-api-isolated-to-one-crate.md`).
The entire coupling is the vendored `proto/canopy.proto` — never add a
path or git dependency on a cloud-canopy crate.
See [`CanopyExecutor`] for runtime semantics and
`docs/specs/freeform/canopy-integration.md` for the env contract.

<storycode:copy id="f-5017d9438178-config">
//! Cloud Canopy executor for Hickory Docs.
//!
//! This crate is the ONLY code in the repository that knows Cloud Canopy's
//! API (see `docs/guarantees/execution/canopy-api-isolated-to-one-crate.md`).
//! The entire coupling is the vendored `proto/canopy.proto` — never add a
//! path or git dependency on a cloud-canopy crate.
//!
//! See [`CanopyExecutor`] for runtime semantics and
//! `docs/specs/freeform/canopy-integration.md` for the env contract.

mod config;
</storycode:copy>
<a id="f-5017d9438178-executor"></a>
### `executor`

<storycode:copy id="f-5017d9438178-executor">
mod executor;
</storycode:copy>
<a id="f-5017d9438178-frame"></a>
### `frame`

<storycode:copy id="f-5017d9438178-frame">
mod frame;

</storycode:copy>
<a id="f-5017d9438178-pb"></a>
### `pb`

Generated gRPC bindings for the vendored `proto/canopy.proto`.
Public so the in-crate contract tests can implement a mock
`CanopyAgent` server against the exact wire contract. Nothing outside
this crate may use it (enforced by `tests/isolation.rs`).

<storycode:copy id="f-5017d9438178-pb">
/// Generated gRPC bindings for the vendored `proto/canopy.proto`.
///
/// Public so the in-crate contract tests can implement a mock
/// `CanopyAgent` server against the exact wire contract. Nothing outside
/// this crate may use it (enforced by `tests/isolation.rs`).
#[allow(clippy::all, clippy::pedantic)]
pub mod pb {
    tonic::include_proto!("canopy.v1");
}

</storycode:copy>
<a id="f-5017d9438178-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-5017d9438178-use_declaration">
pub use config::{CanopyConfig, image_map_summary};
</storycode:copy>
<a id="f-5017d9438178-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-5017d9438178-use_declaration-2">
pub use executor::CanopyExecutor;

</storycode:copy>
<a id="f-5017d9438178-use_declaration-3"></a>
### `use_declaration`

Exposed for the in-crate contract tests' mock guest; not a public API.

<storycode:copy id="f-5017d9438178-use_declaration-3">
// Exposed for the in-crate contract tests' mock guest; not a public API.
#[doc(hidden)]
pub use frame::{
    FrameItem, FrameParser, READY_BANNER, SENTINEL_END, SENTINEL_START, decode_script,
    encode_script, exec_line, shell_quote, strip_ansi,
};
</storycode:copy>

### Assemble `crates/hickory-executor-canopy/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-canopy/src/lib.rs"><storycode:paste select="#f-5017d9438178-config"/><storycode:paste select="#f-5017d9438178-executor"/><storycode:paste select="#f-5017d9438178-frame"/><storycode:paste select="#f-5017d9438178-pb"/><storycode:paste select="#f-5017d9438178-use_declaration"/><storycode:paste select="#f-5017d9438178-use_declaration-2"/><storycode:paste select="#f-5017d9438178-use_declaration-3"/></storycode:file>

<a id="output-e998bd7eb08e"></a>
## `crates/hickory-executor-canopy/tests/contract.rs`

Related contracts: [Only `hickory-executor-canopy` Knows Cloud Canopy's API](contract.md).

<a id="f-e998bd7eb08e-TOKEN_KEY"></a>
### `TOKEN_KEY`

Contract test: the full `CanopyExecutor` round-trip against an in-crate
mock gRPC server implementing SpawnSandbox / AttachSandbox /
DestroySandbox with a fake pty-echoing guest.
The mock reproduces the guest channel's observable behaviour — ready
banner, pty echo of every input line, bracketed-paste escapes around the
sentinels, `\r\n` line endings, merged stdout/stderr — and runs the
decoded scripts with a real `/bin/sh` (each sandbox mapped to its own
host temp dir), so output, state accumulation, and exit codes are real.
Protects: docs/guarantees/execution/canopy-api-isolated-to-one-crate.md
(the vendored proto is exercised end-to-end from this crate alone).

<storycode:copy id="f-e998bd7eb08e-TOKEN_KEY">
//! Contract test: the full `CanopyExecutor` round-trip against an in-crate
//! mock gRPC server implementing SpawnSandbox / AttachSandbox /
//! DestroySandbox with a fake pty-echoing guest.
//!
//! The mock reproduces the guest channel's observable behaviour — ready
//! banner, pty echo of every input line, bracketed-paste escapes around the
//! sentinels, `\r\n` line endings, merged stdout/stderr — and runs the
//! decoded scripts with a real `/bin/sh` (each sandbox mapped to its own
//! host temp dir), so output, state accumulation, and exit codes are real.
//!
//! Protects: docs/guarantees/execution/canopy-api-isolated-to-one-crate.md
//! (the vendored proto is exercised end-to-end from this crate alone).

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hickory_executor::{Executor, TranscriptEvent};
use hickory_executor_canopy::{CanopyConfig, CanopyExecutor, decode_script, pb};
use pb::canopy_agent_server::{CanopyAgent, CanopyAgentServer};
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

const TOKEN_KEY: &str = "x-canopy-capability-bin";

</storycode:copy>
<a id="f-e998bd7eb08e-MockState"></a>
### `MockState`

<storycode:copy id="f-e998bd7eb08e-MockState">
#[derive(Default)]
struct MockState {
    spawned: Vec<pb::SpawnSandboxRequest>,
    destroyed: Vec<String>,
    dirs: HashMap<String, std::path::PathBuf>,
}

</storycode:copy>
<a id="f-e998bd7eb08e-MockAgent"></a>
### `MockAgent`

<storycode:copy id="f-e998bd7eb08e-MockAgent">
struct MockAgent {
    state: Arc<Mutex<MockState>>,
    root: Arc<tempfile::TempDir>,
    /// When set, every RPC must carry exactly these token bytes.
    expected_token: Option<Vec<u8>>,
}

</storycode:copy>
<a id="f-e998bd7eb08e-impl-MockAgent"></a>
### `impl-MockAgent`

<storycode:copy id="f-e998bd7eb08e-impl-MockAgent">
impl MockAgent {
    // tonic's Status is what the server trait returns; its size is not ours
    // to shrink in a mock.
    #[allow(clippy::result_large_err)]
    fn check_token<T>(&self, req: &Request<T>) -> Result<(), Status> {
        if let Some(expected) = &self.expected_token {
            let got = req
                .metadata()
                .get_bin(TOKEN_KEY)
                .ok_or_else(|| Status::unauthenticated("missing capability token"))?
                .to_bytes()
                .map_err(|_| Status::unauthenticated("malformed capability token"))?;
            if got.as_ref() != expected.as_slice() {
                return Err(Status::permission_denied("wrong capability token"));
            }
        }
        Ok(())
    }
}

</storycode:copy>
<a id="f-e998bd7eb08e-run_fake_guest"></a>
### `run_fake_guest`

The fake guest: pty semantics over the attach stream.

<storycode:copy id="f-e998bd7eb08e-run_fake_guest">
/// The fake guest: pty semantics over the attach stream.
async fn run_fake_guest(
    workdir: std::path::PathBuf,
    first_payload: Vec<u8>,
    mut inbound: Streaming<pb::AttachSandboxRequest>,
    tx: tokio::sync::mpsc::Sender<Result<pb::AttachSandboxChunk, Status>>,
) {
    let send = |tx: tokio::sync::mpsc::Sender<Result<pb::AttachSandboxChunk, Status>>,
                s: String| async move {
        let _ = tx
            .send(Ok(pb::AttachSandboxChunk {
                data: s.into_bytes(),
            }))
            .await;
    };

    // Connection handed to a shell: banner, then a bracketed-paste prompt.
    send(
        tx.clone(),
        "\u{1b}[?2004hcanopy-guest ready\r\n$ ".to_string(),
    )
    .await;

    let mut buf: Vec<u8> = first_payload;
    loop {
        // Process complete input lines.
        while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = buf.drain(..=nl).collect();
            let line = String::from_utf8_lossy(&line_bytes).trim_end().to_string();
            // Pty echo: the input comes back, escape-wrapped, CRLF-ended.
            send(tx.clone(), format!("\u{1b}[?2004l{line}\r\n")).await;

            // Emulate `echo __CANOPY_S__; echo <b64> | base64 -d | /bin/sh;
            // echo __CANOPY_E__$?` the way the guest shell would.
            let Some(b64) = line
                .split("echo ")
                .nth(2)
                .and_then(|s| s.split(" | base64 -d").next())
            else {
                send(tx.clone(), "sh: syntax error\r\n$ ".to_string()).await;
                continue;
            };
            let script = decode_script(b64).expect("client sends valid base64");
            // The mock guest has no VM: map its absolute workdir onto a
            // per-sandbox host directory.
            let script = script.replace("/hickory-work", &workdir.to_string_lossy());

            send(tx.clone(), "\u{1b}[?2004l__CANOPY_S__\r\n".to_string()).await;
            let out = tokio::task::spawn_blocking(move || {
                use std::io::Write as _;
                use std::process::{Command, Stdio};
                let mut child = Command::new("/bin/sh")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("spawn sh");
                // `exec 2>&1` merges stderr into stdout, as a pty would.
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(format!("exec 2>&1\n{script}").as_bytes())
                    .unwrap();
                child.wait_with_output().expect("wait sh")
            })
            .await
            .expect("script task");
            let code = out.status.code().unwrap_or(1);
            let text = String::from_utf8_lossy(&out.stdout).replace('\n', "\r\n");
            send(tx.clone(), text).await;
            send(tx.clone(), format!("__CANOPY_E__{code}\r\n$ ")).await;
        }
        match inbound.next().await {
            Some(Ok(msg)) => buf.extend_from_slice(&msg.data),
            _ => break,
        }
    }
}

</storycode:copy>
<a id="f-e998bd7eb08e-impl-CanopyAgent-for-MockAgent"></a>
### `impl-CanopyAgent for MockAgent`

<storycode:copy id="f-e998bd7eb08e-impl-CanopyAgent-for-MockAgent">
#[tonic::async_trait]
impl CanopyAgent for MockAgent {
    async fn spawn_sandbox(
        &self,
        req: Request<pb::SpawnSandboxRequest>,
    ) -> Result<Response<pb::SpawnSandboxResponse>, Status> {
        self.check_token(&req)?;
        let r = req.into_inner();
        let dir = self.root.path().join(&r.sandbox_id);
        std::fs::create_dir_all(&dir).unwrap();
        let id = r.sandbox_id.clone();
        let mut state = self.state.lock().unwrap();
        state.dirs.insert(id.clone(), dir);
        state.spawned.push(r);
        Ok(Response::new(pb::SpawnSandboxResponse {
            sandbox_id: id,
            channel_path: "/nonexistent/channel.sock".into(),
            deadline: "2099-01-01T00:00:00Z".into(),
            error: String::new(),
        }))
    }

    async fn destroy_sandbox(
        &self,
        req: Request<pb::DestroySandboxRequest>,
    ) -> Result<Response<pb::DestroySandboxResponse>, Status> {
        self.check_token(&req)?;
        let r = req.into_inner();
        self.state.lock().unwrap().destroyed.push(r.sandbox_id);
        Ok(Response::new(pb::DestroySandboxResponse {
            destroyed: true,
            error: String::new(),
        }))
    }

    async fn list_sandboxes(
        &self,
        _req: Request<pb::ListSandboxesRequest>,
    ) -> Result<Response<pb::ListSandboxesResponse>, Status> {
        let state = self.state.lock().unwrap();
        Ok(Response::new(pb::ListSandboxesResponse {
            sandbox_ids: state.dirs.keys().cloned().collect(),
        }))
    }

    type AttachSandboxStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<pb::AttachSandboxChunk, Status>> + Send>>;

    async fn attach_sandbox(
        &self,
        req: Request<Streaming<pb::AttachSandboxRequest>>,
    ) -> Result<Response<Self::AttachSandboxStream>, Status> {
        self.check_token(&req)?;
        let mut inbound = req.into_inner();
        let first = inbound
            .next()
            .await
            .transpose()?
            .ok_or_else(|| Status::invalid_argument("stream closed before naming a sandbox"))?;
        if first.sandbox_id.is_empty() {
            return Err(Status::invalid_argument(
                "first message must set sandbox_id",
            ));
        }
        let workdir = self
            .state
            .lock()
            .unwrap()
            .dirs
            .get(&first.sandbox_id)
            .cloned()
            .ok_or_else(|| Status::not_found(format!("no sandbox '{}'", first.sandbox_id)))?;

        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(run_fake_guest(workdir, first.data, inbound, tx));
        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    async fn get_state(
        &self,
        _req: Request<pb::GetStateRequest>,
    ) -> Result<Response<pb::GetStateResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
    async fn diff(
        &self,
        _req: Request<pb::DiffRequest>,
    ) -> Result<Response<pb::DiffResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
    async fn apply_migration(
        &self,
        _req: Request<pb::ApplyMigrationRequest>,
    ) -> Result<Response<pb::ApplyMigrationResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
    async fn get_ledger(
        &self,
        _req: Request<pb::GetLedgerRequest>,
    ) -> Result<Response<pb::GetLedgerResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
    async fn adopt_drift(
        &self,
        _req: Request<pb::AdoptDriftRequest>,
    ) -> Result<Response<pb::AdoptDriftResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
    async fn get_my_limits(
        &self,
        _req: Request<pb::GetMyLimitsRequest>,
    ) -> Result<Response<pb::GetMyLimitsResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
    async fn request_egress_decision(
        &self,
        _req: Request<pb::RequestEgressDecisionRequest>,
    ) -> Result<Response<pb::RequestEgressDecisionResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
    async fn pending_egress(
        &self,
        _req: Request<pb::PendingEgressRequest>,
    ) -> Result<Response<pb::PendingEgressResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
    async fn decide_egress(
        &self,
        _req: Request<pb::DecideEgressRequest>,
    ) -> Result<Response<pb::DecideEgressResponse>, Status> {
        Err(Status::unimplemented("mock"))
    }
}

</storycode:copy>
<a id="f-e998bd7eb08e-Mock"></a>
### `Mock`

<storycode:copy id="f-e998bd7eb08e-Mock">
struct Mock {
    addr: String,
    state: Arc<Mutex<MockState>>,
    _root: Arc<tempfile::TempDir>,
}

</storycode:copy>
<a id="f-e998bd7eb08e-start_mock"></a>
### `start_mock`

<storycode:copy id="f-e998bd7eb08e-start_mock">
async fn start_mock(expected_token: Option<Vec<u8>>) -> Mock {
    let state = Arc::new(Mutex::new(MockState::default()));
    let root = Arc::new(tempfile::tempdir().unwrap());
    let agent = MockAgent {
        state: state.clone(),
        root: root.clone(),
        expected_token,
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(CanopyAgentServer::new(agent))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener)),
    );
    Mock {
        addr: format!("127.0.0.1:{}", addr.port()),
        state,
        _root: root,
    }
}

</storycode:copy>
<a id="f-e998bd7eb08e-config_for"></a>
### `config_for`

<storycode:copy id="f-e998bd7eb08e-config_for">
fn config_for(mock: &Mock, token: Option<Vec<u8>>) -> CanopyConfig {
    CanopyConfig {
        agent: mock.addr.clone(),
        token,
        node: Some("mock-node".into()),
        image_map: [
            (
                "python:3.12".to_string(),
                "/nix/store/aaaa-py-img".to_string(),
            ),
            (
                "alpine:3.20".to_string(),
                "/nix/store/bbbb-alpine-img".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        vcpus: 2,
        mem_mib: 768,
        lifetime_secs: 1200,
        egress_hosts: vec!["pypi.org".into()],
        boot_timeout: Duration::from_secs(30),
        exec_timeout: Duration::from_secs(30),
    }
}

</storycode:copy>
<a id="f-e998bd7eb08e-full_execute_round_trip_with_transcript_timing_and_exit_code"></a>
### `full_execute_round_trip_with_transcript_timing_and_exit_code`

<storycode:copy id="f-e998bd7eb08e-full_execute_round_trip_with_transcript_timing_and_exit_code">
#[tokio::test(flavor = "multi_thread")]
async fn full_execute_round_trip_with_transcript_timing_and_exit_code() {
    let mock = start_mock(None).await;
    let ex = CanopyExecutor::new(config_for(&mock, None));

    ex.ensure_started("c1", "python:3.12").await.unwrap();
    let out = ex.execute("c1", "printf 'hello\\n'").await.unwrap();
    assert_eq!(out, "hello\n");

    // The spawn carried the mapped Nix store path and the configured shape.
    {
        let state = mock.state.lock().unwrap();
        assert_eq!(state.spawned.len(), 1);
        let s = &state.spawned[0];
        assert_eq!(s.image, "/nix/store/aaaa-py-img");
        assert_eq!(s.vcpus, 2);
        assert_eq!(s.mem_mib, 768);
        assert_eq!(s.lifetime_secs, 1200);
        assert_eq!(s.egress_hosts, vec!["pypi.org".to_string()]);
        assert!(s.sandbox_id.starts_with("hickory-c1-"), "{}", s.sandbox_id);
    }

    let ts = ex.transcripts();
    let entries = &ts["c1"];
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].commands, vec!["printf 'hello\\n'".to_string()]);
    assert_eq!(entries[0].output, "hello\n");
    let kinds: Vec<&str> = entries[0]
        .events
        .iter()
        .map(|e| match e {
            TranscriptEvent::Cmd { .. } => "cmd",
            TranscriptEvent::Out { .. } => "out",
            TranscriptEvent::Err { .. } => "err",
            TranscriptEvent::Exit { .. } => "exit",
        })
        .collect();
    assert_eq!(kinds.first(), Some(&"cmd"));
    assert_eq!(kinds.last(), Some(&"exit"));
    assert!(kinds.contains(&"out"));
    assert!(!kinds.contains(&"err"), "pty output is merged into out");
    assert!(matches!(
        entries[0].events.last(),
        Some(TranscriptEvent::Exit { code: 0, .. })
    ));
    // Timestamps are monotonically non-decreasing.
    let ts_ms: Vec<u64> = entries[0].events.iter().map(|e| e.t_offset_ms()).collect();
    assert!(ts_ms.windows(2).all(|w| w[0] <= w[1]), "{ts_ms:?}");
}

</storycode:copy>
<a id="f-e998bd7eb08e-state_persists_across_execs_and_nonzero_exit_is_recorded"></a>
### `state_persists_across_execs_and_nonzero_exit_is_recorded`

<storycode:copy id="f-e998bd7eb08e-state_persists_across_execs_and_nonzero_exit_is_recorded">
#[tokio::test(flavor = "multi_thread")]
async fn state_persists_across_execs_and_nonzero_exit_is_recorded() {
    let mock = start_mock(None).await;
    let ex = CanopyExecutor::new(config_for(&mock, None));
    ex.ensure_started("c", "alpine:3.20").await.unwrap();

    ex.execute("c", "echo data > f.txt").await.unwrap();
    let out = ex.execute("c", "cat f.txt").await.unwrap();
    assert_eq!(out, "data\n");

    let err = ex.execute("c", "echo oops >&2; exit 3").await.unwrap_err();
    assert!(err.to_string().contains("exit 3"), "err: {err}");
    assert!(err.to_string().contains("oops"), "err: {err}");
    let ts = ex.transcripts();
    assert!(
        ts["c"]
            .last()
            .unwrap()
            .events
            .iter()
            .any(|e| matches!(e, TranscriptEvent::Exit { code: 3, .. }))
    );
}

</storycode:copy>
<a id="f-e998bd7eb08e-stdin_is_honored"></a>
### `stdin_is_honored`

<storycode:copy id="f-e998bd7eb08e-stdin_is_honored">
#[tokio::test(flavor = "multi_thread")]
async fn stdin_is_honored() {
    let mock = start_mock(None).await;
    let ex = CanopyExecutor::new(config_for(&mock, None));
    ex.ensure_started("c", "alpine:3.20").await.unwrap();
    let out = ex
        .execute_with_stdin("c", "cat", "piped input")
        .await
        .unwrap();
    assert_eq!(out, "piped input");
}

</storycode:copy>
<a id="f-e998bd7eb08e-fork_replays_command_history_in_a_fresh_sandbox"></a>
### `fork_replays_command_history_in_a_fresh_sandbox`

<storycode:copy id="f-e998bd7eb08e-fork_replays_command_history_in_a_fresh_sandbox">
#[tokio::test(flavor = "multi_thread")]
async fn fork_replays_command_history_in_a_fresh_sandbox() {
    let mock = start_mock(None).await;
    let ex = CanopyExecutor::new(config_for(&mock, None));
    ex.ensure_started("base", "alpine:3.20").await.unwrap();
    ex.execute("base", "echo shared > seed.txt").await.unwrap();
    ex.register_fork("child", "base", None).await.unwrap();
    ex.ensure_started("child", "alpine:3.20").await.unwrap();

    // Two sandboxes were spawned — the fork is not the same VM.
    assert_eq!(mock.state.lock().unwrap().spawned.len(), 2);

    let out = ex.execute("child", "cat seed.txt").await.unwrap();
    assert_eq!(out, "shared\n");
    // Divergence: writes in the fork do not affect the source.
    ex.execute("child", "echo forked > only.txt").await.unwrap();
    assert!(ex.execute("base", "cat only.txt").await.is_err());
}

</storycode:copy>
<a id="f-e998bd7eb08e-volume_roundtrip_via_base64_tar_over_the_attach_stream"></a>
### `volume_roundtrip_via_base64_tar_over_the_attach_stream`

<storycode:copy id="f-e998bd7eb08e-volume_roundtrip_via_base64_tar_over_the_attach_stream">
#[tokio::test(flavor = "multi_thread")]
async fn volume_roundtrip_via_base64_tar_over_the_attach_stream() {
    let mock = start_mock(None).await;
    let ex = CanopyExecutor::new(config_for(&mock, None));
    ex.ensure_started("w", "alpine:3.20").await.unwrap();
    ex.create_mount_point("w", "/data").await.unwrap();
    ex.execute("w", "echo v1 > data/file.txt").await.unwrap();
    let tar_bytes = ex.extract_volume("w", "/data").await.unwrap();

    // The bytes really are a tar with the file in it.
    let mut names = Vec::new();
    let mut archive = tar::Archive::new(&tar_bytes[..]);
    for entry in archive.entries().unwrap() {
        names.push(
            entry
                .unwrap()
                .path()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        );
    }
    assert!(
        names.iter().any(|n| n.ends_with("file.txt")),
        "tar entries: {names:?}"
    );

    ex.ensure_started("r", "alpine:3.20").await.unwrap();
    ex.inject_volume("r", "/incoming", &tar_bytes)
        .await
        .unwrap();
    let out = ex.execute("r", "cat incoming/file.txt").await.unwrap();
    assert_eq!(out, "v1\n");
}

</storycode:copy>
<a id="f-e998bd7eb08e-unknown_image_fails_before_spawning_and_lists_configured_images"></a>
### `unknown_image_fails_before_spawning_and_lists_configured_images`

<storycode:copy id="f-e998bd7eb08e-unknown_image_fails_before_spawning_and_lists_configured_images">
#[tokio::test(flavor = "multi_thread")]
async fn unknown_image_fails_before_spawning_and_lists_configured_images() {
    let mock = start_mock(None).await;
    let ex = CanopyExecutor::new(config_for(&mock, None));
    let err = ex
        .ensure_started("c", "node:22")
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("node:22"), "err: {err}");
    assert!(err.contains("alpine:3.20"), "err: {err}");
    assert!(err.contains("python:3.12"), "err: {err}");
    assert!(mock.state.lock().unwrap().spawned.is_empty());
}

</storycode:copy>
<a id="f-e998bd7eb08e-shutdown_destroys_every_spawned_sandbox"></a>
### `shutdown_destroys_every_spawned_sandbox`

<storycode:copy id="f-e998bd7eb08e-shutdown_destroys_every_spawned_sandbox">
#[tokio::test(flavor = "multi_thread")]
async fn shutdown_destroys_every_spawned_sandbox() {
    let mock = start_mock(None).await;
    let ex = CanopyExecutor::new(config_for(&mock, None));
    ex.ensure_started("a", "alpine:3.20").await.unwrap();
    ex.ensure_started("b", "python:3.12").await.unwrap();
    ex.shutdown().await.unwrap();

    let state = mock.state.lock().unwrap();
    let spawned: Vec<String> = state.spawned.iter().map(|s| s.sandbox_id.clone()).collect();
    let mut destroyed = state.destroyed.clone();
    destroyed.sort();
    let mut expected = spawned.clone();
    expected.sort();
    assert_eq!(destroyed, expected);
}

</storycode:copy>
<a id="f-e998bd7eb08e-capability_token_is_sent_as_binary_metadata_on_every_rpc"></a>
### `capability_token_is_sent_as_binary_metadata_on_every_rpc`

<storycode:copy id="f-e998bd7eb08e-capability_token_is_sent_as_binary_metadata_on_every_rpc">
#[tokio::test(flavor = "multi_thread")]
async fn capability_token_is_sent_as_binary_metadata_on_every_rpc() {
    let token = b"raw-biscuit-bytes".to_vec();
    let mock = start_mock(Some(token.clone())).await;

    // Without the token the mock refuses.
    let ex = CanopyExecutor::new(config_for(&mock, None));
    assert!(ex.ensure_started("c", "alpine:3.20").await.is_err());

    // With it, spawn + attach + destroy all pass the token check.
    let ex = CanopyExecutor::new(config_for(&mock, Some(token)));
    ex.ensure_started("c", "alpine:3.20").await.unwrap();
    assert_eq!(ex.execute("c", "echo ok").await.unwrap(), "ok\n");
    ex.shutdown().await.unwrap();
    assert!(!mock.state.lock().unwrap().destroyed.is_empty());
}

</storycode:copy>
<a id="f-e998bd7eb08e-resource_stats_report_boot_and_command_durations"></a>
### `resource_stats_report_boot_and_command_durations`

<storycode:copy id="f-e998bd7eb08e-resource_stats_report_boot_and_command_durations">
#[tokio::test(flavor = "multi_thread")]
async fn resource_stats_report_boot_and_command_durations() {
    let mock = start_mock(None).await;
    let ex = CanopyExecutor::new(config_for(&mock, None));
    ex.ensure_started("c", "alpine:3.20").await.unwrap();
    ex.execute("c", "true").await.unwrap();
    let stats = ex.resource_stats();
    let s = &stats["c"];
    assert!(s.boot_duration > Duration::ZERO);
    assert_eq!(s.command_durations.len(), 1);
}
</storycode:copy>

### Assemble `crates/hickory-executor-canopy/tests/contract.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-canopy/tests/contract.rs"><storycode:paste select="#f-e998bd7eb08e-TOKEN_KEY"/><storycode:paste select="#f-e998bd7eb08e-MockState"/><storycode:paste select="#f-e998bd7eb08e-MockAgent"/><storycode:paste select="#f-e998bd7eb08e-impl-MockAgent"/><storycode:paste select="#f-e998bd7eb08e-run_fake_guest"/><storycode:paste select="#f-e998bd7eb08e-impl-CanopyAgent-for-MockAgent"/><storycode:paste select="#f-e998bd7eb08e-Mock"/><storycode:paste select="#f-e998bd7eb08e-start_mock"/><storycode:paste select="#f-e998bd7eb08e-config_for"/><storycode:paste select="#f-e998bd7eb08e-full_execute_round_trip_with_transcript_timing_and_exit_code"/><storycode:paste select="#f-e998bd7eb08e-state_persists_across_execs_and_nonzero_exit_is_recorded"/><storycode:paste select="#f-e998bd7eb08e-stdin_is_honored"/><storycode:paste select="#f-e998bd7eb08e-fork_replays_command_history_in_a_fresh_sandbox"/><storycode:paste select="#f-e998bd7eb08e-volume_roundtrip_via_base64_tar_over_the_attach_stream"/><storycode:paste select="#f-e998bd7eb08e-unknown_image_fails_before_spawning_and_lists_configured_images"/><storycode:paste select="#f-e998bd7eb08e-shutdown_destroys_every_spawned_sandbox"/><storycode:paste select="#f-e998bd7eb08e-capability_token_is_sent_as_binary_metadata_on_every_rpc"/><storycode:paste select="#f-e998bd7eb08e-resource_stats_report_boot_and_command_durations"/></storycode:file>

<a id="output-c57123bb6fac"></a>
## `crates/hickory-executor-canopy/tests/isolation.rs`

Related contracts: [Only `hickory-executor-canopy` Knows Cloud Canopy's API](contract.md).

<a id="f-c57123bb6fac-FORBIDDEN_API_TOKENS"></a>
### `FORBIDDEN_API_TOKENS`

CI-greppable enforcement of
only `crates/hickory-executor-canopy` may know Cloud Canopy's API.
Two rules, checked over the whole workspace:
1. No Rust source outside this crate mentions a canopy API token
(proto package, RPC names, metadata key, pty sentinels, `CANOPY_*`
env names). The bare word "canopy" stays legal — executor selection
(`HICKORY_EXECUTOR=canopy`) and docs may name the backend without
knowing its API.
2. Only allowlisted crates (the executor-selection points) may depend on
or import `hickory-executor-canopy` itself.

<storycode:copy id="f-c57123bb6fac-FORBIDDEN_API_TOKENS">
//! CI-greppable enforcement of
//! docs/guarantees/execution/canopy-api-isolated-to-one-crate.md:
//! only `crates/hickory-executor-canopy` may know Cloud Canopy's API.
//!
//! Two rules, checked over the whole workspace:
//! 1. No Rust source outside this crate mentions a canopy API token
//!    (proto package, RPC names, metadata key, pty sentinels, `CANOPY_*`
//!    env names). The bare word "canopy" stays legal — executor selection
//!    (`HICKORY_EXECUTOR=canopy`) and docs may name the backend without
//!    knowing its API.
//! 2. Only allowlisted crates (the executor-selection points) may depend on
//!    or import `hickory-executor-canopy` itself.

use std::path::{Path, PathBuf};

/// API-level knowledge of canopy. Any of these outside the adapter crate is
/// a guarantee violation.
const FORBIDDEN_API_TOKENS: &[&str] = &[
    "canopy.v1",
    "CanopyAgent",
    "SpawnSandbox",
    "AttachSandbox",
    "DestroySandbox",
    "spawn_sandbox",
    "attach_sandbox",
    "destroy_sandbox",
    "x-canopy-capability",
    "__CANOPY_S__",
    "__CANOPY_E__",
    "canopy-guest",
    "CANOPY_AGENT",
    "CANOPY_TOKEN",
    "CANOPY_NODE",
    "CANOPY_IMAGE_MAP",
    "CANOPY_VCPUS",
    "CANOPY_MEM_MIB",
    "CANOPY_LIFETIME_SECS",
    "CANOPY_EGRESS_HOSTS",
];

</storycode:copy>
<a id="f-c57123bb6fac-ADAPTER_DEPENDENTS_ALLOWED"></a>
### `ADAPTER_DEPENDENTS_ALLOWED`

Crates allowed to depend on the adapter: the executor-selection points
that construct it behind the `Executor` trait.

<storycode:copy id="f-c57123bb6fac-ADAPTER_DEPENDENTS_ALLOWED">
/// Crates allowed to depend on the adapter: the executor-selection points
/// that construct it behind the `Executor` trait.
const ADAPTER_DEPENDENTS_ALLOWED: &[&str] = &["hickory-cli", "hickory-server"];

</storycode:copy>
<a id="f-c57123bb6fac-ADAPTER_CRATE"></a>
### `ADAPTER_CRATE`

<storycode:copy id="f-c57123bb6fac-ADAPTER_CRATE">
const ADAPTER_CRATE: &str = "hickory-executor-canopy";

</storycode:copy>
<a id="f-c57123bb6fac-workspace_root"></a>
### `workspace_root`

<storycode:copy id="f-c57123bb6fac-workspace_root">
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<name> has a workspace root two levels up")
        .to_path_buf()
}

</storycode:copy>
<a id="f-c57123bb6fac-rust_files_under"></a>
### `rust_files_under`

<storycode:copy id="f-c57123bb6fac-rust_files_under">
fn rust_files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == "target" || name == "node_modules" || name.starts_with('.') {
                continue; // hidden dirs include .git and .claude/worktrees
            }
            rust_files_under(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

</storycode:copy>
<a id="f-c57123bb6fac-no_rust_source_outside_the_adapter_crate_knows_canopys_api"></a>
### `no_rust_source_outside_the_adapter_crate_knows_canopys_api`

<storycode:copy id="f-c57123bb6fac-no_rust_source_outside_the_adapter_crate_knows_canopys_api">
#[test]
fn no_rust_source_outside_the_adapter_crate_knows_canopys_api() {
    let root = workspace_root();
    let adapter = root.join("crates").join(ADAPTER_CRATE);
    let mut files = Vec::new();
    // Scan every Rust file in the repo (crates, apps, examples, build
    // scripts), not just crates/, so a new location cannot dodge the rule.
    rust_files_under(&root, &mut files);
    assert!(
        files.iter().any(|f| f.starts_with(&adapter)),
        "sanity: the scan must see the adapter crate itself"
    );

    let mut violations = Vec::new();
    for file in files {
        if file.starts_with(&adapter) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&file) else {
            continue;
        };
        for token in FORBIDDEN_API_TOKENS {
            if content.contains(token) {
                violations.push(format!("{}: contains {token:?}", file.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "Cloud Canopy API knowledge leaked outside crates/{ADAPTER_CRATE} \
         (docs/guarantees/execution/canopy-api-isolated-to-one-crate.md):\n{}",
        violations.join("\n")
    );
}

</storycode:copy>
<a id="f-c57123bb6fac-only_allowlisted_crates_depend_on_the_adapter"></a>
### `only_allowlisted_crates_depend_on_the_adapter`

<storycode:copy id="f-c57123bb6fac-only_allowlisted_crates_depend_on_the_adapter">
#[test]
fn only_allowlisted_crates_depend_on_the_adapter() {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut violations = Vec::new();
    for entry in std::fs::read_dir(&crates_dir)
        .expect("crates/ exists")
        .flatten()
    {
        let crate_dir = entry.path();
        let name = crate_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if name == ADAPTER_CRATE {
            continue;
        }
        let manifest = crate_dir.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(&manifest)
            && content.contains(ADAPTER_CRATE)
            && !ADAPTER_DEPENDENTS_ALLOWED.contains(&name.as_str())
        {
            violations.push(format!(
                "{}: depends on {ADAPTER_CRATE} but is not an allowed executor-selection point",
                manifest.display()
            ));
        }
        // The import side of the same rule.
        let mut files = Vec::new();
        rust_files_under(&crate_dir.join("src"), &mut files);
        rust_files_under(&crate_dir.join("tests"), &mut files);
        if !ADAPTER_DEPENDENTS_ALLOWED.contains(&name.as_str()) {
            for file in files {
                if std::fs::read_to_string(&file)
                    .is_ok_and(|c| c.contains("hickory_executor_canopy"))
                {
                    violations.push(format!(
                        "{}: imports hickory_executor_canopy",
                        file.display()
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "unexpected dependents of the canopy adapter \
         (docs/guarantees/execution/canopy-api-isolated-to-one-crate.md):\n{}",
        violations.join("\n")
    );
}
</storycode:copy>

### Assemble `crates/hickory-executor-canopy/tests/isolation.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-canopy/tests/isolation.rs"><storycode:paste select="#f-c57123bb6fac-FORBIDDEN_API_TOKENS"/><storycode:paste select="#f-c57123bb6fac-ADAPTER_DEPENDENTS_ALLOWED"/><storycode:paste select="#f-c57123bb6fac-ADAPTER_CRATE"/><storycode:paste select="#f-c57123bb6fac-workspace_root"/><storycode:paste select="#f-c57123bb6fac-rust_files_under"/><storycode:paste select="#f-c57123bb6fac-no_rust_source_outside_the_adapter_crate_knows_canopys_api"/><storycode:paste select="#f-c57123bb6fac-only_allowlisted_crates_depend_on_the_adapter"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-executor-canopy/src/executor.rs](../../../mechanisms/crates/hickory-executor-canopy/src/executor.rs.md#output-47988291742d)
- [crates/hickory-executor-canopy/src/frame.rs](../../verification/an-expectation-means-the-same-on-every-platform/contract.md#output-eaa988f0d416)
- [crates/hickory-executor-canopy/src/lib.rs](contract.md#output-5017d9438178)

### Tests and fixtures

- [crates/hickory-cli/tests/test_command_tests.rs](../../../mechanisms/crates/hickory-cli/tests/test_command_tests.rs.md#output-87d6e5478294)
- [crates/hickory-executor-canopy/tests/contract.rs](contract.md#output-e998bd7eb08e)
- [crates/hickory-executor-canopy/tests/isolation.rs](contract.md#output-c57123bb6fac)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
