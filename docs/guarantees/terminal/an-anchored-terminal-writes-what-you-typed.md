# An Anchored Terminal Writes What You Typed

Given a terminal anchored to a container in a document, when you type a
command into it, then that line becomes part of that container's cell — and
the cell is always **a prefix of the session that reproduces**:

- **The anchor is the container**, not the document. Several `hick:exec`
  blocks naming one container is already how the language says "commands that
  share state", so a terminal bound to `sdk` *is* that shell. Say "anchored to
  a container", never "attached to a document".
- **One cell that grows**, not one cell per line. A session of forty
  exploratory commands is one cell, because a document is something a person
  reads.
- **The line recorded is the line you typed**, as the shell assembled it — a
  pipeline is one line, a `for` loop is one line, a heredoc keeps its
  newlines.
- **Suspend, never filter.** A line that may not be recorded stops the
  recording rather than being skipped over. A cell missing its third line
  claims a run that cannot reproduce, because lines four onward depended on
  it; a cell that stops at line two is simply true.
- **Suspension is sticky, and resuming starts a new cell.** After a
  suspension the shell holds state the document does not describe, so
  continuing would claim that state came from the lines above.
- **The scan gates the write and never undoes one.** The document is a live
  CRDT that autosaves and syncs, so a byte that reached it may already be on
  another machine.
- **Never anchor silently.** A terminal that is writing says which document
  and which container for as long as it is doing so, and a shell hick has no
  hook for is **refused by name** rather than left looking anchored while
  recording nothing.
- **A leading space means do not record**, in every shell hick anchors. That
  convention is honoured by hick rather than left to the shell, because the
  shells disagree about it completely — see the verification notes.

What is claimed about secrets is exactly: **a line that looks like a secret
stops the recording.** Never "your secrets are safe here" — a scanner catches
published prefixes and high-entropy strings and cannot catch a short
password, and this product does not say things it cannot prove.

---

Last LLM verification:
- Date: 2026-08-27
- Reviewer: Claude (Opus 5)
- Result: verified against real bash **and real zsh**, which turned out to
  disagree in a way that mattered
- Evidence, in the order the bytes travel:
  - `crates/hick-term/src/shell_integration.rs` — the hook, added beside the
    OSC 7 one the same mechanism already installs. bash uses **`PS0`**;
    the two obvious alternatives are wrong and the comment says why.
  - `crates/hick-term/src/command.rs` — the OSC 633 wire format and its
    parser. Percent-encoded byte-wise, the same encoding OSC 7 uses, so a
    command carrying newlines or quotes needs no second escaping scheme.
  - `crates/hick-term/src/session.rs` — `typed_commands()` broadcasts them,
    `reports_commands()` says whether this shell can be anchored at all, and
    `inject()` says something IN the terminal without saying it TO the shell.
  - `crates/hick-term/src/anchor.rs` — the decision. Pure.
  - `crates/hickory-cli/src/anchor.rs` — where the line lands. Pure.
  - `crates/hickory-cli/src/serve/anchored.rs` — the table and the task, and
    nothing else.
  - `apps/web/src/terminal/AnchorBar.tsx` — what the person sees.
- Test coverage:
  - `crates/hick-term/tests/typed_commands.rs` — **against a real PTY, for
    every hooked shell installed**, and it fails rather than passes if none
    is: a pipeline and a `for` loop each arrive as ONE line, nothing is
    reported before anything is typed, an empty Enter reports nothing, and a
    line the shell kept out of its history never arrives as a fresh,
    recordable report.
  - `crates/hick-term/src/anchor.rs` unit tests — a secret suspends, the
    suspension is sticky, an unchanged history number suspends rather than
    repeating the previous line, resuming reports that it is a new cell, and
    the scanner leaves ordinary build vocabulary alone.
  - `crates/hickory-cli/src/anchor.rs` unit tests — the cell grows, everything
    outside it comes back byte for byte, a self-closing cell is a declaration
    and is not grown, a bare document does not gain a wrapper, and what is
    written still parses.
  - `crates/hickory-cli/tests/anchored_terminal.rs` — **the whole path**,
    through the HTTP API, against **both** shells: typing grows one cell in
    order, unanchoring stops the document receiving, an `export …=sk-…` never
    reaches the document while everything after it is left out too, a line
    typed with a leading space never reaches it either, the anchor says why
    in words that do not overclaim, resuming starts a second cell, and an
    un-hookable shell is refused with a message naming bash and zsh and
    saying the terminal still works. The zsh runs additionally exercise the
    `ZDOTDIR` forwarding, which is the part of the existing integration the
    new hook had to be added to without breaking a person's own startup
    files.
- Caveat requiring review:
  - ~~zsh is written and never run.~~ **Resolved 2026-08-27** — zsh 5.9 was
    installed and measured, and the assumption it was carrying was wrong in
    both directions. `$HISTCMD` inside `preexec` is the slot a line *would*
    take, not a record of one taken: a line hidden by `HIST_IGNORE_SPACE`
    advances it and then gives it back, so the **next** genuine command
    reuses the number. Under the old rule that meant zsh recorded the hidden
    line (its number had advanced) and then suspended the innocent one after
    it. Neither wrote anything false, and both were wrong.
    The fix is the leading-space rule above, applied in hick rather than
    inferred from the shell: bash never reports such a line at all, zsh
    reports it in full, and hick stops on it either way. What is NOT done is
    detecting zsh's history options (`HIST_IGNORE_SPACE`,
    `HIST_IGNORE_DUPS`) — re-implementing another program's rules in a hook
    is how a hook drifts, and the uniform rule needs none of it. The
    consequence worth stating: **a leading space suspends recording even when
    the shell would have kept the line**, which is a hick rule and not a
    shell one.
  - **Windows is undesigned.** ConPTY has no process groups; the shell hook
    itself is shell-shaped rather than PTY-shaped so it may well work, but
    nothing has run there and no `cmd`/PowerShell hook exists.
  - **The foreground-program suspension is nearly unreachable, by design and
    by accident.** Inside a REPL the shell's `PS0` never fires, so nothing is
    recorded without anything refusing — measured. `Recording::foreign_program`
    exists so a caller that detects it with `tcgetpgrp` can *say* so, and
    **nothing calls it yet**: the terminal does not currently announce "you
    are in `less`, recording is paused". That is a missing message, not a
    missing gate.
  - **The secret scanner is a heuristic and its tuning is unmeasured.** The
    published prefixes are exact; the entropy rule (32+ characters, mixed
    case and digits, base64-ish alphabet) was chosen by reasoning and tested
    against a handful of real build commands, not against a corpus. False
    positives cost a suspension somebody must notice and clear.
  - **Output is not scanned at all.** A command that *prints* a token is
    recorded by the transcript exactly as it always has been. The spec is
    clear this is the same exposure and not new here, and equally clear that
    the same suspend belongs on the output path. It is not built.
  - **A dropped command stops the recording, and losing the broadcast is how
    it is detected.** If the anchor task falls 256 commands behind, `Lagged`
    unanchors the session with a message. That is correct and it has never
    been observed, because nothing has typed that fast.
  - **Nothing was watched in a browser.** The `AnchorBar` is unit-tested and
    mounted, but no human or automated pass has looked at an anchored
    terminal in the running app.
