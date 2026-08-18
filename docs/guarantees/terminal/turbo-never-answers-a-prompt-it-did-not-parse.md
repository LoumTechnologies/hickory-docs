# Turbo Never Answers A Prompt It Did Not Parse

Given turbo is on, when a session is waiting on a prompt, then that prompt is
answered automatically **only** if all three hold:

1. the prompt was **declared** by the program itself, with its own choices —
   never one recognised by its shape on screen;
2. exactly one of those choices is non-destructive;
3. that choice is the one taken.

Anything else is left for a person: a guessed prompt, a prompt whose choices
are all destructive, a prompt with two harmless answers (a choice between two
harmless outcomes is still a decision), and a prompt with no choices at all.

Turbo is off until it is turned on, and is not persisted — an auto-answerer
that comes back after a restart you did not ask for is a surprise nobody wants
twice.

The reason for rule 1 is that a guessed prompt is exactly that: we can see
that something is being asked, not what the answers mean. Raising a session to
*needs you* on a guess costs a glance. Answering one on a guess costs an
answer the person never gave.

Auto-answering happens in one named place — `sweep_turbo`, called explicitly
by the list route — rather than inside anything that reads state, so reading
the queue never changes the world.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `hick_term::turbo::turbo_choice` is the whole rule, pure, and
  refuses on `PromptSource::Guessed` before looking at anything else.
  `hick_term::registry::Terminals::sweep_turbo` is the only caller, invoked
  from `serve::terminal::list`; `Terminals::turbo` starts false and is set
  only by `PUT /api/terminals/turbo`, with nothing writing it to disk.
  Guessed prompts are produced by `hick_term::prompt::looks_like_a_question`,
  which is confined to that module and considers only the last non-empty line
  so an answered question does not hold a session hostage.
  The client half is `apps/web/src/terminal/AttentionCard.tsx`, which renders
  a plain input line — never invented buttons — for a prompt with no choices.
- Test coverage: `hick_term::turbo::tests` covers all four refusals and the
  one acceptance. `hick_term::registry::tests::
  turbo_leaves_a_guessed_prompt_for_a_person_even_when_it_is_on` and
  `turbo_off_answers_nothing_at_all` cover the sweep.
  `apps/web/src/terminal/AttentionCard.test.tsx::
  never_invents_buttons_for_a_prompt_it_only_guessed_at` covers the card.
