# Caching and Freeze

Cache container execution results to avoid expensive WASM boot + command execution on repeated runs when source hasn't changed.

## Motivation

Every `hick` run currently boots WASM containers from scratch (120s boot timeout), runs all commands sequentially, and captures output. For a document with multiple containers and many exec steps, this can take minutes. During authoring, most runs change only prose text while exec commands remain the same. Caching avoids re-executing unchanged commands.

Quarto solves this with two mechanisms: **cache** (per-cell, for speed) and **freeze** (per-document, for collaboration). Hickory Docs needs a single mechanism that respects the security model — cached outputs must be invalidated when anything affecting the execution changes.

## Design

### Cache Model

Cache entries are keyed by a content hash of everything that affects execution output:

```
cache_key = SHA-256(
    container_image,
    container_capabilities (sorted, canonical),
    command_text (trimmed),
    upstream_volume_hashes (if any),
    secret_names (not values — just which secrets are injected),
)
```

Each entry stores:
- The `ExecTranscriptEntry` (commands + output)
- The hash of any volumes written by this exec (for downstream invalidation)

### Cache Directory

```
.hick-cache/
  transcripts/
    <container_name>/
      <exec_hash>.json       # cached ExecTranscriptEntry
  meta.json                  # cache format version, creation timestamp
```

The `.hick-cache/` directory lives alongside the `.hick` files (project root). It should be added to `.gitignore` by default — unlike Quarto's `_freeze/`, these caches contain real execution output from the user's machine and aren't portable across environments.

### Freeze Mode

A separate `--freeze` flag stores results and prevents re-execution entirely. Unlike cache (which re-executes on hash mismatch), freeze never re-executes — it fails if cached output is missing.

| Mode | Behavior |
|------|----------|
| `hick file.hick` (default) | Execute all, no cache |
| `hick --cache file.hick` | Use cache when hash matches, execute and update cache on miss |
| `hick --freeze file.hick` | Use cached results only, error if any exec is not cached |
| `hick --clear-cache file.hick` | Delete cache directory and execute fresh |

### Cache Invalidation

Automatic invalidation happens when:
- Command text changes (part of the hash)
- Container image changes (part of the hash)
- Container capabilities change (part of the hash)
- An upstream volume-writing exec's output changes (downstream hashes include upstream output hash)
- Secret names change (not values — we can't hash encrypted secrets safely)

Manual invalidation:
- `--clear-cache` deletes the cache directory
- Deleting `.hick-cache/` manually

### Security Considerations

- Cached outputs are not signed or encrypted. They're local developer convenience, not a trust boundary.
- Secret values are never stored in the cache. Only secret names are included in the hash key, so changing which secrets are injected invalidates the cache, but the actual decrypted values don't leak into cache files.
- In CI, `--cache` should generally not be used (fresh execution ensures correctness). `--freeze` may be useful if pre-computed results are checked in.

## Implementation

### Step 1: Cache Key Computation

**New file:** `crates/hick/src/cache.rs`

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecCacheEntry {
    pub commands: Vec<String>,
    pub output: String,
    pub output_hash: String,  // SHA-256 of output, for downstream invalidation
}

#[derive(Debug)]
pub struct CacheConfig {
    pub cache_dir: PathBuf,
    pub enabled: bool,
    pub freeze: bool,
}

impl CacheConfig {
    pub fn new(project_dir: &Path, enabled: bool, freeze: bool) -> Self {
        Self {
            cache_dir: project_dir.join(".hick-cache").join("transcripts"),
            enabled,
            freeze,
        }
    }
}

/// Compute the cache key for an exec.
pub fn exec_cache_key(
    container_image: &str,
    capabilities_canonical: &str,
    command: &str,
    upstream_hashes: &[&str],
    secret_names: &[&str],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"image:");
    hasher.update(container_image.as_bytes());
    hasher.update(b"\ncaps:");
    hasher.update(capabilities_canonical.as_bytes());
    hasher.update(b"\ncmd:");
    hasher.update(command.trim().as_bytes());
    for h in upstream_hashes {
        hasher.update(b"\nupstream:");
        hasher.update(h.as_bytes());
    }
    for s in secret_names {
        hasher.update(b"\nsecret:");
        hasher.update(s.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Look up a cached exec result.
pub fn cache_lookup(
    config: &CacheConfig,
    container: &str,
    key: &str,
) -> Result<Option<ExecCacheEntry>> {
    let path = config.cache_dir.join(container).join(format!("{key}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)?;
    let entry: ExecCacheEntry = serde_json::from_str(&data)?;
    Ok(Some(entry))
}

/// Store a result in the cache.
pub fn cache_store(
    config: &CacheConfig,
    container: &str,
    key: &str,
    entry: &ExecCacheEntry,
) -> Result<()> {
    let dir = config.cache_dir.join(container);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{key}.json"));
    let data = serde_json::to_string_pretty(entry)?;
    std::fs::write(&path, data)?;
    Ok(())
}

/// Delete the entire cache directory.
pub fn cache_clear(config: &CacheConfig) -> Result<()> {
    if config.cache_dir.exists() {
        std::fs::remove_dir_all(&config.cache_dir)?;
    }
    Ok(())
}
```

### Step 2: Add CLI Flags

**File:** `crates/hick/src/main.rs`

```rust
#[derive(Parser)]
struct Cli {
    // ... existing fields ...

    #[arg(long)]
    cache: bool,

    #[arg(long)]
    freeze: bool,

    #[arg(long)]
    clear_cache: bool,
}
```

### Step 3: Integrate Cache into Live Pipeline

**File:** `crates/hick/src/lib.rs`

Add `cache_config: Option<&CacheConfig>` parameter to `run_pipeline_live`:

```rust
pub async fn run_pipeline_live(
    sources: &[(&str, &str)],
    config: &PipelineConfig,
    cache_config: Option<&CacheConfig>,
) -> Result<PipelineResult> {
```

In the exec loop, check cache before executing:

```rust
for exec_id in flow_dag.topological_order() {
    let exec_info = /* ... */;
    let cache_key = exec_cache_key(
        image,
        &canonical_caps(&container_defs, &exec_info.container),
        &exec_info.command,
        &upstream_hashes_for(exec_info, &completed_hashes),
        &secret_names_for(&container_defs, &exec_info.container),
    );

    if let Some(cc) = cache_config {
        if let Some(cached) = cache_lookup(cc, &exec_info.container, &cache_key)? {
            info!("Cache hit for exec in '{}': {}", exec_info.container, &cache_key[..12]);
            // Store the cached transcript entry
            executor.inject_transcript_entry(&exec_info.container, cached.into());
            completed_hashes.insert(exec_info.id, cached.output_hash.clone());
            continue;
        } else if cc.freeze {
            bail!(
                "Freeze mode: no cached result for exec in '{}' (command: {})",
                exec_info.container,
                exec_info.command.lines().next().unwrap_or("?"),
            );
        }
    }

    // Cache miss — execute for real
    executor.ensure_started(&exec_info.container, image).await?;
    let output = executor.execute(&exec_info.container, &exec_info.command).await?;

    // Store in cache
    if let Some(cc) = cache_config {
        let entry = ExecCacheEntry {
            commands: /* from transcript */,
            output: output.clone(),
            output_hash: sha256_hex(&output),
        };
        cache_store(cc, &exec_info.container, &cache_key, &entry)?;
        completed_hashes.insert(exec_info.id, entry.output_hash.clone());
    }
}
```

### Step 4: Canonical Capabilities Serialization

**File:** `crates/hick/src/cache.rs`

```rust
/// Produce a canonical string representation of capabilities for hashing.
/// Rules are sorted to ensure deterministic output.
pub fn canonical_caps(
    container_defs: &HashMap<String, ContainerCapabilities>,
    container: &str,
) -> String {
    let Some(caps) = container_defs.get(container) else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    for rule in &caps.network_rules {
        parts.push(format!("net:{rule:?}"));
    }
    for rule in &caps.file_rules {
        parts.push(format!("file:{rule:?}"));
    }
    for rule in &caps.secret_rules {
        parts.push(format!("secret:{}:{}", rule.env_var, rule.secret_name));
    }
    parts.sort();
    parts.join(";")
}
```

### Step 5: Add `inject_transcript_entry` to Executor

**File:** `crates/hick/src/executor.rs`

```rust
/// Inject a transcript entry from cache (no actual execution).
pub fn inject_transcript_entry(&mut self, container: &str, entry: ExecTranscriptEntry) {
    self.transcripts
        .entry(container.to_string())
        .or_default()
        .push(entry);
}
```

### Step 6: Dependencies

**File:** `crates/hick/Cargo.toml`

```toml
sha2 = "0.10"
serde_json = "1"
# serde already available via hick-token
```

### Step 7: Tests

**File:** `crates/hick/tests/cache_tests.rs`

```rust
#[test]
fn test_cache_key_deterministic() {
    let k1 = exec_cache_key("alpine", "net:deny_all", "echo hi", &[], &[]);
    let k2 = exec_cache_key("alpine", "net:deny_all", "echo hi", &[], &[]);
    assert_eq!(k1, k2);
}

#[test]
fn test_cache_key_changes_with_command() {
    let k1 = exec_cache_key("alpine", "", "echo a", &[], &[]);
    let k2 = exec_cache_key("alpine", "", "echo b", &[], &[]);
    assert_ne!(k1, k2);
}

#[test]
fn test_cache_store_and_lookup() {
    let dir = std::env::temp_dir().join("hick-cache-test");
    let _ = std::fs::remove_dir_all(&dir);
    let config = CacheConfig::new(&dir, true, false);

    let entry = ExecCacheEntry {
        commands: vec!["echo hi".into()],
        output: "hi".into(),
        output_hash: "abc123".into(),
    };
    cache_store(&config, "demo", "key1", &entry).unwrap();

    let found = cache_lookup(&config, "demo", "key1").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().output, "hi");

    let _ = std::fs::remove_dir_all(&dir);
}
```

## Files Modified

| File | Change |
|------|--------|
| `crates/hick/src/cache.rs` | New file: cache key computation, lookup, store, clear |
| `crates/hick/src/lib.rs` | Add `mod cache`, integrate cache into `run_pipeline_live()` |
| `crates/hick/src/executor.rs` | Add `inject_transcript_entry()` |
| `crates/hick/src/main.rs` | `--cache`, `--freeze`, `--clear-cache` CLI flags |
| `crates/hick/Cargo.toml` | Add `sha2`, `serde_json` dependencies |
| `crates/hick/tests/cache_tests.rs` | New test file |

## Open Questions

1. **Should `.hick-cache/` be in `.gitignore` by default?** Recommended yes — cache contains machine-specific output. Freeze mode users who want portable caches could check it in manually.
2. **Cache format versioning**: The `meta.json` file should contain a format version. If the format changes, the entire cache is invalidated (just delete and rebuild).
3. **Volume content hashing**: For volume-flow dependencies, the cache key includes upstream output hashes. This means changing a writer's output automatically invalidates all downstream readers. For large volume outputs, this hashing could be expensive — we may want to hash a manifest (file names + sizes) instead of full content.
