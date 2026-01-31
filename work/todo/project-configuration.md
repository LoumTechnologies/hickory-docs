# Project Configuration

A `_hick.yml` file for project-level settings: file ordering, shared defaults, output directory, and default variables.

## Motivation

Currently, all configuration is implicit or passed via CLI flags. For a project with multiple `.hick` files, users must remember which files to pass, in what order, and which `--param` flags to set. Quarto solves this with `_quarto.yml`, which defines project type, file lists, shared metadata, and format options. Hickory Docs needs a lighter-weight equivalent.

## Design

### `_hick.yml` File

A YAML file at the project root that configures the pipeline:

```yaml
# _hick.yml

# Files to process, in order. Globs supported.
files:
  - docs/variables.hick
  - docs/guide.hick
  - docs/reference.hick

# Default variables (overridden by --param)
vars:
  version: "1.0.0"
  env: dev

# Default container image (used when exec has no image attribute)
defaults:
  image: alpine:3.20
  images-dir: ~/.local/share/hick/images

# Output directory (file paths are relative to this)
output-dir: dist/

# Secrets configuration
secrets:
  key-file: ~/.config/hick/key.txt
  secrets-dir: ~/.config/hick/secrets
```

### Discovery

When `hick` is run without file arguments, it searches for `_hick.yml` in the current directory and parent directories (up to the filesystem root or a `.git` boundary). If found, it uses the config. If not found, file arguments are required.

```bash
# Uses _hick.yml in current or parent directory
hick

# Explicit files override _hick.yml file list (but other settings still apply)
hick docs/guide.hick

# Explicit config file
hick --config path/to/_hick.yml
```

### Field Definitions

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `files` | `string[]` | (required if no CLI args) | Ordered list of `.hick` files to process. Supports globs (`docs/*.hick`). |
| `vars` | `map<string, string>` | `{}` | Default variable values. Lowest priority (CLI `--param` > doc `<hick:var>` > config `vars`). |
| `defaults.image` | `string` | `"alpine"` | Default container image when none specified. |
| `defaults.images-dir` | `string` | auto-detected | Directory containing `.wasm` images. |
| `output-dir` | `string` | `.` (current dir) | Base directory for file outputs. |
| `secrets.key-file` | `string` | `~/.config/hick/key.txt` | Age identity file path. |
| `secrets.secrets-dir` | `string` | `~/.config/hick/secrets` | Secrets directory path. |

### Priority Chain (Lowest to Highest)

1. `_hick.yml` `vars` values
2. `<hick:var>` declarations in documents
3. CLI `--param key=value`

For paths (`images-dir`, `key-file`, `secrets-dir`, `output-dir`):
1. `_hick.yml` values
2. CLI flags (`--images-dir`, `--key-file`, `--secrets-dir`)

### Glob Expansion for Files

The `files` field supports glob patterns:

```yaml
files:
  - docs/shared/*.hick    # all .hick files in docs/shared/
  - docs/guide.hick        # specific file
  - chapters/**/*.hick     # recursive glob
```

Files are processed in the order they appear after glob expansion. Within a glob, files are sorted alphabetically.

## Implementation

### Step 1: Config Data Structure

**New file:** `crates/hick/src/config.rs`

```rust
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct HickConfig {
    #[serde(default)]
    pub files: Vec<String>,

    #[serde(default)]
    pub vars: std::collections::HashMap<String, String>,

    #[serde(default)]
    pub defaults: Defaults,

    #[serde(rename = "output-dir")]
    pub output_dir: Option<String>,

    #[serde(default)]
    pub secrets: SecretsConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct Defaults {
    pub image: Option<String>,

    #[serde(rename = "images-dir")]
    pub images_dir: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SecretsConfig {
    #[serde(rename = "key-file")]
    pub key_file: Option<String>,

    #[serde(rename = "secrets-dir")]
    pub secrets_dir: Option<String>,
}

impl HickConfig {
    /// Load config from a YAML file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let config: HickConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(config)
    }

    /// Resolve file globs relative to config file's directory.
    pub fn resolve_files(&self, base_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut result = Vec::new();
        for pattern in &self.files {
            let full_pattern = base_dir.join(pattern);
            let pattern_str = full_pattern.to_string_lossy();
            let mut matches: Vec<PathBuf> = glob::glob(&pattern_str)
                .with_context(|| format!("invalid glob pattern: {pattern}"))?
                .filter_map(|r| r.ok())
                .collect();
            matches.sort();
            if matches.is_empty() {
                log::warn!("Glob pattern '{pattern}' matched no files");
            }
            result.extend(matches);
        }
        Ok(result)
    }
}

/// Search for `_hick.yml` starting from `start_dir`, walking up to `.git` or root.
pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join("_hick.yml");
        if candidate.is_file() {
            return Some(candidate);
        }
        // Stop at .git boundary
        if dir.join(".git").exists() {
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}
```

### Step 2: Integrate into CLI

**File:** `crates/hick/src/main.rs`

```rust
use crate::config::{HickConfig, find_config};

#[derive(Parser)]
struct Cli {
    /// .hick files to process (overrides _hick.yml files list)
    files: Vec<PathBuf>,

    /// Path to config file (default: auto-discover _hick.yml)
    #[arg(long)]
    config: Option<PathBuf>,

    // ... existing fields ...
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config
    let config_path = cli.config.clone()
        .or_else(|| find_config(&std::env::current_dir().unwrap_or_default()));

    let config = if let Some(ref path) = config_path {
        info!("Using config: {}", path.display());
        HickConfig::load(path)?
    } else {
        HickConfig::default()
    };

    let config_dir = config_path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(Path::new("."));

    // Resolve files: CLI args take precedence, then config
    let files = if !cli.files.is_empty() {
        cli.files.clone()
    } else {
        config.resolve_files(config_dir)?
    };

    if files.is_empty() {
        bail!("No .hick files specified. Provide files as arguments or create a _hick.yml.");
    }

    // Merge config vars with CLI params (CLI wins)
    let mut params: Vec<(String, String)> = config.vars
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (k, v) in &cli.params {
        // Remove any config var with same name, then add CLI param
        params.retain(|(pk, _)| pk != k);
        params.push((k.clone(), v.clone()));
    }

    // Resolve paths with config fallbacks
    let key_file = cli.key_file_override()
        .or_else(|| config.secrets.key_file.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("~/.config/hick/key.txt"));

    let secrets_dir = cli.secrets_dir_override()
        .or_else(|| config.secrets.secrets_dir.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("~/.config/hick/secrets"));

    let images_dir = cli.images_dir.clone()
        .or_else(|| config.defaults.images_dir.as_ref().map(PathBuf::from));

    let output_dir = config.output_dir
        .as_ref()
        .map(|d| config_dir.join(d))
        .unwrap_or_else(|| PathBuf::from("."));

    // ... rest of main, using output_dir when writing files ...
}
```

### Step 3: Output Directory Support

**File:** `crates/hick/src/main.rs`

When writing files, prepend the output directory:

```rust
// Write outputs to disk
for (path, content) in &result.files {
    let full_path = output_dir.join(path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&full_path, content)?;
    info!("Wrote {}", full_path.display());
}
```

### Step 4: Default Image Support

**File:** `crates/hick/src/lib.rs`

Add a `default_image` field to `PipelineConfig`:

```rust
pub struct PipelineConfig {
    pub images_dir: PathBuf,
    pub default_image: String,
}
```

Use it when resolving exec images:

```rust
let image = exec_info.image.as_deref()
    .unwrap_or(&config.default_image);
```

For dry-run mode, pass the default image separately or embed it in the pipeline function signature.

### Step 5: Dependencies

**File:** `crates/hick/Cargo.toml`

```toml
serde_yaml = "0.9"   # or serde_yml
glob = "0.3"
```

### Step 6: Tests

**File:** `crates/hick/tests/config_tests.rs`

```rust
use hick::config::{HickConfig, find_config};
use std::path::Path;

#[test]
fn test_parse_minimal_config() {
    let yaml = "files:\n  - guide.hick\n";
    let config: HickConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.files, vec!["guide.hick"]);
}

#[test]
fn test_parse_full_config() {
    let yaml = r#"
files:
  - docs/*.hick
  - reference.hick

vars:
  version: "2.0.0"
  env: staging

defaults:
  image: alpine:3.20
  images-dir: /opt/hick/images

output-dir: dist/

secrets:
  key-file: /etc/hick/key.txt
  secrets-dir: /etc/hick/secrets
"#;
    let config: HickConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.files.len(), 2);
    assert_eq!(config.vars.get("version").unwrap(), "2.0.0");
    assert_eq!(config.defaults.image.as_deref(), Some("alpine:3.20"));
    assert_eq!(config.output_dir.as_deref(), Some("dist/"));
}

#[test]
fn test_find_config_in_current_dir() {
    let dir = std::env::temp_dir().join("hick-config-test-find");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_hick.yml"), "files: []\n").unwrap();

    let found = find_config(&dir);
    assert_eq!(found, Some(dir.join("_hick.yml")));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_find_config_walks_up() {
    let root = std::env::temp_dir().join("hick-config-test-walk");
    let _ = std::fs::remove_dir_all(&root);
    let sub = root.join("a").join("b");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(root.join("_hick.yml"), "files: []\n").unwrap();

    let found = find_config(&sub);
    assert_eq!(found, Some(root.join("_hick.yml")));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_find_config_stops_at_git() {
    let root = std::env::temp_dir().join("hick-config-test-git");
    let _ = std::fs::remove_dir_all(&root);
    let sub = root.join("project");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::create_dir_all(sub.join(".git")).unwrap();
    // Config is above the .git boundary
    std::fs::write(root.join("_hick.yml"), "files: []\n").unwrap();

    let found = find_config(&sub);
    assert!(found.is_none(), "should not find config above .git");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_resolve_files_glob() {
    let dir = std::env::temp_dir().join("hick-config-test-glob");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.hick"), "").unwrap();
    std::fs::write(dir.join("b.hick"), "").unwrap();
    std::fs::write(dir.join("c.txt"), "").unwrap();

    let config = HickConfig {
        files: vec!["*.hick".to_string()],
        ..Default::default()
    };
    let files = config.resolve_files(&dir).unwrap();
    assert_eq!(files.len(), 2);

    let _ = std::fs::remove_dir_all(&dir);
}
```

## Files Modified

| File | Change |
|------|--------|
| `crates/hick/src/config.rs` | New file: `HickConfig`, `find_config()`, YAML parsing, glob resolution |
| `crates/hick/src/main.rs` | Config discovery, merging CLI + config, `--config` flag, output-dir support |
| `crates/hick/src/lib.rs` | `PipelineConfig.default_image`, pass config vars as params |
| `crates/hick/Cargo.toml` | Add `serde_yaml`, `glob` dependencies |
| `crates/hick/tests/config_tests.rs` | New test file |

## Design Decisions

- **YAML, not TOML or XML**: YAML is the de facto standard for project configuration (Quarto, GitHub Actions, Docker Compose). It's more readable than TOML for nested structures and avoids the verbosity of XML for config.
- **`_hick.yml` naming**: The underscore prefix follows Quarto's convention (`_quarto.yml`) and signals "project metadata, not content."
- **Git boundary for discovery**: Walking up directories indefinitely could find configs from unrelated projects. Stopping at `.git` ensures the config belongs to the current repo.
- **Glob ordering**: Alphabetical sort within each glob ensures deterministic builds. Explicit ordering between globs is preserved.
- **Config vars are lowest priority**: This matches the principle that code (document) overrides config, and CLI overrides everything. Users can set defaults in config and override per-run.

## Relationship to Other Features

- **Variables/Parameters**: Config `vars` feed into the same variable system. Config vars have the lowest priority.
- **Caching**: The `.hick-cache/` directory is relative to the project root (where `_hick.yml` lives).
- **File Includes**: Include paths are relative to the including document, not the project root. The project root just determines which files are processed.
