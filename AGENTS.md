## Ticketry — ticket management

This repository uses **ticketry** for ticket tracking. Tickets are Markdown files
committed to git anywhere in the repo (default: `work/`). A local SQLite
index (`.ticketry/index.db`, never committed) enables fast queries across branches
via the `ticketry` MCP server.

### MCP tools

Use these tools to manage tickets without touching the filesystem directly.

| Tool | Key parameters | Description |
|------|---------------|-------------|
| `list_tickets` | `status`, `assignee`, `milestone`, `label`, `all_branches` | List tickets with optional filters |
| `get_ticket` | `id` | Fetch full detail for one ticket |
| `create_ticket` | `title`, `status`, `parent`, `milestones[]`, `path` | Create a ticket (auto-assigns next ID) |
| `move_ticket` | `id`, `status` | Change a ticket's status (edits frontmatter in-place) |
| `delete_ticket` | `id` | Delete a ticket |
| `list_milestones` | — | List all milestones |
| `create_milestone` | `title`, `status` | Create a milestone |
| `sync` | — | Re-index all branches into the database |

### File format

Tickets use the `.task.md` extension, milestones use `.milestone.md`.
All fields — including `id`, `status`, and `parent_id` — live in YAML frontmatter.
Files can be placed anywhere in the repo; the indexer scans the whole tree.

```
work/
  task-1.task.md   ← top-level ticket
  task-1.1.task.md ← child ticket (sub-task of task-1)
  milestone-1.milestone.md ← milestone
```

Valid statuses: **todo, in-progress, done, blocked**
Default status for new tickets: **todo**

### Frontmatter fields

| Field | Description |
|-------|-------------|
| `id` | Unique ticket ID (e.g. `task-1`) |
| `status` | Current status (one of: todo, in-progress, done, blocked) |
| `parent_id` | Parent ticket ID for sub-tasks |
| `is_milestone` | `true` for milestone files |
| `title` | Short title |
| `assignee`, `labels`, `milestones`, `depends_on`, `priority` | Optional metadata |
| `created_at`, `updated_at` | ISO-8601 timestamps |

### ID scheme

- `task-1` — top-level ticket
- `task-1.1` — first child of `task-1`
- `task-1.1.2` — second child of `task-1.1`
- `milestone-1` — milestone

### Milestones

A ticket joins a milestone by listing the milestone's ID in its `milestones`
frontmatter array. One ticket can belong to multiple milestones simultaneously.

### Workflow examples

```
# Create a task and a child sub-task
create_ticket(title="Add login page", status="todo")
create_ticket(title="Write login form HTML", parent="task-1", status="todo")

# Move work forward (edits frontmatter in-place — file never moves)
move_ticket(id="task-1.1", status="in-progress")
move_ticket(id="task-1.1", status="done")

# Group tasks under a milestone
create_milestone(title="v1.0 launch")
create_ticket(title="Deploy to prod", milestones=["milestone-1"])

# Place ticket in a non-default directory
create_ticket(title="Ops runbook", path="ops/runbooks/deploy-checklist.task.md")

# Query
list_tickets(status="in-progress")
list_tickets(milestone="milestone-1")
list_tickets(all_branches=true)
```

### Keeping the index fresh

The index updates automatically when you use the CLI or MCP tools. If you move
or edit files by hand, run:

```sh
ticketry index   # current branch only
ticketry sync    # all local branches
```

### Migrating from the old format

If this repo previously used the old `work/<status>/task-N.md` layout, run:

```sh
ticketry migrate
```

This injects `id`, `status`, and `parent_id` into frontmatter and renames files
to `.task.md` / `.milestone.md` in-place.
