//! Read Docker credentials from `~/.docker/config.json`.

use anyhow::Result;
use oci_client::secrets::RegistryAuth;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct DockerConfig {
    #[serde(default)]
    auths: HashMap<String, AuthEntry>,
}

#[derive(Debug, Deserialize)]
struct AuthEntry {
    auth: Option<String>,
}

/// Resolve registry credentials for a given registry hostname.
///
/// Checks (in order):
/// 1. Explicit username/password if provided
/// 2. `~/.docker/config.json`
/// 3. Falls back to anonymous
pub fn resolve_auth(
    registry: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<RegistryAuth> {
    // Explicit credentials take priority
    if let (Some(user), Some(pass)) = (username, password) {
        return Ok(RegistryAuth::Basic(user.to_string(), pass.to_string()));
    }

    // Try reading from Docker config
    if let Some(auth) = read_docker_config_auth(registry)? {
        return Ok(auth);
    }

    Ok(RegistryAuth::Anonymous)
}

fn docker_config_path() -> Option<PathBuf> {
    // Check DOCKER_CONFIG env var first
    if let Ok(dir) = std::env::var("DOCKER_CONFIG") {
        let path = PathBuf::from(dir).join("config.json");
        if path.exists() {
            return Some(path);
        }
    }

    // Fall back to ~/.docker/config.json
    dirs::home_dir().map(|home| home.join(".docker").join("config.json"))
}

fn read_docker_config_auth(registry: &str) -> Result<Option<RegistryAuth>> {
    let path = match docker_config_path() {
        Some(p) if p.exists() => p,
        _ => return Ok(None),
    };

    let content = std::fs::read_to_string(&path)?;
    let config: DockerConfig = serde_json::from_str(&content)?;

    // Try exact match first, then common aliases
    let keys_to_try = registry_auth_keys(registry);

    for key in &keys_to_try {
        if let Some(entry) = config.auths.get(*key) {
            if let Some(ref encoded) = entry.auth {
                return decode_auth(encoded).map(Some);
            }
        }
    }

    Ok(None)
}

fn registry_auth_keys(registry: &str) -> Vec<&str> {
    let mut keys = vec![registry];

    // Docker Hub has multiple common keys
    if registry == "docker.io" || registry == "registry-1.docker.io" {
        keys.push("https://index.docker.io/v1/");
        keys.push("index.docker.io");
        keys.push("docker.io");
        keys.push("registry-1.docker.io");
    }

    keys
}

fn decode_auth(encoded: &str) -> Result<RegistryAuth> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD.decode(encoded)?;
    let decoded = String::from_utf8(decoded)?;

    let (user, pass) = decoded.split_once(':').unwrap_or((&decoded, ""));

    Ok(RegistryAuth::Basic(user.to_string(), pass.to_string()))
}
