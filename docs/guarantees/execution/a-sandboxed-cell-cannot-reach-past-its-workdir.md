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

**Windows gets nothing, and is told so.** There is no sandbox this executor
can drive there, so it **refuses to run** and names the alternatives (WSL2,
the Docker executor, or the local executor's stated lack of isolation).
Silently running unconfined would be the worst outcome available: the user
asked for isolation, believes they have it, and does not.

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
- Result: verified on Linux/bubblewrap; macOS unrun; Windows is a refusal, not
  a behaviour
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
