//! The on-disk key file behind the desktop app's Settings page.
//!
//! The CLI reads provider keys from the environment, and that stays true.
//! The desktop app cannot: it is launched from a dock or a Start menu, where
//! there is no shell profile to export anything, so its keys are typed into
//! a Settings page and kept in one JSON file under the app's config
//! directory — `{"anthropic": "sk-…", "openai": null, …}`, keyed by provider
//! selector. This module is that file.
//!
//! Three properties this type is responsible for:
//!
//! - **The file is private.** [`KeyStore::save`] writes atomically with
//!   `0600` on Unix (owner read/write, nothing for group or world); Windows
//!   inherits the profile directory's ACLs, which are already per-user.
//! - **Key material never leaks through logging.** The `Debug` impl prints a
//!   masked fragment, never the key, so an enclosing struct's `{:?}` cannot
//!   betray it.
//! - **An unknown provider is refused at `set`,** naming the valid
//!   selectors — a typo must not silently store a key nothing will read.
//!
//! Precedence against the environment lives in `provider.rs`
//! ([`crate::provider::resolve_selector_with_store`] /
//! [`crate::provider::client_for_with_store`]): stored key first, environment
//! second.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use anyhow::Context as _;

use crate::provider::ProviderSelection;

/// Provider API keys, keyed by canonical selector name
/// (`ProviderSelection::name()`).
#[derive(Clone, Default, PartialEq, Eq)]
pub struct KeyStore {
    keys: BTreeMap<String, String>,
}

/// A displayable fragment of a key: at most the first 4 and last 2
/// characters, and nothing at all for a key short enough that showing them
/// would show most of it. Never the full key, whatever the length.
pub fn masked_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() < 12 {
        return "…".to_string();
    }
    let head: String = chars[..4].iter().collect();
    let tail: String = chars[chars.len() - 2..].iter().collect();
    format!("{head}…{tail}")
}

impl fmt::Debug for KeyStore {
    /// Redacted: providers and masked fragments only. A `KeyStore` routinely
    /// sits inside state structs that get `{:?}`-logged on error paths, and
    /// a derive here would put every key into that log line.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for (name, key) in &self.keys {
            map.entry(name, &masked_key(key));
        }
        map.finish()
    }
}

impl KeyStore {
    /// The canonical storage name for a selector, or `None` for an unknown
    /// one. `grok` and `xai` are one provider, so they canonicalize to one
    /// name — whichever alias set the key, the same lookup finds it.
    fn canonical(selector: &str) -> Option<&'static str> {
        ProviderSelection::parse(selector).map(ProviderSelection::name)
    }

    /// Load the store from `path`. An absent file is an empty store — the
    /// state of every fresh install — not an error; an unreadable or
    /// malformed file is an error naming the file and what to do about it.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = match std::fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("could not read the key file {}", path.display()));
            }
        };
        let parsed: BTreeMap<String, Option<String>> =
            serde_json::from_str(&raw).with_context(|| {
                format!(
                    "the key file {} is not valid JSON — it should be an object like \
                     {{\"anthropic\": \"sk-…\"}}. Fix it, or delete it and re-enter the \
                     keys in Settings.",
                    path.display()
                )
            })?;
        let mut keys = BTreeMap::new();
        for (name, value) in parsed {
            // Unknown names are skipped, not fatal: a file written by a
            // newer version with one more provider must not brick this one.
            let Some(canonical) = Self::canonical(&name) else {
                continue;
            };
            if let Some(v) = value.filter(|v| !v.trim().is_empty()) {
                keys.insert(canonical.to_string(), v);
            }
        }
        Ok(Self { keys })
    }

    /// The stored key for `selector`, if any.
    pub fn key_for(&self, selector: &str) -> Option<String> {
        Self::canonical(selector).and_then(|name| self.keys.get(name).cloned())
    }

    /// The stored key for the provider whose key environment variable is
    /// `var` (`OPENAI_API_KEY` → the `openai` entry). This is what lets the
    /// store slot into the same lookup-by-variable precedence chain
    /// `resolve_selector` already runs.
    pub fn key_for_env(&self, var: &str) -> Option<String> {
        ProviderSelection::all()
            .into_iter()
            .find(|sel| sel.key_env() == var)
            .and_then(|sel| self.keys.get(sel.name()).cloned())
    }

    /// Store (`Some`) or clear (`None`, or a blank string) the key for
    /// `selector`. An unknown selector is an error naming the valid ones.
    pub fn set(&mut self, selector: &str, key: Option<String>) -> anyhow::Result<()> {
        let canonical = Self::canonical(selector).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown provider {selector:?}; expected one of: {}",
                ProviderSelection::ALL.join(", ")
            )
        })?;
        match key.filter(|k| !k.trim().is_empty()) {
            Some(k) => self.keys.insert(canonical.to_string(), k),
            None => self.keys.remove(canonical),
        };
        Ok(())
    }

    /// Write the store to `path`: atomically (write-then-rename, so a crash
    /// mid-write cannot leave a torn file), and owner-only (`0600`) on Unix.
    /// Every provider appears in the file, `null` when unset, so a user who
    /// opens it sees the full form to fill in.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let mut body = serde_json::Map::new();
        for sel in ProviderSelection::all() {
            let value = self
                .keys
                .get(sel.name())
                .map(|k| serde_json::Value::String(k.clone()))
                .unwrap_or(serde_json::Value::Null);
            body.insert(sel.name().to_string(), value);
        }
        let json = serde_json::to_string_pretty(&serde_json::Value::Object(body))
            .expect("a string map serializes");

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("could not create {}", parent.display()))?;
        }

        // The temp file sits beside the target so the rename stays on one
        // filesystem (a cross-device rename is a copy, which is not atomic).
        let tmp = path.with_extension("tmp");
        {
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create(true).truncate(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt as _;
                // 0600 at creation, not after: there must be no instant in
                // which the file exists world-readable with keys in it.
                opts.mode(0o600);
            }
            let mut file = opts
                .open(&tmp)
                .with_context(|| format!("could not write the key file {}", tmp.display()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;
                // `mode` only applies on creation; a leftover tmp file from
                // a previous run keeps its old bits unless reset here.
                file.set_permissions(std::fs::Permissions::from_mode(0o600))
                    .with_context(|| format!("could not restrict {}", tmp.display()))?;
            }
            use std::io::Write as _;
            file.write_all(json.as_bytes())
                .and_then(|()| file.sync_all())
                .with_context(|| format!("could not write the key file {}", tmp.display()))?;
        }
        std::fs::rename(&tmp, path)
            .with_context(|| format!("could not move the key file into {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_file_is_an_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = KeyStore::load(&dir.path().join("does-not-exist.json")).unwrap();
        assert_eq!(store, KeyStore::default());
        assert!(store.key_for("anthropic").is_none());
    }

    #[test]
    fn keys_round_trip_through_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("llm-keys.json");

        let mut store = KeyStore::default();
        store
            .set("anthropic", Some("sk-ant-roundtrip-000042".into()))
            .unwrap();
        // The alias stores under the canonical name…
        store
            .set("grok", Some("xai-roundtrip-000042".into()))
            .unwrap();
        store.save(&path).unwrap();

        let loaded = KeyStore::load(&path).unwrap();
        assert_eq!(
            loaded.key_for("anthropic").as_deref(),
            Some("sk-ant-roundtrip-000042")
        );
        // …so either alias finds it again.
        assert_eq!(
            loaded.key_for("xai").as_deref(),
            Some("xai-roundtrip-000042")
        );
        assert_eq!(
            loaded.key_for("grok").as_deref(),
            Some("xai-roundtrip-000042")
        );

        // Clearing removes it from the file too.
        let mut loaded = loaded;
        loaded.set("anthropic", None).unwrap();
        loaded.save(&path).unwrap();
        assert!(
            KeyStore::load(&path)
                .unwrap()
                .key_for("anthropic")
                .is_none()
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_saved_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("llm-keys.json");
        let mut store = KeyStore::default();
        store
            .set("openai", Some("sk-perm-check-000042".into()))
            .unwrap();
        store.save(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "keys must not be group- or world-readable");
    }

    #[test]
    fn an_unknown_selector_is_refused_naming_the_valid_ones() {
        let mut store = KeyStore::default();
        let err = store
            .set("gemini", Some("sk-x".into()))
            .unwrap_err()
            .to_string();
        assert!(err.contains("gemini"), "{err}");
        for name in ["anthropic", "openai", "deepseek", "grok", "openrouter"] {
            assert!(err.contains(name), "the error must name {name}: {err}");
        }
        assert!(!err.contains("sk-x"), "never echo key material: {err}");
    }

    #[test]
    fn debug_masks_every_key() {
        let mut store = KeyStore::default();
        store
            .set("anthropic", Some("sk-ant-very-secret-000042".into()))
            .unwrap();
        store.set("openai", Some("short".into())).unwrap();
        let dump = format!("{store:?}");
        assert!(!dump.contains("very-secret"), "{dump}");
        assert!(!dump.contains("short"), "{dump}");
        assert!(
            dump.contains("sk-a…42"),
            "the masked fragment shows: {dump}"
        );
    }

    #[test]
    fn masking_never_reveals_a_short_key() {
        assert_eq!(masked_key("sk-abcdefgh0042"), "sk-a…42");
        // Anything under 12 chars would be mostly revealed by 4+2; show
        // nothing instead.
        assert_eq!(masked_key("tiny"), "…");
        assert_eq!(masked_key("elevenchars"), "…");
        assert_eq!(masked_key(""), "…");
    }

    #[test]
    fn a_malformed_file_names_itself_and_the_fix() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("llm-keys.json");
        std::fs::write(&path, "not json at all").unwrap();
        let err = format!("{:#}", KeyStore::load(&path).unwrap_err());
        assert!(err.contains("llm-keys.json"), "{err}");
        assert!(err.contains("delete"), "{err}");
    }

    #[test]
    fn an_unknown_provider_in_the_file_is_skipped_not_fatal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("llm-keys.json");
        std::fs::write(
            &path,
            r#"{"anthropic": "sk-ant-kept-000042", "future-provider": "sk-f"}"#,
        )
        .unwrap();
        let store = KeyStore::load(&path).unwrap();
        assert_eq!(
            store.key_for("anthropic").as_deref(),
            Some("sk-ant-kept-000042")
        );
    }
}
