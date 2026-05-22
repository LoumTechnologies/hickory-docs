# Crate Naming Inconsistencies

## Summary

Nine crates have names that are either inconsistent with the `hick-` prefix
convention or generic enough to be ambiguous. The fixes are mechanical:
`Cargo.toml` renames + `use` statement updates. No behaviour changes.

---

## 1. `hickory-xml` → `hick-xml`

**Location:** `crates/hickory-xml/`

**Problem:** Every other crate uses `hick-` as its prefix. `hickory-xml` is
the sole exception and shares a root with the unrelated `hickory` DNS crate on
crates.io, which is confusing when searching docs or error messages.

**Fix:**
```
crates/hickory-xml/Cargo.toml  →  name = "hick-xml"
# grep -r "hickory-xml\|hickory_xml" --include="*.toml" --include="*.rs"
# update workspace dep in root Cargo.toml
```

No options — this is a clear correction.

---

## 2. `wasm-net` crates: generic names

**Location:** `wasm-net/crates/`

All five crates in this subsystem use names that are ambiguous without the
`wasm-net/` directory context. In a workspace `Cargo.toml` or a dependency
list they read as system-level crates, not project-specific ones.

| Current name | Proposed name | Notes |
|---|---|---|
| `network` | `hick-net` | Core WASM virtual network stack |
| `bridge` | `hick-net-bridge` | Host↔WASM packet bridge |
| `tcp-nat` | `hick-tcp-nat` | NAT layer for WASM containers |
| `net-config` | `hick-net-config` | Network config types |
| `connection-filter` | `hick-connection-filter` | Outbound connection rules |

**Option A — Full rename (recommended):** Apply the table above. Every
dependent crate gets clearer dependency lines.

**Option B — Prefix directory group:** Keep current names but add
`hick-wasm-` prefix (`hick-wasm-network`, `hick-wasm-bridge`, etc.). Clearer
about the WASM context but longer.

**Option C — No rename, add alias:** Add `package = "network"` to workspace
`[dependencies]` and alias it as `hick-net = { package = "network", … }`.
Avoids the grep-and-replace but adds a permanent workaround.

---

## 3. `container` → `hick-container`

**Location:** `hick-container/crates/container/`

**Problem:** The crate is named `container` but lives in a directory group
called `hick-container`. Importing `container = { workspace = true }` is
confusing next to, say, `tokio` or `tower`.

**Fix:** Rename the crate to `hick-container`.

Note: the sibling `container-testbed` would become `hick-container-testbed`.

---

## 4. `bench` → `hick-bench`

**Location:** `crates/bench/`

**Problem:** Generic name; unclear in CI output or `cargo bench` invocations
whether it belongs to this project.

**Fix:** Rename to `hick-bench`. No structural changes needed.

---

## 5. `network` in workspace `[dependencies]` section

Once `network` is renamed to `hick-net` (item 2), the workspace `Cargo.toml`
entry changes from:

```toml
network = { version = "0.1", path = "wasm-net/crates/network" }
```

to:

```toml
hick-net = { version = "0.1", path = "wasm-net/crates/hick-net" }
```

All `network.workspace = true` references in dependent crates must also be
updated to `hick-net.workspace = true`.

---

## Mechanical checklist

```
# For each rename:
# 1. Edit Cargo.toml name field
# 2. Update workspace Cargo.toml path entry
# 3. Rename the directory (optional but recommended for clarity)
# 4. sed -i 's/old_crate_name/new_crate_name/g' on all *.toml and *.rs files
# 5. cargo check --workspace
```
