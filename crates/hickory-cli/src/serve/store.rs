//! The local [`DocStore`]: a room's durable state is the file itself.
//!
//! The hosted server keeps a room's text in a Postgres row and its encoded
//! CRDT state in a column beside it. Here the text *is* the `.hick` file on
//! disk — which is the point of the whole mode: a collaborator's keystroke
//! lands in the host's working tree, where their editor, their `git diff`, and
//! `hick test` all see it. The CRDT state, which no human reads, goes in a
//! sidecar under `.hick-cache/` (already gitignored by `hick init`).
//!
//! See `docs/specs/freeform/local-collaboration.md`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use hickory_collab::{DocKey, DocStore};

/// Every `.hick` document under the served root, keyed by a stable id.
///
/// Ids are derived from the path rather than minted, so a share link still
/// resolves after the host restarts `hick serve` — a link that dies when
/// the laptop sleeps is not a link.
pub struct DocIndex {
    root: PathBuf,
    /// id → path relative to the root.
    ///
    /// Behind a lock because a document can be created while the app is
    /// running: the scan happens once at startup, and a new file that nothing
    /// knows about is a file nothing can open.
    by_id: std::sync::RwLock<HashMap<String, String>>,
}

impl DocIndex {
    /// Scan `root` for documents. A single file may be served directly, in
    /// which case its parent directory is the root.
    pub fn scan(root: &Path) -> Result<Self> {
        let mut by_id = HashMap::new();
        for path in crate::expand_docs(root)? {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            by_id.insert(doc_id(&rel), rel);
        }
        Ok(Self {
            root: root.to_path_buf(),
            by_id: std::sync::RwLock::new(by_id),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Relative path for a document id.
    pub fn path_of(&self, id: &str) -> Option<String> {
        self.by_id.read().ok()?.get(id).cloned()
    }

    /// Remember a document created while the server was running, and return
    /// its id.
    pub fn add(&self, rel: &str) -> String {
        let id = doc_id(rel);
        if let Ok(mut map) = self.by_id.write() {
            map.insert(id.clone(), rel.to_string());
        }
        id
    }

    /// Absolute path for a document id.
    pub fn absolute(&self, id: &str) -> Option<PathBuf> {
        self.path_of(id).map(|rel| self.root.join(rel))
    }

    /// Id for a path relative to the root, whether or not it was scanned —
    /// provenance can name an upstream document the scan did not reach.
    pub fn id_for_path(&self, rel: &str) -> String {
        doc_id(rel)
    }

    /// Every document, as `(id, relative path)`, sorted by path.
    pub fn entries(&self) -> Vec<(String, String)> {
        let Ok(map) = self.by_id.read() else {
            return Vec::new();
        };
        let mut out: Vec<(String, String)> = map
            .iter()
            .map(|(id, rel)| (id.clone(), rel.clone()))
            .collect();
        out.sort_by(|a, b| a.1.cmp(&b.1));
        out
    }

    /// The single document, when exactly one was found. `hick serve
    /// doc.hick` opens straight into it rather than a one-item list.
    pub fn sole(&self) -> Option<(String, String)> {
        match self.entries().as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        }
    }
}

/// Stable, short, filesystem-safe id for a relative document path.
///
/// FNV-1a in hex: deterministic across machines and restarts, no state to
/// keep, and short enough to read in a URL.
pub fn doc_id(rel_path: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in rel_path.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Files on disk, standing in for Postgres.
pub struct FileDocStore {
    index: Arc<DocIndex>,
}

impl FileDocStore {
    pub fn new(index: Arc<DocIndex>) -> Arc<Self> {
        Arc::new(Self { index })
    }

    /// Where the encoded CRDT state for a document lives.
    fn crdt_path(&self, key: &DocKey) -> PathBuf {
        self.index
            .root()
            .join(".hick-cache")
            .join("crdt")
            .join(format!("{key}.bin"))
    }

    fn source_path(&self, key: &DocKey) -> Result<PathBuf> {
        self.index
            .absolute(key)
            .with_context(|| format!("no document with id {key} under the served directory"))
    }
}

#[async_trait]
impl DocStore for FileDocStore {
    async fn load_source(&self, key: &DocKey) -> Result<String> {
        let path = self.source_path(key)?;
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
    }

    async fn load_crdt(&self, key: &DocKey) -> Result<Option<Vec<u8>>> {
        let path = self.crdt_path(key);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    async fn save(&self, key: &DocKey, source: &str, crdt: &[u8]) -> Result<()> {
        let path = self.source_path(key)?;
        // Write the document through a temp file in the same directory and
        // rename: a host whose editor is watching this file must never see a
        // half-written document, and a crash mid-save must not truncate their
        // work to nothing.
        write_atomic(&path, source.as_bytes())
            .with_context(|| format!("writing {}", path.display()))?;

        let crdt_path = self.crdt_path(key);
        if let Some(parent) = crdt_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // The CRDT sidecar is a cache: losing it costs a re-seed, not data, so
        // a failure here is reported and never fails the document write.
        if let Err(e) = write_atomic(&crdt_path, crdt) {
            log::warn!("could not store CRDT state at {}: {e}", crdt_path.display());
        }
        Ok(())
    }
}

/// Write `bytes` to `path` via a sibling temp file and a rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write as _;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|e| e.to_str()).unwrap_or("part")
    ));
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_and_path_derived() {
        // A share link has to survive a restart, so this must not be random.
        assert_eq!(doc_id("docs/tour.hick"), doc_id("docs/tour.hick"));
        assert_ne!(doc_id("docs/tour.hick"), doc_id("docs/other.hick"));
        assert_eq!(doc_id("a.hick").len(), 16);
    }

    #[tokio::test]
    async fn the_store_round_trips_a_document_and_its_crdt_state() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.hick"), "original\n").unwrap();
        let index = Arc::new(DocIndex::scan(dir.path()).unwrap());
        let (id, rel) = index.sole().unwrap();
        assert_eq!(rel, "a.hick");

        let store = FileDocStore::new(index);
        assert_eq!(store.load_source(&id).await.unwrap(), "original\n");
        assert_eq!(store.load_crdt(&id).await.unwrap(), None);

        store.save(&id, "edited\n", b"crdt-bytes").await.unwrap();
        // The edit landed in the user's actual file — that is the whole point.
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.hick")).unwrap(),
            "edited\n"
        );
        assert_eq!(
            store.load_crdt(&id).await.unwrap().as_deref(),
            Some(b"crdt-bytes".as_slice())
        );
        // …and no stray temp files were left beside it.
        let names: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(!names.iter().any(|n| n.contains("tmp")), "{names:?}");
    }

    #[tokio::test]
    async fn an_unknown_id_names_what_went_wrong() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.hick"), "x").unwrap();
        let index = Arc::new(DocIndex::scan(dir.path()).unwrap());
        let store = FileDocStore::new(index);
        let err = store
            .load_source(&"deadbeef".to_string())
            .await
            .unwrap_err();
        assert!(err.to_string().contains("no document with id"), "{err}");
    }
}
