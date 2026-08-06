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

#[derive(Default)]
struct MockState {
    spawned: Vec<pb::SpawnSandboxRequest>,
    destroyed: Vec<String>,
    dirs: HashMap<String, std::path::PathBuf>,
}

struct MockAgent {
    state: Arc<Mutex<MockState>>,
    root: Arc<tempfile::TempDir>,
    /// When set, every RPC must carry exactly these token bytes.
    expected_token: Option<Vec<u8>>,
}

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

struct Mock {
    addr: String,
    state: Arc<Mutex<MockState>>,
    _root: Arc<tempfile::TempDir>,
}

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
