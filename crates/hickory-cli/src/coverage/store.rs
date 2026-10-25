use super::*;
use sha2::{Digest, Sha256};
use std::path::Component;

pub fn safe_path(root: &Path, rel: &str) -> Result<PathBuf> {
    ensure!(
        !Path::new(rel).is_absolute()
            && Path::new(rel)
                .components()
                .all(|p| matches!(p, Component::Normal(_) | Component::CurDir)),
        "Path must be inside the workspace"
    );
    let path = root.join(rel).canonicalize()?;
    ensure!(
        path.starts_with(root.canonicalize()?),
        "Path escapes workspace through a symlink"
    );
    Ok(path)
}
pub(super) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
pub(super) fn fresh_id() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("OS randomness");
    hex::encode(bytes)
}
pub(super) fn dir(root: &Path) -> Result<PathBuf> {
    let mut dir = root.to_path_buf();
    for part in [".hick-cache", "coverage"] {
        dir.push(part);
        if !dir.exists() {
            std::fs::create_dir(&dir)?;
        }
        ensure!(
            !std::fs::symlink_metadata(&dir)?.file_type().is_symlink(),
            "Coverage storage cannot be a symlink"
        );
    }
    Ok(dir)
}
fn path(root: &Path, id: &str) -> Result<PathBuf> {
    ensure!(
        id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid run id"
    );
    Ok(dir(root)?.join(format!("{id}.json")))
}
pub(super) fn save(root: &Path, record: &Record) -> Result<()> {
    let path = path(root, &record.id)?;
    let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    use std::io::Write;
    temp.write_all(&serde_json::to_vec(record)?)?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
pub(super) fn load(root: &Path, id: &str) -> Result<Record> {
    let path = path(root, id)?;
    ensure!(
        !std::fs::symlink_metadata(&path)?.file_type().is_symlink(),
        "Run record cannot be a symlink"
    );
    ensure!(
        std::fs::metadata(&path)?.len() <= 64 * 1024 * 1024,
        "Run record too large"
    );
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

pub fn snapshot(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    let walk = ignore::WalkBuilder::new(root)
        .hidden(false)
        .follow_links(false)
        .filter_entry(|e| {
            !e.file_name().to_str().is_some_and(|n| {
                [
                    ".git",
                    ".hick-cache",
                    "node_modules",
                    "target",
                    "bin",
                    "obj",
                    "dist",
                    ".venv",
                    "venv",
                    "__pycache__",
                    ".pytest_cache",
                    ".dev",
                    ".dex",
                    ".code-review-graph",
                ]
                .contains(&n)
            })
        })
        .build();
    for entry in walk {
        let entry = entry?;
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with(".coverage")
            || name.ends_with(".profraw")
            || name.ends_with(".profdata")
            || name == "coverage.json"
            || name == "coverage.xml"
            || name == "lcov.info"
        {
            continue;
        }
        // Hash source/config/data, including hidden configuration, but not secrets or outputs.
        if name == ".env" || name.starts_with(".env.") || name.ends_with(".log") {
            continue;
        }
        ensure!(
            files.len() < 100_000,
            "Source snapshot exceeds 100,000 files"
        );
        if entry.metadata()?.len() > 4 * 1024 * 1024 {
            continue;
        }
        files.insert(
            relative(root, path),
            format!("{:x}", Sha256::digest(std::fs::read(path)?)),
        );
    }
    Ok(files)
}
pub(super) fn new_record(root: &Path, project: &Path, request: &Request) -> Result<Record> {
    let metadata = config(project)?;
    let group = request.group.clone();
    if let Some(name) = &group {
        ensure!(
            metadata.groups.contains_key(name),
            "Unknown test group {name}"
        );
    }
    let labels: Vec<_> = request
        .labels
        .iter()
        .cloned()
        .chain(group.iter().cloned())
        .chain(
            group
                .as_ref()
                .and_then(|g| metadata.groups.get(g))
                .into_iter()
                .flat_map(|g| g.labels.iter().cloned()),
        )
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(Record {
        id: fresh_id(),
        mode: if request.action == "manual-start" {
            "manual"
        } else if request.action == "import" {
            "import"
        } else {
            "automated"
        }
        .into(),
        status: "running".into(),
        started_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis(),
        project: relative(root, project),
        language: request.language.clone(),
        group,
        metadata,
        labels,
        identity: Identity {
            collector: "pending".into(),
            version: "unknown".into(),
            configuration: String::new(),
            scope: relative(root, project),
            source_hashes: snapshot(root)?,
        },
        argv: vec![],
        tests: vec![],
        evidence: None,
        output: String::new(),
        error: None,
        exit_code: None,
        source_changed_during_run: false,
        manual_verdict: None,
        notes: request.notes.clone(),
        checkpoints: vec![],
        baseline: None,
        profile: request.profile.clone(),
        parent: None,
        mappings: BTreeMap::new(),
    })
}
fn summary(root: &Path, mut record: Record, include_raw: bool) -> Result<Value> {
    let current = record.identity.source_hashes.iter().any(|(p, hash)| {
        safe_path(root, p)
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
            .as_ref()
            != Some(hash)
    });
    let unknown = record.identity.source_hashes.is_empty()
        || record.evidence.as_ref().is_some_and(|e| {
            e.report
                .files
                .keys()
                .any(|p| !record.identity.source_hashes.contains_key(p))
        });
    record.baseline = None;
    if !include_raw {
        if let Some(e) = &mut record.evidence {
            e.report.raw.clear();
            e.report.files.clear();
            e.identity.source_hashes.clear();
            e.tests.clear();
        }
        record.identity.source_hashes.clear();
        record.mappings.clear();
        record.tests.clear();
        record.metadata = Config::default();
        record.output.clear();
    }
    let mut value = json!(record);
    value["source_state"] = json!(if unknown {
        "unknown"
    } else if current || record.source_changed_during_run {
        "historical"
    } else {
        "current"
    });
    if record.status == "running" && !super::runner::active(root, &record.id) {
        value["status"] = json!("interrupted");
    }
    Ok(value)
}
pub(super) fn view(root: &Path, id: &str) -> Result<Value> {
    summary(root, load(root, id)?, true)
}
pub(super) fn list(root: &Path) -> Result<Value> {
    let mut records = Vec::new();
    for e in std::fs::read_dir(dir(root)?)? {
        let path = e?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
            records.push(load(root, id)?);
        }
    }
    records.sort_by_key(|r| std::cmp::Reverse(r.started_ms));
    let total = records.len();
    let runs: Result<Vec<_>> = records
        .into_iter()
        .take(100)
        .map(|r| summary(root, r, false))
        .collect();
    Ok(json!({"runs":runs?,"total":total,"limit":100}))
}
