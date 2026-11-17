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
        let metadata=std::fs::symlink_metadata(&dir)?;
        ensure!(
            metadata.is_dir()&&!metadata.file_type().is_symlink(),
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
    ensure!(record.workspace_identity_kind.as_deref()!=Some("physical-root-v1")||record.workspace_id==crate::serve::resource_inputs::RootIdentity::fingerprint(root).ok(),"Original collector workspace is unavailable; result remains retained in this process");
    ensure!(record.id.len()==32&&record.id.bytes().all(|byte|byte.is_ascii_hexdigit()),"Invalid run id");
    #[cfg(unix)] {
      use std::{ffi::CString,os::fd::{AsRawFd,FromRawFd},io::Write};
      // Every child component and rename is relative to an already-open parent.
      // Replacing the public root or cache path cannot redirect this write.
      fn directory(parent:&std::fs::File,name:&str)->Result<std::fs::File>{
        let name=CString::new(name)?;
        let created=unsafe{libc::mkdirat(parent.as_raw_fd(),name.as_ptr(),0o700)};
        if created!=0 {let error=std::io::Error::last_os_error();ensure!(error.kind()==std::io::ErrorKind::AlreadyExists,"Cannot create collector storage: {error}");}
        let fd=unsafe{libc::openat(parent.as_raw_fd(),name.as_ptr(),libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC)};
        ensure!(fd>=0,"Cannot open held collector storage: {}",std::io::Error::last_os_error());Ok(unsafe{std::fs::File::from_raw_fd(fd)})
      }
      let scope=crate::serve::resource_inputs::RootIdentity::bind(root);
      let held=std::fs::File::open(root)?;
      ensure!(same_file::Handle::from_file(held.try_clone()?)?==same_file::Handle::from_path(root)?,"Collector root changed");
      scope.check(root).map_err(anyhow::Error::msg)?;
      ensure!(record.workspace_identity_kind.as_deref()!=Some("physical-root-v1")||record.workspace_id==crate::serve::resource_inputs::RootIdentity::fingerprint(root).ok(),"Original collector root changed");
      static STORES:std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String,std::sync::Arc<std::fs::File>>>>=std::sync::OnceLock::new();
      let stores=STORES.get_or_init(||std::sync::Mutex::new(std::collections::HashMap::new()));
      let mut stores=stores.lock().unwrap();
      let identity=crate::serve::resource_inputs::RootIdentity::fingerprint(root).map_err(anyhow::Error::msg)?;
      let records=if let Some(records)=stores.get(&identity){records.clone()}else{let cache=directory(&held,".hick-cache")?;let records=std::sync::Arc::new(directory(&cache,"coverage")?);stores.insert(identity.clone(),records.clone());records};
      drop(stores);
      ensure!(same_file::Handle::from_file(records.try_clone()?)?==same_file::Handle::from_path(root.join(".hick-cache/coverage"))?,"Collector record storage changed; outcome remains retained in this process");
      let temporary=CString::new(format!(".{}.{}.tmp",record.id,fresh_id()))?;
      let destination=CString::new(format!("{}.json",record.id))?;
      let fd=unsafe{libc::openat(records.as_raw_fd(),temporary.as_ptr(),libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL|libc::O_NOFOLLOW|libc::O_CLOEXEC,0o600)};
      ensure!(fd>=0,"Cannot reserve collector record: {}",std::io::Error::last_os_error());
      let mut file=unsafe{std::fs::File::from_raw_fd(fd)};
      let result=(||->Result<()>{file.write_all(&serde_json::to_vec(record)?)?;file.sync_all()?;ensure!(unsafe{libc::renameat(records.as_raw_fd(),temporary.as_ptr(),records.as_raw_fd(),destination.as_ptr())}==0,"Cannot publish collector record: {}",std::io::Error::last_os_error());Ok(())})();
      if result.is_err(){unsafe{libc::unlinkat(records.as_raw_fd(),temporary.as_ptr(),0)};}
      result
    }
    #[cfg(not(unix))] {bail!("Held-directory collector persistence is unavailable on this host")}
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
    let workspace=crate::serve::resource_inputs::RootIdentity::fingerprint(root).ok();
    Ok(Record {
        workspace_id: workspace.clone(),
        workspace_identity_kind: workspace.map(|_|"physical-root-v1".into()),
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
    let origin_current=record.workspace_identity_kind.as_deref()==Some("physical-root-v1")&&record.workspace_id==crate::serve::resource_inputs::RootIdentity::fingerprint(root).ok();
    let current = record.identity.source_hashes.iter().any(|(p, hash)| {
        safe_path(root, p)
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
            .as_ref()
            != Some(hash)
    });
    let unknown = !origin_current || record.identity.source_hashes.is_empty()
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
/// Reading and follow-up selection share the same retained terminal authority.
/// A completed result can be visible before its guarded durable save finishes.
pub(super) fn selected(root:&Path,id:&str)->Result<Record>{let _=path(root,id)?;super::runner::retained(root,id).map(Ok).unwrap_or_else(||load(root,id))}
pub(super) fn view(root: &Path, id: &str) -> Result<Value> {
    summary(root, selected(root,id)?, true)
}
pub(super) fn list(root: &Path) -> Result<Value> {
    let mut records = super::runner::retained_records(root);
    let mut directory=root.to_path_buf();let mut exists=true;
    for component in [".hick-cache","coverage"] {directory.push(component);match std::fs::symlink_metadata(&directory){Ok(metadata)=>ensure!(metadata.is_dir()&&!metadata.file_type().is_symlink(),"Coverage storage cannot be a symlink"),Err(error)if error.kind()==std::io::ErrorKind::NotFound=>{exists=false;break},Err(error)=>return Err(error.into())}}
    let entries=if exists{Some(std::fs::read_dir(directory)?)}else{None};
    for e in entries.into_iter().flatten() {
        let path = e?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
            if !records.iter().any(|record|record.id==id){records.push(load(root, id)?);}
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
