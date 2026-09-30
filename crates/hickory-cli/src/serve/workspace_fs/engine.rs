use super::super::{LocalState, acp::record::Record, store};
use super::{View, view};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const MAX_FILE: usize = 64 * 1024 * 1024;
struct Handle {
    path: String,
    view: View,
    original: Vec<u8>,
    bytes: Vec<u8>,
    write: bool,
    dirty: bool,
}
/// One workspace frontend's open files. All operations serialize with ACP/MCP;
/// each handle pins the bytes and lineage it read, rather than reinterpreting
/// offsets against a newer document. Writes become visible on close or sync.
pub struct Engine {
    state: LocalState,
    cached: Option<View>,
    handles: HashMap<String, Handle>,
    reads: HashMap<String, (View, Vec<u8>)>,
    turn: Option<String>,
    record: Arc<Mutex<Record>>,
    pub gate: Arc<Mutex<()>>,
}
impl Engine {
    pub(crate) fn new(state: LocalState, record: Arc<Mutex<Record>>, gate: Arc<Mutex<()>>) -> Self {
        Self {
            state,
            cached: None,
            handles: HashMap::new(),
            reads: HashMap::new(),
            turn: None,
            record,
            gate,
        }
    }
    pub fn open(state: LocalState, session: std::path::PathBuf) -> Result<Self> {
        let record = Record::open(session, state.index.root())?;
        Ok(Self::new(
            state,
            Arc::new(Mutex::new(record)),
            Arc::new(Mutex::new(())),
        ))
    }
    async fn view(&mut self) -> Result<View> {
        let sources = View::sources(&self.state).await?;
        if let Some(view) = &self.cached
            && view.sources == sources
        {
            return Ok(view.clone());
        }
        let view = View::weave(&self.state, sources).await?;
        self.cached = Some(view.clone());
        Ok(view)
    }
    pub async fn request(&mut self, p: &Value) -> Result<Value> {
        let op = p["op"].as_str().context("operation is required")?;
        let rel = p["path"].as_str().unwrap_or("");
        let root = self.state.index.root().to_path_buf();
        view::confined(&root, rel)?;
        match op {
            "space" => Ok(
                json!({"total":fs2::total_space(&root)?,"available":fs2::available_space(&root)?,"free":fs2::free_space(&root)?}),
            ),
            "turn" => {
                self.turn = p["turn"].as_str().map(str::to_string);
                self.reads.clear();
                Ok(json!({}))
            }
            "read_text" => {
                let view = self.view().await?;
                let bytes = view.bytes(&self.state, rel)?;
                ensure!(bytes.len() <= MAX_FILE, "file exceeds the buffer limit");
                let text = std::str::from_utf8(&bytes)?;
                let start = p["line"].as_u64().unwrap_or(1).saturating_sub(1) as usize;
                let limit = p["limit"].as_u64().map_or(usize::MAX, |n| n as usize);
                let count = text.lines().count().max(1);
                let shown = count.saturating_sub(start).min(limit);
                if shown > 0 {
                    let read = hickory_agent::ContextRead {
                        path: rel.into(),
                        commit: None,
                        sha256: view::hash(&bytes),
                        first_line: start + 1,
                        last_line: start + shown,
                    };
                    self.record
                        .lock()
                        .await
                        .event(hickory_agent::SessionEvent::Read { read: &read })?;
                }
                let content = if p.get("line").is_none() && p.get("limit").is_none() {
                    text.to_string()
                } else {
                    text.split_inclusive('\n').skip(start).take(limit).collect()
                };
                self.reads.insert(rel.into(), (view, bytes));
                Ok(json!({"content":content}))
            }
            "write_text" => {
                let text = p["content"].as_str().context("content is required")?;
                let (view, before) = self
                    .reads
                    .remove(rel)
                    .context("read this file before writing it, so its revision can be checked")?;
                self.commit(rel, &view, &before, text.as_bytes()).await?;
                Ok(json!({}))
            }
            "stat" => {
                let mut entry = self.view().await?.entry(&self.state, rel)?;
                if let Some(h) = self.handles.values().find(|h| h.path == rel && h.dirty) {
                    entry.size = h.bytes.len();
                }
                Ok(serde_json::to_value(entry)?)
            }
            "list" => Ok(serde_json::to_value(
                self.view().await?.list(&self.state, rel)?,
            )?),
            "open" => {
                let write = p["write"].as_bool().unwrap_or(false);
                if write {
                    view::writable(rel)?;
                }
                let view = self.view().await?;
                let bytes = view.bytes(&self.state, rel)?;
                ensure!(
                    bytes.len() <= MAX_FILE,
                    "file exceeds the agent workspace's 64 MiB buffer limit"
                );
                let id = format!("{:016x}", super::super::rand_id());
                self.handles.insert(
                    id.clone(),
                    Handle {
                        path: rel.into(),
                        view,
                        original: bytes.clone(),
                        bytes,
                        write,
                        dirty: false,
                    },
                );
                Ok(json!({"handle":id}))
            }
            "read" => {
                use base64::Engine as _;
                let turn = self.turn.clone();
                let h = self.handle(p)?;
                let offset = number(p, "offset")?;
                let length = number(p, "length")?.min(MAX_FILE);
                let start = offset.min(h.bytes.len());
                let end = start.saturating_add(length).min(h.bytes.len());
                let data = h.bytes[start..end].to_vec();
                let evidence = json!({"path":h.path,"revision":h.view.revision,"sha256":view::hash(&h.bytes),"offset":start,"length":data.len(),"kind":"filesystem-access","turn":turn,"uncommitted":h.dirty,"lineage":if h.bytes == h.original { h.view.outputs.get(&h.path).map(|o| &o.lineage) } else { None }});
                let path = h.path.clone();
                let view = h.view.clone();
                let original = h.original.clone();
                self.reads.insert(path, (view, original));
                self.record
                    .lock()
                    .await
                    .context("filesystem-access", &evidence)?;
                Ok(json!({"data":base64::engine::general_purpose::STANDARD.encode(&data)}))
            }
            "write" => {
                use base64::Engine as _;
                let offset = number(p, "offset")?;
                let data = base64::engine::general_purpose::STANDARD
                    .decode(p["data"].as_str().context("data is required")?)?;
                let h = self.handle(p)?;
                ensure!(h.write, "file was not opened for writing");
                let end = offset.checked_add(data.len()).context("write overflow")?;
                ensure!(
                    end <= MAX_FILE,
                    "file exceeds the agent workspace's 64 MiB buffer limit"
                );
                h.bytes.resize(end.max(h.bytes.len()), 0);
                h.bytes[offset..end].copy_from_slice(&data);
                h.dirty = true;
                Ok(json!({"written":data.len()}))
            }
            "truncate" => {
                let length = number(p, "length")?;
                ensure!(length <= MAX_FILE, "file exceeds buffer limit");
                let h = self.handle(p)?;
                ensure!(h.write, "file was not opened for writing");
                h.bytes.resize(length, 0);
                h.dirty = true;
                Ok(json!({}))
            }
            "close" | "flush" => {
                let id = p["handle"].as_str().context("handle is required")?;
                let h = self.handles.remove(id).context("unknown file handle")?;
                if h.dirty
                    && let Err(e) = self.commit(&h.path, &h.view, &h.original, &h.bytes).await
                {
                    self.record.lock().await.context(
                        "filesystem-refusal",
                        &json!({"turn":self.turn,"path":h.path,"error":format!("{e:#}")}),
                    )?;
                    self.handles.insert(id.into(), h);
                    return Err(e);
                }
                if op == "flush" {
                    let view = self.view().await?;
                    let bytes = view.bytes(&self.state, &h.path)?;
                    self.handles.insert(
                        id.into(),
                        Handle {
                            view,
                            original: bytes.clone(),
                            bytes,
                            dirty: false,
                            ..h
                        },
                    );
                }
                Ok(json!({}))
            }
            "sync" => {
                let ids: Vec<_> = self.handles.keys().cloned().collect();
                // Each save is one act. This does not invent a multi-file
                // transaction boundary from a volume-wide fsync.
                for id in ids {
                    let h = self.handles.remove(&id).unwrap();
                    if h.dirty
                        && let Err(e) = self.commit(&h.path, &h.view, &h.original, &h.bytes).await
                    {
                        self.handles.insert(id, h);
                        return Err(e);
                    }
                    let view = self.view().await?;
                    let bytes = view.bytes(&self.state, &h.path)?;
                    self.handles.insert(
                        id,
                        Handle {
                            view,
                            original: bytes.clone(),
                            bytes,
                            dirty: false,
                            ..h
                        },
                    );
                }
                Ok(json!({}))
            }
            "create" => {
                view::writable(rel)?;
                let view = self.view().await?;
                let path = view::confined(&root, rel)?;
                ensure!(
                    !path.exists() && !view.outputs.contains_key(rel),
                    "file already exists"
                );
                if p["directory"] == true {
                    std::fs::create_dir(&path)?;
                } else {
                    std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&path)?;
                }
                Ok(serde_json::to_value(
                    self.view().await?.entry(&self.state, rel)?,
                )?)
            }
            "rename" => {
                let to = p["to"].as_str().context("destination is required")?;
                view::writable(rel)?;
                view::writable(to)?;
                let view = self.view().await?;
                ensure!(
                    !view
                        .sources
                        .keys()
                        .chain(view.outputs.keys())
                        .any(|p| p == rel || p.starts_with(&format!("{rel}/"))),
                    "move documents and generated files through Hickory's Files pane"
                );
                ensure!(
                    !self.handles.values().any(|h| h.path == rel && h.dirty),
                    "close the temporary file before renaming it"
                );
                let from = view::confined(&root, rel)?;
                let dest = view::confined(&root, to)?;
                if view.sources.contains_key(to)
                    || view.outputs.contains_key(to)
                    || to.ends_with(".md")
                {
                    let before = if dest.exists() || view.outputs.contains_key(to) {
                        view.bytes(&self.state, to)?
                    } else {
                        Vec::new()
                    };
                    let after = std::fs::read(&from)?;
                    let (base, before) =
                        if view.sources.contains_key(to) || view.outputs.contains_key(to) {
                            self.reads.remove(to).context(
                            "read the destination before replacing a document or generated file",
                        )?
                        } else {
                            (view.clone(), before)
                        };
                    self.commit(to, &base, &before, &after).await?;
                    std::fs::remove_file(from)?;
                } else {
                    std::fs::rename(from, dest)?;
                }
                Ok(json!({}))
            }
            "remove" => {
                view::writable(rel)?;
                let view = self.view().await?;
                ensure!(
                    !view
                        .sources
                        .keys()
                        .chain(view.outputs.keys())
                        .any(|p| p == rel || p.starts_with(&format!("{rel}/"))),
                    "delete documents and generated files through Hickory's Files pane"
                );
                let path = view::confined(&root, rel)?;
                if path.is_dir() {
                    std::fs::remove_dir(path)?;
                } else {
                    std::fs::remove_file(path)?;
                }
                Ok(json!({}))
            }
            _ => anyhow::bail!("unsupported filesystem operation {op}"),
        }
    }
    fn handle(&mut self, p: &Value) -> Result<&mut Handle> {
        self.handles
            .get_mut(p["handle"].as_str().context("handle is required")?)
            .context("unknown file handle")
    }
    async fn commit(&mut self, rel: &str, base: &View, before: &[u8], after: &[u8]) -> Result<()> {
        if before == after {
            return Ok(());
        }
        view::writable(rel)?;
        let current = self.view().await?;
        ensure!(
            base.revision == current.revision
                && (if view::confined(self.state.index.root(), rel)?.exists()
                    || current.sources.contains_key(rel)
                    || current.outputs.contains_key(rel)
                {
                    current.bytes(&self.state, rel)?
                } else {
                    Vec::new()
                }) == before,
            "workspace changed since this file was opened; reopen it before saving"
        );
        let mut sources = base.sources.clone();
        let mut changed = Vec::new();
        if sources.contains_key(rel) || (rel.ends_with(".md") && !base.outputs.contains_key(rel)) {
            let source = std::str::from_utf8(after)?;
            hick_lang::parse(source)?;
            sources.insert(rel.into(), source.into());
            changed.push(rel.to_string());
        } else if let Some(output) = base.outputs.get(rel) {
            let old = std::str::from_utf8(before)?;
            let new = std::str::from_utf8(after)?;
            let edits = crate::up::reverse::source_edits_for_save(old, new, &output.lineage)?;
            let absolute: HashMap<_, _> = sources
                .iter()
                .map(|(p, s)| {
                    (
                        self.state.index.root().join(p).display().to_string(),
                        s.clone(),
                    )
                })
                .collect();
            let updated = hickory_lineage::apply_source_edits(&absolute, &edits)?;
            for (path, source) in updated {
                let rel = std::path::Path::new(&path)
                    .strip_prefix(self.state.index.root())?
                    .to_string_lossy()
                    .replace('\\', "/");
                view::confined(self.state.index.root(), &rel)?;
                sources.insert(rel.clone(), source);
                changed.push(rel);
            }
            ensure!(
                changed.len() <= 1,
                "this save changes several source documents; use Hickory's document tools to review the batch"
            );
        } else {
            store::write_atomic(&view::confined(self.state.index.root(), rel)?, after)?;
            if rel.ends_with(".md") {
                self.state.index.add(rel);
            }
            self.record.lock().await.context("filesystem-write", &json!({"turn":self.turn,"path":rel,"before":view::hash(before),"after":view::hash(after)}))?;
            return Ok(());
        }
        let candidate = View::weave(&self.state, sources.clone()).await?;
        ensure!(
            candidate.bytes(&self.state, rel)? == after,
            "the document would not reproduce the proposed output; nothing was saved"
        );
        // Validate EVERYTHING before the single source-file publication. The
        // mount derives its outputs from the source, never from disk products.
        ensure!(
            self.view().await?.revision == base.revision,
            "document changed while this save was being validated; reopen the file"
        );
        for rel in changed {
            let id = self.state.index.add(&rel);
            let original = base.sources.get(&rel).map(String::as_str).unwrap_or("");
            let path = view::confined(self.state.index.root(), &rel)?;
            if !path.exists() {
                store::write_atomic(&path, b"")?;
            }
            let published = self
                .state
                .rooms
                .replace_source_if_current(&id, original, &sources[&rel], || {
                    self.state
                        .write_source(&id, &sources[&rel])
                        .map_err(|e| anyhow::anyhow!("{e:?}"))
                })
                .await?;
            ensure!(
                published,
                "the live document changed before publication; reopen the file"
            );
            self.record.lock().await.context("filesystem-write", &json!({"turn":self.turn,"path":rel,"surface":rel,"revision":candidate.revision,"sha256":view::hash(sources[&rel].as_bytes())}))?;
            let source = &sources[&rel];
            let original = base.sources.get(&rel).map(String::as_str).unwrap_or("");
            let diff = similar::TextDiff::from_lines(original, source);
            let ranges: Vec<_> = diff
                .ops()
                .iter()
                .filter(|op| op.tag() != similar::DiffTag::Equal)
                .map(|op| op.new_range())
                .collect();
            let lines: Vec<_> = source.lines().collect();
            let first = (ranges.first().map_or(0, |r| r.start) + 1).min(lines.len().max(1));
            let last = ranges
                .last()
                .map_or(first, |r| r.end.max(first))
                .min(lines.len().max(first));
            let wrote = hickory_agent::Wrote {
                file: rel,
                first_line: first,
                last_line: last,
                hashes: lines
                    .iter()
                    .skip(first - 1)
                    .take(last - first + 1)
                    .map(|line| hickory_agent::hashline::line_hash(line))
                    .collect(),
            };
            self.record
                .lock()
                .await
                .event(hickory_agent::SessionEvent::Wrote { wrote: &wrote })?;
        }
        Ok(())
    }
}
fn number(p: &Value, key: &str) -> Result<usize> {
    p[key]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .with_context(|| format!("{key} must be a non-negative integer"))
}
