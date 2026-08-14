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
- Date: 2026-08-13
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
- Caveats — what LLM review could NOT establish:
  - **macOS has never been run.** The Seatbelt profile is written from its
    documented syntax and has no test on real hardware. `sandbox-exec` is also
    deprecated by Apple; if it is removed, this degrades to a refusal on macOS
    too.
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
  (7 tests against the real sandbox, skipped loudly where none exists) plus 8
  policy unit tests.
