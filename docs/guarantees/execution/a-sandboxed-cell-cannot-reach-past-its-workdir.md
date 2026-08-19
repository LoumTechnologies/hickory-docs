# A Sandboxed Cell Cannot Reach Past Its Workdir

Given `HICKORY_EXECUTOR=sandbox`, when a document's cells run, then each cell
may write only its own container workdir and a private `/tmp`; it sees an
empty home directory rather than yours; it cannot read another container's
workdir; and it has no network unless the document declared one with
`<hick:allow>`.

This is the executor for a document you did not write. The default
`LocalExecutor` runs a cell as you, with your files and your network — it says
so in its own documentation and compares itself to `make` — which is a fair
deal for your own work and a bad one for a document somebody sent you or an
agent produced. "Read every cell before running it" is advice nobody follows
twice.

## What it does not claim

**It is not reproducibility.** The interpreter a cell runs is still whichever
one your machine has, so two machines can still disagree. Isolation and
determinism are different problems: a sandbox restricts what already exists,
while a VM image brings a toolchain with it. The second is what
`HICKORY_EXECUTOR=canopy` is for, pointed at a node you run.

**It is not equal on every platform.** Enforcement is bubblewrap on Linux and
Seatbelt on macOS, and they are not the same strength — Seatbelt cannot give a
cell its own process namespace, and hides your home by denying reads rather
than by mounting an empty one. `Sandbox::describe` states which is in force.

**The private `/tmp` is a Linux-only part of this promise, and that is a
boundary rather than a bug.** Bubblewrap bind-mounts each container's own
directory at `/tmp`, so the path is private per container. Seatbelt has no
mount namespaces: the literal path `/tmp` is one directory for every process on
the machine and there is nowhere to redirect it to. Denying `/tmp` outright was
considered and rejected — it would break every document that writes a scratch
file there, trading a real capability for an isolation property macOS will not
grant either way. So on macOS, **two containers share `/tmp`**. Everything else
holds: each container's workdir and its own tmp directory are private, which is
what `a_cell_cannot_read_another_containers_workdir` covers. A run that needs
`/tmp` isolation wants the Docker executor.

**Windows confines through AppContainer, and that is the newest and least
proven of the three.** `appcontainer.rs` runs a cell at Low integrity under its
own package SID, grants that SID an ACE on the workdir and nowhere else, gives
it no capability SIDs unless the document declared a network, and puts it in a
job object with `KILL_ON_JOB_CLOSE`. Because AppContainer is applied by the
*parent* at process creation, `hick` re-invokes itself as the hidden
`__sandbox-run` subcommand rather than exec'ing a wrapper the way bubblewrap and
Seatbelt do.

**It had never been run, and running it found two defects that no amount of
reading would have.** Measured on a Windows 11 guest, 2026-08-19:

1. **Every confined cell failed before it started.** The confined line quotes
   the launcher, the workdir, and the command — six quote characters — and
   `cmd /?` documents that `cmd /C` preserves quoting only when there are
   *exactly two*; otherwise it strips the leading quote and the **last** one.
   The program name therefore reached `CreateProcess` as `C:\...\hick.exe"`,
   a filename Windows rejects, and the cell died with "The filename, directory
   name, or volume label syntax is incorrect". Fixed by wrapping the whole line
   in one more pair of quotes (`LocalExecutor::shell_command`).
2. **The cell's own shell syntax was executed by the outer shell.** `cmd`
   applies `> < & | ^ ( )` before `CommandLineToArgvW` splits the line, and —
   contrary to what quoting suggests — it does so *inside* double quotes too. A
   cell running `echo hello > note.txt && cat note.txt` had its argument
   swallowed by the outer redirect and the second half run **unconfined**.
   Fixed by caret-escaping those characters in `windows_quote`, which `cmd`
   consumes, so the confined process receives the bare text. The escaping is
   **asymmetric**, which measuring caught and reasoning had not: `cmd` resolves
   the *program* before it consumes carets, so escaping the launcher's path
   makes it unfindable (`C:\Program Files ^(x86^)\...` → "The system cannot
   find the path specified"). The program is quoted plain; every argument is
   escaped. A literal caret in an argument must be doubled for the same reason
   — a directory named `has^caret` reached the confined process as `hascaret`.

A third defect is a product bug rather than a Windows one: confinement
re-invokes `current_exe()`, which is only the CLI when the CLI is what is
running. The desktop app (`Hickory Docs.exe`) and any test binary have no
`__sandbox-run` subcommand, so both would have re-invoked something that cannot
confine. `policy::launcher` now resolves a real `hick` — explicitly via
`HICKORY_SANDBOX_LAUNCHER`, else itself, else one beside or one directory above
— and confinement **refuses** when it finds none rather than running the wrong
binary.

What is still weaker than Linux and macOS: `%VAR%` in a cell's command is
expanded by the outer `cmd` before the cell sees it, which `sh -c` with single
quotes does not do on the other two platforms. `cmd` has no escape for `%`, so
closing that means not routing the confined line through an outer shell at
all — the launcher already runs `cmd.exe /C` inside the AppContainer, so the
outer one buys nothing.

Where a machine has no sandbox at all, the executor still **refuses to run**
rather than silently running unconfined, and names the alternatives (WSL2, the
Docker executor, or the local executor's stated lack of isolation) — the user
asked for isolation, and believing they have it when they do not is the worst
outcome available.

## Deliberate details

- **The empty home.** Read-only was not enough. A cell that can *list*
  `~/.ssh` has already told its author which keys exist, and "it cannot
  exfiltrate because the network is off" is one mistake away from false. The
  cell gets an empty writable home instead, which also makes it more
  reproducible: a cell that behaves differently because of somebody's dotfiles
  is a cell nobody can re-run.
- **The transcript records the cell, not the sandbox.** Transcripts are woven
  into the document, so they show the command as written. A page of
  `bwrap --ro-bind …` in the middle of somebody's documentation would be
  noise about our implementation.
- **Mount order is load-bearing.** The workdir lives under `/tmp`, and
  bubblewrap applies mounts in sequence — a private `/tmp` mounted after the
  workdir bind hides the one directory the cell needs, failing with
  "Can't chdir", which reads like a bug in the executor.
- **A fork inherits and attenuates.** A forked container starts from its
  parent's grants intersected with its own, so a branch can never reach
  further than what it came from.

---

Last LLM verification:
- Date: 2026-08-18
- Reviewer: Claude (Opus 5)
- Result: verified on Linux/bubblewrap and on macOS/Seatbelt (two defects found
  and fixed there, below); **Windows/AppContainer is implemented and entirely
  unrun**
- Evidence:
  - `crates/hickory-executor-sandbox/` — `policy.rs` builds the argv and the
    Seatbelt profile; `lib.rs` wraps `LocalExecutor` so transcripts, volumes,
    forks and mounts have exactly one implementation.
  - `tests/confinement.rs` runs the real sandbox: a cell writes its own
    workdir; a write to `/etc` fails and the file does not appear; a write to
    `$HOME` succeeds into the private tmpfs and the real home never sees it;
    `~/.ssh` is not listable; one container cannot read another's workdir; a
    Python `socket.create_connection` to a public address is BLOCKED; and the
    transcript contains `echo hello` rather than `--ro-bind`.
  - Driven end to end with a deliberately hostile document. Under
    `HICKORY_EXECUTOR=local` it wrote `~/hickory-pwned` and listed real key
    filenames. Under `HICKORY_EXECUTOR=sandbox` the same document reports
    `ls: cannot access '/home/…/.ssh': No such file or directory`, the write
    lands nowhere, and the host's home is untouched.
- **Measured on macOS 2026-08-18, which found two real defects.** Nothing in CI
  had ever run this crate on a Mac (`ci.yml` was Ubuntu only), and the release
  job's macOS smoke test passed because a GitHub runner sets `TMPDIR` under the
  workspace rather than to `/var/folders/…`.
  - **A cell could not write its own workdir.** `seatbelt_profile` put the
    workdir into the policy as given, and Seatbelt matches the path the kernel
    resolves to — on macOS `/var` is a symlink to `/private/var`, so the grant
    named a path the kernel never sees. Every cell that wrote a file failed with
    `Operation not permitted`, and `hick test examples/text-tools-tour.hick`
    failed on its first cell. The profile now resolves the path (`resolve`), and
    `the_seatbelt_profile_grants_the_resolved_workdir_not_the_symlinked_one`
    pins it.
  - **A cell could read every other container's workdir.** The profile's
    `(allow file-read*)` is global, and bubblewrap's isolation here comes from
    namespaces Seatbelt does not have. It now denies each **named peer** —
    `LocalExecutor::peer_dirs` lists the other containers' workdirs and tmp
    directories at the moment the command runs. Pinned by
    `the_seatbelt_profile_hides_sibling_containers` and by
    `a_cell_cannot_read_another_containers_workdir`, which now passes on both
    platforms.

    Denying the *directory the containers share* was tried first and is wrong,
    which is worth recording because it looks tidier: that directory is also
    what a `<hick:volume>` mounted into the cell's own workdir resolves through,
    so denying it made `cd project` fail with `Not a directory` in a document
    that had done nothing wrong. `the_seatbelt_profile_does_not_deny_the_directory_containers_share`
    keeps it from coming back. A container started *after* the command began is
    not in the list, since it did not exist when the policy was written.
  - A macOS job was added to `ci.yml`, and it forces
    `TMPDIR="$(getconf DARWIN_USER_TEMP_DIR)"` so the runner cannot go on hiding
    the symlink hazard the way it did. With these fixes the full workspace suite
    passes on macOS — 1418 tests, 0 failures — which it had never done.
- Caveats — what LLM review could NOT establish:
  - **Windows has never executed a single confined cell.** `appcontainer.rs`
    is 400 lines of Win32 that nothing has run: not CI (Linux and macOS only),
    not the release smoke test (`--version` on Windows, by matrix flag), and not
    a developer — the crate's own `confinement.rs` would exercise it, since
    `available()` skips only when `Sandbox::detect()` is `None` and it returns
    `AppContainer` on any Windows box, but no job runs those tests there.
    Everything this file says about Windows is therefore read off the source.
    Closing it needs a Windows machine running `cargo test -p
    hickory-executor-sandbox`, and a flavor that exercises the shipped binary on
    a guest — neither exists yet.
  - **macOS is now run, on one machine and one version.** Everything above was
    exercised on macOS 15.7.7 (Intel), and `ci.yml`'s macOS job runs on
    `macos-14` (arm64). Neither is every Mac. `sandbox-exec` also remains
    deprecated by Apple; if it is removed this degrades to a refusal on macOS,
    the way Windows already does.
  - **Reads of the system are permitted by design**, so a cell can still read
    world-readable files outside your home — `/etc/passwd`, any repository on
    a shared path. The threat model is "cannot write your machine, cannot
    exfiltrate", not "cannot see anything".
  - `--unshare-all` gives a network namespace, but `/sys` is bind-mounted from
    the host, so `ls /sys/class/net` still lists the host's interfaces. That
    is cosmetic — a connection attempt fails — but it looks alarming, and the
    test asserts reachability rather than the listing for exactly that reason.
  - Nothing bounds CPU, memory, or wall time yet. A cell can still spin
    forever; it just cannot touch your files while doing it.
- Test coverage: `crates/hickory-executor-sandbox/tests/confinement.rs`
  (10 tests against the real sandbox, skipped loudly where none exists — and
  where a property belongs to one sandbox rather than the guarantee, the test
  now says which and why instead of asserting the mechanism) plus 21 policy
  unit tests, three of them added for the defects above.
