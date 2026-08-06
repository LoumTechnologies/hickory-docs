//! Environment-driven configuration for [`crate::CanopyExecutor`].
//!
//! See `docs/specs/freeform/canopy-integration.md` for the env contract.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use base64::Engine as _;

/// Everything the executor needs to reach a canopy node agent and shape the
/// sandboxes it spawns. Parse once at startup with [`CanopyConfig::from_env`];
/// nothing in the executor reads the environment after that.
#[derive(Debug, Clone)]
pub struct CanopyConfig {
    /// Node agent gRPC endpoint: an absolute unix socket path (`/run/...`)
    /// or a `host:port` mesh address.
    pub agent: String,
    /// Decoded capability token, attached to every RPC as the
    /// `x-canopy-capability-bin` binary metadata (raw biscuit bytes). `None`
    /// over the local unix socket, where reaching the socket is the trust.
    pub token: Option<Vec<u8>>,
    /// Node name, recorded for error messages only (a token is minted per
    /// node; the agent itself knows which node it is).
    pub node: Option<String>,
    /// `.hick` image ref (`python:3.12`) → Nix store image path declared in
    /// the tenant ledger.
    pub image_map: HashMap<String, String>,
    /// Sandbox shape requested on spawn.
    pub vcpus: u32,
    pub mem_mib: u64,
    pub lifetime_secs: u64,
    /// Extra egress hosts requested for every sandbox.
    pub egress_hosts: Vec<String>,
    /// How long to wait for the guest agent's ready banner after spawn.
    pub boot_timeout: Duration,
    /// Per-exec wall-clock limit.
    pub exec_timeout: Duration,
}

fn env_opt(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) if !v.trim().is_empty() => Some(v.trim().to_string()),
        _ => None,
    }
}

fn env_parse<T: std::str::FromStr>(name: &str, default: T) -> Result<T>
where
    T::Err: std::fmt::Display,
{
    match env_opt(name) {
        None => Ok(default),
        Some(v) => v
            .parse()
            .map_err(|e| anyhow::anyhow!("{name}={v:?} is not a valid value: {e}")),
    }
}

impl CanopyConfig {
    /// Read the `CANOPY_*` environment. Fails with actionable messages: a
    /// missing `CANOPY_AGENT` or malformed `CANOPY_IMAGE_MAP` should be
    /// fixed in deployment config, not discovered mid-pipeline.
    pub fn from_env() -> Result<Self> {
        let agent = env_opt("CANOPY_AGENT").context(
            "CANOPY_AGENT is not set. HICKORY_EXECUTOR=canopy needs the node agent's \
             gRPC endpoint: an absolute unix socket path (e.g. /run/canopy/agent.sock) \
             or a mesh host:port (e.g. 10.77.0.1:7433). Set CANOPY_AGENT, or use \
             HICKORY_EXECUTOR=local",
        )?;

        let token = match env_opt("CANOPY_TOKEN") {
            Some(t) => Some(
                base64::engine::general_purpose::STANDARD
                    .decode(t.trim())
                    .context("CANOPY_TOKEN is not valid base64 (it should be the capability token exactly as minted)")?,
            ),
            None => None,
        };

        let image_map: HashMap<String, String> = match env_opt("CANOPY_IMAGE_MAP") {
            Some(raw) => serde_json::from_str(&raw).context(
                "CANOPY_IMAGE_MAP is not a JSON object of image ref -> Nix store path, \
                 e.g. {\"python:3.12\":\"/nix/store/...-canopy-sandbox-image\"}",
            )?,
            None => HashMap::new(),
        };

        let egress_hosts = env_opt("CANOPY_EGRESS_HOSTS")
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            agent,
            token,
            node: env_opt("CANOPY_NODE"),
            image_map,
            vcpus: env_parse("CANOPY_VCPUS", 1u32)?,
            mem_mib: env_parse("CANOPY_MEM_MIB", 512u64)?,
            lifetime_secs: env_parse("CANOPY_LIFETIME_SECS", 900u64)?,
            egress_hosts,
            boot_timeout: Duration::from_secs(env_parse("CANOPY_BOOT_TIMEOUT_SECS", 180u64)?),
            exec_timeout: Duration::from_secs(env_parse("CANOPY_EXEC_TIMEOUT_SECS", 300u64)?),
        })
    }

    /// Map a `.hick` image ref to its Nix store path. Unknown refs fail with
    /// the configured set, so the fix (add a mapping, or use one that
    /// exists) is visible in the error itself.
    pub fn resolve_image(&self, image: &str) -> Result<String> {
        if let Some(path) = self.image_map.get(image) {
            return Ok(path.clone());
        }
        let mut known: Vec<&str> = self.image_map.keys().map(String::as_str).collect();
        known.sort_unstable();
        if known.is_empty() {
            bail!(
                "image '{image}' is not mapped to a canopy sandbox image, and \
                 CANOPY_IMAGE_MAP is empty. Set CANOPY_IMAGE_MAP to a JSON object \
                 mapping image refs to the Nix store paths declared in the tenant \
                 ledger, e.g. {{\"{image}\":\"/nix/store/...-canopy-sandbox-image\"}}"
            );
        }
        bail!(
            "image '{image}' is not in CANOPY_IMAGE_MAP. Configured images: {}. \
             Add a mapping for '{image}' (the Nix store path must be in the tenant \
             ledger's image allowlist), or change the document to use a configured image",
            known.join(", ")
        )
    }
}

/// Best-effort summary of the configured image map (`.hick` image ref →
/// Nix store path) for display purposes (e.g. the server's `GET
/// /api/executor`). Unlike [`CanopyConfig::from_env`] this never fails:
/// unset or malformed configuration yields an empty map — resolution
/// errors surface at run time with actionable messages, not here.
pub fn image_map_summary() -> HashMap<String, String> {
    env_opt("CANOPY_IMAGE_MAP")
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(map: &[(&str, &str)]) -> CanopyConfig {
        CanopyConfig {
            agent: "/tmp/x.sock".into(),
            token: None,
            node: None,
            image_map: map
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            vcpus: 1,
            mem_mib: 512,
            lifetime_secs: 900,
            egress_hosts: vec![],
            boot_timeout: Duration::from_secs(180),
            exec_timeout: Duration::from_secs(300),
        }
    }

    #[test]
    fn known_image_maps_to_store_path() {
        let c = cfg(&[("python:3.12", "/nix/store/abc-img")]);
        assert_eq!(
            c.resolve_image("python:3.12").unwrap(),
            "/nix/store/abc-img"
        );
    }

    #[test]
    fn unknown_image_error_lists_configured_images() {
        let c = cfg(&[
            ("python:3.12", "/nix/store/abc-img"),
            ("alpine:3.20", "/nix/store/def-img"),
        ]);
        let err = c.resolve_image("node:22").unwrap_err().to_string();
        assert!(err.contains("node:22"), "err: {err}");
        assert!(err.contains("alpine:3.20, python:3.12"), "err: {err}");
    }

    #[test]
    fn empty_map_error_explains_canopy_image_map() {
        let c = cfg(&[]);
        let err = c.resolve_image("python:3.12").unwrap_err().to_string();
        assert!(err.contains("CANOPY_IMAGE_MAP is empty"), "err: {err}");
        assert!(err.contains("/nix/store/"), "err: {err}");
    }
}
