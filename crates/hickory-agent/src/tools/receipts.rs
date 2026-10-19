use super::*;

/// One thing a tool showed the model: a file, at a content hash and (when
/// the file is in a git repository) a commit, over a 1-based inclusive line
/// range. Derived by the tool, never declared by the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRead {
    /// The path as the session knows it — the document's name for the
    /// primary, the upstream's path, an output's path, or the path given to
    /// `read_file` resolved relative to the primary document.
    pub path: String,
    /// Exact host-resolved source path when available. `path` above remains the
    /// historical citation locator and may be relative to a different tool root.
    pub source_path: Option<std::path::PathBuf>,
    /// `git rev-parse HEAD` of the repository containing the file, if any.
    /// The working tree may differ from HEAD; `sha256` is the truth about
    /// the bytes, `commit` is where to look for them later.
    pub commit: Option<String>,
    /// SHA-256 of the whole file's bytes as read.
    pub sha256: String,
    /// First line shown, 1-based.
    pub first_line: usize,
    /// Last line shown, 1-based, inclusive.
    pub last_line: usize,
    /// Exact returned source bytes, retained for historical inspection.
    pub content: String,
    /// Half-open UTF-8 byte range in the exact snapshot hashed above.
    pub byte_span: (usize, usize),
    /// Uniqueness witnessed in the original full snapshot, including overlapping matches.
    pub unique: bool,
}

/// The lines an edit left in a file, 1-based inclusive, with their content
/// hashes — the anchors by which a later reader finds them again after the
/// file has moved on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrote {
    pub file: String,
    pub first_line: usize,
    pub last_line: usize,
    /// SHA-256 hashline hashes of those lines, in order.
    pub hashes: Vec<String>,
    /// SHA-256 per written line; short anchors are never attribution evidence.
    pub fingerprints: Vec<String>,
}

/// SHA-256 of `bytes`, lowercase hex.
pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// `git rev-parse HEAD` for the repository containing `path`, if there is
/// one and git is installed. Anything else is `None`: outside a repository
/// there is no commit to name, and the read is still recorded by its hash.
pub(super) fn head_commit_for(path: &Path) -> Option<String> {
    let dir = if path.is_dir() {
        path
    } else {
        match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        }
    };
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

/// The lines of `new` that differ from `old`: the first and last (1-based,
/// inclusive) line indices into `new` outside the common prefix and suffix,
/// with their hashes. `None` when nothing changed or the change was a pure
/// deletion (no lines of `new` were written).
pub(super) fn changed_lines(old: &str, new: &str, file: &str) -> Option<Wrote> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let mut prefix = 0;
    while prefix < a.len() && prefix < b.len() && a[prefix] == b[prefix] {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < a.len() - prefix
        && suffix < b.len() - prefix
        && a[a.len() - 1 - suffix] == b[b.len() - 1 - suffix]
    {
        suffix += 1;
    }
    let last = b.len().checked_sub(suffix)?;
    if last <= prefix {
        return None;
    }
    Some(Wrote {
        file: file.to_string(),
        first_line: prefix + 1,
        last_line: last,
        fingerprints: b[prefix..last]
            .iter()
            .map(|line| hashline::line_fingerprint(line))
            .collect(),
        hashes: b[prefix..last]
            .iter()
            .map(|l| hashline::line_hash(l))
            .collect(),
    })
}

impl super::EditSession {
    /// A context record for showing the whole of `source`, known as `path`.
    pub(super) fn read_of(&self, path: &std::path::Path, name: &str, source: &str) -> ContextRead {
        let lines = source.lines().count().max(1);
        ContextRead {
            path: name.to_string(),
            source_path: Some(path.to_path_buf()),
            commit: super::head_commit_for(path),
            sha256: super::sha256_hex(source.as_bytes()),
            first_line: 1,
            last_line: lines,
            content: source.to_string(),
            byte_span: (0, source.len()),
            unique: !source.is_empty(),
        }
    }
}
