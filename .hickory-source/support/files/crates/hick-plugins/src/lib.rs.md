<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `fb3bc671b954bf343382764613fe81ab59a1842264894ab07ec473fa5c6bf0d6`.

<book:entry path="crates/hick-plugins/src/lib.rs">
//! Built-in language plugins and their typed contributions. Registry inspection
//! never installs tools, executes application code, or grants sandbox access.
//! Hosts initiate execution; providers own ecosystem-specific behavior.
pub mod build;
pub mod capabilities;
mod csharp;
pub mod dap_discovery;
mod debug;
mod install;
pub mod java;
pub mod java_debug;
mod languages;
pub mod lsp_discovery;
pub mod metadata;
pub mod routing;
pub mod scaffold;
pub mod session_writes;
pub mod testing;
pub mod typed_client;

pub use debug::{DebugCandidate, DebugRecipe, PORT_PLACEHOLDER, Transport};
pub use install::{Asset, Installer};
use std::collections::HashSet;
use std::path::{Component, Path};
use std::sync::LazyLock;

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

#[derive(Debug, Clone, Copy)]
pub struct LspCandidate {
    pub bin: &'static str,
    pub args: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormulaBackend {
    pub language: &'static str,
    pub aliases: &'static [&'static str],
    pub file_name: &'static str,
    pub interpreters: &'static [&'static str],
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum EnvValue {
    Literal(&'static str),
    /// Relative to a host-selected state directory, never a permission grant.
    StatePath(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct Environment {
    pub name: &'static str,
    pub value: EnvValue,
}

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

#[derive(Debug)]
pub struct Plugin {
    pub id: &'static str,
    pub languages: &'static [&'static str],
    pub interfaces: &'static [Interface],
}

/// Immutable ownership registry. Registration order never resolves conflicts.
pub struct Registry {
    plugins: &'static [&'static Plugin],
}
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
fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.contains(':')
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
fn valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
pub fn registry() -> &'static Registry {
    static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
        Registry::new(languages::PLUGINS).expect("built-in plugin declarations are valid")
    });
    &REGISTRY
}
#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
</book:entry>

</book:doc>
