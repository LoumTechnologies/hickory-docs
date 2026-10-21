#![cfg(unix)]
//! Check process diagnostics at the real subprocess boundary.
use hick_dap::{Adapter, Transport};

const SECRET: &str = "private-adapter-answer-must-not-escape";

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "private_adapter_child", "--nocapture"])
        .env("HICK_TEST_PRIVATE_ADAPTER_CHILD", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(SECRET));
}

#[tokio::test]
async fn private_adapter_child() {
    if std::env::var_os("HICK_TEST_PRIVATE_ADAPTER_CHILD").is_none() {
        return;
    }
    // Each fake adapter writes a diagnostic before waiting for the client.
    // TCP adds stdout as an independent non-protocol channel.
    for transport in [Transport::Stdio, Transport::Tcp] {
        let script = if transport == Transport::Stdio {
            format!(
                "import sys; print({SECRET:?}, file=sys.stderr, flush=True); sys.stdin.buffer.read()"
            )
        } else {
            format!(
                "import sys,socket; print({SECRET:?},flush=True); print({SECRET:?},file=sys.stderr,flush=True); s=socket.socket(); s.bind(('127.0.0.1',int(sys.argv[1]))); s.listen(); c,_=s.accept(); c.recv(4096)"
            )
        };
        let mut command = vec!["python3".into(), "-c".into(), script];
        if transport == Transport::Tcp {
            command.push("{port}".into());
        }
        let adapter = Adapter::start_private(&command, transport).await.unwrap();
        adapter.shutdown().await;
    }
    assert!(
        Adapter::start_private(&[], Transport::Attached(1234))
            .await
            .is_err()
    );
}
