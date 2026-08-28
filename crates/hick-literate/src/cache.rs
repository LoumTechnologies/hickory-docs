//! Execution result caching for `--cache` and `--freeze` modes.
//!
//! Cache entries are keyed by a SHA-256 hash of everything that affects
//! execution output (image, capabilities, command text, secret names).
//! Results are stored as JSON files in `.hick-cache/transcripts/<container>/`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use hick_token::ContainerCapabilities;

/// A cached execution result, serialized to JSON.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecCacheEntry {
    /// Individual command lines sent to the container.
    pub commands: Vec<String>,
    /// Captured output from the command execution.
    pub output: String,
    /// SHA-256 of the output, for downstream invalidation.
    pub output_hash: String,
}

/// What a cell does about its recording — one setting on one axis, rather
/// than two booleans with an impossible fourth state.
///
/// The axis is **what a missing recording means**, and the three answers are
/// the only three there are: it means nothing (`Off`), it means "execute and
/// remember this" (`Reuse`), or it means "there is no baseline yet" —
/// `Require`, which `run` answers by establishing one and `test` answers by
/// reporting the cell unverifiable.
///
/// This is both the run-wide default (`hick run` with no flag is `Off`,
/// `--cache` is `Reuse`, `--freeze` is `Require`) and, after a cell's own
/// `freeze=` attribute has had its say, the per-cell decision:
/// `freeze="true"` is `Require` and `freeze="false"` is `Off`, whatever the
/// run-wide default is.
///
/// Guarantee: `docs/guarantees/verification/freeze-is-declared-per-cell.md`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CacheMode {
    /// Execute the cell, ignore any recording it has, and record nothing.
    #[default]
    Off,
    /// Answer the cell from a recording that still matches its cache key;
    /// on a miss, execute it and record the result.
    Reuse,
    /// The cell should execute at most once, ever: its recording is the
    /// answer. On a miss there is no baseline — `hick run` establishes one
    /// by executing the cell once and recording it, while `hick test`
    /// reports the cell as *unverifiable* and writes nothing, so a verifier
    /// can never manufacture the baseline it then compares against.
    Require,
}

impl CacheMode {
    /// Whether a recording is looked up for this cell at all.
    pub fn consults(self) -> bool {
        self != CacheMode::Off
    }

    /// Whether executing this cell leaves a recording behind.
    ///
    /// **Always.** This is the half of the axis that is not about what a
    /// missing recording means — it is about whether a cell that really ran
    /// is remembered, and there is no mode in which forgetting is useful.
    ///
    /// It used to be `self != Off`, which made `hick run` — the default,
    /// with no flag — record nothing. The consequence was not a slower weave
    /// but a false statement: a later weave found no recording and wrote
    /// `[never run]` over an artifact a real run had produced, and that
    /// marker reads as "this has never executed" when what it meant was "I
    /// have no recording of this". With `run` recording nothing, the two came
    /// apart constantly. Anyone with the app open on a folder they also use a
    /// terminal in watched a chart or a generated file turn into four words
    /// on the next keystroke.
    ///
    /// The rule that must not bend is a different one, and it is enforced
    /// elsewhere: a VERIFIER writes no recording, ever, or a check could
    /// manufacture the baseline it then compares against. That is
    /// `PipelineConfig::collect_unverifiable`, set only by `hick test`, and
    /// checked at both store sites. Weaving never reaches them because it
    /// never executes. So the only thing this now records is a cell that
    /// genuinely ran, under a verb that genuinely runs.
    pub fn records(self) -> bool {
        true
    }
}

/// Configuration for the cache subsystem.
#[derive(Debug)]
pub struct CacheConfig {
    /// Root directory for cached transcripts.
    pub cache_dir: PathBuf,
    /// The directory the document's relative paths resolve against.
    ///
    /// Kept here because a cache key now covers the cell's inputs, and the
    /// weave path — which never executes, and so never seeds a volume of its
    /// own — has to seed the same volumes from the same place to arrive at
    /// the same key. Without it a woven document could not find any of the
    /// recordings a run had just written.
    pub project_dir: PathBuf,
    /// The **run-wide default** mode, set by `hick run`'s `--cache` /
    /// `--freeze` flags.
    ///
    /// Freeze is a per-cell property: a `freeze=` attribute on a cell
    /// overrides this value in either direction, so a run can be `Off`
    /// while one cell in it is `Require`. That is the ordinary case — a
    /// document that declares `freeze="true"` on one cell and is run with
    /// plain `hick run`.
    pub mode: CacheMode,
}

impl CacheConfig {
    /// Create a new cache config rooted at `project_dir/.hick-cache/transcripts`.
    pub fn new(project_dir: &Path, mode: CacheMode) -> Self {
        Self {
            cache_dir: project_dir.join(".hick-cache").join("transcripts"),
            project_dir: project_dir.to_path_buf(),
            mode,
        }
    }

    /// The mode for one cell: its own `freeze=` attribute if it declared one,
    /// else the run-wide default.
    pub fn mode_for(&self, freeze: Option<bool>) -> CacheMode {
        match freeze {
            Some(true) => CacheMode::Require,
            Some(false) => CacheMode::Off,
            None => self.mode,
        }
    }
}

/// The mode for one cell when the run may have no cache config at all.
///
/// A run with no cache directory (the server's preview, watch, the agent's
/// own tool calls) can neither read nor write a recording, so `Require`
/// degrades to `Off`: the cell executes. It is a declaration this caller
/// cannot honour, not an error — see `run_pipeline_live`.
pub fn cell_mode(cache_config: Option<&CacheConfig>, freeze: Option<bool>) -> CacheMode {
    cache_config.map_or(CacheMode::Off, |cc| cc.mode_for(freeze))
}

/// Compute the cache key (SHA-256 hex) for an exec step.
///
/// **The key covers the cell's inputs, not just its text.** A recording
/// answers "what did this cell produce last time", and that question is only
/// well posed once the answer is pinned to everything that could change the
/// output. Four of those are properties of the cell itself — the image, the
/// capabilities it was granted, the command, and which secrets were in scope.
/// The other two are what it read:
///
/// * `input_digest` — the contents of the volumes mounted into this cell, from
///   [`inputs_digest`]. This is what makes a `<hick:file>` product an input:
///   the document assembles `analysis.py`, the file is seeded into a volume,
///   and the cell that runs it re-executes when it changes. Keyed on the
///   command alone, editing that file would serve the old recording and the
///   document would report numbers produced by code it no longer contains.
/// * `upstream_keys` — the keys of this cell's predecessors in the flow DAG.
///   Chaining them makes invalidation transitive without hashing anything
///   twice: if an upstream cell's inputs changed, its key changed, so every
///   cell downstream of it has a different key too. Container state, volume
///   flow, and copy/paste edges are all carried by this one term, because the
///   DAG already models each of them as an edge.
///
/// Order matters for `upstream_keys`; callers pass them in the DAG's
/// deterministic predecessor order so the same graph always yields the same
/// key.
pub fn exec_cache_key(
    container_image: &str,
    capabilities_canonical: &str,
    command: &str,
    secret_names: &[&str],
    input_digest: &str,
    upstream_keys: &[&str],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"image:");
    hasher.update(container_image.as_bytes());
    hasher.update(b"\ncaps:");
    hasher.update(capabilities_canonical.as_bytes());
    hasher.update(b"\ncmd:");
    hasher.update(command.trim().as_bytes());
    for s in secret_names {
        hasher.update(b"\nsecret:");
        hasher.update(s.as_bytes());
    }
    hasher.update(b"\ninputs:");
    hasher.update(input_digest.as_bytes());
    for k in upstream_keys {
        hasher.update(b"\nupstream:");
        hasher.update(k.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Digest the files a cell can read, as `(path, bytes)` pairs.
///
/// Sorted by path before hashing, because the order files come out of a volume
/// is not a property of the volume — an unsorted digest would make the same
/// inputs produce different keys on different runs and no cell would ever hit
/// its recording. The length of each path and body is folded in so that
/// `("ab", "c")` and `("a", "bc")` cannot collide.
///
/// An empty input set has its own digest rather than the empty string, so
/// "this cell reads nothing" is a statement the key records rather than an
/// absence it cannot distinguish from "inputs not computed".
pub fn inputs_digest(files: &[(String, Vec<u8>)]) -> String {
    let mut sorted: Vec<&(String, Vec<u8>)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    hasher.update(b"inputs-v1:");
    for (path, body) in sorted {
        hasher.update(path.len().to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update(body.len().to_le_bytes());
        hasher.update(body);
    }
    format!("{:x}", hasher.finalize())
}

/// Compute the cache key (SHA-256 hex) for a `<hick:agent>` cell.
///
/// **The prompt and the model are both in the key, and both have to be.** A
/// recording answers the question "what did this cell produce last time"; for
/// an agent cell the question is only well posed once you say *what was asked*
/// and *who was asked*. Key on the prompt alone and editing the prompt serves
/// the old recording — freeze would then verify a claim the document no longer
/// makes. Key on the model alone and the same failure happens the other way.
///
/// Deliberately NOT in the key: `max-turns`. It bounds how hard the cell may
/// try, not what it was asked, and a recording made under a larger budget is
/// still an honest answer to the same question.
///
/// The domain separator differs from [`exec_cache_key`]'s (`agent:` vs
/// `image:`), so an agent cell and an exec cell can never collide on a key
/// even if their text matched byte for byte.
pub fn agent_cache_key(model: &str, prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"agent:");
    hasher.update(b"\nmodel:");
    hasher.update(model.as_bytes());
    hasher.update(b"\nprompt:");
    hasher.update(prompt.trim().as_bytes());
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
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read cache entry {}", path.display()))?;
    let entry: ExecCacheEntry = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse cache entry {}", path.display()))?;
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
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create cache dir {}", dir.display()))?;
    let path = dir.join(format!("{key}.json"));
    let data = serde_json::to_string_pretty(entry)?;
    std::fs::write(&path, data)
        .with_context(|| format!("failed to write cache entry {}", path.display()))?;
    Ok(())
}

/// Delete the entire cache directory.
pub fn cache_clear(config: &CacheConfig) -> Result<()> {
    // Walk up from transcripts/ to .hick-cache/
    if let Some(root) = config.cache_dir.parent()
        && root.exists()
    {
        std::fs::remove_dir_all(root)
            .with_context(|| format!("failed to clear cache at {}", root.display()))?;
    }
    Ok(())
}

/// Produce a canonical string representation of capabilities for hashing.
/// Rules are sorted to ensure deterministic output regardless of definition order.
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

/// Extract secret names for a container (used in cache key computation).
pub fn secret_names_for(
    container_defs: &HashMap<String, ContainerCapabilities>,
    container: &str,
) -> Vec<String> {
    container_defs
        .get(container)
        .map(|caps| {
            caps.secret_rules
                .iter()
                .map(|r| r.secret_name.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// Compute SHA-256 hex of a string.
pub fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The digest of a cell that reads nothing, so the tests below vary one
    /// term at a time.
    fn no_inputs() -> String {
        inputs_digest(&[])
    }

    #[test]
    fn cache_key_deterministic() {
        let k1 = exec_cache_key("alpine", "net:deny_all", "echo hi", &[], &no_inputs(), &[]);
        let k2 = exec_cache_key("alpine", "net:deny_all", "echo hi", &[], &no_inputs(), &[]);
        assert_eq!(k1, k2);
    }

    #[test]
    fn cache_key_changes_with_command() {
        let k1 = exec_cache_key("alpine", "", "echo a", &[], &no_inputs(), &[]);
        let k2 = exec_cache_key("alpine", "", "echo b", &[], &no_inputs(), &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_changes_with_image() {
        let k1 = exec_cache_key("alpine", "", "echo hi", &[], &no_inputs(), &[]);
        let k2 = exec_cache_key("python:3.12", "", "echo hi", &[], &no_inputs(), &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_changes_with_caps() {
        let k1 = exec_cache_key("alpine", "net:deny_all", "echo hi", &[], &no_inputs(), &[]);
        let k2 = exec_cache_key("alpine", "", "echo hi", &[], &no_inputs(), &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_changes_with_secrets() {
        let k1 = exec_cache_key("alpine", "", "echo hi", &[], &no_inputs(), &[]);
        let k2 = exec_cache_key("alpine", "", "echo hi", &["API_KEY"], &no_inputs(), &[]);
        assert_ne!(k1, k2);
    }

    /// The point of the whole change: a cell whose command never changed, but
    /// whose input file did, must not be answered from the old recording.
    #[test]
    fn cache_key_changes_with_inputs() {
        let before = inputs_digest(&[("src@proj/analysis.py".into(), b"x = 1".to_vec())]);
        let after = inputs_digest(&[("src@proj/analysis.py".into(), b"x = 2".to_vec())]);
        let k1 = exec_cache_key("alpine", "", "python analysis.py", &[], &before, &[]);
        let k2 = exec_cache_key("alpine", "", "python analysis.py", &[], &after, &[]);
        assert_ne!(k1, k2);
    }

    /// Invalidation is transitive: a cell downstream of a changed cell has a
    /// different key even though nothing about it changed directly.
    #[test]
    fn cache_key_changes_with_upstream() {
        let k1 = exec_cache_key("alpine", "", "echo hi", &[], &no_inputs(), &["upstream-a"]);
        let k2 = exec_cache_key("alpine", "", "echo hi", &[], &no_inputs(), &["upstream-b"]);
        assert_ne!(k1, k2);
        // And having an upstream at all is different from having none.
        let k3 = exec_cache_key("alpine", "", "echo hi", &[], &no_inputs(), &[]);
        assert_ne!(k1, k3);
    }

    #[test]
    fn inputs_digest_is_order_independent() {
        let a = inputs_digest(&[
            ("b.py".into(), b"two".to_vec()),
            ("a.py".into(), b"one".to_vec()),
        ]);
        let b = inputs_digest(&[
            ("a.py".into(), b"one".to_vec()),
            ("b.py".into(), b"two".to_vec()),
        ]);
        assert_eq!(a, b, "a volume's iteration order is not part of its state");
    }

    /// Length-prefixing means a path/body split cannot be shifted to produce
    /// the same digest from different content.
    #[test]
    fn inputs_digest_does_not_collide_on_a_shifted_boundary() {
        let a = inputs_digest(&[("ab".into(), b"c".to_vec())]);
        let b = inputs_digest(&[("a".into(), b"bc".to_vec())]);
        assert_ne!(a, b);
    }

    #[test]
    fn inputs_digest_distinguishes_empty_from_present() {
        assert_ne!(
            inputs_digest(&[]),
            inputs_digest(&[("a.py".into(), Vec::new())]),
            "a cell that reads an empty file did not read nothing"
        );
    }

    // The four tests below protect
    // docs/guarantees/agent/an-agent-recording-is-keyed-by-prompt-and-model.md.

    #[test]
    fn agent_key_deterministic() {
        assert_eq!(
            agent_cache_key("claude-sonnet-5", "write the greeting"),
            agent_cache_key("claude-sonnet-5", "write the greeting")
        );
    }

    #[test]
    fn agent_key_changes_with_prompt() {
        assert_ne!(
            agent_cache_key("claude-sonnet-5", "write the greeting"),
            agent_cache_key("claude-sonnet-5", "write the farewell"),
            "an edited prompt must retire its recording, or freeze verifies a \
             claim the document no longer makes"
        );
    }

    #[test]
    fn agent_key_changes_with_model() {
        assert_ne!(
            agent_cache_key("claude-sonnet-5", "write the greeting"),
            agent_cache_key("gpt-5", "write the greeting"),
            "a different model is a different answer to the same question"
        );
    }

    #[test]
    fn agent_and_exec_keys_never_collide() {
        // Same text on both sides; only the domain separator differs.
        assert_ne!(
            agent_cache_key("alpine", "echo hi"),
            exec_cache_key("alpine", "", "echo hi", &[], &no_inputs(), &[])
        );
    }

    #[test]
    fn cache_store_and_lookup() {
        let dir = std::env::temp_dir().join("hick-cache-test-store");
        let _ = std::fs::remove_dir_all(&dir);
        let config = CacheConfig::new(&dir, CacheMode::Reuse);

        let entry = ExecCacheEntry {
            commands: vec!["echo hi".into()],
            output: "hi\n".into(),
            output_hash: sha256_hex("hi\n"),
        };
        cache_store(&config, "demo", "key1", &entry).unwrap();

        let found = cache_lookup(&config, "demo", "key1").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.output, "hi\n");
        assert_eq!(found.commands, vec!["echo hi"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cache_lookup_returns_none_for_missing() {
        let dir = std::env::temp_dir().join("hick-cache-test-miss");
        let _ = std::fs::remove_dir_all(&dir);
        let config = CacheConfig::new(&dir, CacheMode::Reuse);

        let found = cache_lookup(&config, "demo", "nonexistent").unwrap();
        assert!(found.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cache_clear_removes_directory() {
        let dir = std::env::temp_dir().join("hick-cache-test-clear");
        let _ = std::fs::remove_dir_all(&dir);
        let config = CacheConfig::new(&dir, CacheMode::Reuse);

        let entry = ExecCacheEntry {
            commands: vec!["echo hi".into()],
            output: "hi\n".into(),
            output_hash: sha256_hex("hi\n"),
        };
        cache_store(&config, "demo", "key1", &entry).unwrap();
        assert!(config.cache_dir.exists());

        cache_clear(&config).unwrap();
        assert!(!config.cache_dir.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn canonical_caps_sorted() {
        let mut caps = ContainerCapabilities::new();
        caps.secret_rules.push(hick_token::SecretRule {
            env_var: "B_KEY".into(),
            secret_name: "b-secret".into(),
        });
        caps.secret_rules.push(hick_token::SecretRule {
            env_var: "A_KEY".into(),
            secret_name: "a-secret".into(),
        });

        let mut defs = HashMap::new();
        defs.insert("test".to_string(), caps);

        let canonical = canonical_caps(&defs, "test");
        // Should be sorted, so a-secret comes before b-secret
        let parts: Vec<&str> = canonical.split(';').collect();
        assert!(parts[0] < parts[1], "parts should be sorted: {canonical}");
    }
}
