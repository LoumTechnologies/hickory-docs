# Eval Store Spec

## Scope

This document specifies the on-disk format, file naming convention,
storage layout, and git sync semantics of the `hick-eval-store` crate
(`hick-agent/crates/hick-eval-store/`). The eval coordinator (TASK-50)
MUST conform to this spec when writing and reading eval records.

---

## Storage Layout

### Store directory resolution

The eval store directory MUST be resolved in this order:

1. `<git-repo-root>/.hick/eval/` — when the process is running inside
   a git repository.
2. `~/.local/share/hick/eval/` — fallback when outside a repository.

Implementations MUST call `resolve_store_dir()` to obtain the path;
they MUST NOT hard-code either location.

### Machine ID

Each machine has a stable UUID stored in
`~/.local/share/hick/machine-id`. On first run the file does not exist
and MUST be created with a freshly generated UUID v4. Subsequent runs
MUST read the existing file and MUST NOT regenerate the UUID.

### File naming

Each machine appends exclusively to its own file named:

```
<sanitized-hostname>_<uuid>.ndjson
```

Where:
- `<sanitized-hostname>` is the machine's hostname with all characters
  that are not alphanumeric or `-` replaced by `_`.
- `<uuid>` is the stable UUID from `~/.local/share/hick/machine-id`.

No two machines share a file. This invariant MUST be maintained so
that `git merge` is always conflict-free.

---

## Record Format

Each line in a `.ndjson` file is a self-contained JSON object. Records
MUST be written one per line with a trailing newline. Blank lines MUST
be ignored when reading.

### Required fields (all record types)

| Field | Type | Notes |
|-------|------|-------|
| `record_type` | String | Discriminator; see record types below. |
| `machine_id` | String | The UUID of the writing machine. |
| `timestamp` | ISO-8601 | UTC timestamp of when the record was written. |

### Record Types

#### `eval_result`

Written by the agent after completing a query.

| Field | Required | Notes |
|-------|----------|-------|
| `config_name` | **REQUIRED** | Name of the agent config evaluated. |
| `query` | optional | The user query text. |
| `response` | optional | The agent's response text. |
| `scores` | optional | Object mapping metric name to f64 score. |
| `token_cost` | optional | See `TokenCost` shape below. |
| `original_turn_id` | optional | Decimal NodeId of the original turn being replayed (retroactive replay records). |
| `wall_secs` | optional | Wall-clock seconds from UserTurn to Done. |
| `rework_count` | optional | Number of failed action steps (proxy for backtracking). |

#### `prune`

Appended to mark a config as excluded from future eval scheduling.
The original `eval_result` lines MUST NOT be deleted.

| Field | Required | Notes |
|-------|----------|-------|
| `config_name` | optional | Config being pruned. MAY be absent in legacy records. |
| `reason` | optional | Human-readable explanation. |
| `pruned_before` | optional | Exclude records with `timestamp < pruned_before` (legacy field). |
| `scores` | optional | Scores at the time of auto-pruning decision. |
| `confidence` | optional | Wilson CI upper bound at time of pruning. |
| `trigger` | optional | `"manual"` or `"auto"`. |

#### `unprune`

Reversal of a prior `prune` record. The original `prune` record MUST
NOT be deleted; current state MUST be derived by replaying the log.

| Field | Required | Notes |
|-------|----------|-------|
| `config_name` | **REQUIRED** | Config being re-enabled. |
| `reason` | optional | Human-readable explanation. |

#### `judge_score`

Written by the background LLM-as-judge task (TASK-48). Never
overwrites an `eval_result` record — always a separate append.

| Field | Required | Notes |
|-------|----------|-------|
| `config_name` | **REQUIRED** | Config whose response was scored. |
| `judge_model` | **REQUIRED** | Model identifier used as judge. |
| `prompt_hash` | **REQUIRED** | Hex hash of the judge prompt template. |
| `query` | **REQUIRED** | The query that was evaluated. |
| `scores` | **REQUIRED** | Object mapping dimension name to f64 score. |
| `backtracking_events` | optional | Array of `{description, avoidable}` objects. Omitted when empty. |
| `token_cost` | optional | Token cost of the judge call. |
| `wall_secs` | optional | Wall-clock seconds of the evaluated turn (copied from eval result). |
| `rework_count` | optional | Rework count of the evaluated turn (copied from eval result). |

#### `human_rating`

Human score that confirms or overrides a `judge_score`. The original
`judge_score` record MUST NOT be modified.

| Field | Required | Notes |
|-------|----------|-------|
| `config_name` | **REQUIRED** | Config being rated. |
| `query` | **REQUIRED** | The query that was evaluated. |
| `scores` | **REQUIRED** | All dimensions present; overrides judge scores. |
| `judge_timestamp` | optional | Timestamp of the `judge_score` this rating responds to. |

#### Unknown record types

A reader MUST NOT drop records with an unrecognised `record_type`. It
MUST preserve them as opaque JSON objects so that older readers can
process files written by newer versions without data loss.

### `TokenCost` shape

```json
{
  "input_tokens": 1234,
  "output_tokens": 567,
  "total_cost_usd": 0.0123
}
```

All three fields are optional and MUST be omitted when `null`.

---

## Append-Only Invariant

Lines in `.ndjson` files MUST NEVER be deleted or modified after they
are written. State changes (pruning, unpruning) MUST be expressed as
new appended records. Readers derive current state by replaying the
full log in timestamp order.

---

## Git Sync Semantics

### Startup (pull)

On `GitBackend::new()`, the implementation MUST attempt
`git pull --ff-only`. This operation is **non-fatal**: errors (no
remote, not a repo, conflict) MUST be silently ignored so the agent
can continue without network access.

Pull MUST be skipped when no remote is configured for the repo.

### Session end (push)

After a session completes, `GitBackend::sync_session()` MUST:

1. `git add <machine-file>` — stage only this machine's file.
2. `git commit -m "eval: update <uuid>"` — commit with a fixed message
   format.
3. `git push` — push to the default remote.

All three steps are **non-fatal**: errors at any step MUST be silently
ignored. The agent MUST NOT be interrupted or blocked by a git failure.

Push MUST be skipped when no remote is configured for the repo.

### Conflict resolution

Because each machine writes exclusively to its own file, git merges
are always conflict-free. Implementations MUST NOT attempt manual
conflict resolution for eval store files.

### Home-folder fallback

When the store directory resolves to `~/.local/share/hick/eval/` (not
inside a git repo), git sync MUST be skipped entirely. Records are
stored locally only.

---

## Aggregation

`GitBackend::read_all()` MUST aggregate records from every `.ndjson`
file in the store directory, across all machines. Files that are not
`.ndjson` MUST be ignored. Malformed lines MUST be silently skipped so
a corrupted file on one machine cannot block the reader.
