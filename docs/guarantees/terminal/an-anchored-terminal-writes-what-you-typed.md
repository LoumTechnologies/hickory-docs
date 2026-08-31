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
  recording nothing. *Having a hook is a property of the binary, not of the
  shell's name*: bash reports commands only from **4.4**, the version that
  added `PS0`, and the `/bin/bash` macOS ships is 3.2.
- **A leading space means do not record**, in every shell hick anchors. That
  convention is honoured by hick rather than left to the shell, because the
  shells disagree about it completely — see the verification notes.

What is claimed about secrets is exactly: **a line that looks like a secret
stops the recording.** Never "your secrets are safe here" — a scanner catches
published prefixes and high-entropy strings and cannot catch a short
password, and this product does not say things it cannot prove.

---

Last LLM verification:
- Date: 2026-08-31
- Reviewer: Claude (Opus 5)
- Result: verified, after two defects that both hid behind a common default
- What changed:
  - **A bash too old for `PS0` no longer looks anchorable.**
    `Integration::reports_commands` splits the two halves of the shell
    integration, which had been conflated: reporting the working directory
    (OSC 7, `PROMPT_COMMAND`) works on every bash ever shipped, while
    reporting the command (OSC 633, `PS0`) needs 4.4+. On a stock Mac the
    rcfile installed cleanly, the prompt and directory worked, and `PS0` sat
    there as an ordinary variable bash 3.2 never expands — so the terminal
    offered to anchor and would have recorded nothing. `has_ps0` now asks the
    binary that will actually be spawned (`shell -c 'printf … $BASH_VERSINFO'`)
    rather than assuming, so a Homebrew bash 5 works on the same Mac where
    `/bin/bash` does not, and a probe that cannot be run answers *false* —
    a maybe is treated as a no, because the other failure is a terminal that
    says it is recording and is not. `POST /api/terminals/{id}/anchor` names
    the version and `brew install bash` rather than repeating "bash and zsh
    only", which was true by name and false in fact.
  - **The leading space now survives the hook.** `history 1` prints
    `<padding><number><2 chars><command verbatim>`; the hook stripped *all*
    leading whitespace after the number, which got a plain command right and
    silently ate the user's own leading space. That space is the whole input
    to `Suspension::HiddenByLeadingSpace`, so the rule this guarantee states
    — a leading space means do not record — could not fire in bash at all.
    The separator is now cut by length (`${body:2}`), measured on bash 5.3.9.
    It survived because `HISTCONTROL=ignorespace` is a common distro default
    and hides the line entirely; CI's Linux image sets it, this developer's
    Mac does not, and the assertion had been passing vacuously.
- Evidence:
  - `crates/hick-term/src/shell_integration.rs` — `has_ps0`,
    `BASH_PS0_SINCE`, the `${body:2}` separator cut, and two unit tests
    (`a_shell_that_cannot_say_its_version_does_not_claim_to_report_commands`,
    `an_old_bash_still_integrates_but_reports_no_commands`).
  - `crates/hick-term/src/session.rs` — `Session::reports_commands` asks the
    integration instead of testing it for presence.
  - `crates/hick-term/tests/typed_commands.rs` — `for_each_shell` skips a
    shell the *product* says cannot report, so the test and the anchor
    endpoint can never disagree about which shells are hookable.
- Caveat: no bash 3.2 runs in CI. The gate is covered by unit tests that
  substitute a program printing no version; the real 3.2 path was measured by
  hand on macOS 15.7.7 (`/bin/bash` 3.2.57) on 2026-08-31.

Previous verification:
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
  - ~~The foreground-program suspension is nearly unreachable…~~ **Built
    2026-08-27, and it is not a suspension.** Modelling it as one was wrong:
    the spec's own words are "the terminal is yours; the document resumes
    when it exits", so nothing has to be resumed. It is a *notice*
    (`hick_term::anchor::ForeignInput`), said in the terminal once per
    program and shown standing in the bar while it lasts, and the anchor
    keeps recording either side of it.
    The other half that reasoning got wrong: `tcgetpgrp(master) !=
    shell_pgid` is true for **every** ordinary command — `dotnet build` holds
    the terminal exactly as `less` does — so it is not a signal that anything
    is amiss. The question is not "is a child running" but "are the keys
    being typed right now going to one", which is why this is checked on the
    **input path** and nowhere else.
    Verified against both shells: typing into `python3` inside an anchored
    terminal records neither `print(...)` nor `quit()`, says why, keeps the
    lines either side of it, and clears the note when the shell returns.
    Naming the program is Linux-only (`/proc/<pid>/comm`); elsewhere the
    message says "a program" and still says the useful half.
  - **The secret scanner is a heuristic and its tuning is unmeasured.** The
    published prefixes are exact; the entropy rule (32+ characters, mixed
    case and digits, base64-ish alphabet) was chosen by reasoning and tested
    against a handful of real build commands, not against a corpus. False
    positives cost a suspension somebody must notice and clear.
  - **Output is scanned, and only warned about — a deliberate departure from
    the spec, not an omission.** `hick run` reads every cell's recorded
    output with the same scanner and names the cell when a line looks like a
    credential, saying plainly that it HAS been recorded.
    The spec asks for "the same suspend" on the output path, and its own
    argument for stopping does not carry over. In a terminal the recording is
    automatic and unattended, and a false positive costs a suspension a person
    can see and resume from. In a document there is nothing to resume:
    declining to record a cell's output changes what the document weaves, so
    the same false positive would report drift, fail `hick test`, and keep
    failing until somebody changed their program's output. A heuristic that
    can break a build is a different trade from one that can pause a
    recording, and the author is present at `hick run` in a way they are not
    when a terminal is recording them.
    What is missing before this could become a refusal is a way for a
    document to say "yes, I meant that" — and that does not exist. Recorded
    here rather than decided quietly.
  - **`recorded` counts lines that reached the document**, not lines that
    were accepted. It used to be incremented before the write and was
    therefore briefly false — a caller trusting the count read a document the
    line had not reached yet. Found by a test that waited on the count and
    then read the file.
  - **A dropped command stops the recording, and losing the broadcast is how
    it is detected.** If the anchor task falls 256 commands behind, `Lagged`
    unanchors the session with a message. That is correct and it has never
    been observed, because nothing has typed that fast.
  - **Nothing was watched in a browser.** The `AnchorBar` is unit-tested and
    mounted, but no human or automated pass has looked at an anchored
    terminal in the running app.
