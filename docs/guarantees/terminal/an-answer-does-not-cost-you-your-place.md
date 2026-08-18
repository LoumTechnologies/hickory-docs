# An Answer Does Not Cost You Your Place

Given a session at the top of the attention queue, when its prompt is answered
from the attention card, then the answer reaches that session's terminal, the
session leaves the queue on the same request, and the pane the person was
working in is not closed, rearranged, or unfocused.

The card is the point of the queue. If answering meant going to a terminal,
reading it, replying, and coming back, the queue would only be a prettier way
of checking on things.

Two details carry it:

- `POST /api/terminals/:id/answer` writes **and** clears the declared prompt,
  so the session leaves the queue immediately rather than on the next poll —
  otherwise the card sits there for up to a second looking unanswered, and
  gets answered twice.
- The card shows the program's own choices when the prompt was declared, and a
  plain input line when it was only guessed at. It never invents a button for
  a prompt whose answers we do not know (see
  `turbo-never-answers-a-prompt-it-did-not-parse.md`).

Opening the terminal is offered on the card, never done for you.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: partially verified — see caveat
- Evidence: `crates/hickory-cli/src/serve/terminal.rs::answer` writes the
  choice's `send` and then calls `Session::declare_prompt(None)`, returning
  the fresh summary. `apps/web/src/terminal/AttentionCard.tsx` renders choices
  or an input line by `prompt.source`/`choices.length`, and its buttons call
  back rather than navigating. In `WorkspaceView`, the card's `onAnswer` calls
  `terminals.answer` only; `onOpen` is the one path that touches the layout,
  and it runs from a click.
- Test coverage: `apps/web/src/terminal/AttentionCard.test.tsx` covers the
  choices, the exact bytes sent, the guessed-prompt input line, the
  destructive marking, and the finished-with-changes wording.
- Caveat: the "does not cost you your place" half — that answering leaves the
  layout and the focused pane untouched — is not yet covered by a test. It
  rests on the card's handlers not calling `setLayout`, which an LLM reviewer
  should re-read rather than assume. A workspace-level test that answers a
  prompt and asserts the layout is identity-equal afterwards would close this.
