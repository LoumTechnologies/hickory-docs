use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

const MAX_DRAFT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Remote {
    pub machine: String,
    pub doc_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Draft {
    pub id: String,
    pub title: String,
    pub source: String,
    pub revision: u64,
    pub remote: Option<Remote>,
    pub y_state: Option<Vec<u8>>,
    #[serde(default)]
    pub outputs: Option<serde_json::Value>,
}

pub struct Drafts {
    dir: PathBuf,
}
impl Drafts {
    pub fn new(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        Ok(Self {
            dir: dir.to_owned(),
        })
    }

    fn path(&self, id: &str) -> Result<PathBuf> {
        if id.is_empty()
            || id.len() > 100
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            bail!("Invalid draft identifier; the previous document was not changed");
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    pub fn load(&self, id: &str) -> Result<Draft> {
        let path = self.path(id)?;
        let meta = std::fs::symlink_metadata(&path)?;
        if !meta.is_file() || meta.len() > MAX_DRAFT_BYTES as u64 {
            bail!("Draft is not a regular document smaller than 8 MB");
        }
        let draft: Draft = serde_json::from_slice(&std::fs::read(&path)?)
            .with_context(|| format!("Could not recover draft {id}; keep its file for recovery"))?;
        if draft.id != id {
            bail!("Draft identity does not match its filename");
        }
        Ok(draft)
    }

    pub fn list(&self) -> Result<Vec<Draft>> {
        let mut drafts = Vec::new();
        for entry in std::fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .context("Invalid draft filename")?;
                drafts.push(self.load(id)?);
            }
        }
        drafts.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(drafts)
    }
    pub fn save(&self, mut draft: Draft) -> Result<Draft> {
        let path = self.path(&draft.id)?;
        let revision = match std::fs::symlink_metadata(&path) {
            Ok(_) => self.load(&draft.id)?.revision,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error.into()),
        };
        if revision != draft.revision {
            bail!("A newer draft was saved; reopen it before saving again");
        }
        draft.revision = revision
            .checked_add(1)
            .context("Draft revision exhausted")?;
        let bytes = serde_json::to_vec(&draft)?;
        if bytes.len() > MAX_DRAFT_BYTES {
            bail!("Draft exceeds 8 MB; export or split it before saving");
        }
        let mut file = tempfile::NamedTempFile::new_in(&self.dir)?;
        file.write_all(&bytes)?;
        file.as_file().sync_all()?;
        file.persist(&path).map_err(|e| e.error)?;
        Ok(draft)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restart_recovers_text_and_crdt_and_refuses_a_late_save() {
        let dir = tempfile::tempdir().unwrap();
        let store = Drafts::new(dir.path()).unwrap();
        let draft = Draft {
            id: "note-1".into(),
            title: "Notes".into(),
            source: "Hello 🦀".into(),
            revision: 0,
            remote: Some(Remote {
                machine: "desktop".into(),
                doc_id: "a".into(),
            }),
            y_state: Some(vec![1, 2, 3]),
            outputs: Some(serde_json::json!({"captured_at":"test", "render":{"blocks":[]}})),
        };
        let accepted = store.save(draft.clone()).unwrap();
        assert!(store.save(draft).is_err());
        let reopened = Drafts::new(dir.path()).unwrap();
        assert_eq!(reopened.list().unwrap(), vec![accepted]);
    }
    #[test]
    fn draft_names_cannot_escape_storage() {
        let dir = tempfile::tempdir().unwrap();
        let store = Drafts::new(dir.path()).unwrap();
        for id in ["../escape", "/tmp/escape", "", "a/b", "a\\b"] {
            assert!(store.load(id).is_err());
        }
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
