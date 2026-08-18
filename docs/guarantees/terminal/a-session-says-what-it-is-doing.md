# A Session Says What It Is Doing

Given any terminal session, when it is asked for its state, then it answers
exactly one of **needs-you**, **working**, **idle**, **finished**, or
**failed**, decided in this order:

1. an exited child is `finished` (exit 0) or `failed` (anything else) —
   whatever was on screen when it died;
2. a session waiting on a prompt is `needs-you`;
3. a session with a foreground child other than its own shell is `working`,
   however quiet it is;
4. otherwise silence decides: `working` under 1 500 ms since the last byte,
   `idle` at or beyond it.

Asking twice in a row must answer twice. This is a real constraint, not a
truism: the client polls, so the second ask — the one where nothing has
changed — is the common path, and taking the state lock twice in one pass
hangs the whole server rather than failing.

Rule 3 is the reason the quiet timer alone will not do: a compile that prints
nothing for a minute is not idle. Where the platform will not name the
foreground process (Windows, where `portable-pty` reports no process group
leader), rule 4 carries the classification alone and the state is a weaker
claim — a long silent build there reads as idle.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `hick_term::classify::classify` is the whole decision, pure, over
  a `Signals` struct the caller fills. `Session::summary` gathers the signals
  (`exit_code` via `Child::try_wait`, `foreground_child` by comparing
  `MasterPty::process_group_leader` with the shell's pid, `quiet_ms` from the
  reader thread's last write) and takes `state_since` in one scoped lock.
  `IDLE_AFTER_MS` is the 1 500 ms above.
- Test coverage: the six tests in `hick_term::classify::tests` cover each rule
  and the ordering between them, including the exited-with-a-prompt case and
  the Windows `foreground_child: None` path.
  `hick_term::registry::tests::asking_what_a_session_is_doing_twice_answers_twice`
  bounds the second-ask case with a timeout, because the failure it guards
  against hangs rather than fails.
  `crates/hickory-cli/tests/serve_terminals.rs::
  a_command_that_fails_is_failed_and_leads_the_queue` asserts `finished` and
  `failed` over the wire.
