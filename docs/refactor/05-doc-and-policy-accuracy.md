# Documentation and Policy Accuracy

CLAUDE.md and the specs directory contain guidance that is either outdated
after recent consolidations or missing entirely.

---

## 1. CLAUDE.md — "Binary crates" section is obsolete

**Current text (CLAUDE.md, "AI Coding Agent Architecture"):**

> **Binary crates** (adjective-noun names, e.g. `copper-ledge`, `iron-peak`):
> Each binary wires a specific combination of extensions together for A/B
> evaluation. The combination is explicit in `Cargo.toml` — no hidden feature
> gates. When evaluation is done, the winning combination becomes the public
> release binary.

**What actually happened:** Task-43 retired both `iron-peak` and `copper-ledge`
in favour of a single `hick` binary (`hick-agent` crate). Configuration is now
done via TOML config files (`hick-agent/configs/standard.toml`,
`hick-edit.toml`), not by choosing a binary.

**Required update:** Replace the "Binary crates" paragraph with:

> **The `hick` binary** (`hick-agent/crates/hick-agent/`): A single universal
> binary wiring all extensions. Active extensions are determined at startup from
> `.hick/agent.toml` (project config) or `~/.hick/agent.toml` (user default).
> Named config presets live in `hick-agent/configs/` (e.g. `standard.toml`,
> `hick-edit.toml`).

The "Extensions" and "Core" paragraphs remain accurate; only "Binary crates"
needs replacement.

---

## 2. CLAUDE.md — "Extensions" section lists non-existent evaluation status

The current text mentions `hick-agent-script-first` and `hick-agent-hick-edit`
as "Optional capabilities evaluated for inclusion." Both are now always
compiled into the binary (they're unconditional deps in `hick-agent/Cargo.toml`
— activation is config-driven, not build-time). Update the description to
reflect that extensions are always compiled but runtime-activated via config.

---

## 3. CLAUDE.md — No mention of config-driven activation

There is no guidance anywhere in CLAUDE.md explaining that extension activation
is controlled by `[extensions]` in `agent.toml`, nor how to add a new
extension. Add a brief section:

> **Adding an extension:** Implement the extension behaviour in a
> `hick-agent-<name>` crate, add it as a dependency of `hick-agent`, add a
> boolean key to `ExtensionsSection` in `src/config.rs`, and add the
> conditional wiring in `src/main.rs`. Add the key to all configs in
> `hick-agent/configs/`.

---

## 4. `docs/specs/` — No spec for the config file format

The `.hick/agent.toml` config schema is defined in code
(`hick-agent/src/config.rs`) but not in `docs/specs/`. This means:
- Contract changes are invisible unless you read the Rust source
- The `eval`, `machine`, and `[search]` sections have no normative spec

**Create:** `docs/specs/agent-config.md` covering:
- Required vs optional sections
- `[model]` section constraints (provider enum, model_id requirement)
- `[extensions]` key semantics (compiled-in but runtime-activated)
- `[security]` MUST/MUST NOT rules (e.g. `deny_all_network` default)
- `[machine]` declaration format and eval coordinator interaction
- Search space config (`[search]` section, `expand_trials()`)

---

## 5. `docs/specs/` — No spec for the `hick-eval-store` format

`hick-eval-store` stores per-machine NDJSON eval results. The format and
storage layout are defined only in code. A spec is needed before the eval
coordinator (TASK-50, referenced in config comments) is implemented.

Key things to specify:
- File naming convention (path pattern per machine-id)
- Required fields in each eval record
- Git sync semantics (when to push, conflict resolution)
- Home-folder fallback conditions

---

## 6. `docs/specs/` — Existing spec files for review

Confirm these specs still match reality after recent changes:

| File | Risk of staleness |
|------|------------------|
| Any spec referencing `iron-peak` or `copper-ledge` | HIGH — binary names retired |
| Any spec referencing `AgentEngine` or re-export paths | MEDIUM |
| Session log XML format spec | LOW — recently added in task-36 |

```bash
# Quick scan for retired names in specs
grep -r "iron-peak\|copper-ledge\|hick-agent-engine\|AgentEngine" docs/specs/
```

---

## 7. Global `~/.claude/CLAUDE.md` — No mention of Hickory XML dialect rule

The project `CLAUDE.md` has a critical section ("Hickory XML Is Not Standard
XML") that explains why bare `<`, `>`, `&` must not be escaped in `.hick`
files. This rule is easy to violate when an LLM assumes standard XML escaping.
The section already exists in the project CLAUDE.md, so no change is needed
there — but confirm this section is not accidentally duplicated or contradicted
elsewhere.
