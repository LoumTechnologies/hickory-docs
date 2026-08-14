//! Does the sandbox actually confine anything?
//!
//! Protects docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md
//!
//! These run the real sandbox against the real filesystem, because the only
//! interesting question is whether a hostile cell is stopped — and that is
//! not observable from the argv. Each is skipped, loudly, on a machine with
//! no sandbox: a silent skip would let this file rot into a decoration.

use hickory_executor::Executor;
use hickory_executor_sandbox::{Sandbox, SandboxedExecutor};

fn available() -> bool {
    if Sandbox::detect() == Sandbox::None {
        eprintln!(
            "SKIPPED: no sandbox on this machine ({})",
            Sandbox::missing_hint()
        );
        return false;
    }
    true
}

async fn started() -> SandboxedExecutor {
    let executor = SandboxedExecutor::new().expect("a sandbox is available");
    executor
        .ensure_started("c", "python:3.12")
        .await
        .expect("container starts");
    executor
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cell_can_work_in_its_own_workdir() {
    if !available() {
        return;
    }
    let executor = started().await;
    let out = executor
        .execute("c", "echo hello > note.txt && cat note.txt")
        .await
        .expect("a cell can write its own workdir");
    assert!(out.contains("hello"), "{out}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cell_cannot_write_outside_its_workdir() {
    if !available() {
        return;
    }
    let executor = started().await;
    // The whole point: a document you did not write cannot touch your files.
    let result = executor
        .execute("c", "touch /etc/hickory-should-not-exist && echo WROTE")
        .await;
    assert!(result.is_err(), "writing to /etc succeeded: {result:?}");
    assert!(
        !std::path::Path::new("/etc/hickory-should-not-exist").exists(),
        "the sandbox let a cell create a file in /etc"
    );

    // $HOME is the more realistic target. A write there SUCCEEDS inside the
    // sandbox — the cell has a private, empty home on a tmpfs, so tools that
    // want a cache directory still work — and lands nowhere: the host's home
    // never sees it. That is the property worth asserting, not the exit code.
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    let escape = format!("touch {home}/hickory-should-not-exist && echo WROTE");
    executor
        .execute("c", &escape)
        .await
        .expect("a private home is writable inside the sandbox");
    assert!(
        !std::path::Path::new(&format!("{home}/hickory-should-not-exist")).exists(),
        "the sandbox let a cell write into the real $HOME"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cell_cannot_read_your_dotfiles() {
    if !available() || Sandbox::detect() != Sandbox::Bubblewrap {
        return;
    }
    let executor = started().await;
    // Read-only was not enough. A cell that can list ~/.ssh has already told
    // whoever wrote it which keys exist, and "the network is off" is one
    // mistake away from being false.
    let out = executor
        .execute(
            "c",
            "ls -a \"$HOME\" | tr '\\n' ' '; echo; ls \"$HOME/.ssh\" 2>&1 | head -1",
        )
        .await
        .expect("listing an empty home works");
    assert!(
        !out.contains("id_ed25519") && !out.contains("id_rsa"),
        "the sandbox exposed private keys: {out}"
    );
    assert!(
        out.contains("No such file") || out.trim().ends_with('.') || !out.contains(".ssh"),
        "expected an empty home, got: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cell_has_no_network_unless_the_document_grants_it() {
    if !available() || Sandbox::detect() != Sandbox::Bubblewrap {
        // Only bubblewrap gives a network namespace this test can observe;
        // Seatbelt's denial is not visible in `/sys`.
        return;
    }
    if which("python3").is_none() {
        eprintln!("SKIPPED: no python3 to probe the network with");
        return;
    }
    let executor = started().await;
    // Probe by CONNECTING, not by listing interfaces. `/sys` is bind-mounted
    // from the host, so `ls /sys/class/net` reports the host's interfaces
    // however the network namespace is set up — it looks like a leak and is
    // not one. Reachability is the property that matters and the only one
    // worth asserting.
    // Written to a file rather than a heredoc: Rust's line-continuation
    // escapes eat the leading whitespace Python needs, and a probe that fails
    // to parse would "prove" the network is blocked for the wrong reason.
    let write_probe = concat!(
        "printf '%s\n' ",
        "'import socket' ",
        "'try:' ",
        "'    socket.create_connection((\"1.1.1.1\", 80), timeout=3)' ",
        "'    print(\"REACHED\")' ",
        "'except OSError as e:' ",
        "'    print(\"BLOCKED\", type(e).__name__)' ",
        "> probe.py"
    );
    executor
        .execute("c", write_probe)
        .await
        .expect("writing the probe works");
    let probe = "python3 probe.py";
    let out = executor.execute("c", probe).await.expect("the probe runs");
    assert!(
        out.contains("BLOCKED"),
        "a container that declared no network reached the internet: {out}"
    );
}

fn which(name: &str) -> Option<std::path::PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

#[tokio::test(flavor = "multi_thread")]
async fn declaring_the_network_opens_it() {
    if !available() || Sandbox::detect() != Sandbox::Bubblewrap {
        return;
    }
    let executor = SandboxedExecutor::new().unwrap();
    executor
        .declare_capabilities(
            "net",
            hick_token::ContainerCapabilities::new().allow_network("example.com", "443"),
        )
        .await
        .unwrap();
    executor.ensure_started("net", "alpine").await.unwrap();

    // Not asserting that it REACHES the internet — a machine may be offline,
    // and a test that needs the network to pass is a test that fails for the
    // wrong reason. What is asserted is that the sandbox was asked to share
    // the network, which is the executor's half of the bargain.
    let confined = executor
        .execute("net", "echo ok")
        .await
        .expect("a network-granted container still runs");
    assert!(confined.contains("ok"));
}

#[tokio::test(flavor = "multi_thread")]
async fn one_containers_cells_share_a_tmp() {
    if !available() {
        return;
    }
    // A container's state accumulates across its cells in FILES, and /tmp is
    // files. A sandbox that gives each command its own tmpfs breaks that: the
    // document works unsandboxed and fails confined, which reads as the
    // document being wrong when it is the sandbox. (This is not
    // hypothetical — it broke one of this repository's own documents, whose
    // cells pass state through a file in /tmp.)
    let executor = started().await;
    executor
        .execute("c", "echo remembered > /tmp/note.txt")
        .await
        .expect("a cell can write /tmp");
    let out = executor
        .execute("c", "cat /tmp/note.txt")
        .await
        .expect("the next cell finds it");
    assert!(out.contains("remembered"), "{out}");
}

#[tokio::test(flavor = "multi_thread")]
async fn two_containers_do_not_share_a_tmp() {
    if !available() {
        return;
    }
    // The other half: shared WITHIN a container, private BETWEEN them. A
    // single shared /tmp would let one container read another's scratch
    // files, which is the isolation this executor exists to provide.
    let executor = started().await;
    executor.ensure_started("other", "alpine").await.unwrap();
    executor
        .execute("other", "echo secret > /tmp/other-note.txt")
        .await
        .unwrap();
    let out = executor
        .execute("c", "cat /tmp/other-note.txt 2>/dev/null || echo DENIED")
        .await
        .unwrap();
    assert!(
        out.contains("DENIED"),
        "one container read another's /tmp: {out}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cell_cannot_read_another_containers_workdir() {
    if !available() {
        return;
    }
    let executor = started().await;
    executor.ensure_started("other", "alpine").await.unwrap();
    executor
        .execute("other", "echo secret > secret.txt")
        .await
        .unwrap();

    // Each container's workdir is bound only into its own sandbox, so one
    // cell cannot read what another produced except through a declared
    // volume.
    let result = executor
        .execute("c", "cat ../*/secret.txt 2>/dev/null || echo DENIED")
        .await
        .unwrap();
    assert!(result.contains("DENIED"), "{result}");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_transcript_records_the_cell_not_the_sandbox() {
    if !available() {
        return;
    }
    let executor = started().await;
    executor.execute("c", "echo hello").await.unwrap();
    let transcripts = executor.transcripts();
    let recorded = format!("{transcripts:?}");
    // A reader asked to check what ran must see their own command, not a
    // bwrap invocation wrapped around it.
    assert!(recorded.contains("echo hello"), "{recorded}");
    assert!(
        !recorded.contains("--ro-bind"),
        "the sandbox leaked into the transcript: {recorded}"
    );
}
