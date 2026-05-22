# Dead Code and Cruft

Items that can be deleted outright with no architectural decisions required.
All of these are safe to do in isolation before any other refactor work.

---

## 1. `hick-agent-mcp-graphql` — compiled but never enabled

**Location:** `hick-agent/crates/hick-agent-mcp-graphql/`

**Evidence:** Every config file in `hick-agent/configs/` sets
`mcp_graphql = false`. The `mcp_graphql` key is parsed in
`hick-agent/src/config.rs` and present in `ExtensionsSection` but the
conditional branch that would activate the extension is never reached in
practice.

The crate is still compiled into the workspace on every `cargo build` and
`cargo test` run.

### Option A — Delete the crate (recommended)

Remove `hick-agent/crates/hick-agent-mcp-graphql/`, remove the dep from
`hick-agent/Cargo.toml`, remove `mcp_graphql` from `ExtensionsSection` and the
config parser.

If GraphQL-over-MCP is revisited, it can be added back from the git history.

### Option B — Gate behind a `#[cfg]` / cargo feature

Move the extension behind a `graphql` Cargo feature flag so it does not
compile by default. This preserves the code while eliminating the build tax.

---

## 2. `src/main.rs` at repo root

**Location:** `/src/main.rs`

**Content:**
```rust
fn main() {
    let message = "Hello from hick!";
    println!("{}", message);
    // ... trivial even/odd loop
}
```

This is a toy scratch file. It is not a workspace member (not listed in
`Cargo.toml`'s `members`), so it doesn't compile. It has no purpose.

**Fix:** Delete `/src/main.rs` and the `/src/` directory.

---

## 3. `work/` directory

**Location:** `/work/`

**Content:** A single file with the text `todo`.

Stale placeholder from an earlier work-tracking approach (the project now uses
Backlog.md for task management). Safe to delete.

---

## 4. Root-level scratch files

These informal documents were never moved to a proper location:

| File | Content | Disposition |
|------|---------|-------------|
| `hello.py` | `print("hello world")` scratch | Delete |
| `Data Lineage Thoughts.md` | Informal design notes | Move to `docs/` or delete |
| `structured_messages.md` | Notes on message formats | Move to `docs/` or delete |
| `PLAN.md` | Untracked planning doc | Review; if still relevant, move to `docs/`; otherwise delete |

None of these files are workspace members or pipeline sources.

---

## 5. `blue-iron-cube/output/` — generated artifacts in git

**Location:** `blue-iron-cube/output/backend/`

This directory contains a `Cargo.toml` (`name = "blue-iron-cube"`) and
generated source files. It appears to be a snapshot of the `backend/` pipeline
output committed to the `blue-iron-cube/` source tree rather than the canonical
`backend/` workspace member.

**Check:** Run `grep -r 'weave="blue-iron-cube/output' --include="*.hick"` to
confirm this is generated and not a source. If confirmed:

- Add `blue-iron-cube/output/` to `.gitignore`
- Delete the committed files

The canonical output is `backend/src/` (a workspace member). A duplicate at
`blue-iron-cube/output/` only creates confusion about which version is
authoritative.

---

## 6. `wasm-test-tool` and `wasi-example`

**Location:** `crates/wasm-test-tool/`, `crates/wasi-example/`

Both crates appear to be development/test scaffolding. `wasi-example` has no
dependencies listed. Neither appears to be depended upon by any production
crate.

**Action:** Confirm with `cargo tree --workspace -e no-dev | grep wasm-test-tool`
and `cargo tree --workspace -e no-dev | grep wasi-example`. If neither appears
in the production dep tree, move them to a top-level `examples/` directory or
delete them. They should not be in `crates/` alongside production library
crates.

---

## Deletion checklist

```bash
# Verify nothing depends on mcp-graphql before deleting
grep -r "hick-agent-mcp-graphql\|hick_agent_mcp_graphql" \
  --include="*.toml" --include="*.rs" | grep -v target

# Verify src/ is not a workspace member
grep "\"src\"" Cargo.toml  # should return nothing

# Check output/ is generated
grep -r 'weave="blue-iron-cube/output' --include="*.hick"

# Check test/example crates have no prod deps
cargo tree --workspace -e no-dev | grep -E "wasm-test-tool|wasi-example"
```
