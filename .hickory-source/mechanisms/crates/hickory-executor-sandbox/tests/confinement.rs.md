<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-executor-sandbox/tests/confinement.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c8a04313c63c"></a>
## `crates/hickory-executor-sandbox/tests/confinement.rs`

Related contracts: [A Cell Cannot Hang A Run](../../../../guarantees/execution/a-cell-cannot-hang-a-run/contract.md), [A Sandboxed Cell Cannot Reach Past Its Workdir](../../../../guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir/contract.md), [Every Desktop Platform Can Confine A Cell](../../../../guarantees/execution/every-desktop-platform-can-confine-a-cell/contract.md).

[Read the test `per_shell` beside its requirement](../../../../guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir/contract.md#f-c8a04313c63c-per_shell).
<a id="f-c8a04313c63c-available"></a>
### `available`

<storycode:copy id="f-c8a04313c63c-available">
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

</storycode:copy>
<a id="f-c8a04313c63c-started"></a>
### `started`

<storycode:copy id="f-c8a04313c63c-started">
async fn started() -> SandboxedExecutor {
    let executor = SandboxedExecutor::new().expect("a sandbox is available");
    executor
        .ensure_started("c", "python:3.12")
        .await
        .expect("container starts");
    executor
}

</storycode:copy>
<a id="f-c8a04313c63c-a_cell_can_work_in_its_own_workdir"></a>
### `a_cell_can_work_in_its_own_workdir`

<storycode:copy id="f-c8a04313c63c-a_cell_can_work_in_its_own_workdir">
#[tokio::test(flavor = "multi_thread")]
async fn a_cell_can_work_in_its_own_workdir() {
    if !available() {
        return;
    }
    let executor = started().await;
    let out = executor
        .execute(
            "c",
            &per_shell(
                "echo hello > note.txt && cat note.txt",
                "echo hello > note.txt && type note.txt",
            ),
        )
        .await
        .expect("a cell can write its own workdir");
    assert!(out.contains("hello"), "{out}");
}

</storycode:copy>
<a id="f-c8a04313c63c-a_cell_cannot_write_outside_its_workdir"></a>
### `a_cell_cannot_write_outside_its_workdir`

<storycode:copy id="f-c8a04313c63c-a_cell_cannot_write_outside_its_workdir">
#[tokio::test(flavor = "multi_thread")]
async fn a_cell_cannot_write_outside_its_workdir() {
    if !available() {
        return;
    }
    let executor = started().await;
    // The whole point: a document you did not write cannot touch your files.
    //
    // The Windows target is `C:\Users\Public` rather than a system directory
    // on purpose. `touch /etc/...` is msys on Windows and fails whatever the
    // sandbox does, so the assertion held for a run in which nothing was ever
    // confined — and a system directory would fail for lack of Administrator
    // instead. Public is writable by an ordinary user, so a cell that reaches
    // it really did escape.
    let outside = if cfg!(windows) {
        std::path::PathBuf::from(r"C:\Users\Public\hickory-should-not-exist.txt")
    } else {
        std::path::PathBuf::from("/etc/hickory-should-not-exist")
    };
    let _ = std::fs::remove_file(&outside);
    let result = executor
        .execute(
            "c",
            &per_shell(
                &format!("touch {} && echo WROTE", outside.display()),
                &format!("echo x> \"{}\" && echo WROTE", outside.display()),
            ),
        )
        .await;
    assert!(
        result.is_err(),
        "writing to {} succeeded: {result:?}",
        outside.display()
    );
    assert!(
        !outside.exists(),
        "the sandbox let a cell create {}",
        outside.display()
    );

    // $HOME is the more realistic target, and the property worth asserting is
    // the same on both sandboxes even though they reach it differently: the
    // host's home never sees the write. Bubblewrap mounts a private tmpfs
    // home, so the write SUCCEEDS and lands nowhere — tools that want a cache
    // directory still work. Seatbelt cannot mount anything, so it denies the
    // write outright. Asserting the exit code would be asserting the
    // mechanism; asserting the host's home is asserting the guarantee.
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Default".into())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| "/root".into())
    };
    let escape = per_shell(
        &format!("touch {home}/hickory-should-not-exist && echo WROTE"),
        &format!("echo x> \"{home}\\hickory-should-not-exist\" && echo WROTE"),
    );
    let wrote = executor.execute("c", &escape).await;
    if Sandbox::detect() == Sandbox::Bubblewrap {
        wrote.expect("a private home is writable inside the sandbox");
    }
    assert!(
        !std::path::Path::new(&home)
            .join("hickory-should-not-exist")
            .exists(),
        "the sandbox let a cell write into the real home directory"
    );
}

</storycode:copy>
<a id="f-c8a04313c63c-a_cell_cannot_read_your_dotfiles"></a>
### `a_cell_cannot_read_your_dotfiles`

<storycode:copy id="f-c8a04313c63c-a_cell_cannot_read_your_dotfiles">
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

</storycode:copy>
<a id="f-c8a04313c63c-a_cell_has_no_network_unless_the_document_grants_it"></a>
### `a_cell_has_no_network_unless_the_document_grants_it`

<storycode:copy id="f-c8a04313c63c-a_cell_has_no_network_unless_the_document_grants_it">
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

</storycode:copy>
<a id="f-c8a04313c63c-which"></a>
### `which`

<storycode:copy id="f-c8a04313c63c-which">
fn which(name: &str) -> Option<std::path::PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

</storycode:copy>
<a id="f-c8a04313c63c-declaring_the_network_opens_it"></a>
### `declaring_the_network_opens_it`

<storycode:copy id="f-c8a04313c63c-declaring_the_network_opens_it">
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

</storycode:copy>
<a id="f-c8a04313c63c-one_containers_cells_share_a_tmp"></a>
### `one_containers_cells_share_a_tmp`

<storycode:copy id="f-c8a04313c63c-one_containers_cells_share_a_tmp">
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
    // On Windows the per-container tmp is not something this code binds:
    // Windows redirects an AppContainer's TEMP into the package's own store,
    // and the package SID is derived from the workdir — so it is already
    // per-container. The test is the same question either way.
    executor
        .execute(
            "c",
            &per_shell(
                "echo remembered > /tmp/note.txt",
                r"echo remembered> %TEMP%\note.txt",
            ),
        )
        .await
        .expect("a cell can write /tmp");
    let out = executor
        .execute(
            "c",
            &per_shell("cat /tmp/note.txt", r"type %TEMP%\note.txt"),
        )
        .await
        .expect("the next cell finds it");
    assert!(out.contains("remembered"), "{out}");
}

</storycode:copy>
<a id="f-c8a04313c63c-two_containers_do_not_share_a_tmp"></a>
### `two_containers_do_not_share_a_tmp`

<storycode:copy id="f-c8a04313c63c-two_containers_do_not_share_a_tmp">
#[tokio::test(flavor = "multi_thread")]
async fn two_containers_do_not_share_a_tmp() {
    if !available() {
        return;
    }
    // The other half: shared WITHIN a container, private BETWEEN them. A
    // single shared /tmp would let one container read another's scratch
    // files, which is the isolation this executor exists to provide.
    //
    // Bubblewrap only. **Seatbelt cannot do this and it is not an oversight**:
    // it has no mount namespaces, so the literal path `/tmp` is one directory
    // for every process on the machine and there is nowhere to redirect it to.
    // The alternative — denying `/tmp` outright — would break the sibling test
    // above and every document that writes a scratch file there, trading a
    // real capability for an isolation property macOS will not give either
    // way. Recorded as a boundary in
    // docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md;
    // a run that needs it wants the Docker executor. Each container's *own*
    // tmp directory is still private — that part is covered by
    // `a_cell_cannot_read_another_containers_workdir`.
    if Sandbox::detect() != Sandbox::Bubblewrap {
        eprintln!(
            "skipped: {:?} cannot give each container a private /tmp — see the \
             guarantee's Boundary section",
            Sandbox::detect()
        );
        return;
    }
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

</storycode:copy>
<a id="f-c8a04313c63c-a_cell_cannot_read_another_containers_workdir"></a>
### `a_cell_cannot_read_another_containers_workdir`

<storycode:copy id="f-c8a04313c63c-a_cell_cannot_read_another_containers_workdir">
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
    // `for /d` rather than a wildcard path: `type ..\*\secret.txt` does not
    // expand a wildcard in a DIRECTORY component, so it reports failure even
    // when the peer is perfectly readable — a test built on it would pass
    // whether or not anything was confining. Checked both ways on Windows 11.
    let result = executor
        .execute(
            "c",
            &per_shell(
                "cat ../*/secret.txt 2>/dev/null || echo DENIED",
                r#"for /d %d in (..\*) do @type "%d\secret.txt" 2>nul"#,
            ),
        )
        .await
        .unwrap();
    // The absence of the data, not the presence of a fallback message: what
    // the guarantee promises is that the bytes do not arrive.
    assert!(
        !result.contains("secret"),
        "a cell read another container's workdir: {result}"
    );
}

</storycode:copy>
<a id="f-c8a04313c63c-the_transcript_records_the_cell_not_the_sandbox"></a>
### `the_transcript_records_the_cell_not_the_sandbox`

<storycode:copy id="f-c8a04313c63c-the_transcript_records_the_cell_not_the_sandbox">
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

</storycode:copy>
[Read the test `a_confined_cell_is_killed_at_its_timeout` beside its requirement](../../../../guarantees/execution/a-cell-cannot-hang-a-run/contract.md#f-c8a04313c63c-a_confined_cell_is_killed_at_its_timeout).

### Assemble `crates/hickory-executor-sandbox/tests/confinement.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-sandbox/tests/confinement.rs"><storycode:paste select="#f-c8a04313c63c-per_shell"/><storycode:paste select="#f-c8a04313c63c-available"/><storycode:paste select="#f-c8a04313c63c-started"/><storycode:paste select="#f-c8a04313c63c-a_cell_can_work_in_its_own_workdir"/><storycode:paste select="#f-c8a04313c63c-a_cell_cannot_write_outside_its_workdir"/><storycode:paste select="#f-c8a04313c63c-a_cell_cannot_read_your_dotfiles"/><storycode:paste select="#f-c8a04313c63c-a_cell_has_no_network_unless_the_document_grants_it"/><storycode:paste select="#f-c8a04313c63c-which"/><storycode:paste select="#f-c8a04313c63c-declaring_the_network_opens_it"/><storycode:paste select="#f-c8a04313c63c-one_containers_cells_share_a_tmp"/><storycode:paste select="#f-c8a04313c63c-two_containers_do_not_share_a_tmp"/><storycode:paste select="#f-c8a04313c63c-a_cell_cannot_read_another_containers_workdir"/><storycode:paste select="#f-c8a04313c63c-the_transcript_records_the_cell_not_the_sandbox"/><storycode:paste select="#f-c8a04313c63c-a_confined_cell_is_killed_at_its_timeout"/></storycode:file>

</storycode:doc>
