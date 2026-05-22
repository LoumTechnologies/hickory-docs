# Agent Configuration Spec

## Scope

This document specifies the schema and resolution semantics of the
`.hick/agent.toml` configuration file consumed by the `hick` binary
(`hick-agent/crates/hick-agent/src/config.rs`).

---

## Config Resolution Order

The agent MUST load the first config file found in this order:

1. `--config <path>` passed on the command line (explicit override).
2. `.hick/agent.toml` in the current project directory.
3. `~/.hick/agent.toml` in the user's home directory.

If `--config <path>` is given but the file does not exist, the agent
MUST fail with an error. If neither the project nor home config exists
and no explicit path is given, the agent MUST fail with a descriptive
error.

---

## Config Shapes

A config file is one of two mutually exclusive shapes:

- **Point config**: no `[search]` section. Ready to run immediately.
- **Search space config**: contains a `[search]` section with at least
  one axis. MUST be expanded to concrete trial configs via
  `SearchSpaceConfig::expand_trials()` before use.

---

## `[model]` Section

The `[model]` section is **required**. The agent MUST fail at parse
time if `[model]` is absent.

| Key | Type | Required | Default | Notes |
|-----|------|----------|---------|-------|
| `model_id` | String | **REQUIRED** | — | Model identifier, e.g. `"claude-opus-4-7-20251101"`. |
| `provider` | Enum | optional | `"anthropic"` | One of: `"anthropic"`, `"openai"`, `"xai"`. |
| `api_key_env` | String | optional | provider default | Name of the env var holding the API key. |
| `context_window` | u32 | optional | — | Informational hint; not enforced by the config layer. |
| `max_tokens` | u32 | optional | — | Maximum output tokens per request. |

The `model_id` key MUST be present. Any config omitting it MUST be
rejected at parse time.

---

## `[extensions]` Section

The `[extensions]` section is optional. All keys default to `false`.

Extensions are **always compiled into the binary**. This section
controls which are activated at runtime — it is not a build-time gate.

| Key | Type | Default | Extension crate |
|-----|------|---------|-----------------|
| `script_first` | bool | `false` | `hick-agent-script-first` |
| `hick_edit` | bool | `false` | `hick-agent-hick-edit` |
| `mcp_wasm` | bool | `false` | `hick-agent-mcp-wasm` |
| `mcp_sdk` | bool | `false` | `hick-agent-mcp-sdk` |
| `mcp_xml` | bool | `false` | `hick-agent-mcp-xml` |
| `mcp_graphql` | bool | `false` | `hick-agent-mcp-graphql` |

An unrecognised key in `[extensions]` SHOULD produce a warning but
MUST NOT be treated as a fatal error (unknown keys are silently
ignored by the serde deserialiser).

---

## `[security]` Section

The `[security]` section is optional. When absent, the agent MUST
apply the built-in defaults defined in `AgentSecurityConfig::default`.

### Defaults

| Key | Default value |
|-----|---------------|
| `deny_all_network` | `true` |
| `allowed_hosts` | `pypi.org:443`, `files.pythonhosted.org:443` |
| `file_access.read` | `["/workspace/**"]` |
| `file_access.write` | `["/workspace/**"]` |

### Normative rules

- `deny_all_network` MUST default to `true`. A config that explicitly
  sets it to `false` opts out of network denial; this is a deliberate
  and visible choice.
- `allowed_hosts` entries MUST have a `host` key. The `port` key is
  optional and defaults to `"*"` (all ports).
- The `policy_script` key, when present, MUST be a path to a Python
  policy script evaluated for data-flow rules.
- `[[sinks]]` and `[[mcp_tool_sinks]]` entries classify and gate
  script-execution and MCP tool calls respectively. The default sink
  rejects `{"sensitivity": "secret"}` from both.

---

## `[mcp]` Section

The `[mcp]` section is optional.

| Key | Default |
|-----|---------|
| `servers_file` | `".hick/mcp-servers.toml"` |
| `secrets_file` | `".hick/mcp-secrets.toml"` |
| `response_format.default` | `"json"` |
| `response_format.by_shape.list` | absent |
| `response_format.by_shape.nested` | absent |
| `response_format.by_shape.scalar` | absent |
| `response_format.by_shape.object` | absent |

---

## `[machine]` Section

The `[machine]` section is optional. When absent, the machine is
treated as unconstrained (no size limit, no declared backends).

| Key | Type | Default | Notes |
|-----|------|---------|-------|
| `backends` | `Vec<String>` | `[]` | Recognised values: `"Metal"`, `"CUDA"`, `"CPU"`. Empty means no declared preference. |
| `max_model_gb` | f64 | absent | Maximum model size the machine can load. Absent means no limit. |
| `eval_configs` | `Vec<String>` | `[]` | Names of eval configs this machine attempts. Empty means "all". |
| `[[machine.model_paths]]` | array | `[]` | GGUF model files available on this machine. |

Each `[[machine.model_paths]]` entry MUST have:
- `name` (String) — short identifier used in eval config references.
- `path` (PathBuf) — absolute or config-file-relative path to the `.gguf` file.
- `size_gb` (f64) — approximate model size in gigabytes.

The agent MUST validate that every declared `model_paths` entry exists
on disk at startup. A missing path MUST produce a startup error naming
the entry and the resolved path.

The eval coordinator MUST use `can_run_model(size_gb)` to skip
combinations that exceed `max_model_gb`. When `max_model_gb` is
absent, all model sizes are considered feasible.

---

## `[eval]` Section

The `[eval]` section is optional.

| Key | Type | Default | Notes |
|-----|------|---------|-------|
| `auto_prune_threshold` | f64 | absent | Confidence threshold for auto-pruning (TASK-49). When set, configs whose Wilson CI upper bound falls below `best_win_rate * threshold` are pruned. |

### `[eval.judge]` Sub-section

| Key | Type | Required | Notes |
|-----|------|----------|-------|
| `model` | String | **REQUIRED** (if section present) | Judge model identifier. |
| `provider` | Enum | optional (`"anthropic"`) | Provider for the judge model. |
| `api_key_env` | String | optional | Env var for the judge API key. |
| `prompt_template` | String | optional | Template with `{query}`, `{response}`, `{dimensions}` placeholders. Defaults to the built-in backtracking-aware template. |
| `dimensions` | array | optional | Scoring dimensions with `name` and `weight` fields. Defaults to five built-in dimensions. |

Default dimensions and weights:

| Dimension | Weight |
|-----------|--------|
| `task_completion` | 0.30 |
| `tool_efficiency` | 0.20 |
| `unnecessary_steps` | 0.15 |
| `backtracking` | 0.25 |
| `output_quality` | 0.10 |

---

## `[search]` Section

Presence of `[search]` with at least one axis makes the config a
**search space config**. Absence makes it a **point config**.

| Key | Type | Default | Notes |
|-----|------|---------|-------|
| `strategy` | String | `"grid"` | One of `"grid"` or `"ofat"`. Unknown values MUST be rejected. |
| `<axis-name>` | `Vec<Value>` | — | Each non-reserved key is a search axis. Arrays MUST have at least one element. |
| `[[search.constraints]]` | array | `[]` | Constraint entries with `if_keys` and `then_keys` tables. |

### Strategies

- **`grid`**: Cartesian product of all axes. Two binary axes produce
  four trials.
- **`ofat`** (one-factor-at-a-time): Baseline (first value of every
  axis) plus one variation per non-baseline axis value. For `n` axes
  with lengths `[l₁, …, lₙ]`, produces `1 + Σ(lᵢ − 1)` trials.

### Constraints

A constraint entry MUST have `if_keys` and `then_keys` tables. A trial
that satisfies all `if_keys` MUST also satisfy all `then_keys`;
otherwise the trial MUST be pruned from the expansion.

```toml
[[search.constraints]]
if_keys   = { hick_edit = true }
then_keys = { script_first = true }
```

---

## Top-Level Optional Keys

| Key | Type | Default | Notes |
|-----|------|---------|-------|
| `system_prompt_extra` | String | absent | Path (relative to project dir) to a file whose content is appended to the agent system prompt. |

---

## Validation Rules

1. `[model].model_id` MUST be present; absence MUST produce a parse
   error.
2. Unknown keys in `[intent_parsers]` MUST produce a startup error
   naming the offending key and the section.
3. Unknown keys in `[classifiers]` MUST produce a startup error naming
   the offending key.
4. All `[machine.model_paths]` entries MUST reference files that exist
   on disk; a missing path MUST produce a startup error.
5. An unknown `[search].strategy` value MUST produce a parse error.
6. A `[search]` axis array MUST have at least one element; an empty
   array MUST produce a parse error.
