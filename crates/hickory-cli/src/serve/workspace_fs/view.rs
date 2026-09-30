use super::super::LocalState;
use anyhow::{Result, ensure};
use hickory_lineage::Provenance;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, serde::Serialize)]
pub struct Entry {
    pub path: String,
    pub directory: bool,
    pub size: usize,
    pub mode: u32,
}
#[derive(Clone)]
pub struct Output {
    pub bytes: Vec<u8>,
    pub lineage: Vec<Provenance>,
}
#[derive(Clone)]
pub struct View {
    pub revision: String,
    pub sources: BTreeMap<String, String>,
    pub outputs: BTreeMap<String, Output>,
}
pub fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Lexical confinement precedes canonical confinement. Symlinks are refused:
/// a mount must never expose the backing tree through a link.
pub fn confined(root: &Path, rel: &str) -> Result<PathBuf> {
    ensure!(
        !Path::new(rel).is_absolute(),
        "workspace paths must be relative"
    );
    ensure!(
        Path::new(rel)
            .components()
            .all(|c| matches!(c, Component::Normal(_))),
        "invalid workspace path"
    );
    let mut path = root.to_path_buf();
    for part in Path::new(rel).components() {
        path.push(part.as_os_str());
        if let Ok(meta) = std::fs::symlink_metadata(&path) {
            ensure!(
                !meta.file_type().is_symlink(),
                "symbolic links are not exposed in the agent workspace"
            );
        }
    }
    Ok(path)
}
pub fn writable(rel: &str) -> Result<()> {
    ensure!(!rel.is_empty(), "the workspace root cannot be changed");
    ensure!(
        !Path::new(rel).components().any(|c| c.as_os_str() == ".git"
            || c.as_os_str() == ".hick-cache"
            || c.as_os_str() == "sessions"),
        "repository history, caches, and session records are read-only here"
    );
    Ok(())
}
impl View {
    pub async fn sources(state: &LocalState) -> Result<BTreeMap<String, String>> {
        let mut sources = BTreeMap::new();
        for (id, rel) in state.index.entries() {
            let source = if let Some(room) = state.rooms.get(&id).await {
                room.text().await
            } else {
                std::fs::read_to_string(state.index.root().join(&rel))?
            };
            sources.insert(rel, source);
        }
        Ok(sources)
    }
    pub async fn capture(state: &LocalState) -> Result<Self> {
        Self::weave(state, Self::sources(state).await?).await
    }
    pub async fn weave(state: &LocalState, sources: BTreeMap<String, String>) -> Result<Self> {
        let mut outputs = BTreeMap::new();
        for (rel, source) in &sources {
            let path = state.index.root().join(rel);
            let name = path.display().to_string();
            let doc = hick_lang::parse(source)?;
            ensure!(
                hick_blocks::attribute_errors(source, &doc).is_empty(),
                "invalid attributes in {rel}"
            );
            let cc = hick_literate::cache::CacheConfig::new(
                path.parent().unwrap(),
                hick_literate::cache::CacheMode::Reuse,
            );
            let result = hick_literate::run_pipeline_weave(
                &[(&name, source)],
                &state.params,
                cc.cache_dir.is_dir().then_some(&cc),
            )
            .await?;
            let dir = Path::new(rel).parent().unwrap();
            for (key, content) in &result.files {
                let target = dir.join(key).to_string_lossy().replace('\\', "/");
                confined(state.index.root(), &target)?;
                ensure!(
                    !sources.contains_key(&target),
                    "generated output {target} would overwrite a source document"
                );
                ensure!(
                    !outputs.contains_key(&target),
                    "multiple documents generate {target}"
                );
                let bytes = match content {
                    hick_exec::node::FileContent::Text(s) => s.as_bytes().to_vec(),
                    hick_exec::node::FileContent::Binary(b) => b.to_bytes()?,
                };
                let lineage = result
                    .provenance_maps
                    .get(key)
                    .map(hickory_lineage::from_provenance_map)
                    .unwrap_or_default();
                outputs.insert(target, Output { bytes, lineage });
            }
        }
        let revision = hash(serde_json::to_string(&sources)?.as_bytes());
        Ok(Self {
            revision,
            sources,
            outputs,
        })
    }
    pub fn bytes(&self, state: &LocalState, rel: &str) -> Result<Vec<u8>> {
        let path = confined(state.index.root(), rel)?;
        if let Some(source) = self.sources.get(rel) {
            return Ok(source.as_bytes().to_vec());
        }
        if let Some(output) = self.outputs.get(rel) {
            return Ok(output.bytes.clone());
        }
        Ok(std::fs::read(path)?)
    }
    pub fn entry(&self, state: &LocalState, rel: &str) -> Result<Entry> {
        if rel.is_empty() {
            return Ok(Entry {
                path: rel.into(),
                directory: true,
                size: 0,
                mode: 0o755,
            });
        }
        let path = confined(state.index.root(), rel)?;
        let directory = path.is_dir()
            || self
                .outputs
                .keys()
                .any(|p| p.starts_with(&format!("{rel}/")));
        let size = if directory {
            0
        } else {
            self.bytes(state, rel)?.len()
        };
        Ok(Entry {
            path: rel.into(),
            directory,
            size,
            mode: if directory { 0o755 } else { 0o644 },
        })
    }
    pub fn list(&self, state: &LocalState, rel: &str) -> Result<Vec<Entry>> {
        let path = confined(state.index.root(), rel)?;
        let mut children = std::collections::BTreeSet::new();
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                if !entry.file_type()?.is_symlink() {
                    children.insert(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        let prefix = if rel.is_empty() {
            String::new()
        } else {
            format!("{rel}/")
        };
        for key in self.outputs.keys().chain(self.sources.keys()) {
            if let Some(rest) = key.strip_prefix(&prefix) {
                children.insert(rest.split('/').next().unwrap().to_string());
            }
        }
        children
            .into_iter()
            .map(|name| self.entry(state, &format!("{prefix}{name}")))
            .collect()
    }
}
