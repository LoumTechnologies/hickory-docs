//! Launch discovery is read-only. Providers and manual profiles produce one model.
use hick_lang::{EntryKind, ProjectTree};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CONFIG: &str = ".hick-launch.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default)]
    pub artifact: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default = "root")]
    pub cwd: String,
    #[serde(default)]
    pub build: Vec<String>,
    #[serde(default)]
    pub default: bool,
}
fn root() -> String {
    ".".into()
}
#[derive(Clone, Debug, Serialize)]
pub struct Profile {
    #[serde(flatten)]
    pub definition: Definition,
    pub origin: String,
    pub can_debug: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    version: u32,
    profiles: Vec<Definition>,
}

/// Each provider receives the same immutable project, without starting tools.
pub trait Provider {
    fn discover(&self, tree: &ProjectTree) -> Result<Vec<Profile>, String>;
}
pub struct Applications;
pub struct PackageScripts;
pub struct PythonTests;

fn profile(
    id: String,
    name: String,
    program: Option<String>,
    command: Vec<String>,
    cwd: String,
    origin: &str,
) -> Profile {
    let can_debug = program
        .as_ref()
        .is_some_and(|p| hick_dap::language_of(std::path::Path::new(p)).is_some());
    Profile {
        definition: Definition {
            id,
            name,
            program,
            command,
            artifact: None,
            cwd,
            args: vec![],
            env: BTreeMap::new(),
            build: vec![],
            default: false,
        },
        origin: origin.into(),
        can_debug,
    }
}
/// Existing test gutters and command providers share the same launch model.
pub fn command_profile(
    id: String,
    name: String,
    command: Vec<String>,
    cwd: String,
    origin: &str,
) -> Profile {
    profile(id, name, None, command, cwd, origin)
}
fn text<'a>(tree: &'a ProjectTree, path: &str) -> Option<&'a str> {
    let entry = tree.get(path)?;
    (entry.kind == EntryKind::File)
        .then(|| std::str::from_utf8(&entry.bytes).ok())
        .flatten()
}
/// Conservative syntax-shaped discovery, not a claim about call-graph reachability.
impl Provider for Applications {
    fn discover(&self, tree: &ProjectTree) -> Result<Vec<Profile>, String> {
        let mut found = vec![];
        for path in tree.entries().keys() {
            let Some(source) = text(tree, path) else {
                continue;
            };
            let is_test = path
                .rsplit('/')
                .next()
                .is_some_and(|p| p.starts_with("test_") || p.ends_with("_test.py"));
            let runnable = if path.ends_with(".py") && !is_test {
                source.lines().any(|line| {
                    if line.starts_with(char::is_whitespace) {
                        return false;
                    }
                    let line = line.trim();
                    line.starts_with("if __name__")
                        || line.starts_with("print(")
                        || line.starts_with("asyncio.run(")
                })
            } else if path.ends_with(".js") || path.ends_with(".mjs") || path.ends_with(".cjs") {
                source.contains("console.")
                    || source.contains(".listen(")
                    || path.ends_with("/index.js")
                    || path == "index.js"
            } else if path.ends_with(".rs") {
                (path.ends_with("main.rs") || path.contains("/src/bin/"))
                    && source.contains("fn main(")
            } else if path.ends_with(".go") {
                source.contains("func main(")
            } else if path.ends_with(".cs") {
                path.ends_with("Program.cs")
                    || source.contains("static void Main(")
                    || source.contains("static async Task Main(")
            } else {
                false
            };
            if runnable {
                let manifest = if path.ends_with(".rs") {
                    Some("Cargo.toml")
                } else if path.ends_with(".cs") {
                    Some(".csproj")
                } else {
                    None
                };
                let cwd = manifest
                    .and_then(|name| {
                        tree.entries()
                            .keys()
                            .filter(|p| p.ends_with(name))
                            .filter_map(|p| {
                                let dir = p.rsplit_once('/').map_or("", |(d, _)| d);
                                (dir.is_empty() || path.starts_with(&format!("{dir}/")))
                                    .then_some(dir)
                            })
                            .max_by_key(|p| p.len())
                    })
                    .filter(|p| !p.is_empty())
                    .unwrap_or(".");
                found.push(profile(
                    format!("app:{path}"),
                    path.clone(),
                    Some(path.clone()),
                    vec![],
                    cwd.into(),
                    "application",
                ));
            }
        }
        Ok(found)
    }
}
impl Provider for PackageScripts {
    fn discover(&self, tree: &ProjectTree) -> Result<Vec<Profile>, String> {
        let mut found = vec![];
        for path in tree
            .entries()
            .keys()
            .filter(|p| p.as_str() == "package.json" || p.ends_with("/package.json"))
        {
            let Some(source) = text(tree, path) else {
                continue;
            };
            let manifest: serde_json::Value =
                serde_json::from_str(source).map_err(|e| format!("Cannot read {path}: {e}"))?;
            let cwd = path.rsplit_once('/').map_or(".", |(p, _)| p);
            if let Some(scripts) = manifest["scripts"].as_object() {
                for name in scripts.keys() {
                    found.push(profile(
                        format!("npm:{path}:{name}"),
                        format!("{cwd} · npm run {name}"),
                        None,
                        vec!["npm".into(), "run".into(), name.clone()],
                        cwd.into(),
                        "package script",
                    ));
                }
            }
        }
        Ok(found)
    }
}
impl Provider for PythonTests {
    fn discover(&self, tree: &ProjectTree) -> Result<Vec<Profile>, String> {
        let mut found = vec![];
        for path in tree.entries().keys().filter(|p| p.ends_with(".py")) {
            let Some(source) = text(tree, path) else {
                continue;
            };
            for line in source.lines() {
                let Some(name) = line
                    .strip_prefix("def ")
                    .or_else(|| line.strip_prefix("async def "))
                    .and_then(|s| s.split_once('(').map(|s| s.0))
                else {
                    continue;
                };
                if name.starts_with("test_")
                    && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    found.push(profile(
                        format!("pytest:{path}:{name}"),
                        format!("{path}::{name}"),
                        None,
                        vec![
                            "python3".into(),
                            "-m".into(),
                            "pytest".into(),
                            format!("{path}::{name}"),
                        ],
                        root(),
                        "test",
                    ));
                }
            }
        }
        Ok(found)
    }
}

pub fn discover_with(
    tree: &ProjectTree,
    providers: &[&dyn Provider],
) -> Result<Vec<Profile>, String> {
    let mut found = BTreeMap::new();
    for provider in providers {
        for profile in provider.discover(tree)? {
            validate(&profile.definition, tree, &profile.origin)?;
            found.insert(profile.definition.id.clone(), profile);
        }
    }
    if let Some(source) = text(tree, CONFIG) {
        let config: Config =
            serde_json::from_str(source).map_err(|e| format!("Fix {CONFIG}: {e}"))?;
        if config.version != 1 {
            return Err(format!(
                "{CONFIG} needs version 1; found {}",
                config.version
            ));
        }
        let mut ids = BTreeSet::new();
        for definition in config.profiles {
            if !ids.insert(definition.id.clone()) {
                return Err(format!(
                    "Duplicate launch profile {} in {CONFIG}; give each profile a unique id",
                    definition.id
                ));
            }
            validate(&definition, tree, CONFIG)?;
            let can_debug = definition
                .program
                .as_ref()
                .is_some_and(|p| hick_dap::language_of(std::path::Path::new(p)).is_some());
            found.insert(
                definition.id.clone(),
                Profile {
                    definition,
                    origin: CONFIG.into(),
                    can_debug,
                },
            );
        }
    } else if tree.get(CONFIG).is_some() {
        return Err(format!(
            "{CONFIG} must be a UTF-8 regular file. Replace its symlink or binary contents."
        ));
    }
    let result: Vec<_> = found.into_values().collect();
    if result.iter().filter(|p| p.definition.default).count() > 1 {
        return Err(format!("Choose only one default profile in {CONFIG}"));
    }
    Ok(result)
}
pub fn discover(tree: &ProjectTree) -> Result<Vec<Profile>, String> {
    discover_with(tree, &[&Applications, &PackageScripts, &PythonTests])
}
pub fn select(tree: &ProjectTree, id: Option<&str>) -> Result<Profile, String> {
    let profiles = discover(tree)?;
    if let Some(id) = id {
        return profiles.into_iter().find(|p| p.definition.id == id).ok_or_else(|| format!("Launch profile {id} is absent from this project version. Choose another profile."));
    }
    if let Some(default) = profiles.iter().find(|p| p.definition.default) {
        return Ok(default.clone());
    }
    let apps: Vec<_> = profiles.iter().filter(|p| p.can_debug).collect();
    if apps.len() == 1 {
        return Ok(apps[0].clone());
    }
    Err(format!(
        "{} launch targets found. Choose a launch profile, or add one in {CONFIG}.",
        profiles.len()
    ))
}
fn validate(p: &Definition, tree: &ProjectTree, origin: &str) -> Result<(), String> {
    let fail = |message: &str| format!("Launch profile {} from {origin}: {message}", p.id);
    if p.id.trim().is_empty() || p.name.trim().is_empty() {
        return Err(fail("id and name cannot be empty"));
    }
    if p.program.is_some() != p.command.is_empty() {
        return Err(fail("specify exactly one of program or command"));
    }
    if let Some(path) = &p.program {
        hick_lang::validate_project_path(path).map_err(|e| fail(&e))?;
        if tree.get(path).is_none_or(|e| e.kind != EntryKind::File) {
            return Err(fail(
                "program must name a regular file in this project version",
            ));
        }
    }
    if let Some(path) = &p.artifact {
        hick_lang::validate_project_path(path).map_err(|e| fail(&e))?;
    }
    if p.cwd != "." {
        hick_lang::validate_project_path(&p.cwd).map_err(|e| fail(&e))?;
    }
    if p.command.first().is_some_and(|s| s.is_empty())
        || p.build.first().is_some_and(|s| s.is_empty())
    {
        return Err(fail("command/build executable cannot be empty"));
    }
    if p.args
        .iter()
        .chain(p.command.iter())
        .chain(p.build.iter())
        .any(|s| s.contains('\0'))
    {
        return Err(fail("arguments cannot contain NUL bytes"));
    }
    if p.env.iter().any(|(k, v)| {
        k.is_empty()
            || k.starts_with(char::is_numeric)
            || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            || v.contains('\0')
    }) {
        return Err(fail("environment names/values are invalid"));
    }
    Ok(())
}

impl Profile {
    pub fn run_argv(&self, root: &std::path::Path) -> Result<Vec<String>, String> {
        let d = &self.definition;
        let mut argv = if let Some(artifact) = &d.artifact {
            let artifact = root.join(artifact).to_string_lossy().into_owned();
            if d.program.as_ref().is_some_and(|p| p.ends_with(".cs")) {
                vec!["dotnet".into(), artifact]
            } else {
                vec![artifact]
            }
        } else if let Some(path) = &d.program {
            let absolute = root.join(path).to_string_lossy().into_owned();
            match hick_dap::language_of(std::path::Path::new(path)) {
                Some("python") => vec!["python3".into(), absolute.clone()],
                Some("javascript") => vec!["node".into(), absolute.clone()],
                Some("rust") => vec!["cargo".into(), "run".into(), "--".into()],
                Some("go") => vec!["go".into(), "run".into(), absolute.clone()],
                Some("csharp") => vec!["dotnet".into(), "run".into(), "--".into()],
                _ => {
                    return Err(format!(
                        "No runner for {path}. Configure command in {CONFIG}."
                    ));
                }
            }
        } else {
            d.command.clone()
        };
        argv.extend(d.args.clone());
        Ok(argv)
    }
}
