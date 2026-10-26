<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/tests/private_adapter.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b2669b67dd81"></a>
## `crates/hick-dap/tests/private_adapter.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-b2669b67dd81-SECRET"></a>
### `SECRET`

<storycode:copy id="f-b2669b67dd81-SECRET">
#![cfg(unix)]
//! Check process diagnostics at the real subprocess boundary.
use hick_dap::{Adapter, Transport};

const SECRET: &str = "private-adapter-answer-must-not-escape";

</storycode:copy>
[Read the test `private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr` beside its requirement](../../../../contract.md#f-b2669b67dd81-private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr).
<a id="f-b2669b67dd81-private_adapter_child"></a>
### `private_adapter_child`

<storycode:copy id="f-b2669b67dd81-private_adapter_child">
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
</storycode:copy>

### Assemble `crates/hick-dap/tests/private_adapter.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/private_adapter.rs"><storycode:paste select="#f-b2669b67dd81-SECRET"/><storycode:paste select="#f-b2669b67dd81-private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr"/><storycode:paste select="#f-b2669b67dd81-private_adapter_child"/></storycode:file>

</storycode:doc>
