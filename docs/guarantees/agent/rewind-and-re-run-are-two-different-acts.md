# Rewind And Re-Run Are Named Apart, And The Difference Is What Is Kept

Given a conversation in the dock, when a person changes their mind, then the
two available acts are distinct and each says what it keeps at the point the
choice is made:

| | Where it lives | The change of mind is |
|---|---|---|
| **Rewind** | one session file, one conversation, an abandoned branch in the tree | **kept** — visible in the file forever |
| **Re-run** | a second session file; the first is a draft you may discard | **not kept** — nothing in session 2 refers to session 1 |

**A re-run is not a reenactment.** Every byte of the second session is written
by the harness, the tools really ran, the result really came out. It is a
complete record of itself and stands without the first ever having existed —
which the product already treats correctly, because `hick init` gitignores
`sessions/`, so discarding the first attempt is the default rather than a
decision.

**The carry is what moves between them**, and it is not the transcript: a
better opening prompt, the tests you kept, the approaches you ruled out.
`hick ingest --from carry <session>` writes the opening prompt into an ordinary `.hick`
document and leaves the rest as empty, named slots.

Corollaries that are part of the guarantee:

- **It does not distil for you.** The tests worth carrying are the ones you
  READ and kept. They were written by an agent that had already seen one
  implementation, so they tend to pin that implementation's incidental choices
  — call order, internal names, an error string — rather than the behaviour you
  wanted, and carrying all of them binds the next attempt to the first one's
  arbitrary decisions while appearing to have chosen freely. The guard is not
  mechanical, and the tool must not pretend otherwise.
- **A carry invents no fourth artifact.** It is an ordinary `.hick` document:
  no new file type, no new store, no new gitignore rule.
- **A carry is written outside `sessions/`**, and therefore committed by
  default. `sessions/` is gitignored, and a carry that vanished with the
  session it distils would be pointless — and a carried requirement whose only
  support is a session nobody has reads as pulled out of the air.
- **An existing carry is never overwritten.** The whole point is what you kept.
- **Unfilled slots are reported, never enforced.** An empty slot is the normal
  state of a carry a second old.
- **Tests are one instance and not the mechanism.** Building "carry the tests"
  as a primitive would be building the example rather than the thing.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `apps/web/src/components/ChatDock.tsx` — `/rerun` in `parseSlash`,
    `REWIND_TIP` / `RERUN_TIP` on the affordances, `SLASH_HELP`, and the
    `rerun` handler's note.
  - `crates/hickory-cli/src/carry.rs` — `opening_prompt`, `carry_source` (the
    three slots and what each says), `unfilled_slots`, `carry_from_session`
    (the outside-`sessions/` default, the never-overwrite refusal, the
    not-a-session refusal).
  - `crates/hickory-cli/src/main.rs` — `hick ingest --from carry` and its report.
  - Tests: `crates/hickory-cli/src/carry.rs` unit tests (5);
    `apps/web/src/components/ChatDock.test.tsx` (3).
- Caveat requiring LLM review: self-containment is not checked. A carried
  requirement whose only support is a discarded session is a dangling
  citation in shape, and `hick cites` is where that check belongs — it is step
  4 of that design and is not built. Edited sessions (the declared layer over
  a frozen base) are step 3 and are also not built.
