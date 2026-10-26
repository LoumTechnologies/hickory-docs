<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/prediction/environment.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a7fe04771e89"></a>
## `crates/hick-literate/src/prediction/environment.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../contract.md).

<a id="f-a7fe04771e89-Profile"></a>
### `Profile`

Host-owned runtime revalidation, shared by execution adapters and evidence export.
Fingerprints are observations before/after a run, not an immutable OS sandbox.

<storycode:copy id="f-a7fe04771e89-Profile">
//! Host-owned runtime revalidation, shared by execution adapters and evidence export.
//! Fingerprints are observations before/after a run, not an immutable OS sandbox.
use anyhow::{Result, ensure};
use hick_observe::prediction::{Identity, digest};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Stdio,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Profile {
    PythonDebugpy { project: PathBuf },
}
</storycode:copy>
<a id="f-a7fe04771e89-impl-Profile"></a>
### `impl-Profile`

<storycode:copy id="f-a7fe04771e89-impl-Profile">
impl Profile {
    pub async fn capture(&self) -> Result<BTreeMap<String, String>> {
        let profile = self.clone();
        tokio::task::spawn_blocking(move || match profile {
            Self::PythonDebugpy { project } => capture_python(&project),
        })
        .await?
        .map_err(|_| anyhow::anyhow!("Private runtime identity unavailable"))
    }
    pub(super) async fn begin(profile: Option<&Self>, identity: &Identity) -> Result<Option<Self>> {
        if let Some(profile) = profile {
            ensure!(
                profile.capture().await? == identity.environment,
                "Private runtime differs from the sealed environment"
            );
        }
        Ok(profile.cloned())
    }
    pub(super) async fn unchanged(profile: &Option<Self>, identity: &Identity) -> Result<bool> {
        match profile {
            Some(profile) => Ok(profile.capture().await? == identity.environment),
            None => Ok(true),
        }
    }
}
</storycode:copy>
<a id="f-a7fe04771e89-capture_python"></a>
### `capture_python`

Captures interpreter/stdlib/import-environment/runtime-library/debugpy hashes.
It deliberately does not certify every OS library, transient mutation or side effect.

<storycode:copy id="f-a7fe04771e89-capture_python">
/// Captures interpreter/stdlib/import-environment/runtime-library/debugpy hashes.
/// It deliberately does not certify every OS library, transient mutation or side effect.
pub fn capture_python(project: &Path) -> Result<BTreeMap<String, String>> {
    let adapter = hick_dap::discover("python", project)
        .ok_or_else(|| anyhow::anyhow!("Private adapter unavailable"))?;
    ensure!(
        adapter.command.len() == 3
            && adapter.command[1] == "-m"
            && adapter.command[2] == "debugpy.adapter"
            && !adapter.hosted
            && !adapter.multi_session,
        "Unsupported private runtime recipe"
    );
    let mut identities = BTreeMap::new();
    identities.insert("adapter_recipe".into(),digest(&serde_json::json!({"command":adapter.command,"transport":format!("{:?}",adapter.transport),"launch":adapter.launch_extra}))?);
    for (name, program, is_adapter) in [
        ("output_python", "python3", false),
        ("adapter_python", adapter.command[0].as_str(), true),
    ] {
        let mut command = std::process::Command::new(program);
        command
            .current_dir(project)
            .args(["-I", "-B", "-c", include_str!("python_identity.py")])
            .stdin(Stdio::null())
            .stderr(Stdio::null());
        if is_adapter {
            command.arg("--adapter");
        }
        let result = command
            .output()
            .map_err(|_| anyhow::anyhow!("Private runtime capture unavailable"))?;
        ensure!(
            result.status.success() && result.stdout.len() <= 65536,
            "Private runtime capture unavailable"
        );
        let captured: serde_json::Value = serde_json::from_slice(&result.stdout)
            .map_err(|_| anyhow::anyhow!("Invalid private runtime fingerprint"))?;
        identities.insert(name.into(), digest(&captured)?);
    }
    Ok(identities)
}
</storycode:copy>

### Assemble `crates/hick-literate/src/prediction/environment.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/prediction/environment.rs"><storycode:paste select="#f-a7fe04771e89-Profile"/><storycode:paste select="#f-a7fe04771e89-impl-Profile"/><storycode:paste select="#f-a7fe04771e89-capture_python"/></storycode:file>

</storycode:doc>
