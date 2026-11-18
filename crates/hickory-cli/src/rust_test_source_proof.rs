#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceEvidence {
    Selected,
    Other,
    Unknown,
}
fn rust_compiled_source_evidence(
    raw: &serde_json::Value,
    crate_name: &str,
    declaration: &hick_plugins::testing::rust_debug_source::RustTestDeclaration,
    selected: &Path,
    is_crate_root: bool,
) -> Result<std::collections::BTreeMap<String, SourceEvidence>> {
    anyhow::ensure!(
        raw.as_object().is_some_and(|v| v.len() == 3
            && v.keys()
                .all(|k| matches!(k.as_str(), "version" | "producer" | "functions"))),
        "Unexpected Rust compiled source proof fields"
    );
    anyhow::ensure!(
        raw.get("version").and_then(|v| v.as_u64()) == Some(1)
            && raw.get("producer").and_then(|v| v.as_str()) == Some("lldb-image-source-v1"),
        "Unknown Rust compiled source proof"
    );
    let functions = raw
        .get("functions")
        .and_then(|v| v.as_array())
        .context("Rust compiled source symbols are unavailable")?;
    anyhow::ensure!(
        functions.len() <= 1024,
        "Rust compiled source symbol budget exceeded"
    );
    let selected = selected.canonicalize()?;
    let mut readings = std::collections::BTreeMap::new();
    let mut work = 0usize;
    let mut local = declaration.namespace.clone();
    local.push(declaration.name.clone());
    let local = local.join("::");
    for row in functions {
        anyhow::ensure!(
            row.as_object().is_some_and(|v| v.len() == 3
                && v.contains_key("name")
                && v.contains_key("entries")
                && v.contains_key("optimized")),
            "Unexpected Rust compiled function fields"
        );
        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .context("Rust compiled function name is missing")?;
        let optimized = row
            .get("optimized")
            .and_then(|v| v.as_bool())
            .context("Rust compiled function optimization state is missing")?;
        anyhow::ensure!(
            name.len() <= 8192,
            "Rust compiled function name budget exceeded"
        );
        let mut parts = name.split("::").collect::<Vec<_>>();
        if let Some(last) = parts.last() {
            if last.len() == 17
                && last.starts_with('h')
                && last[1..].bytes().all(|b| b.is_ascii_hexdigit())
                && *last != declaration.name
            {
                parts.pop();
            }
        }
        if parts.len() < 2 || parts[0] != crate_name.replace('-', "_") {
            continue;
        }
        parts.remove(0);
        if parts.last().copied() != Some(declaration.name.as_str())
            || parts.iter().any(|part| {
                part.is_empty()
                    || !part.chars().all(|c| {
                        c == '_'
                            || c.is_alphanumeric()
                            || !c.is_ascii() && !c.is_control() && !c.is_whitespace()
                    })
            })
        {
            continue;
        }
        let qualified = parts.join("::");
        let namespace = if is_crate_root {
            qualified == local
        } else {
            qualified == local || qualified.ends_with(&("::".to_owned() + &local))
        };
        let entries = row
            .get("entries")
            .and_then(|v| v.as_array())
            .context("Rust compiled function has no source reading")?;
        anyhow::ensure!(
            entries.len() <= 4096,
            "Rust compiled source entry budget exceeded"
        );
        let (mut matched, mut known) = (false, false);
        for entry in entries {
            anyhow::ensure!(
                entry.as_object().is_some_and(|v| v.len() == 4
                    && v.keys()
                        .all(|k| matches!(k.as_str(), "file" | "line" | "column" | "address"))),
                "Unexpected Rust source entry fields"
            );
            work += 1;
            anyhow::ensure!(
                work <= 100000,
                "Rust compiled source reading budget exceeded"
            );
            let line = entry
                .get("line")
                .and_then(|v| v.as_u64())
                .context("Rust compiled source line is missing")?;
            let file = entry
                .get("file")
                .and_then(|v| v.as_str())
                .context("Rust compiled source path is missing")?;
            anyhow::ensure!(
                file.len() <= 32768,
                "Rust compiled source path budget exceeded"
            );
            let path = Path::new(file);
            if !path.is_absolute() {
                continue;
            }
            if let Ok(path) = path.canonicalize() {
                known = true;
                if path == selected
                    && line >= declaration.first_line as u64
                    && line <= declaration.last_line as u64
                {
                    matched = true
                }
            }
        }
        let evidence = if optimized {
            SourceEvidence::Unknown
        } else if matched && namespace {
            SourceEvidence::Selected
        } else if matched {
            SourceEvidence::Unknown
        } else if known {
            SourceEvidence::Other
        } else {
            SourceEvidence::Unknown
        };
        let slot = readings.entry(qualified).or_insert(evidence);
        if *slot != evidence {
            *slot = SourceEvidence::Unknown
        }
    }
    Ok(readings)
}
#[cfg(test)]
fn rust_compiled_source_identity(
    raw: &serde_json::Value,
    crate_name: &str,
    declaration: &hick_plugins::testing::rust_debug_source::RustTestDeclaration,
    selected: &Path,
    is_crate_root: bool,
) -> Result<String> {
    let mut selected =
        rust_compiled_source_evidence(raw, crate_name, declaration, selected, is_crate_root)?
            .into_iter()
            .filter_map(|(name, state)| (state == SourceEvidence::Selected).then_some(name))
            .collect::<Vec<_>>();
    anyhow::ensure!(
        selected.len() == 1,
        "Selected Rust declaration has no unique compiled source/qualified identity"
    );
    Ok(selected.remove(0))
}
use super::*;
use std::io::{Read, Seek};
struct SourcePin {
    path: PathBuf,
    canonical: PathBuf,
    held: same_file::Handle,
    identity: RustExecutableIdentity,
    changed: (i64, i64),
    text: String,
}
fn change_time(meta: &std::fs::Metadata) -> (i64, i64) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        (meta.ctime(), meta.ctime_nsec())
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        (0, 0)
    }
}
impl SourcePin {
    fn capture(path: &Path) -> Result<Self> {
        let original = std::fs::symlink_metadata(path)?;
        anyhow::ensure!(
            original.is_file() && !original.file_type().is_symlink() && original.len() <= 1048576,
            "Rust source proof needs a bounded ordinary file"
        );
        let file = std::fs::File::open(path)?;
        let held = same_file::Handle::from_file(file)?;
        let canonical = path.canonicalize()?;
        anyhow::ensure!(
            same_file::Handle::from_path(path)? == held,
            "Rust source ownership changed while capturing proof"
        );
        let identity = rust_executable_identity(&canonical)?;
        anyhow::ensure!(
            identity.modified.is_some(),
            "Rust source revision metadata is unavailable"
        );
        let changed = change_time(&held.as_file().metadata()?);
        let mut file = held.as_file().try_clone()?;
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.take(1048577).read_to_end(&mut bytes)?;
        anyhow::ensure!(
            bytes.len() <= 1048576,
            "Rust source proof byte budget exceeded"
        );
        let text = String::from_utf8(bytes).context("Rust source is not UTF-8")?;
        let result = Self {
            path: path.to_owned(),
            canonical,
            held,
            identity,
            changed,
            text,
        };
        result.current()?;
        Ok(result)
    }
    fn current(&self) -> Result<()> {
        anyhow::ensure!(
            self.path.canonicalize()? == self.canonical
                && same_file::Handle::from_path(&self.path)? == self.held
                && rust_executable_identity(&self.canonical)? == self.identity
                && change_time(&self.held.as_file().metadata()?) == self.changed,
            "Rust source identity/revision changed during compiler proof"
        );
        let mut file = self.held.as_file().try_clone()?;
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.take(1048577).read_to_end(&mut bytes)?;
        anyhow::ensure!(
            bytes == self.text.as_bytes(),
            "Rust source bytes changed during compiler proof"
        );
        anyhow::ensure!(
            same_file::Handle::from_path(&self.path)? == self.held,
            "Rust source address was replaced during compiler proof"
        );
        Ok(())
    }
}
pub(super) struct Selection {
    selected: SourcePin,
    manifest: SourcePin,
    declaration: hick_plugins::testing::rust_debug_source::RustTestDeclaration,
    line: u32,
    character: u32,
}
impl Selection {
    pub(super) fn capture(
        path: &Path,
        line: u32,
        character: u32,
        hash: &str,
        name: &str,
        runtime: &Path,
    ) -> Result<Self> {
        let manifest = SourcePin::capture(&runtime.join("Cargo.toml"))?;
        let selected = SourcePin::capture(path)?;
        anyhow::ensure!(
            crate::serve::source_tests::digest(selected.text.as_bytes()) == hash,
            "Selected Rust source no longer matches the admitted compiler output"
        );
        let declaration = hick_plugins::testing::rust_debug_source::selected_declaration(
            &selected.text,
            line,
            character,
            name,
        )
        .map_err(anyhow::Error::msg)?;
        Ok(Self {
            selected,
            manifest,
            declaration,
            line,
            character,
        })
    }
    pub(super) fn current(&self) -> Result<()> {
        self.selected.current()?;
        self.manifest.current()
    }
}
struct SourceReadBudget {
    started: std::time::Instant,
    reads: usize,
    bytes: usize,
}
impl SourceReadBudget {
    fn new() -> Self {
        Self {
            started: std::time::Instant::now(),
            reads: 0,
            bytes: 0,
        }
    }
    fn charge(&mut self, bytes: usize) -> Result<(), String> {
        self.reads = self.reads.saturating_add(1);
        self.bytes = self.bytes.saturating_add(bytes);
        if self.reads > 1024
            || self.bytes > 32 * 1024 * 1024
            || self.started.elapsed() > std::time::Duration::from_secs(10)
        {
            return Err("Aggregate Cargo module-reading budget exceeded".into());
        }
        Ok(())
    }
}
pub(super) struct Preparation {
    selection: Selection,
    files: std::collections::BTreeMap<PathBuf, SourcePin>,
    graphs: std::collections::BTreeMap<
        PathBuf,
        hick_plugins::testing::rust_debug_source::RustModuleTests,
    >,
}
impl Preparation {
    pub(super) fn capture(
        selection: Selection,
        metadata: &serde_json::Value,
        runtime: &Path,
        physical: Option<(&Path, &std::collections::BTreeMap<String, String>)>,
    ) -> Result<Self> {
        selection.current()?;
        let runtime = runtime.canonicalize()?;
        let manifest = runtime.join("Cargo.toml").canonicalize()?;
        let packages = metadata
            .get("packages")
            .and_then(|v| v.as_array())
            .context("Cargo metadata has no package reading")?;
        anyhow::ensure!(
            packages.len() <= 4096,
            "Cargo metadata package budget exceeded"
        );
        let packages = packages
            .iter()
            .filter(|p| {
                p.get("manifest_path")
                    .and_then(|v| v.as_str())
                    .and_then(|p| Path::new(p).canonicalize().ok())
                    .as_ref()
                    == Some(&manifest)
            })
            .collect::<Vec<_>>();
        anyhow::ensure!(
            packages.len() == 1,
            "Cargo metadata has no unique admitted package manifest"
        );
        let targets = packages[0]
            .get("targets")
            .and_then(|v| v.as_array())
            .context("Cargo metadata has no targets")?;
        anyhow::ensure!(
            targets.len() <= 128,
            "Cargo test target proof budget exceeded"
        );
        let mut files = std::collections::BTreeMap::<PathBuf, SourcePin>::new();
        let mut total = selection.selected.text.len();
        let mut graphs = std::collections::BTreeMap::new();
        let mut reading_budget = SourceReadBudget::new();
        for target in targets {
            if target.get("test").and_then(|v| v.as_bool()) != Some(true) {
                continue;
            }
            let path = target
                .get("src_path")
                .and_then(|v| v.as_str())
                .context("Cargo test root is missing")?;
            let root = Path::new(path).canonicalize()?;
            anyhow::ensure!(
                root.starts_with(&runtime),
                "Cargo test root escaped its admitted package"
            );
            if graphs.contains_key(&root) {
                continue;
            }
            let mut read = |path: &Path| -> Result<Option<String>, String> {
                reading_budget.charge(0)?;
                if path == selection.selected.canonical {
                    reading_budget.charge(selection.selected.text.len())?;
                    return Ok(Some(selection.selected.text.clone()));
                }
                if let Some(pin) = files.get(path) {
                    reading_budget.charge(pin.text.len())?;
                    return Ok(Some(pin.text.clone()));
                }
                match std::fs::symlink_metadata(path) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                    Err(e) => return Err(e.to_string()),
                    _ => {}
                }
                let pin = SourcePin::capture(path).map_err(|e| e.to_string())?;
                if pin.canonical != path || !pin.canonical.starts_with(&runtime) {
                    return Err("Rust module mapping is not a proven ordinary package path".into());
                }
                if let Some((workspace, open)) = physical {
                    let relative = pin
                        .canonical
                        .strip_prefix(workspace)
                        .map_err(|_| "Rust module escaped the saved workspace")?
                        .to_str()
                        .ok_or("Rust module path is not UTF-8")?;
                    if open
                        .get(relative)
                        .is_some_and(|source| source.as_bytes() != pin.text.as_bytes())
                    {
                        return Err(format!(
                            "Save the related Rust module {relative} before Debug test"
                        ));
                    }
                }
                total += pin.text.len();
                if total > 8 * 1024 * 1024 || files.len() >= 256 {
                    return Err("Cargo module source proof budget exceeded".into());
                }
                reading_budget.charge(pin.text.len())?;
                let text = pin.text.clone();
                files.insert(pin.canonical.clone(), pin);
                Ok(Some(text))
            };
            let text = read(&root)
                .map_err(anyhow::Error::msg)?
                .context("Cargo test root is unreadable")?;
            hick_plugins::testing::rust_debug_source::builtin_crate_harness(&text)
                .map_err(anyhow::Error::msg)?;
            let graph =
                hick_plugins::testing::rust_debug_source::module_test_graph(&root, &mut read)
                    .map_err(anyhow::Error::msg)?;
            reading_budget.charge(0).map_err(anyhow::Error::msg)?;
            graphs.insert(root, graph);
        }
        let result = Self {
            selection,
            files,
            graphs,
        };
        result.current()?;
        anyhow::ensure!(
            result
                .graphs
                .values()
                .any(|g| g.tests.values().any(|case| result.selected(case))),
            "Selected Rust declaration has no proven active module/target identity; unknown cfg/path/include mappings were not replaced by a sibling"
        );
        Ok(result)
    }
    fn selected(&self, case: &hick_plugins::testing::rust_debug_source::RustModuleTest) -> bool {
        case.file == self.selection.selected.canonical
            && case.line == self.selection.line
            && case.character == self.selection.character
            && case.name.trim_start_matches("r#") == self.selection.declaration.name
    }
    pub(super) fn current(&self) -> Result<()> {
        self.selection.current()?;
        for pin in self.files.values() {
            pin.current()?
        }
        Ok(())
    }
    pub(super) fn ignored(&self) -> bool {
        self.selection.declaration.ignored
    }
}
pub(super) struct MetadataTool {
    path: PathBuf,
    canonical: PathBuf,
    held: same_file::Handle,
    identity: RustExecutableIdentity,
}
impl MetadataTool {
    pub(super) fn from_adapter(name: &str, command: &[String]) -> Result<Self> {
        anyhow::ensure!(
            name == "lldb-dap",
            "The selected Rust adapter does not expose verified read-only source metadata; no suffix-name fallback was used"
        );
        let adapter = Path::new(command.first().context("Rust adapter command is missing")?);
        anyhow::ensure!(
            adapter.is_absolute(),
            "Rust source metadata requires a pinned adapter executable"
        );
        let path = adapter
            .parent()
            .context("Rust adapter has no tool directory")?
            .join(if cfg!(windows) { "lldb.exe" } else { "lldb" });
        let canonical = path
            .canonicalize()
            .context("The selected LLVM adapter has no compatible LLDB metadata reader")?;
        let held = same_file::Handle::from_path(&canonical)?;
        let identity = rust_executable_identity(&canonical)?;
        let result = Self {
            path,
            canonical,
            held,
            identity,
        };
        result.current()?;
        Ok(result)
    }
    fn current(&self) -> Result<()> {
        anyhow::ensure!(
            self.path.canonicalize()? == self.canonical
                && same_file::Handle::from_path(&self.path)? == self.held
                && rust_executable_identity(&self.canonical)? == self.identity,
            "LLDB metadata reader identity changed"
        );
        Ok(())
    }
}

impl Preparation {
    pub(super) fn admit_artifact(&self, artifact: &RustTestArtifact) -> Result<()> {
        self.current()?;
        anyhow::ensure!(
            self.graphs.contains_key(&artifact.source),
            "Compiled Rust target has no captured built-in-harness/module graph; refused before listing"
        );
        Ok(())
    }
    pub(super) fn listed_identity(
        &self,
        artifact: &RustTestArtifact,
        names: &[String],
    ) -> Result<Option<String>> {
        let graph = self
            .graphs
            .get(&artifact.source)
            .context("Compiled Rust target has no captured module graph")?;
        let mut selected = None;
        for name in names {
            let normalized = name
                .split("::")
                .map(|part| part.strip_prefix("r#").unwrap_or(part))
                .collect::<Vec<_>>()
                .join("::");
            let case=graph.tests.get(&normalized).context("A matching compiled test has an unknown cfg/module/source association; no sibling substitution was used")?;
            if self.selected(case) {
                anyhow::ensure!(
                    selected.is_none(),
                    "Selected Rust declaration has ambiguous compiled test identities"
                );
                selected = Some(name.clone())
            }
        }
        Ok(selected)
    }
    pub(super) async fn verify_compiled(
        &self,
        tool: &MetadataTool,
        artifact: &RustTestArtifact,
        name: &str,
        crate_name: &str,
    ) -> Result<()> {
        self.current()?;
        tool.current()?;
        rust_artifact_current(artifact)?;
        let held = same_file::Handle::from_path(&artifact.executable)?;
        let temporary =
            tempfile::tempdir().context("Cannot allocate an isolated metadata reader directory")?;
        let query = serde_json::json!({"base":self.selection.declaration.name,"first":self.selection.declaration.first_line,"last":self.selection.declaration.last_line,"image":artifact.executable.canonicalize()?});
        let script = include_str!("rust_test_source_metadata.py");
        let mut command = tokio::process::Command::new(&tool.path);
        command
            .current_dir(temporary.path())
            .args([
                "--no-lldbinit",
                "--batch",
                "-O",
                "settings set target.load-script-from-symbol-file false",
                "-f",
            ])
            .arg(&artifact.executable)
            .arg("-o")
            .arg(format!("script exec({})", serde_json::to_string(script)?))
            .env("HICKORY_RUST_META_QUERY", serde_json::to_string(&query)?)
            .env_remove("PYTHONPATH")
            .env_remove("PYTHONHOME")
            .env("PYTHONNOUSERSITE", "1")
            .env("PYTHONSAFEPATH", "1");
        let (output, _) =
            rust_test_command_output_with_limit(command, std::time::Duration::from_secs(20))
                .await?;
        self.current()?;
        tool.current()?;
        rust_artifact_current(artifact)?;
        anyhow::ensure!(
            same_file::Handle::from_path(&artifact.executable)? == held,
            "Compiled Rust image identity changed during metadata inspection"
        );
        let text = std::str::from_utf8(&output).context("LLDB source metadata was not UTF-8")?;
        let values = text
            .lines()
            .filter_map(|line| line.strip_prefix("HICKORY_RUST_SOURCE_V1="))
            .collect::<Vec<_>>();
        anyhow::ensure!(
            values.len() == 1,
            "LLDB did not return one bounded compiled source reading"
        );
        let raw: serde_json::Value = serde_json::from_str(values[0])?;
        let evidence = rust_compiled_source_evidence(
            &raw,
            crate_name,
            &self.selection.declaration,
            &self.selection.selected.canonical,
            artifact.source == self.selection.selected.canonical,
        )?;
        let normalized = name
            .split("::")
            .map(|part| part.strip_prefix("r#").unwrap_or(part))
            .collect::<Vec<_>>()
            .join("::");
        anyhow::ensure!(
            evidence.get(&normalized) == Some(&SourceEvidence::Selected),
            "The emitted Rust test does not prove the selected source declaration; Debug refused rather than substituting a sibling"
        );
        Ok(())
    }
}
#[cfg(test)]
mod source_reader_tests {
    use super::*;
    use serde_json::json;
    fn reading(file: &Path, line: u32, name: &str) -> serde_json::Value {
        json!({"version":1,"producer":"lldb-image-source-v1","functions":[{"name":name,"optimized":false,"entries":[{"file":file,"line":line,"column":0,"address":"100"}]}]})
    }
    fn declaration() -> hick_plugins::testing::rust_debug_source::RustTestDeclaration {
        hick_plugins::testing::rust_debug_source::RustTestDeclaration {
            name: "chosen".into(),
            namespace: vec!["cases".into()],
            first_line: 4,
            last_line: 7,
            ignored: false,
        }
    }
    #[test]
    fn file_span_and_exact_namespace_refuse_cfg_disabled_siblings() {
        let root = tempfile::tempdir().unwrap();
        let selected = root.path().join("lib.rs");
        std::fs::write(&selected, "").unwrap();
        let value = reading(&selected, 5, "owned::cases::chosen::h0123456789abcdef");
        assert_eq!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &selected, true)
                .unwrap(),
            "cases::chosen"
        );
        assert!(
            rust_compiled_source_identity(
                &reading(&selected, 13, "owned::cases::chosen::h0123456789abcdef"),
                "owned",
                &declaration(),
                &selected,
                true
            )
            .is_err()
        );
        assert!(
            rust_compiled_source_identity(
                &reading(&selected, 5, "owned::other::chosen::h0123456789abcdef"),
                "owned",
                &declaration(),
                &selected,
                true
            )
            .is_err()
        );
    }
    #[test]
    fn a_same_basename_is_not_the_selected_physical_source() {
        let root = tempfile::tempdir().unwrap();
        let a = root.path().join("one/lib.rs");
        let b = root.path().join("two/lib.rs");
        std::fs::create_dir_all(a.parent().unwrap()).unwrap();
        std::fs::create_dir_all(b.parent().unwrap()).unwrap();
        std::fs::write(&a, b"same").unwrap();
        std::fs::write(&b, b"same").unwrap();
        assert!(
            rust_compiled_source_identity(
                &reading(&b, 5, "owned::cases::chosen::h0123456789abcdef"),
                "owned",
                &declaration(),
                &a,
                true
            )
            .is_err()
        );
    }
    #[test]
    fn external_module_mapping_requires_one_emitted_full_identity() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("module.rs");
        std::fs::write(&source, "").unwrap();
        let mut value = reading(
            &source,
            5,
            "owned::ordinary::cases::chosen::h0123456789abcdef",
        );
        assert_eq!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &source, false).unwrap(),
            "ordinary::cases::chosen"
        );
        let row =
            reading(&source, 5, "owned::other::cases::chosen::h0123456789abcdef")["functions"][0]
                .clone();
        value["functions"].as_array_mut().unwrap().push(row);
        assert!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &source, false).is_err()
        );
    }
    #[test]
    fn unknown_metadata_effects_and_relative_paths_never_become_proof() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("lib.rs");
        std::fs::write(&source, "").unwrap();
        let mut value = reading(&source, 5, "owned::cases::chosen::h0123456789abcdef");
        value["command"] = json!("run sibling");
        assert!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &source, true).is_err()
        );
        let value = reading(
            Path::new("lib.rs"),
            5,
            "owned::cases::chosen::h0123456789abcdef",
        );
        assert!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &source, true).is_err()
        );
    }
}

#[cfg(test)]
mod captured_source_tests {
    use super::*;
    #[test]
    fn repeated_target_readings_share_byte_count_and_time_limits() {
        let mut bytes = SourceReadBudget::new();
        assert!(bytes.charge(32 * 1024 * 1024 + 1).is_err());
        let mut count = SourceReadBudget::new();
        for _ in 0..1024 {
            count.charge(0).unwrap()
        }
        assert!(count.charge(0).is_err());
        let mut time = SourceReadBudget::new();
        time.started = std::time::Instant::now() - std::time::Duration::from_secs(11);
        assert!(time.charge(0).is_err());
    }

    #[test]
    fn selected_graph_never_admits_a_cfg_disabled_sibling() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        let path = root.path().join("src/lib.rs");
        let text = "#[cfg(any())]mod inactive {\n #[test] fn chosen(){}\n}\nmod active {\n #[test] fn chosen(){}\n}\n";
        std::fs::write(&path, text).unwrap();
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname='proof'\nversion='0.1.0'\n",
        )
        .unwrap();
        let candidates = hick_plugins::testing::candidates::rust("rust", text)
            .unwrap()
            .candidates;
        let metadata = serde_json::json!({"packages":[{"manifest_path":root.path().join("Cargo.toml"),"targets":[{"test":true,"src_path":path}]}]});
        let hash = crate::serve::source_tests::digest(text.as_bytes());
        let inactive = &candidates[0];
        let selection = Selection::capture(
            &path,
            inactive.line,
            inactive.character,
            &hash,
            &inactive.name,
            root.path(),
        )
        .unwrap();
        assert!(Preparation::capture(selection, &metadata, root.path(), None).is_err());
        let active = &candidates[1];
        let selection = Selection::capture(
            &path,
            active.line,
            active.character,
            &hash,
            &active.name,
            root.path(),
        )
        .unwrap();
        let proof = Preparation::capture(selection, &metadata, root.path(), None).unwrap();
        let artifact = RustTestArtifact {
            executable: path.clone(),
            source: path.canonicalize().unwrap(),
            identity: rust_executable_identity(&path).unwrap(),
        };
        proof.admit_artifact(&artifact).unwrap();
        let foreign_artifact = RustTestArtifact {
            source: root.path().join("unknown-root.rs"),
            executable: artifact.executable.clone(),
            identity: artifact.identity.clone(),
        };
        assert!(
            proof
                .admit_artifact(&foreign_artifact)
                .unwrap_err()
                .to_string()
                .contains("refused before listing")
        );
        assert_eq!(
            proof
                .listed_identity(&artifact, &["active::chosen".into()])
                .unwrap(),
            Some("active::chosen".into())
        );
        assert!(
            proof
                .listed_identity(&artifact, &["inactive::chosen".into()])
                .is_err()
        );
        let replacement = path.with_extension("replacement");
        std::fs::write(&replacement, text).unwrap();
        std::fs::rename(replacement, &path).unwrap();
        assert!(proof.current().is_err());
    }
}
