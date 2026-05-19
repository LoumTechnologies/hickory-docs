# hick-agent-hick-edit

Hickory XML-aware extension layered on top of `hick-agent-sdk`.

Gives the agent the ability to understand and modify hick pipelines, not just
run arbitrary code.

## Key types

- `HickEditFormat` / `HickEditService` — runs pipelines and computes
  reverse-provenance source edits
- `PipelineWatcher` — watches `.hick` files via `notify` and transparently
  re-runs the pipeline on changes
- `PasteSlotRegistry` — indexes named `<hick:paste>` composition points
- `HickEditAgent` — composable behaviour object that adds pipeline editing
  to any agent runtime

## System prompt integration

`system_prompt_fragment()` injects Hickory-specific guidance (no-escaping rules,
generated-file policy) into any agent's system prompt, ensuring LLMs don't
try to edit pipeline outputs directly.
