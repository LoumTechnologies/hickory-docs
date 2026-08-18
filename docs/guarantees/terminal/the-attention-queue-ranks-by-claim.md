# The Attention Queue Ranks By Claim

Given every terminal session in the window, when the attention queue is read,
then it contains exactly the sessions making a claim on a person, ordered:

1. **needs-you** — blocked on a decision;
2. **failed**;
3. **finished** with uncommitted changes;
4. **finished** clean;

and within each band, the oldest waiter first, ties broken on session id so
the order never depends on storage order.

Working and idle sessions never enter the queue: one is busy and one is done
being busy, and neither is waiting on you. **Monitors never enter it either**,
whatever state they are in — a dev server that just fell over belongs in the
dock (see `a-monitor-never-steals-focus.md`).

Finished-with-uncommitted-changes outranks a clean finish however much older
the clean one is, because it still has a decision in it — commit, amend, throw
away — and a clean finish is only news.

⌘J walks this order and nothing else. Its cursor is a session **id**, never an
index: between two presses the session you were on may have been answered,
closed, or gone back to work, and an index would then point at whoever
shuffled into that slot. When nothing is waiting, ⌘J says so rather than
appearing not to work.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `hick_term::attention::attention_order` is the ordering, pure,
  with the bands in `band()`. `hick_term::registry::Terminals::attention`
  filters monitors out before building the claims, and
  `crates/hickory-cli/src/serve/terminal.rs::list` serves that one order to
  every surface, so the queue, the session rows, and ⌘J cannot disagree.
  The cursor step is `apps/web/src/lib/attentionCursor.ts::nextInQueue`,
  driven by `WorkspaceView`'s `nextAttention`, which sets `nothingWaiting`
  when the queue is empty.
- Test coverage: `hick_term::attention::tests` covers the band order, the
  dirty-outranks-clean case, oldest-first, the exclusion of working/idle, and
  storage-order independence. `hick_term::registry::tests::
  a_monitor_never_enters_the_queue_however_loudly_it_fails` covers monitors.
  `apps/web/src/lib/attentionCursor.test.ts` covers the wrap, the
  session-left-the-queue case, and the empty queue.
  `crates/hickory-cli/tests/serve_terminals.rs::
  a_command_that_fails_is_failed_and_leads_the_queue` and
  `a_monitor_stays_out_of_the_queue` assert the order over the wire.
