<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/capabilities.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-452bde5e61c3"></a>
## `crates/hick-plugins/src/capabilities.rs`

<a id="f-452bde5e61c3-Activation"></a>
### `Activation`

Workspace capability planning. Inspection never installs or executes tools.
Providers own detection and implementation; hosts own probing, scheduling,
confinement and lifecycle. Every interface is enumerated, including future ones.

<storycode:copy id="f-452bde5e61c3-Activation">
//! Workspace capability planning. Inspection never installs or executes tools.
//! Providers own detection and implementation; hosts own probing, scheduling,
//! confinement and lifecycle. Every interface is enumerated, including future ones.
use crate::{Interface, InterfaceKind, Plugin, Registry};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    /// Safe to prepare without running application code.
    Background,
    /// Build, test, debug and profiling require an execution request.
    OnRequest,
}

</storycode:copy>
<a id="f-452bde5e61c3-Availability"></a>
### `Availability`

<storycode:copy id="f-452bde5e61c3-Availability">
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// Provider exists; the host has not checked prerequisites yet.
    Unchecked,
    Available,
    Installable,
    Missing,
    Unsupported,
}

</storycode:copy>
<a id="f-452bde5e61c3-Capability"></a>
### `Capability`

<storycode:copy id="f-452bde5e61c3-Capability">
#[derive(Debug, Clone)]
pub struct Capability {
    pub plugin: &'static str,
    pub language: &'static str,
    pub kind: InterfaceKind,
    pub workspace: PathBuf,
    pub activation: Activation,
    pub availability: Availability,
    pub interface: &'static Interface,
}

</storycode:copy>
<a id="f-452bde5e61c3-impl-Interface"></a>
### `impl-Interface`

<storycode:copy id="f-452bde5e61c3-impl-Interface">
impl Interface {
    pub fn activation(&self) -> Activation {
        match self {
            Self::Scaffold { .. }
            | Self::ProjectEnvironment { .. }
            | Self::Build { .. }
            | Self::TestRunner { .. }
            | Self::Dap { .. }
            | Self::Formula(_)
            | Self::Indexer { .. }
            | Self::RuntimeObservation { .. }
            | Self::Profiling { .. }
            | Self::CodeModel { .. } => Activation::OnRequest,
            _ => Activation::Background,
        }
    }
}

</storycode:copy>
<a id="f-452bde5e61c3-CapabilityManager"></a>
### `CapabilityManager`

One view over the same registry every protocol consumer uses. No capability
whitelist: adding a provider makes it visible to the manager automatically.

<storycode:copy id="f-452bde5e61c3-CapabilityManager">
/// One view over the same registry every protocol consumer uses. No capability
/// whitelist: adding a provider makes it visible to the manager automatically.
pub struct CapabilityManager<'a> {
    registry: &'a Registry,
}

</storycode:copy>
<a id="f-452bde5e61c3-impl-CapabilityManager-a"></a>
### `impl-CapabilityManager<'a>`

<storycode:copy id="f-452bde5e61c3-impl-CapabilityManager-a">
impl<'a> CapabilityManager<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Self { registry }
    }

    /// Resolve the file's language plus project manifests along its ancestor
    /// chain. Each ecosystem uses its nearest root, bounded by the workspace.
    /// A `.csproj` therefore enables C# without pretending XML is C# source.
    pub fn inspect(&self, root: &Path, file: &Path) -> Result<Vec<Capability>, String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        if file.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err("capability path must stay inside the workspace".into());
        }
        let file = if file.is_absolute() {
            file.to_path_buf()
        } else {
            root.join(file)
        };
        let file = file.canonicalize().map_err(|e| e.to_string())?;
        if !file.starts_with(&root) {
            return Err("capability path resolves outside the workspace".into());
        }
        let language = self.registry.language_id(&file.to_string_lossy());
        let start = if file.is_dir() {
            file.as_path()
        } else {
            file.parent().unwrap_or(&root)
        };
        let mut capabilities = Vec::new();
        for plugin in self.registry.plugins() {
            let project = nearest_project(plugin, start, &root);
            let direct = language.filter(|language| plugin.languages.contains(language));
            if direct.is_none() && project.is_none() {
                continue;
            }
            let Some(language) = direct.or_else(|| plugin.languages.first().copied()) else {
                continue;
            };
            let workspace = project.unwrap_or_else(|| root.clone());
            for interface in plugin.interfaces.iter().filter(|i| i.supports(language)) {
                capabilities.push(Capability {
                    plugin: plugin.id,
                    language,
                    kind: interface.kind(),
                    workspace: workspace.clone(),
                    activation: interface.activation(),
                    availability: Availability::Unchecked,
                    interface,
                });
            }
        }
        Ok(capabilities)
    }

    /// Host probes are explicit so metadata inspection cannot spawn a tool or
    /// mistake "a provider was declared" for "the service is running".
    pub fn probe(
        &self,
        capabilities: &mut [Capability],
        mut probe: impl FnMut(&Capability) -> Availability,
    ) {
        for capability in capabilities {
            capability.availability = probe(capability);
        }
    }
}

</storycode:copy>
<a id="f-452bde5e61c3-nearest_project"></a>
### `nearest_project`

<storycode:copy id="f-452bde5e61c3-nearest_project">
fn nearest_project(plugin: &Plugin, start: &Path, root: &Path) -> Option<PathBuf> {
    let markers = plugin.interfaces.iter().find_map(|i| match i {
        Interface::Project { markers } => Some(*markers),
        _ => None,
    })?;
    for dir in start.ancestors().take_while(|dir| dir.starts_with(root)) {
        let entries = std::fs::read_dir(dir).ok();
        if entries.into_iter().flatten().flatten().any(|entry| {
            // Do not follow manifest symlinks outside the workspace.
            entry
                .path()
                .canonicalize()
                .is_ok_and(|p| p.starts_with(root) && p.is_file())
                && markers.iter().any(|marker| {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    marker
                        .strip_prefix('*')
                        .map_or(name == *marker, |suffix| name.ends_with(suffix))
                })
        }) {
            return Some(dir.to_path_buf());
        }
    }
    None
}

</storycode:copy>
<a id="f-452bde5e61c3-tests"></a>
### `tests`

<storycode:copy id="f-452bde5e61c3-tests">
#[cfg(test)]
mod tests {
    use super::*;
    fn write(root: &Path, name: &str) -> PathBuf {
        let path = root.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "").unwrap();
        path
    }

    #[test]
    fn csproj_activates_csharp_without_misclassifying_xml() {
        let root = tempfile::tempdir().unwrap();
        let project = write(root.path(), "app/App.csproj");
        let manager = CapabilityManager::new(crate::registry());
        let capabilities = manager.inspect(root.path(), &project).unwrap();
        assert!(
            capabilities
                .iter()
                .any(|c| c.language == "xml" && c.kind == InterfaceKind::Syntax)
        );
        let plugin = crate::registry()
            .plugins()
            .find(|p| p.id == "csharp")
            .unwrap();
        for interface in plugin.interfaces {
            let capability = capabilities
                .iter()
                .find(|c| c.plugin == plugin.id && c.kind == interface.kind())
                .unwrap();
            assert_eq!(
                capability.workspace,
                project.parent().unwrap().canonicalize().unwrap()
            );
            assert_eq!(capability.availability, Availability::Unchecked);
        }
        assert!(!root.path().join(".hick-cache").exists());
    }

    #[test]
    fn every_routed_language_exposes_every_contribution() {
        let root = tempfile::tempdir().unwrap();
        let manager = CapabilityManager::new(crate::registry());
        for (extension, language, _) in crate::routing::extensions() {
            let file = write(root.path(), &format!("source.{extension}"));
            let actual = manager.inspect(root.path(), &file).unwrap();
            for plugin in crate::registry()
                .plugins()
                .filter(|p| p.languages.contains(language))
            {
                for interface in plugin.interfaces.iter().filter(|i| i.supports(language)) {
                    assert!(
                        actual.iter().any(|c| c.language == *language
                            && c.plugin == plugin.id
                            && c.kind == interface.kind()),
                        "{language}: {:?}",
                        interface.kind()
                    );
                }
            }
        }
    }

    #[test]
    fn nearest_project_roots_do_not_leak_between_siblings() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "Cargo.toml");
        write(root.path(), "crates/first/Cargo.toml");
        write(root.path(), "crates/second/Other.csproj");
        let file = write(root.path(), "crates/first/src/main.rs");
        let actual = CapabilityManager::new(crate::registry())
            .inspect(root.path(), &file)
            .unwrap();
        assert!(
            actual
                .iter()
                .filter(|c| c.plugin == "rust")
                .all(|c| c.workspace == root.path().canonicalize().unwrap().join("crates/first"))
        );
        assert!(!actual.iter().any(|c| c.plugin == "csharp"));
    }

    #[test]
    fn react_has_intelligence_and_tests_but_no_standalone_debug_target() {
        let root = tempfile::tempdir().unwrap();
        let file = write(root.path(), "App.tsx");
        let actual = CapabilityManager::new(crate::registry())
            .inspect(root.path(), &file)
            .unwrap();
        assert!(actual.iter().any(|c| c.kind == InterfaceKind::Lsp));
        assert!(actual.iter().any(|c| c.kind == InterfaceKind::TestRunner));
        assert!(!actual.iter().any(|c| c.kind == InterfaceKind::Dap));
    }

    #[test]
    fn metadata_does_not_claim_services_are_ready_or_invent_coverage() {
        let root = tempfile::tempdir().unwrap();
        let file = write(root.path(), "app.py");
        let manager = CapabilityManager::new(crate::registry());
        let mut actual = manager.inspect(root.path(), &file).unwrap();
        assert!(
            actual
                .iter()
                .all(|c| c.availability == Availability::Unchecked)
        );
        assert!(actual.iter().any(|c| c.kind == InterfaceKind::Profiling));
        assert!(!actual.iter().any(|c| c.kind == InterfaceKind::Coverage));
        manager.probe(&mut actual, |_| Availability::Missing);
        assert!(
            actual
                .iter()
                .all(|c| c.availability == Availability::Missing)
        );
        assert!(!root.path().join(".hick-cache").exists());
    }

    #[test]
    fn executable_features_require_an_execution_request() {
        for interface in crate::registry().plugins().flat_map(|p| p.interfaces) {
            if matches!(
                interface,
                Interface::Build { .. }
                    | Interface::Dap { .. }
                    | Interface::TestRunner { .. }
                    | Interface::Formula(_)
                    | Interface::RuntimeObservation { .. }
                    | Interface::Profiling { .. }
                    | Interface::Indexer { .. }
                    | Interface::CodeModel { .. }
            ) {
                assert_eq!(interface.activation(), Activation::OnRequest);
            }
        }
    }

    #[test]
    fn traversal_and_external_files_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let file = write(outside.path(), "app.py");
        let manager = CapabilityManager::new(crate::registry());
        assert!(manager.inspect(root.path(), &file).is_err());
        assert!(
            manager
                .inspect(root.path(), Path::new("../app.py"))
                .is_err()
        );
        #[cfg(unix)]
        {
            let link = root.path().join("escape.py");
            std::os::unix::fs::symlink(&file, &link).unwrap();
            assert!(manager.inspect(root.path(), &link).is_err());
            std::os::unix::fs::symlink(&file, root.path().join("Cargo.toml")).unwrap();
            let plain = write(root.path(), "notes.txt");
            assert!(manager.inspect(root.path(), &plain).unwrap().is_empty());
        }
    }
}

</storycode:copy>
<a id="f-452bde5e61c3-filename_tests"></a>
### `filename_tests`

<storycode:copy id="f-452bde5e61c3-filename_tests">
#[cfg(test)]
mod filename_tests {
    use super::*;
    #[test]
    fn server_providers_are_reachable_from_their_real_files() {
        let root = tempfile::tempdir().unwrap();
        for (file, language) in [
            ("Dockerfile", "dockerfile"),
            ("Containerfile", "dockerfile"),
            ("main.ex", "elixir"),
            ("Main.hs", "haskell"),
            ("main.tf", "terraform"),
        ] {
            std::fs::write(root.path().join(file), "").unwrap();
            let actual = CapabilityManager::new(crate::registry())
                .inspect(root.path(), Path::new(file))
                .unwrap();
            assert!(
                actual
                    .iter()
                    .any(|c| c.language == language && c.kind == InterfaceKind::Lsp),
                "{file}"
            );
            assert_eq!(
                crate::routing::highlight(language),
                crate::routing::Highlight::PlainText
            );
        }
    }
}

</storycode:copy>
<a id="f-452bde5e61c3-reachability_tests"></a>
### `reachability_tests`

<storycode:copy id="f-452bde5e61c3-reachability_tests">
#[cfg(test)]
mod reachability_tests {
    #[test]
    fn every_language_server_can_be_reached_by_opening_a_file() {
        let routed = crate::routing::known_language_ids();
        for language in crate::registry().languages(crate::InterfaceKind::Lsp) {
            assert!(
                routed.contains(&language),
                "{language} has a server but no file route"
            );
        }
    }
}
</storycode:copy>

### Assemble `crates/hick-plugins/src/capabilities.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/capabilities.rs"><storycode:paste select="#f-452bde5e61c3-Activation"/><storycode:paste select="#f-452bde5e61c3-Availability"/><storycode:paste select="#f-452bde5e61c3-Capability"/><storycode:paste select="#f-452bde5e61c3-impl-Interface"/><storycode:paste select="#f-452bde5e61c3-CapabilityManager"/><storycode:paste select="#f-452bde5e61c3-impl-CapabilityManager-a"/><storycode:paste select="#f-452bde5e61c3-nearest_project"/><storycode:paste select="#f-452bde5e61c3-tests"/><storycode:paste select="#f-452bde5e61c3-filename_tests"/><storycode:paste select="#f-452bde5e61c3-reachability_tests"/></storycode:file>

</storycode:doc>
