
# Todo app — requirements



One edge declared. The glossary comes from the domain model one hop up, and
the decisions come from the kickoff meeting **two** hops up — this document
never names the meeting, and does not have to. Change a decision in the
meeting notes and this document, its tickets, and the domain model all go
stale together.

## Vocabulary in force


**Task** — one thing to do. Has an id, a title, and a state. Nothing else;
every field anyone proposed beyond these was left as an open question.

**State** — which of open, blocked, done, or dropped a task is in. Exactly one at a
time; there is no "partially done".

**Dropped** — decided against, deliberately retained. Distinct from deleted,
which the app does not offer. Dropping is a decision worth keeping a record
of.

**List** — the whole set of tasks, in creation order. There is one list; the
app has no concept of multiple lists or projects.


## Decisions in force


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


## Requirements

Each requirement below is also a ticket. The ticket files under `work/` are
tangled from these fragments — the requirement and the ticket are the same
bytes, so they cannot drift. Editing the requirement IS editing the ticket.












### `work/task-rename.task.md`

```markdown
---
slug: task-rename
title: Rename a task
status: todo
---


`todo rename <id> "<new title>"` changes a task's title, leaving its id and
state untouched. This is the requirement the opaque-id decision exists for:
anything referencing a task must survive a retitle. An unknown id exits
nonzero, like the other commands that take one.

```



### `work/task-add.task.md`

```markdown
---
slug: task-add
title: Create a task
status: todo
---


`todo add "<title>"` creates an open task and prints its id. The id is short,
opaque, and assigned at creation; it is never derived from the title.

```



### `work/task-list.task.md`

```markdown
---
slug: task-list
title: List tasks
status: todo
---


`todo list` prints every task in creation order as `<id> <state> <title>`.
Dropped and blocked tasks are included — retaining them is pointless if the
default view hides them. `--state open|blocked|done|dropped` filters.

```



### `work/task-transitions.task.md`

```markdown
---
slug: task-transitions
title: Complete and drop tasks
status: todo
---


`todo done <id>` moves a task to done. `todo drop <id>` moves it to dropped.
`todo block <id>` moves it to blocked. All are idempotent: re-running on a
task already in that state succeeds and changes nothing, so a retried
command is never an error.

```



### `work/task-storage.task.md`

```markdown
---
slug: task-storage
title: Persist tasks to disk
status: todo
---


Tasks persist to `tasks.json` in the working directory, created on first
write. A missing file reads as an empty list, not an error — the first run of
a fresh install must behave like every later run.

```

