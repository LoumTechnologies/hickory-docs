//! Exact revision input adapters. Read-only snapshots; no checkout, index
//! mutation, scratch tree or ancestor lookup is part of source verification.
use anyhow::{Context, Result, bail};
use hick_lang::{EntryKind, ProjectEntry, ProjectInventory, ProjectTree};
use std::{path::Path, process::Command};

pub fn directory(root: &Path, inventory: Option<&ProjectInventory>) -> Result<ProjectTree> {
    let mut tree = ProjectTree::default();
    read_directory(root, root, inventory, &mut tree)?;
    Ok(tree)
}
fn read_directory(
    root: &Path,
    dir: &Path,
    inventory: Option<&ProjectInventory>,
    tree: &mut ProjectTree,
) -> Result<()> {
    for item in std::fs::read_dir(dir).with_context(|| {
        format!(
            "Cannot inventory {}; make it readable and pause other writers before retrying",
            dir.display()
        )
    })? {
        let item = item?;
        let full = item.path();
        let path = full
            .strip_prefix(root)?
            .to_str()
            .context("project contains a non-UTF-8 path")?
            .replace('\\', "/");
        let metadata = std::fs::symlink_metadata(&full).with_context(|| {
            format!(
                "Cannot capture {}; restore access and pause other writers before retrying",
                full.display()
            )
        })?;
        if metadata.is_dir() {
            // Metadata and explicitly excluded dependency/build directories
            // are not read. No gitignore or extension heuristic is consulted.
            if inventory.is_some_and(|i| !i.contains(&format!("{path}/"))) {
                continue;
            }
            read_directory(root, &full, inventory, tree)?;
        } else {
            if inventory.is_some_and(|i| !i.contains(&path)) {
                continue;
            }
            let (kind, bytes) = if metadata.file_type().is_symlink() {
                let target = std::fs::read_link(&full)
                    .with_context(|| format!("Cannot read symlink {}", full.display()))?;
                (
                    EntryKind::Symlink,
                    target
                        .to_str()
                        .context("non-UTF-8 symlink target")?
                        .as_bytes()
                        .to_vec(),
                )
            } else if metadata.is_file() {
                (EntryKind::File, std::fs::read(&full).with_context(|| format!("Cannot capture bytes of {}; make it readable and pause other writers before retrying", full.display()))?)
            } else {
                bail!("unsupported project entry kind: {path}");
            };
            #[cfg(unix)]
            let executable = {
                use std::os::unix::fs::PermissionsExt;
                kind == EntryKind::File && metadata.permissions().mode() & 0o111 != 0
            };
            #[cfg(not(unix))]
            let executable = false;
            tree.insert(
                &path,
                ProjectEntry {
                    bytes: bytes.into(),
                    kind,
                    executable,
                    origins: vec![],
                },
            )
            .map_err(anyhow::Error::msg)?;
        }
    }
    Ok(())
}
fn command(root: &Path, executable: &str, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new(executable)
        .current_dir(root)
        .args(args)
        .output()
        .with_context(|| format!("read {executable} revision"))?;
    if !output.status.success() {
        bail!(
            "{executable} revision read failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.stdout)
}

/// Freeze a commit or the complete stage-zero index. Unmerged index entries
/// fail; working-tree Markdown and outputs are never borrowed.
pub fn git(root: &Path, revision: &str) -> Result<ProjectTree> {
    let prefix = String::from_utf8(command(root, "git", &["rev-parse", "--show-prefix"])?)?
        .trim_end_matches(['\r', '\n'])
        .to_owned();
    let listing = if revision == ":index" {
        command(root, "git", &["ls-files", "--stage", "-z", "--full-name"])?
    } else {
        let revision = String::from_utf8(command(
            root,
            "git",
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{revision}^{{tree}}"),
            ],
        )?)?
        .trim()
        .to_owned();
        command(
            root,
            "git",
            &["ls-tree", "-r", "-z", "--full-tree", &revision],
        )?
    };
    let mut selected = vec![];
    for record in listing.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let (info, path) = record.split_at(
            record
                .iter()
                .position(|b| *b == b'\t')
                .context("malformed git entry")?,
        );
        let info = std::str::from_utf8(info)?;
        let path = std::str::from_utf8(&path[1..])?;
        let Some(path) = path.strip_prefix(&prefix) else {
            continue;
        };
        let fields: Vec<_> = info.split_whitespace().collect();
        let (mode, oid) = if revision == ":index" {
            if fields.len() != 3 || fields[2] != "0" {
                bail!("unmerged Git index entry: {path}");
            }
            (fields[0], fields[1])
        } else {
            if fields.len() != 3 || fields[1] != "blob" {
                bail!("unsupported Git entry: {path}");
            }
            (fields[0], fields[2])
        };
        let (kind, executable) = match mode {
            "100644" => (EntryKind::File, false),
            "100755" => (EntryKind::File, true),
            "120000" => (EntryKind::Symlink, false),
            _ => bail!("unsupported Git mode {mode} for {path}"),
        };
        selected.push((path.to_owned(), kind, executable, oid.to_owned()));
    }
    git_blobs(root, selected)
}

fn git_blobs(root: &Path, selected: Vec<(String, EntryKind, bool, String)>) -> Result<ProjectTree> {
    use std::{io::Write, process::Stdio};
    let mut child = Command::new("git")
        .current_dir(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().context("Git batch input missing")?;
    let names = selected
        .iter()
        .map(|(_, _, _, oid)| format!("{oid}\n"))
        .collect::<String>();
    let writer = std::thread::spawn(move || stdin.write_all(names.as_bytes()));
    let output = child.wait_with_output()?;
    writer
        .join()
        .map_err(|_| anyhow::anyhow!("Git batch input failed"))??;
    if !output.status.success() {
        bail!(
            "Git blob read failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut remaining = output.stdout.as_slice();
    let mut tree = ProjectTree::default();
    for (path, kind, executable, oid) in selected {
        let end = remaining
            .iter()
            .position(|b| *b == b'\n')
            .context("Git batch header missing")?;
        let header = std::str::from_utf8(&remaining[..end])?;
        let parts: Vec<_> = header.split_whitespace().collect();
        anyhow::ensure!(
            parts.len() == 3 && parts[0] == oid && parts[1] == "blob",
            "Unexpected Git batch object: {header}"
        );
        let size: usize = parts[2].parse()?;
        remaining = &remaining[end + 1..];
        anyhow::ensure!(
            remaining.len() > size && remaining[size] == b'\n',
            "Truncated Git blob: {path}"
        );
        let bytes = remaining[..size].to_vec();
        remaining = &remaining[size + 1..];
        tree.insert(
            &path,
            ProjectEntry {
                bytes: bytes.into(),
                kind,
                executable,
                origins: vec![],
            },
        )
        .map_err(anyhow::Error::msg)?;
    }
    Ok(tree)
}

pub fn mercurial(root: &Path, revision: &str) -> Result<ProjectTree> {
    let repository =
        std::path::PathBuf::from(String::from_utf8(command(root, "hg", &["root"])?)?.trim());
    let local = root
        .canonicalize()?
        .strip_prefix(repository.canonicalize()?)?
        .to_str()
        .context("non-UTF-8 Mercurial scope")?
        .replace('\\', "/");
    let prefix = if local.is_empty() {
        String::new()
    } else {
        format!("{local}/")
    };
    // Resolve moving names once so manifest and every cat read one revision.
    let revision = String::from_utf8(command(
        root,
        "hg",
        &["log", "-r", revision, "--template", "{node}"],
    )?)?;
    if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("Mercurial revision must resolve to exactly one changeset");
    }
    let listing = command(
        root,
        "hg",
        &[
            "manifest",
            "-r",
            &revision,
            "--template",
            "{path}\\0{flags}\\0",
        ],
    )?;
    let fields: Vec<_> = listing.split(|b| *b == 0).collect();
    let mut tree = ProjectTree::default();
    for pair in fields[..fields.len().saturating_sub(1)].chunks(2) {
        if pair.len() != 2 {
            bail!("malformed Mercurial manifest");
        }
        let original_path = std::str::from_utf8(pair[0])?;
        let Some(path) = original_path.strip_prefix(&prefix) else {
            continue;
        };
        let flags = std::str::from_utf8(pair[1])?;
        let (kind, executable) = match flags {
            "" => (EntryKind::File, false),
            "x" => (EntryKind::File, true),
            "l" => (EntryKind::Symlink, false),
            _ => bail!("unsupported Mercurial flags for {path}: {flags}"),
        };
        let bytes = command(
            &repository,
            "hg",
            &["cat", "-r", &revision, "--", original_path],
        )?;
        tree.insert(
            path,
            ProjectEntry {
                bytes: bytes.into(),
                kind,
                executable,
                origins: vec![],
            },
        )
        .map_err(anyhow::Error::msg)?;
    }
    Ok(tree)
}

/// Decode already-captured persisted CRDT updates, without opening a room or
/// reading moving state. The caller supplies the exact immutable update bytes.
pub fn persisted_crdt(entries: &[(String, Vec<u8>)]) -> Result<ProjectTree> {
    use yrs::updates::decoder::Decode;
    use yrs::{GetString, Transact, Update};
    let mut tree = ProjectTree::default();
    for (path, update) in entries {
        if tree.get(path).is_some() {
            bail!("duplicate CRDT snapshot path: {path}");
        }
        let doc = yrs::Doc::new();
        let text = doc.get_or_insert_text("source");
        {
            let mut txn = doc.transact_mut();
            txn.apply_update(Update::decode_v1(update)?)?;
        }
        let source = text.get_string(&doc.transact());
        tree.insert(path, ProjectEntry::file(source.into_bytes()))
            .map_err(anyhow::Error::msg)?;
    }
    Ok(tree)
}
