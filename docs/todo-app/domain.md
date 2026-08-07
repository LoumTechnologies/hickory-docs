
# Todo app — domain model



The vocabulary. Terms are tagged `.glossary` so downstream documents can pull
the whole set by class. Decisions are NOT restated here — they arrive from the
meeting by reference, so each is written down in exactly one place and every
document below this one goes stale together when it changes.

## Terms









## Decisions this model rests on


Tasks persist to a single JSON file on disk. No database. The app must run
with no service dependencies, because the point is to exercise the document
chain, not an install.

A task is in exactly one of four states: **open**, **blocked**, **done**, or
**dropped**. Blocked means work cannot proceed until something external
changes; it is not done and not dropped. Dropped is not deleted — it stays
in the file and stays listable, because "what did we decide not to do" is
the question people actually come back for.

Tasks are identified by a short opaque id assigned on creation, never by
their title. Titles are editable; anything that references a task must
survive a retitle.

The interface is a CLI. No web UI in the first cut — a UI would double the
surface without exercising any part of the lifecycle that the CLI does not.


## Why dropped is not deleted

The state decision above is the one that shapes the data model. A deleted
task leaves no trace, so "why isn't this in the list any more" has no answer.
A dropped task answers it. That costs nothing here — the file keeps every
task regardless — and it is the difference between a log and a list.
