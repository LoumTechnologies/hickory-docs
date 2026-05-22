# Shim Crate Elimination

Three crates exist primarily as re-export forwarding layers. They were each
created during consolidation work but not cleaned up. They add indirection
without adding functionality.

---

## 1. `hick-agent-engine`

**Location:** `hick-agent/crates/hick-agent-engine/`

**Description from Cargo.toml:** *"Re-export shim — all agent logic lives in
`hick-agent-script-first` and `hick-agent-sdk`. Only `llm_local` (llama.cpp)
stays here."*

The crate re-exports ~25 symbols from `hick-agent-sdk` and
`hick-agent-script-first`. Its only non-forwarding content is the optional
`llama-cpp-2` integration (behind `#[cfg(feature = "local")]`).

**Problem:** Everything except the llama.cpp wrapper is pure forwarding. The
`hick-agent` binary already depends directly on both source crates and only
pulls in `hick-agent-engine` for the optional `local` feature.

### Option A — Move llama.cpp impl into `hick-agent`, delete `hick-agent-engine` (recommended)

The `local` feature is only meaningful at the binary level. Moving it directly
into `hick-agent` keeps the feature close to the entry point that uses it:

```toml
# hick-agent/Cargo.toml
hick-agent-sdk = { path = "../hick-agent-sdk", features = ["web", "relay"] }
llama-cpp-2 = { version = "...", optional = true }

[features]
local = ["dep:llama-cpp-2"]
local-cuda = ["local", "llama-cpp-2/cuda"]
```

Delete `hick-agent/crates/hick-agent-engine/` entirely.

**Tradeoff:** The `LocalLlmClient` struct moves into `hick-agent/src/`. If a
second binary ever needs local LLM, extract it then into a dedicated
`hick-agent-local-llm` crate.

### Option B — Rename to `hick-agent-local-llm`, strip all re-exports

Keep the crate but remove all `pub use` forwarding. The crate becomes a
focused wrapper for llama.cpp with a single `LocalLlmClient` type. Dependents
that needed the forwarded symbols import from the source crates directly.

**Tradeoff:** Smaller cleanup than Option A but leaves a crate in the workspace
that still must be maintained.

---

## 2. `hick-agent-core`

**Location:** `hick-agent/crates/hick-agent-core/`

**Description from Cargo.toml:** *"Re-export shim — command execution types
(local) and hick-XML edit types from `hick-agent-hick-edit`."*

The crate has a small `command` module (`CommandExecutor`, `CommandOutput`,
`WasmSandboxConfig`, `WasmSandboxExecutor`) and re-exports everything else from
`hick-agent-hick-edit`.

**Problem:** `hick-agent-core` is only depended on by `hick-agent` itself (the
binary). It was likely created to provide a stable import path while
`hick-agent-hick-edit` was evolving. That purpose is served by the crate
boundaries directly now.

### Option A — Inline `command` module into `hick-agent-sdk`, delete `hick-agent-core` (recommended)

The `command` module is WASM sandbox execution infrastructure — conceptually
part of the SDK's execution model. Move it to `hick-agent-sdk/src/command.rs`
and have `hick-agent` import from `hick_agent_sdk` and `hick_agent_hick_edit`
directly.

```rust
// Before (in hick-agent/src/main.rs)
use hick_agent_core::{CommandExecutor, WasmSandboxConfig, WasmSandboxExecutor};

// After
use hick_agent_sdk::{CommandExecutor, WasmSandboxConfig, WasmSandboxExecutor};
```

### Option B — Inline `command` module into `hick-agent-hick-edit`, delete `hick-agent-core`

Since WASM sandbox execution is closely tied to hick pipeline editing, it could
live in `hick-agent-hick-edit` instead. This makes that crate's scope slightly
broader but keeps execution infra together.

### Option C — Merge `hick-agent-core` into the binary (break the lib pattern)

Move the `command` module directly into `hick-agent/src/` and remove
`hick-agent-core` as a separate crate. Viable only if the integration tests
never need `CommandExecutor` types directly — check test imports first.

---

## 3. `hick-agent/src/lib.rs` (binary-as-library)

**Location:** `hick-agent/crates/hick-agent/src/lib.rs`

The `hick-agent` binary crate exposes a lib target that re-exports symbols from
`hick-agent-sdk`, `hick-agent-script-first`, and `hick-agent-hick-edit`. The
integration tests (`tests/`) use it:

```rust
// tests/react_loop_integration.rs
use hick_agent::events::AgentEvent;

// tests/common/mod.rs
use hick_agent::llm::{ChatChunk, LlmClient, Message};
```

**Problem:** The re-exports in `lib.rs` duplicate what's available from the
extension crates. Integration tests import `hick_agent::events::AgentEvent`
when they could import `hick_agent_script_first::events::AgentEvent`.

### Option A — Remove re-exports from `lib.rs`, update test imports

Change test imports to use the source crates directly and slim `lib.rs` down to
only public API surface genuinely owned by `hick-agent` (e.g., config structs).

**Tradeoff:** Small test churn. Removes an implicit "stable re-export API"
that doesn't actually need to exist.

### Option B — Keep lib target, document it as the integration-test facade

Leave `lib.rs` as-is but add a `#[doc(hidden)]` attribute and a note explaining
it is only for integration tests. Prevents future growth of the re-export
surface.

**Tradeoff:** No churn, but the shim lives on.

---

## Combined impact

Eliminating all three shims removes two entire crates from the workspace and
collapses three layers of indirection between `hick-agent`'s entry point and
the SDK/extension crates. Compilation times improve slightly (fewer crates to
link). The import graph becomes a DAG rather than a diamond.
