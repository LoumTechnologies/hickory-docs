# Pre-commit

Before committing, always run the full test suite:

```bash
cargo test --workspace -- --test-threads=1
```

## Hickory XML Is Not Standard XML

Hickory XML is **not** a standard XML dialect. The parser only recognises
elements that carry the `hick:` namespace prefix (e.g. `<hick:doc>`,
`<hick:section>`). Anything that merely *looks* like an XML tag but lacks that
prefix is passed through literally — it is not parsed, not escaped, and not
touched.

Consequence: bare `<`, `>`, and `&` characters in content (Rust generics, C++
templates, HTML, boolean `&&`, shell redirects, etc.) are **preserved as-is**
in pipeline output. You do **not** need CDATA sections or entity escaping for
source code embedded in `.hick` files.

If output files contain `&lt;`, `&gt;`, or `&amp;` as literal text, that
indicates either (a) the content was pre-escaped before being inserted, or (b)
an older/non-hickory tool processed the file. It is not a property of the
current hick pipeline.

## Pipeline-Generated Files

Many files in this repo are **outputs of the hick pipeline**, not sources.
Running `cargo test` or `hick run` regenerates them. Editing generated files
directly is pointless — the next pipeline run overwrites the changes.

**How to identify a generated file:** check whether it appears as a weave
target in any `.hick` source file (`grep -r 'weave="path/to/file"' --include="*.hick"`).
If it does, edit the `.hick` source instead.

Known generated files/directories (non-exhaustive):
- `backend/src/` — generated from `blue-iron-cube/main.hick` and domain `.hick` files
- `.gitignore`, `README.md`, `.editorconfig`, `.devcontainer/` — generated from scaffolding `.hick` files
- `TEMPLATE.md` — test pipeline output, not committed

**Never** manually fix content in a generated file (e.g. to correct entity refs
or formatting). Find the `.hick` source and fix it there.

## AI Coding Agent Architecture

The agents live under `hick-agent/crates/`. Three layers:

**Core** (`hick-agent-sdk`): LLM clients, WASM sandbox, network
filtering, runtime loop. No hick pipeline knowledge. All agents depend
on this; extend it rather than forking it.

**Extensions** (`hick-agent-*` prefix): Optional capabilities evaluated
for inclusion in the final public agent. Current extensions:
- `hick-agent-script-first` — script-first ReAct loop (Python/shell in
  WASM containers)
- `hick-agent-hick-edit` — hick-XML pipeline editing, pipeline watcher,
  reverse provenance

New capabilities under evaluation belong here as a new
`hick-agent-<feature>` crate before being promoted to a binary.

**Binary crates** (adjective-noun names, e.g. `copper-ledge`,
`iron-peak`): Each binary wires a specific combination of extensions
together for A/B evaluation. The combination is explicit in
`Cargo.toml` — no hidden feature gates. When evaluation is done, the
winning combination becomes the public release binary.

## Formal Specifications (`docs/specs/`)

Formal behavioural specifications for this codebase live in `docs/specs/`.
They use RFC-style normative language (`MUST`, `MUST NOT`, `SHOULD`,
`SHOULD NOT`, `MAY`). All spec files MUST use this language consistently —
informal phrasing ("should", "will", "needs to") is not acceptable in a spec.

**During every conversation**, watch for requirements, constraints, or
invariants the user states about how the system must behave. If a stated
requirement belongs in a spec:

1. Check whether a relevant spec file already exists in `docs/specs/`.
2. If one exists, add the requirement to it using normative language.
3. If none exists, create a new spec file in `docs/specs/` with an
   appropriate name and a `## Scope` section before adding the requirement.
4. Tell the user which spec file was updated or created.

Do this proactively — do not wait to be asked. Requirements mentioned in
passing during planning or debugging conversations are just as binding as
those stated explicitly.

## Backlog.md Tool Behaviour: task_list vs task_view

`task_list` only searches `backlog/tasks/`. Completed tasks live in
`backlog/completed/` and are **invisible to `task_list`**. `task_view`
reads both directories, so it can find a task that `task_list` claims
doesn't exist.

Consequence: if a task appears missing or "To Do" when you believe it is
done, call `task_view` with its explicit ID before concluding anything.
Do not trust a `task_list` absence as proof that a task doesn't exist or
hasn't been completed.

<!-- BACKLOG.MD MCP GUIDELINES START -->

<CRITICAL_INSTRUCTION>

## BACKLOG WORKFLOW INSTRUCTIONS

This project uses Backlog.md MCP for all task and project management activities.

**CRITICAL GUIDANCE**

- If your client supports MCP resources, read `backlog://workflow/overview` to understand when and how to use Backlog for this project.
- If your client only supports tools or the above request fails, call `backlog.get_backlog_instructions()` to load the tool-oriented overview. Use the `instruction` selector when you need `task-creation`, `task-execution`, or `task-finalization`.

- **First time working here?** Read the overview resource IMMEDIATELY to learn the workflow
- **Already familiar?** You should have the overview cached ("## Backlog.md Overview (MCP)")
- **When to read it**: BEFORE creating tasks, or when you're unsure whether to track work

These guides cover:
- Decision framework for when to create tasks
- Search-first workflow to avoid duplicates
- Links to detailed guides for task creation, execution, and finalization
- MCP tools reference

You MUST read the overview resource to understand the complete workflow. The information is NOT summarized here.

</CRITICAL_INSTRUCTION>

<!-- BACKLOG.MD MCP GUIDELINES END -->
