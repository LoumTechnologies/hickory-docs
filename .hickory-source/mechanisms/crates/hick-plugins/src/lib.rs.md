<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ff0c576452e2"></a>
## `crates/hick-plugins/src/lib.rs`

<a id="f-ff0c576452e2-build"></a>
### `build`

Built-in language plugins and their typed contributions. Registry inspection
never installs tools, executes application code, or grants sandbox access.
Hosts initiate execution; providers own ecosystem-specific behavior.

<storycode:copy id="f-ff0c576452e2-build">
//! Built-in language plugins and their typed contributions. Registry inspection
//! never installs tools, executes application code, or grants sandbox access.
//! Hosts initiate execution; providers own ecosystem-specific behavior.
pub mod build;
</storycode:copy>
<a id="f-ff0c576452e2-capabilities"></a>
### `capabilities`

<storycode:copy id="f-ff0c576452e2-capabilities">
pub mod capabilities;
</storycode:copy>
<a id="f-ff0c576452e2-csharp"></a>
### `csharp`

<storycode:copy id="f-ff0c576452e2-csharp">
mod csharp;
</storycode:copy>
<a id="f-ff0c576452e2-dap_discovery"></a>
### `dap_discovery`

<storycode:copy id="f-ff0c576452e2-dap_discovery">
pub mod dap_discovery;
</storycode:copy>
<a id="f-ff0c576452e2-debug"></a>
### `debug`

<storycode:copy id="f-ff0c576452e2-debug">
mod debug;
</storycode:copy>
<a id="f-ff0c576452e2-install"></a>
### `install`

<storycode:copy id="f-ff0c576452e2-install">
mod install;
</storycode:copy>
<a id="f-ff0c576452e2-java"></a>
### `java`

<storycode:copy id="f-ff0c576452e2-java">
pub mod java;
</storycode:copy>
<a id="f-ff0c576452e2-java_debug"></a>
### `java_debug`

<storycode:copy id="f-ff0c576452e2-java_debug">
pub mod java_debug;
</storycode:copy>
<a id="f-ff0c576452e2-languages"></a>
### `languages`

<storycode:copy id="f-ff0c576452e2-languages">
mod languages;
</storycode:copy>
<a id="f-ff0c576452e2-lsp_discovery"></a>
### `lsp_discovery`

<storycode:copy id="f-ff0c576452e2-lsp_discovery">
pub mod lsp_discovery;
</storycode:copy>
<a id="f-ff0c576452e2-metadata"></a>
### `metadata`

<storycode:copy id="f-ff0c576452e2-metadata">
pub mod metadata;
</storycode:copy>
<a id="f-ff0c576452e2-routing"></a>
### `routing`

<storycode:copy id="f-ff0c576452e2-routing">
pub mod routing;
</storycode:copy>
<a id="f-ff0c576452e2-scaffold"></a>
### `scaffold`

<storycode:copy id="f-ff0c576452e2-scaffold">
pub mod scaffold;
</storycode:copy>
<a id="f-ff0c576452e2-session_writes"></a>
### `session_writes`

<storycode:copy id="f-ff0c576452e2-session_writes">
pub mod session_writes;
</storycode:copy>
<a id="f-ff0c576452e2-testing"></a>
### `testing`

<storycode:copy id="f-ff0c576452e2-testing">
pub mod testing;
</storycode:copy>
<a id="f-ff0c576452e2-typed_client"></a>
### `typed_client`

<storycode:copy id="f-ff0c576452e2-typed_client">
pub mod typed_client;

</storycode:copy>
<a id="f-ff0c576452e2-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-ff0c576452e2-use_declaration">
pub use debug::{DebugCandidate, DebugRecipe, PORT_PLACEHOLDER, Transport};
</storycode:copy>
<a id="f-ff0c576452e2-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-ff0c576452e2-use_declaration-2">
pub use install::{Asset, Installer};
</storycode:copy>
<a id="f-ff0c576452e2-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-ff0c576452e2-use_declaration-3">
use std::collections::HashSet;
</storycode:copy>
<a id="f-ff0c576452e2-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-ff0c576452e2-use_declaration-4">
use std::path::{Component, Path};
</storycode:copy>
<a id="f-ff0c576452e2-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-ff0c576452e2-use_declaration-5">
use std::sync::LazyLock;

</storycode:copy>
<a id="f-ff0c576452e2-InterfaceKind"></a>
### `InterfaceKind`

<storycode:copy id="f-ff0c576452e2-InterfaceKind">
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterfaceKind {
    SessionWrites,
    Project,
    Syntax,
    Structure,
    Lsp,
    Dap,
    Build,
    TestRunner,
    CodeModel,
    Formula,
    Indexer,
    TypedClient,
    Runtime,
    Scaffold,
    ProjectEnvironment,
    EditorEnvironment,
    RuntimeObservation,
    Profiling,
    Coverage,
}

</storycode:copy>
<a id="f-ff0c576452e2-LspCandidate"></a>
### `LspCandidate`

<storycode:copy id="f-ff0c576452e2-LspCandidate">
#[derive(Debug, Clone, Copy)]
pub struct LspCandidate {
    pub bin: &'static str,
    pub args: &'static [&'static str],
}

</storycode:copy>
<a id="f-ff0c576452e2-FormulaBackend"></a>
### `FormulaBackend`

<storycode:copy id="f-ff0c576452e2-FormulaBackend">
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormulaBackend {
    pub language: &'static str,
    pub aliases: &'static [&'static str],
    pub file_name: &'static str,
    pub interpreters: &'static [&'static str],
    pub source: &'static str,
}

</storycode:copy>
<a id="f-ff0c576452e2-EnvValue"></a>
### `EnvValue`

<storycode:copy id="f-ff0c576452e2-EnvValue">
#[derive(Debug, Clone, Copy)]
pub enum EnvValue {
    Literal(&'static str),
    /// Relative to a host-selected state directory, never a permission grant.
    StatePath(&'static str),
}

</storycode:copy>
<a id="f-ff0c576452e2-Environment"></a>
### `Environment`

<storycode:copy id="f-ff0c576452e2-Environment">
#[derive(Debug, Clone, Copy)]
pub struct Environment {
    pub name: &'static str,
    pub value: EnvValue,
}

</storycode:copy>
<a id="f-ff0c576452e2-Interface"></a>
### `Interface`

<storycode:copy id="f-ff0c576452e2-Interface">
#[derive(Debug)]
pub enum Interface {
    SessionWrites {
        aliases: &'static [&'static str],
        extract: fn(&str) -> Vec<session_writes::WriteOp>,
    },
    EditorEnvironment {
        kinds: &'static [&'static str],
        settings: fn(Option<&str>, &Path) -> serde_json::Value,
        root: fn(&Path, &Path) -> std::path::PathBuf,
    },
    Project {
        markers: &'static [&'static str],
    },
    Syntax {
        file_names: &'static [(&'static str, &'static str, routing::Highlight)],
        extensions: &'static [(&'static str, &'static str, routing::Highlight)],
    },
    Structure {
        languages: &'static [&'static str],
        language: fn(&str) -> &'static str,
        grammar: fn(&str) -> Option<tree_sitter::Language>,
    },
    Lsp {
        candidates: &'static [LspCandidate],
        project_dirs: &'static [&'static str],
        installer: Option<&'static Installer>,
    },
    Dap {
        candidates: &'static [DebugCandidate],
        project_dirs: &'static [&'static str],
        installer: Option<&'static Installer>,
    },
    Build {
        recipe: build::Build,
    },
    TestRunner {
        plan: testing::Planner,
    },
    CodeModel {
        bin: &'static str,
    },
    Formula(FormulaBackend),
    Indexer {
        bin: &'static str,
        args: &'static [&'static str],
        output_flag: &'static str,
        installer: Option<&'static Installer>,
    },
    TypedClient {
        languages: &'static [&'static str],
        target: typed_client::emit::Target,
    },
    Runtime {
        environment: &'static [Environment],
    },
    Scaffold {
        catalog: fn() -> anyhow::Result<scaffold::Catalog>,
    },
    ProjectEnvironment {
        provider: fn() -> Box<dyn hick_project_env::Provider>,
    },
    RuntimeObservation {
        provider: &'static str,
        script: &'static str,
    },
    Profiling {
        provider: &'static str,
        script: &'static str,
    },
}

</storycode:copy>
<a id="f-ff0c576452e2-impl-Interface"></a>
### `impl-Interface`

<storycode:copy id="f-ff0c576452e2-impl-Interface">
impl Interface {
    /// An ecosystem may share tooling while offering fewer features for an alias.
    pub fn supports(&self, language: &str) -> bool {
        match self {
            Self::TypedClient { languages, .. } | Self::Structure { languages, .. } => {
                languages.contains(&language)
            }
            _ => true,
        }
    }
    pub fn kind(&self) -> InterfaceKind {
        match self {
            Self::SessionWrites { .. } => InterfaceKind::SessionWrites,
            Self::Project { .. } => InterfaceKind::Project,
            Self::Syntax { .. } => InterfaceKind::Syntax,
            Self::Structure { .. } => InterfaceKind::Structure,
            Self::Lsp { .. } => InterfaceKind::Lsp,
            Self::Dap { .. } => InterfaceKind::Dap,
            Self::Build { .. } => InterfaceKind::Build,
            Self::TestRunner { .. } => InterfaceKind::TestRunner,
            Self::CodeModel { .. } => InterfaceKind::CodeModel,
            Self::Formula(_) => InterfaceKind::Formula,
            Self::Indexer { .. } => InterfaceKind::Indexer,
            Self::TypedClient { .. } => InterfaceKind::TypedClient,
            Self::Runtime { .. } => InterfaceKind::Runtime,
            Self::Scaffold { .. } => InterfaceKind::Scaffold,
            Self::ProjectEnvironment { .. } => InterfaceKind::ProjectEnvironment,
            Self::EditorEnvironment { .. } => InterfaceKind::EditorEnvironment,
            Self::RuntimeObservation { .. } => InterfaceKind::RuntimeObservation,
            Self::Profiling { .. } => InterfaceKind::Profiling,
        }
    }
    pub fn project_dirs(&self) -> &'static [&'static str] {
        match self {
            Self::Lsp { project_dirs, .. } | Self::Dap { project_dirs, .. } => project_dirs,
            _ => &[],
        }
    }
    pub fn installer(&self) -> Option<&'static Installer> {
        match self {
            Self::Lsp { installer, .. }
            | Self::Dap { installer, .. }
            | Self::Indexer { installer, .. } => *installer,
            _ => None,
        }
    }
}

</storycode:copy>
<a id="f-ff0c576452e2-Plugin"></a>
### `Plugin`

<storycode:copy id="f-ff0c576452e2-Plugin">
#[derive(Debug)]
pub struct Plugin {
    pub id: &'static str,
    pub languages: &'static [&'static str],
    pub interfaces: &'static [Interface],
}

</storycode:copy>
<a id="f-ff0c576452e2-Registry"></a>
### `Registry`

Immutable ownership registry. Registration order never resolves conflicts.

<storycode:copy id="f-ff0c576452e2-Registry">
/// Immutable ownership registry. Registration order never resolves conflicts.
pub struct Registry {
    plugins: &'static [&'static Plugin],
}
</storycode:copy>
<a id="f-ff0c576452e2-impl-Registry"></a>
### `impl-Registry`

<storycode:copy id="f-ff0c576452e2-impl-Registry">
impl Registry {
    pub fn new(plugins: &'static [&'static Plugin]) -> Result<Self, String> {
        let mut ids = HashSet::new();
        let mut owners = HashSet::new();
        let mut environment_names = HashSet::new();
        let mut extensions = HashSet::new();
        let mut file_names_seen = HashSet::new();
        let mut formula_aliases = HashSet::new();
        let mut session_aliases = HashSet::new();
        for plugin in plugins {
            if plugin.id.is_empty()
                || !plugin
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
                || !ids.insert(plugin.id)
            {
                return Err(format!("invalid or duplicate plugin id: {}", plugin.id));
            }
            let mut kinds = HashSet::new();
            for interface in plugin.interfaces {
                if !kinds.insert(interface.kind()) {
                    return Err(format!("duplicate interface in {}", plugin.id));
                }
                for language in plugin
                    .languages
                    .iter()
                    .filter(|language| interface.supports(language))
                {
                    if language.is_empty() || !owners.insert((*language, interface.kind())) {
                        return Err(format!("duplicate or empty language/interface: {language}"));
                    }
                }
                match interface {
                    Interface::Syntax {
                        extensions: routes,
                        file_names,
                    } => {
                        for (extension, language, _) in *routes {
                            if extension.is_empty()
                                || !extension
                                    .bytes()
                                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                                || !plugin.languages.contains(language)
                                || !extensions.insert(*extension)
                            {
                                return Err(format!(
                                    "invalid or duplicate extension in {}: {extension}",
                                    plugin.id
                                ));
                            }
                        }
                        for (name, language, _) in *file_names {
                            if name.is_empty()
                                || name.contains('/')
                                || name.contains('\\')
                                || !plugin.languages.contains(language)
                                || !file_names_seen.insert(name.to_ascii_lowercase())
                            {
                                return Err(format!(
                                    "invalid or duplicate file name in {}: {name}",
                                    plugin.id
                                ));
                            }
                        }
                    }
                    Interface::SessionWrites { aliases, .. } => {
                        if aliases
                            .iter()
                            .any(|alias| alias.is_empty() || !session_aliases.insert(*alias))
                        {
                            return Err(format!("conflicting session language in {}", plugin.id));
                        }
                    }
                    Interface::Formula(backend) => {
                        if !valid_relative_path(backend.file_name)
                            || !plugin.languages.contains(&backend.language)
                            || backend
                                .aliases
                                .iter()
                                .any(|alias| !formula_aliases.insert(*alias))
                        {
                            return Err(format!(
                                "invalid or conflicting formula backend in {}",
                                plugin.id
                            ));
                        }
                    }
                    Interface::Project { markers } => {
                        if markers.iter().any(|marker| {
                            !valid_relative_path(marker.strip_prefix('*').unwrap_or(marker))
                        }) {
                            return Err(format!("invalid project marker in {}", plugin.id));
                        }
                    }
                    Interface::TypedClient { languages, .. }
                    | Interface::Structure { languages, .. } => {
                        if languages.is_empty()
                            || languages
                                .iter()
                                .any(|language| !plugin.languages.contains(language))
                        {
                            return Err(format!(
                                "interface language is outside plugin {}",
                                plugin.id
                            ));
                        }
                    }
                    _ => {}
                }
                for path in interface.project_dirs() {
                    if !valid_relative_path(path) {
                        return Err(format!("tool search path must stay relative: {path}"));
                    }
                }
                if let Some(installer) = interface.installer()
                    && !plugins.iter().any(|owner| {
                        owner.languages.contains(&installer.language)
                            && owner
                                .interfaces
                                .iter()
                                .filter_map(Interface::installer)
                                .any(|i| std::ptr::eq(i, installer))
                    })
                {
                    return Err(format!(
                        "installer language is outside plugin {}",
                        plugin.id
                    ));
                }
                if let Interface::Runtime { environment } = interface {
                    for setting in *environment {
                        if !valid_env_name(setting.name)
                            || !environment_names.insert(setting.name.to_ascii_uppercase())
                        {
                            return Err(format!(
                                "invalid or conflicting environment variable: {}",
                                setting.name
                            ));
                        }
                        if let EnvValue::StatePath(path) = setting.value
                            && !valid_relative_path(path)
                        {
                            return Err(format!("runtime state path must stay relative: {path}"));
                        }
                    }
                }
            }
        }
        Ok(Self { plugins })
    }
    /// Exact basenames take precedence over extension routing.
    pub fn language_id(&self, path: &str) -> Option<&'static str> {
        let name = path.rsplit(['/', '\\']).next()?.to_ascii_lowercase();
        let extension = name.rsplit('.').next()?;
        let routes = || self.plugins().flat_map(|p| p.interfaces);
        routes()
            .find_map(|i| match i {
                Interface::Syntax { file_names, .. } => file_names
                    .iter()
                    .find(|(candidate, _, _)| candidate.eq_ignore_ascii_case(&name))
                    .map(|(_, language, _)| *language),
                _ => None,
            })
            .or_else(|| {
                routes().find_map(|i| match i {
                    Interface::Syntax { extensions, .. } => extensions
                        .iter()
                        .find(|(candidate, _, _)| *candidate == extension)
                        .map(|(_, language, _)| *language),
                    _ => None,
                })
            })
    }

    pub fn plugins(&self) -> impl Iterator<Item = &'static Plugin> + '_ {
        self.plugins.iter().copied()
    }
    pub fn interface(&self, language: &str, kind: InterfaceKind) -> Option<&'static Interface> {
        self.plugins()
            .filter(|p| p.languages.contains(&language))
            .flat_map(|p| p.interfaces)
            .find(|i| i.kind() == kind && i.supports(language))
    }
    pub fn languages(&self, kind: InterfaceKind) -> impl Iterator<Item = &'static str> + '_ {
        self.plugins().flat_map(move |p| {
            p.languages.iter().copied().filter(move |language| {
                p.interfaces
                    .iter()
                    .any(|i| i.kind() == kind && i.supports(language))
            })
        })
    }
    pub fn project_dirs(&self, kind: InterfaceKind) -> impl Iterator<Item = &'static str> + '_ {
        self.plugins()
            .flat_map(|p| p.interfaces)
            .filter(move |i| i.kind() == kind)
            .flat_map(Interface::project_dirs)
            .copied()
    }
    pub fn installers(&self, kind: InterfaceKind) -> impl Iterator<Item = &'static Installer> + '_ {
        let mut seen = HashSet::new();
        self.plugins()
            .flat_map(|p| p.interfaces)
            .filter(move |i| i.kind() == kind)
            .filter_map(Interface::installer)
            .filter(move |i| seen.insert(i.language))
    }
    pub fn environment_registry(&self) -> hick_project_env::Registry {
        hick_project_env::Registry::new(
            self.plugins()
                .flat_map(|p| p.interfaces)
                .filter_map(|i| match i {
                    Interface::ProjectEnvironment { provider } => Some(provider()),
                    _ => None,
                })
                .collect(),
        )
    }
    /// Defaults for confined cells; constructing values grants no permissions.
    pub fn runtime_environment(&self, state_root: &Path) -> Vec<(&'static str, String)> {
        self.plugins()
            .flat_map(|plugin| {
                plugin.interfaces.iter().flat_map(move |interface| {
                    let settings = match interface {
                        Interface::Runtime { environment } => *environment,
                        _ => &[],
                    };
                    settings.iter().map(move |setting| {
                        let value = match setting.value {
                            EnvValue::Literal(value) => value.to_string(),
                            EnvValue::StatePath(path) => state_root
                                .join(plugin.id)
                                .join(path)
                                .to_string_lossy()
                                .into_owned(),
                        };
                        (setting.name, value)
                    })
                })
            })
            .collect()
    }
}
</storycode:copy>
<a id="f-ff0c576452e2-valid_relative_path"></a>
### `valid_relative_path`

<storycode:copy id="f-ff0c576452e2-valid_relative_path">
fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.contains(':')
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
</storycode:copy>
<a id="f-ff0c576452e2-valid_env_name"></a>
### `valid_env_name`

<storycode:copy id="f-ff0c576452e2-valid_env_name">
fn valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
</storycode:copy>
<a id="f-ff0c576452e2-registry"></a>
### `registry`

<storycode:copy id="f-ff0c576452e2-registry">
pub fn registry() -> &'static Registry {
    static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
        Registry::new(languages::PLUGINS).expect("built-in plugin declarations are valid")
    });
    &REGISTRY
}
</storycode:copy>
<a id="f-ff0c576452e2-tests"></a>
### `tests`

<storycode:copy id="f-ff0c576452e2-tests">
#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
</storycode:copy>

### Assemble `crates/hick-plugins/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/lib.rs"><storycode:paste select="#f-ff0c576452e2-build"/><storycode:paste select="#f-ff0c576452e2-capabilities"/><storycode:paste select="#f-ff0c576452e2-csharp"/><storycode:paste select="#f-ff0c576452e2-dap_discovery"/><storycode:paste select="#f-ff0c576452e2-debug"/><storycode:paste select="#f-ff0c576452e2-install"/><storycode:paste select="#f-ff0c576452e2-java"/><storycode:paste select="#f-ff0c576452e2-java_debug"/><storycode:paste select="#f-ff0c576452e2-languages"/><storycode:paste select="#f-ff0c576452e2-lsp_discovery"/><storycode:paste select="#f-ff0c576452e2-metadata"/><storycode:paste select="#f-ff0c576452e2-routing"/><storycode:paste select="#f-ff0c576452e2-scaffold"/><storycode:paste select="#f-ff0c576452e2-session_writes"/><storycode:paste select="#f-ff0c576452e2-testing"/><storycode:paste select="#f-ff0c576452e2-typed_client"/><storycode:paste select="#f-ff0c576452e2-use_declaration"/><storycode:paste select="#f-ff0c576452e2-use_declaration-2"/><storycode:paste select="#f-ff0c576452e2-use_declaration-3"/><storycode:paste select="#f-ff0c576452e2-use_declaration-4"/><storycode:paste select="#f-ff0c576452e2-use_declaration-5"/><storycode:paste select="#f-ff0c576452e2-InterfaceKind"/><storycode:paste select="#f-ff0c576452e2-LspCandidate"/><storycode:paste select="#f-ff0c576452e2-FormulaBackend"/><storycode:paste select="#f-ff0c576452e2-EnvValue"/><storycode:paste select="#f-ff0c576452e2-Environment"/><storycode:paste select="#f-ff0c576452e2-Interface"/><storycode:paste select="#f-ff0c576452e2-impl-Interface"/><storycode:paste select="#f-ff0c576452e2-Plugin"/><storycode:paste select="#f-ff0c576452e2-Registry"/><storycode:paste select="#f-ff0c576452e2-impl-Registry"/><storycode:paste select="#f-ff0c576452e2-valid_relative_path"/><storycode:paste select="#f-ff0c576452e2-valid_env_name"/><storycode:paste select="#f-ff0c576452e2-registry"/><storycode:paste select="#f-ff0c576452e2-tests"/></storycode:file>

</storycode:doc>
