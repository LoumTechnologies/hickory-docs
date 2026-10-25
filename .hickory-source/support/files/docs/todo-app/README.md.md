<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/todo-app/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `9d85a3469797619dec76fa7e4010c7c84b8216693da508a849d340c77031fea2`.

<book:entry path="docs/todo-app/README.md">
# The todo app — a full lifecycle in five documents

This is the demo. It is a working CLI todo app whose entire lifecycle —
meeting, domain model, requirements, tickets, code, acceptance tests — lives
in `.md` documents that verify each other.

```
meetings/2026-08-07-kickoff.md    decisions, each written down exactly once
        ↓ hick:upstream
domain.md                          glossary + those decisions, by reference
        ↓ hick:upstream
requirements.md                    requirements, tangled into work/*.task.md
        ↓ hick:upstream
implementation.md                  the code + acceptance cells that run it
        ↓ hick:file
todo.py                              the shipped program
```

## Try the demo

```sh
hick test docs/todo-app/          # all four documents pass
```

Now change one sentence — in `meetings/2026-08-07-kickoff.md`, make the
states decision say "four states" instead of "three":

```sh
hick test docs/todo-app/          # the meeting, the domain model, and
                                      # the requirements all fail together
```

Nothing scanned for the string "three". The domain model pastes the decision
fragment by reference and the requirements reach it two hops up, so all three
woven outputs changed the moment the source did.

Put it back and they pass again.

## What is verified, and what is not

- **Every requirement is also a ticket.** The files under `work/` are tangled
  from the requirement fragments, so a requirement and its ticket are the same
  bytes. Editing the requirement IS editing the ticket. They are in ticketry's
  format, without hick depending on ticketry.
- **The acceptance cells run the real CLI.** They set titles and filter on
  state rather than pinning the random ids, because pinning a random id would
  fail every run for no reason.
- **The decisions are pasted back into the meeting notes.** `hick:copy` blocks
  are extracted, not rendered — without the paste the notes weave to an empty
  section, and the one document where a decision is written down becomes the
  one document you cannot read it in.

## What an agent did here

The `blocked` task state was added by `hick agent`, not by hand. Asked to
add it, the agent amended the *decision* in the kickoff notes — two hops
upstream from the document it was pointed at — rather than patching the
requirement in front of it, then propagated down through the glossary, the
requirements, the tickets and the Python, added an acceptance cell for
`todo block <id>`, and re-ran everything.

Two harness bugs came out of that run and are fixed: an empty model response
used to kill a session mid-chain, and `verify` used to check only the primary
document, so it reported PASS while leaving the rest of the chain stale.
</book:entry>

</book:doc>
