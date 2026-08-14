# Every Desktop Platform Can Confine A Cell

Given `HICKORY_EXECUTOR=sandbox` on Linux, macOS **or Windows**, when a
document's cell runs, then it may write only its own workdir, reaches the
network only if the document declared it, and cannot outlive the run that
started it. Where nothing can enforce that, the executor **refuses to run**
rather than running the cell unconfined.

The policy is one sentence on every platform. The enforcement is three
different mechanisms, and the differences are stated rather than smoothed
over:

| | Linux | macOS | Windows |
|---|---|---|---|
| Mechanism | bubblewrap | Seatbelt | AppContainer + job object |
| Writes | read-only bind of `/`, workdir bound rw | `(deny default)`, workdir granted | low-integrity package SID, workdir ACL'd for that SID |
| The user's home | empty tmpfs | reads denied | unreadable by default |
| Network | `--unshare-all`, `--share-net` when granted | `(allow network*)` when granted | `internetClient` capability when granted |
| Lifetime | `--die-with-parent` | — | `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` |

## Why Windows needed a different shape

On Linux and macOS the sandbox is a program you exec, and the wrapper confines
whatever it spawns. Windows has no such program: **AppContainer is applied by
the parent at process creation**, and a process cannot put itself into one. So
`hick` re-invokes itself as the hidden `__sandbox-run` subcommand, which makes
the `CreateProcessW` call with the security capabilities attached.

That keeps one shape across all three — `policy::wrap` returns a program and
arguments everywhere — without shipping a second binary that would need
separate installation and separate signing.

Two consequences worth knowing before relying on it:

- **Writes are granted by ACL, not hidden by a mount.** The container's SID
  starts with access to nothing, and the workdir is made writable by granting
  that specific SID an inherited ACE. Nothing else on the disk carries an ACE
  for it, which is the same guarantee bubblewrap gets from a read-only bind,
  arrived at from the opposite direction.
- **Reads are limited to what `ALL APPLICATION PACKAGES` may read.** Machine-
  wide interpreters work. An interpreter installed per-user under
  `%LOCALAPPDATA%` may be invisible to the cell, and the error says so.

---

Last LLM verification:
- Date: 2026-08-14
- Reviewer: Claude (Opus 5)
- Result: **partially verified — the Windows path has never been executed**
- Evidence:
  - `crates/hickory-executor-sandbox/src/policy.rs` — `Sandbox::detect`
    returns `AppContainer` on Windows without probing for a binary (there is
    none to probe); `wrap` builds the `__sandbox-run` re-invocation, with the
    `--` separator before the cell's command so a command starting with a
    flag is not read as one of ours. Tests
    `the_appcontainer_launcher_carries_the_policy_in_its_arguments` and
    `the_appcontainer_launcher_opens_the_network_only_when_granted`.
  - `crates/hickory-executor-sandbox/src/appcontainer.rs` — profile creation
    (reusing an existing one on `ERROR_ALREADY_EXISTS`), the workdir ACE via
    `SetEntriesInAclW` + `SetNamedSecurityInfoW`, the `internetClient`
    capability SID, `PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES` attached to
    a `STARTUPINFOEXW`, and the job object. `container_name` is a stable
    per-workdir hash, tested for stability and for the 64-character package
    name cap.
  - `crates/hickory-cli/src/main.rs` — `__sandbox-run` is hidden, its exit
    code is the confined command's, and a code that does not fit a `u8` is
    reported as failure rather than truncated (truncation can turn a non-zero
    code into zero). On non-Windows it fails with a message naming what to
    use instead, rather than being compiled out.
  - `crates/hickory-executor/src/lib.rs` — `LocalExecutor::shell()` picks
    `cmd.exe /C` on Windows and `sh -c` elsewhere. Without this the Windows
    sandbox could not run at all, since every command went to a shell Windows
    does not have.
  - `crates/hickory-executor-sandbox/src/lib.rs` — `windows_quote` implements
    the `CommandLineToArgvW` rules; tests cover a path with a space, a
    trailing backslash (which would otherwise escape the closing quote), and
    an embedded quote.
  - **Typechecked against the real Win32 API.** The module was compiled for
    `x86_64-pc-windows-msvc` against `windows` 0.58; four signature errors
    were found and fixed that way (`PSID`'s module, the arity of
    `CreateAppContainerProfile`, `DeriveAppContainerSidFromAppContainerName`
    returning its SID, and `CreateProcessW`'s command line not being an
    `Option`).
- Caveats — what LLM review could NOT establish:
  - **Nothing on Windows has been run.** No cell has been confined, no ACL
    applied, no escape attempted. The Linux confinement tests
    (`tests/confinement.rs`) prove the property on Linux only; there is no
    Windows equivalent because there is no Windows machine here. Until
    someone runs it, treat the Windows column above as *implemented and
    typechecked*, not as *verified*.
  - The capability SIDs returned by `DeriveCapabilitySidsFromName` are
    deliberately not freed: the process exits immediately after, and freeing
    them would invalidate the SID handed to `CreateProcessW`. This is sound
    for a launcher and would be a leak anywhere else.
  - Cross-container isolation on Windows rests on each container getting a
    different profile name, which follows from the per-workdir hash but has
    not been observed.
  - Windows has no equivalent of the empty-`$HOME` tmpfs: the user's files
    are unreadable because AppContainer denies by default, not because
    anything hides them.
- Test coverage: the argv, naming and quoting tests above, plus the Linux
  confinement suite that the shared policy is checked against. The Win32
  calls themselves are covered by the cross-compile and by nothing else,
  which is the gap this section names.
