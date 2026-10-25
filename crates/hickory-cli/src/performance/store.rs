use super::*;
use sha2::{Digest, Sha256};
pub(super) fn dir(root: &Path) -> Result<PathBuf> {
    let mut path = root.to_path_buf();
    for part in [".hick-cache", "profiles"] {
        path.push(part);
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }
        ensure!(
            !std::fs::symlink_metadata(&path)?.file_type().is_symlink(),
            "Profile storage cannot be a symlink"
        );
    }
    Ok(path)
}
fn path(root: &Path, id: &str) -> Result<PathBuf> {
    ensure!(
        id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid profile id"
    );
    Ok(dir(root)?.join(format!("{id}.json")))
}
pub(super) fn save(root: &Path, run: &Record) -> Result<()> {
    use std::io::Write;
    let path = path(root, &run.id)?;
    let mut file = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    file.write_all(&serde_json::to_vec(run)?)?;
    file.persist(path).map_err(|e| e.error)?;
    Ok(())
}
pub(super) fn load(root: &Path, id: &str) -> Result<Record> {
    let path = path(root, id)?;
    ensure!(
        !std::fs::symlink_metadata(&path)?.file_type().is_symlink(),
        "Profile record cannot be a symlink"
    );
    ensure!(
        std::fs::metadata(&path)?.len() <= 80 * 1024 * 1024,
        "Profile record too large"
    );
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}
pub(super) fn source_state(root: &Path, run: &Record) -> &'static str {
    if run.identity.source_hashes.is_empty() {
        return "unknown";
    }
    if run.source_changed
        || run.identity.source_hashes.iter().any(|(path, hash)| {
            safe_path(root, path)
                .ok()
                .and_then(|p| std::fs::read(p).ok())
                .map(|b| format!("{:x}", Sha256::digest(b)))
                .as_ref()
                != Some(hash)
        })
    {
        return "historical";
    }
    "current"
}
pub(super) fn view(root: &Path, id: &str, full: bool) -> Result<Value> {
    let mut record = load(root, id)?;
    let state = source_state(root, &record);
    if !full {
        record.sources.clear();
        record.mappings.clear();
        record.identity.source_hashes.clear();
        record.output.clear();
        if let Some(report) = &mut record.report {
            report.raw.clear();
            for t in &mut report.threads {
                t.rows.clear();
            }
        }
    }
    let mut value = json!(record);
    value["source_state"] = json!(state);
    if record.status == "running" && !browser::active(root, id) && !native::active(root, id) {
        value["status"] = json!("interrupted");
    }
    Ok(value)
}
pub(super) fn list(root: &Path) -> Result<Value> {
    let mut records = Vec::new();
    for e in std::fs::read_dir(dir(root)?)? {
        let p = e?.path();
        if p.extension().and_then(|p| p.to_str()) == Some("json") {
            if let Some(id) = p.file_stem().and_then(|p| p.to_str()) {
                records.push(load(root, id)?);
            }
        }
    }
    records.sort_by_key(|r| std::cmp::Reverse(r.started_ms));
    let total = records.len();
    let runs: Result<Vec<_>> = records
        .into_iter()
        .take(100)
        .map(|r| view(root, &r.id, false))
        .collect();
    Ok(json!({"runs":runs?,"total":total,"limit":100}))
}
pub(super) fn finish(root: &Path, record: &mut Record, result: Result<()>) {
    if let Err(error) = result {
        record.status = "unavailable".into();
        record.error = Some(format!("{error:#}"));
    }
    record.source_changed = snapshot(root)
        .map(|s| s != record.identity.source_hashes)
        .unwrap_or(true);
    if let Err(e) = save(root, record) {
        log::error!("Saving profile {}: {e}", record.id);
    }
}
