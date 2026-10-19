//! Host-owned immutable context. Capture is not delivery; polling chooses no refresh policy.
pub mod host;
use anyhow::{Context, Result, bail, ensure};
use hick_context_protocol::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub fn fresh_id() -> String {
    Uuid::new_v4().to_string()
}
pub fn span(start: usize, end: usize) -> Span {
    Span {
        start: start as u64,
        end: end as u64,
        unit: "utf8_bytes".into(),
    }
}
fn slice<'a>(text: &'a str, range: &Span) -> Result<&'a str> {
    ensure!(range.unit == "utf8_bytes", "expected UTF-8 bytes");
    text.get(usize::try_from(range.start)?..usize::try_from(range.end)?)
        .context("invalid UTF-8 range")
}
/// Both original and current revisions must establish unique correspondence.
pub fn unique_passage(snapshot: &str, passage: &str) -> bool {
    passage_positions(snapshot, passage).len() == 1
}
fn passage_positions(snapshot: &str, passage: &str) -> Vec<usize> {
    if passage.is_empty() {
        return vec![];
    }
    let Some(first) = snapshot.find(passage) else {
        return vec![];
    };
    let next = first + snapshot[first..].chars().next().unwrap().len_utf8();
    match snapshot[next..].find(passage) {
        Some(second) => vec![first, next + second],
        None => vec![first],
    }
}
/// Adapters report observations, not guesses about exposure or semantic support.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Head {
    Present { source: Source, text: String },
    Deleted,
    Unavailable,
}
pub trait SourceAdapter {
    fn head(&mut self, source: &Source) -> Result<Head>;
}
/// Instruction metadata is retained separately from ordinary material's bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub node: String,
    pub role: String,
    pub order: u64,
    /// Workspace-relative directory. Applicability is recomputed for each task location.
    pub directory: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Replay {
    fingerprint: String,
    node: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct Data {
    nodes: Vec<Node>,
    captures: Vec<CaptureReceipt>,
    #[serde(default)]
    productions: Vec<ProductionReceipt>,
    sources: BTreeMap<String, Source>,
    heads: BTreeMap<String, Head>,
    keys: BTreeMap<String, Replay>,
    instructions: Vec<Instruction>,
    #[serde(default)]
    unique_anchors: BTreeMap<String, bool>,
}
fn validate_retained(data: &Data) -> Result<()> {
    let bundle = json!({"version":1,"nodes":data.nodes,"captures":data.captures,"deliveries":[],
        "handles":[],"annotations":[],"productions":data.productions,"request":{"id":"validation",
        "branch":"validation","policy":"none","policy_version":"1","observed_cursor":"none",
        "parts":[],"provider_coverage":"host_controlled_only"}});
    validate_bundle(&bundle)?;
    let references: BTreeSet<_> = data.nodes.iter().map(|n| n.id.as_str()).collect();
    ensure!(
        data.keys
            .values()
            .all(|key| references.contains(key.node.as_str()))
            && data
                .instructions
                .iter()
                .all(|i| references.contains(i.node.as_str())),
        "retained metadata has missing_reference"
    );
    ensure!(
        data.nodes.iter().all(|n| n
            .source
            .as_ref()
            .is_none_or(|s| data.sources.contains_key(&s.id))),
        "retained source identity missing"
    );
    Ok(())
}
/// One writer per retained session. Quotas reject new captures rather than evict pinned history.
pub struct Store {
    path: PathBuf,
    _lock: fs::File,
    data: Data,
    epoch: String,
    events: Vec<Change>,
    floor: u64,
    sequence: u64,
    max_bytes: usize,
    max_events: usize,
    reconciled: bool,
}
impl Store {
    pub fn open(path: &Path, max_bytes: usize, max_events: usize) -> Result<Self> {
        ensure!(
            max_bytes > 0 && max_events > 0,
            "positive storage limits required"
        );
        fs::create_dir_all(path.parent().context("store needs parent")?)?;
        let lock = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path.with_extension("lock"))?;
        fs2::FileExt::try_lock_exclusive(&lock).context("context session already in use")?;
        let data: Data = match fs::File::open(path) {
            Ok(file) => {
                use std::io::Read;
                let mut bytes = Vec::new();
                file.take(max_bytes as u64 + 1).read_to_end(&mut bytes)?;
                ensure!(bytes.len() <= max_bytes, "retained store exceeds quota");
                serde_json::from_slice(&bytes)?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Data::default(),
            Err(e) => return Err(e.into()),
        };
        validate_retained(&data)?;
        Ok(Self {
            path: path.into(),
            _lock: lock,
            data,
            epoch: fresh_id(),
            events: vec![],
            floor: 0,
            sequence: 0,
            max_bytes,
            max_events,
            reconciled: false,
        })
    }
    fn commit(&mut self, next: Data) -> Result<()> {
        let bytes = serde_json::to_vec(&next)?;
        ensure!(
            bytes.len() <= self.max_bytes,
            "context storage quota exhausted; retained history unchanged"
        );
        let mut temp = tempfile::NamedTempFile::new_in(self.path.parent().unwrap())?;
        temp.write_all(&bytes)?;
        temp.as_file().sync_all()?;
        temp.persist(&self.path).map_err(|e| e.error)?;
        #[cfg(unix)]
        fs::File::open(self.path.parent().unwrap())?.sync_all()?;
        self.data = next;
        Ok(())
    }
    pub fn node(&self, reference: &str) -> Result<&Node> {
        self.data
            .nodes
            .iter()
            .find(|n| n.id == reference)
            .context("missing_reference")
    }
    pub fn productions(&self) -> &[ProductionReceipt] {
        &self.data.productions
    }
    /// Host-only observation of a completed model response. Genealogy is declared,
    /// never a byte mapping or evidence that the generated claims are true.
    pub fn model_response(
        &mut self,
        request: &str,
        inputs: Vec<String>,
        text: &str,
    ) -> Result<Node> {
        let response: Node = serde_json::from_value(json!({"id":fresh_id(),
            "text":text,"digest":digest(text.as_bytes()),"parents":[],
            "derivation":"root","coverage":"complete","mappings":[]}))?;
        let receipt = fresh_id();
        let output: Node = serde_json::from_value(json!({"id":fresh_id(),
            "text":text,"digest":response.digest,"parents":inputs,
            "derivation":"model_output","coverage":"partial","mappings":[],"receipt":receipt}))?;
        let production: ProductionReceipt = serde_json::from_value(json!({"id":receipt,
            "kind":"model","request":request,"response":response.id,
            "inputs":inputs,"output":output.id}))?;
        let mut next = self.data.clone();
        next.nodes.extend([response, output.clone()]);
        next.productions.push(production);
        validate_retained(&next)?;
        self.commit(next)?;
        Ok(output)
    }
    pub fn nodes(&self) -> &[Node] {
        &self.data.nodes
    }
    pub fn captures(&self) -> &[CaptureReceipt] {
        &self.data.captures
    }
    pub fn sources(&self) -> Vec<Source> {
        self.data
            .sources
            .values()
            .map(|s| match self.data.heads.get(&s.id) {
                Some(Head::Present { source, .. }) => source.clone(),
                _ => s.clone(),
            })
            .collect()
    }
    /// Host registration preserves identity across explicit relocation; equal bytes never merge sources.
    pub fn register(&mut self, kind: &str, locator: &str) -> Result<Source> {
        if let Some(s) = self
            .data
            .sources
            .values()
            .find(|s| s.kind == kind && s.locator == locator)
        {
            return Ok(s.clone());
        }
        let source = Source {
            id: fresh_id(),
            revision: digest(b""),
            locator: locator.into(),
            kind: kind.into(),
        };
        let mut next = self.data.clone();
        next.sources.insert(source.id.clone(), source.clone());
        self.commit(next)?;
        Ok(source)
    }
    pub fn relocate(&mut self, source: &str, locator: &str) -> Result<()> {
        let known = self.data.sources.get(source).context("missing_reference")?;
        ensure!(
            !self
                .data
                .sources
                .values()
                .any(|s| s.id != source && s.kind == known.kind && s.locator == locator),
            "relocation conflicts with an existing source identity"
        );
        let mut next = self.data.clone();
        next.sources
            .get_mut(source)
            .context("unknown source")?
            .locator = locator.into();
        self.commit(next)
    }
    fn replay(&self, key: &str, fingerprint: &str) -> Result<Option<Node>> {
        ensure!(!key.is_empty(), "empty idempotency key");
        if let Some(r) = self.data.keys.get(key) {
            ensure!(r.fingerprint == fingerprint, "idempotency_conflict");
            return Ok(Some(self.node(&r.node)?.clone()));
        }
        Ok(None)
    }
    /// Capture the exact snapshot used by the adapter, not a later reread of disk.
    pub fn capture(
        &mut self,
        request: &CaptureRequest,
        snapshot: &str,
    ) -> Result<(Node, CaptureReceipt)> {
        validate(&serde_json::to_value(request)?, "CaptureRequest")?;
        let fingerprint = digest(&serde_json::to_vec(request)?);
        if let Some(node) = self.replay(&request.idempotency_key, &fingerprint)? {
            let receipt = self
                .data
                .captures
                .iter()
                .find(|r| r.node == node.id)
                .unwrap()
                .clone();
            return Ok((node, receipt));
        }
        let known = self
            .data
            .sources
            .get(&request.source.id)
            .context("unknown source identity")?;
        ensure!(
            known.kind == request.source.kind && known.locator == request.source.locator,
            "source identity/locator mismatch"
        );
        ensure!(
            digest(snapshot.as_bytes()) == request.source.revision,
            "source revision mismatch"
        );
        let range = request
            .span
            .clone()
            .unwrap_or_else(|| span(0, snapshot.len()));
        let text = slice(snapshot, &range)?;
        ensure!(
            text.len() as u64 <= request.max_bytes && text.len() < MAX_FRAME / 4,
            "frame_limit: capture exceeds bounded response; request smaller range"
        );
        let node = Node {
            id: fresh_id(),
            digest: digest(text.as_bytes()),
            text: Some(text.into()),
            artifact: None,
            source: Some(request.source.clone()),
            source_span: Some(range),
            parents: vec![],
            derivation: DerivationKind::Root,
            coverage: Coverage::Complete,
            mappings: vec![],
            receipt: None,
        };
        let receipt = CaptureReceipt {
            id: fresh_id(),
            node: node.id.clone(),
            captured_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
                .to_string(),
            state: "captured".into(),
        };
        let mut next = self.data.clone();
        next.nodes.push(node.clone());
        next.unique_anchors
            .insert(node.id.clone(), unique_passage(snapshot, text));
        next.captures.push(receipt.clone());
        next.keys.insert(
            request.idempotency_key.clone(),
            Replay {
                fingerprint,
                node: node.id.clone(),
            },
        );
        // The source head is an observation, not a modification of the historical capture.
        self.commit(next)?;
        Ok((node, receipt))
    }
    /// Trusted host adapter import of an existing exact read receipt. The adapter, not a
    /// worker DTO, attests the whole-snapshot digest and byte boundaries computed at read time.
    pub fn import_read(
        &mut self,
        source: Source,
        range: Span,
        text: &str,
        key: &str,
    ) -> Result<Node> {
        let unique = digest(text.as_bytes()) == source.revision && !text.is_empty();
        self.import_read_anchored(source, range, text, key, unique)
    }
    pub fn import_read_anchored(
        &mut self,
        source: Source,
        range: Span,
        text: &str,
        key: &str,
        unique: bool,
    ) -> Result<Node> {
        ensure!(
            self.data.sources.contains_key(&source.id),
            "missing_reference"
        );
        ensure!(
            source.revision.len() == 64 && source.revision.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid source revision"
        );
        ensure!(
            range.unit == "utf8_bytes"
                && range.end.checked_sub(range.start) == Some(text.len() as u64),
            "invalid read span"
        );
        ensure!(
            text.len() < MAX_FRAME / 4,
            "frame_limit: capture smaller ranges"
        );
        let fingerprint = digest(&serde_json::to_vec(&(
            source.clone(),
            range.clone(),
            text,
            unique,
        ))?);
        if let Some(node) = self.replay(key, &fingerprint)? {
            return Ok(node);
        }
        let node = Node {
            id: fresh_id(),
            digest: digest(text.as_bytes()),
            text: Some(text.into()),
            artifact: None,
            source: Some(source),
            source_span: Some(range),
            parents: vec![],
            derivation: DerivationKind::Root,
            coverage: Coverage::Complete,
            mappings: vec![],
            receipt: None,
        };
        let mut next = self.data.clone();
        next.nodes.push(node.clone());
        next.unique_anchors.insert(node.id.clone(), unique);
        next.captures.push(CaptureReceipt {
            id: fresh_id(),
            node: node.id.clone(),
            captured_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
                .to_string(),
            state: "captured".into(),
        });
        next.keys.insert(
            key.into(),
            Replay {
                fingerprint,
                node: node.id.clone(),
            },
        );
        self.commit(next)?;
        Ok(node)
    }
    /// Host-side large observations are retained as explicit bounded chunks, with
    /// original whole-source revision and byte ranges on every chunk.
    pub fn import_chunks(
        &mut self,
        source: Source,
        range: Span,
        text: &str,
        key: &str,
    ) -> Result<Vec<Node>> {
        let unique = digest(text.as_bytes()) == source.revision && !text.is_empty();
        self.import_chunks_anchored(source, range, text, key, unique)
    }
    pub fn import_chunks_anchored(
        &mut self,
        source: Source,
        range: Span,
        text: &str,
        key: &str,
        unique: bool,
    ) -> Result<Vec<Node>> {
        ensure!(
            range.end.checked_sub(range.start) == Some(text.len() as u64),
            "invalid read span"
        );
        let mut result = Vec::new();
        let whole = digest(text.as_bytes()) == source.revision;
        let mut offset = 0;
        loop {
            let mut end = (offset + MAX_FRAME / 8).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            let start = range.start as usize + offset;
            let stop = range.start as usize + end;
            let anchor = if offset == 0 && end == text.len() {
                unique
            } else {
                whole && unique_passage(text, &text[offset..end])
            };
            result.push(self.import_read_anchored(
                source.clone(),
                span(start, stop),
                &text[offset..end],
                &format!("{key}/{offset}"),
                anchor,
            )?);
            offset = end;
            if offset == text.len() {
                break;
            }
        }
        Ok(result)
    }
    /// Derivations reuse the protocol's byte/mapping validator. Producer-backed outputs need
    /// host-authenticated production storage; they cannot be invented by this API.
    pub fn derive(&mut self, request: &DeriveRequest) -> Result<Node> {
        validate(&serde_json::to_value(request)?, "DeriveRequest")?;
        let fingerprint = digest(&serde_json::to_vec(request)?);
        if let Some(n) = self.replay(&request.idempotency_key, &fingerprint)? {
            return Ok(n);
        }
        ensure!(
            !self.data.nodes.iter().any(|n| n.id == request.node.id),
            "immutable reference already registered"
        );
        ensure!(
            request.node.artifact.is_none(),
            "artifact storage is not installed"
        );
        ensure!(
            request.node.source.is_none() && request.node.source_span.is_none(),
            "only capture can attest source bytes"
        );
        let mut next = self.data.clone();
        next.nodes.push(request.node.clone());
        validate_retained(&next)?;
        next.keys.insert(
            request.idempotency_key.clone(),
            Replay {
                fingerprint,
                node: request.node.id.clone(),
            },
        );
        self.commit(next)?;
        Ok(request.node.clone())
    }
    pub fn instruction(&mut self, instruction: Instruction) -> Result<()> {
        self.node(&instruction.node)?;
        ensure!(!instruction.role.is_empty(), "instruction role required");
        let directory = Path::new(&instruction.directory);
        ensure!(
            !directory.is_absolute()
                && directory
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "instruction scope must be a relative directory"
        );
        let mut next = self.data.clone();
        next.instructions.push(instruction);
        self.commit(next)
    }
    pub fn applicable(&self, task: &Path) -> Result<Vec<Instruction>> {
        ensure!(
            !task.is_absolute()
                && task
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "task must be workspace-relative"
        );
        let mut instructions: Vec<_> = self
            .data
            .instructions
            .iter()
            .filter(|i| task.starts_with(&i.directory))
            .cloned()
            .collect();
        instructions.sort_by_key(|i| i.order);
        Ok(instructions)
    }
    pub fn compare(&self, request: &CompareRequest) -> Result<Comparison> {
        let old = self.node(&request.old)?;
        let current = self.node(&request.current)?;
        let mut result = Comparison {
            old: old.id.clone(),
            current: Some(current.id.clone()),
            state: ComparisonState::CannotCheck,
            mappings: vec![],
        };
        let (Some(a), Some(b)) = (&old.source, &current.source) else {
            return Ok(result);
        };
        ensure!(
            a.id == b.id && a.kind == b.kind,
            "cannot compare different sources"
        );
        let (Some(left), Some(right), Some(old_span), Some(now_span)) = (
            &old.text,
            &current.text,
            &old.source_span,
            &current.source_span,
        ) else {
            return Ok(result);
        };
        if a.revision == b.revision {
            // Same immutable revision gives exact identity, including duplicate text.
            if old_span.start >= now_span.start && old_span.end <= now_span.end {
                let relative = span(
                    (old_span.start - now_span.start) as usize,
                    (old_span.end - now_span.start) as usize,
                );
                if slice(right, &relative)? == left {
                    result.state = if a.locator == b.locator {
                        ComparisonState::Unchanged
                    } else {
                        ComparisonState::Moved
                    };
                    result.mappings.push(Mapping {
                        parent: old.id.clone(),
                        input: span(0, left.len()),
                        output: relative,
                    });
                }
            }
            return Ok(result);
        }
        // A bounded candidate cannot establish absence or unique correspondence across
        // omitted bytes. Require the full current revision for cross-revision search.
        if left.is_empty() || now_span.start != 0 || digest(right.as_bytes()) != b.revision {
            return Ok(result);
        }
        // Include overlapping matches; match_indices skips overlaps. Never guess among duplicates.
        let matches = passage_positions(right, left);
        result.state = match matches.as_slice() {
            [] => ComparisonState::Changed,
            [p] => {
                if self.data.unique_anchors.get(&old.id) != Some(&true) {
                    result.state = ComparisonState::Ambiguous;
                    return Ok(result);
                }
                result.mappings.push(Mapping {
                    parent: old.id.clone(),
                    input: span(0, left.len()),
                    output: span(*p, *p + left.len()),
                });
                if *p as u64 + now_span.start == old_span.start && a.locator == b.locator {
                    ComparisonState::Unchanged
                } else {
                    ComparisonState::Moved
                }
            }
            _ => ComparisonState::Ambiguous,
        };
        Ok(result)
    }
    pub fn cursor(&self, branch: &str) -> Cursor {
        Cursor {
            epoch: self.epoch.clone(),
            branch: branch.into(),
            position: self.sequence,
        }
    }
    fn event(
        &mut self,
        source: String,
        old_revision: Option<String>,
        new_revision: Option<String>,
        state: ComparisonState,
    ) {
        self.sequence += 1;
        self.events.push(Change {
            sequence: self.sequence,
            source,
            old_revision,
            new_revision,
            state,
        });
        if self.events.len() > self.max_events {
            self.floor = self.events.remove(0).sequence;
        }
    }
    /// Hash reconciliation runs on every poll. No Git dependency or watcher delivery assumption.
    pub fn reconcile(&mut self, adapter: &mut dyn SourceAdapter) -> Result<()> {
        let mut next = self.data.clone();
        let mut events = vec![];
        for source in self.data.sources.values() {
            let head = adapter.head(source).unwrap_or(Head::Unavailable);
            if let Head::Present {
                source: observed,
                text,
            } = &head
            {
                ensure!(
                    observed.id == source.id
                        && observed.kind == source.kind
                        && observed.locator == source.locator
                        && observed.revision == digest(text.as_bytes()),
                    "adapter returned inconsistent source identity/revision"
                );
            }
            if self.data.heads.get(&source.id) != Some(&head) {
                let revision = |h: &Head| match h {
                    Head::Present { source, .. } => Some(source.revision.clone()),
                    _ => None,
                };
                let old = self.data.heads.get(&source.id).and_then(revision);
                let new = revision(&head);
                let state = match &head {
                    Head::Present { .. } => ComparisonState::Changed,
                    Head::Deleted => ComparisonState::Unavailable,
                    Head::Unavailable => ComparisonState::CannotCheck,
                };
                events.push((source.id.clone(), old, new, state));
                next.heads.insert(source.id.clone(), head);
            }
        }
        if !events.is_empty() {
            self.commit(next)?;
        }
        self.reconciled = true;
        for (s, o, n, state) in events {
            self.event(s, o, n, state);
        }
        Ok(())
    }
    /// Derived currentness is transitive and conservative; historical derivations never change.
    pub fn currentness(&self, reference: &str) -> Result<ComparisonState> {
        self.node(reference)?;
        let mut states = BTreeMap::new();
        for node in &self.data.nodes {
            let state = if let Some(source) = &node.source {
                if !self.reconciled {
                    ComparisonState::CannotCheck
                } else {
                    match self.data.heads.get(&source.id) {
                        Some(Head::Present { source: head, .. })
                            if source.revision == head.revision
                                && source.locator == head.locator =>
                        {
                            ComparisonState::Unchanged
                        }
                        Some(Head::Present { .. }) => ComparisonState::Changed,
                        Some(Head::Deleted) => ComparisonState::Unavailable,
                        _ => ComparisonState::CannotCheck,
                    }
                }
            } else if node.parents.is_empty() {
                ComparisonState::CannotCheck
            } else if node
                .parents
                .iter()
                .all(|p| states.get(p) == Some(&ComparisonState::Unchanged))
            {
                if node.coverage == Coverage::Complete {
                    ComparisonState::Unchanged
                } else {
                    ComparisonState::CannotCheck
                }
            } else if node.parents.iter().any(|p| {
                matches!(
                    states.get(p),
                    Some(
                        ComparisonState::Changed
                            | ComparisonState::Moved
                            | ComparisonState::Unavailable
                            | ComparisonState::Invalidated
                    )
                )
            }) {
                ComparisonState::Invalidated
            } else {
                ComparisonState::CannotCheck
            };
            if node.id == reference {
                return Ok(state);
            }
            states.insert(node.id.clone(), state);
        }
        unreachable!("reference validated")
    }
    pub fn changes(&self, request: &ChangesRequest, branch: &str) -> Result<ChangePage> {
        ensure!(
            request.limit > 0 && request.limit <= MAX_PAGE,
            "invalid_page"
        );
        ensure!(
            request.cursor.epoch == self.epoch
                && request.cursor.branch == branch
                && request.cursor.position >= self.floor
                && request.cursor.position <= self.sequence,
            "cursor_expired: reconcile retained references and obtain a new cursor"
        );
        let mut sources = BTreeSet::new();
        let mut pending = request.references.clone();
        let mut visited = BTreeSet::new();
        while let Some(reference) = pending.pop() {
            if !visited.insert(reference.clone()) {
                continue;
            }
            let node = self.node(&reference)?;
            if let Some(source) = &node.source {
                sources.insert(source.id.clone());
            }
            pending.extend(node.parents.iter().cloned());
        }
        let selected: Vec<_> = self
            .events
            .iter()
            .filter(|e| e.sequence > request.cursor.position && sources.contains(&e.source))
            .collect();
        let has_more = selected.len() > request.limit as usize;
        let events: Vec<_> = selected
            .into_iter()
            .take(request.limit as usize)
            .cloned()
            .collect();
        let position = events
            .last()
            .map_or(request.cursor.position, |e| e.sequence);
        let next_cursor = Cursor {
            position,
            ..request.cursor.clone()
        };
        Ok(ChangePage {
            from_cursor: request.cursor.clone(),
            next_cursor,
            events,
            has_more,
            limit: request.limit,
        })
    }
    pub fn head(&self, source: &str) -> Option<&Head> {
        self.data.heads.get(source)
    }
}

/// Files are confined at each read, including after rename/symlink changes. Editor state
/// enters through a separate adapter and cannot fall back to saved bytes when unavailable.
pub struct Files {
    root: PathBuf,
    pub max_bytes: usize,
}
impl Files {
    pub fn new(root: &Path, max_bytes: usize) -> Result<Self> {
        Ok(Self {
            root: root.canonicalize()?,
            max_bytes,
        })
    }
    pub fn read(&self, locator: &str) -> Result<String> {
        let path = self.root.join(locator).canonicalize()?;
        ensure!(path.starts_with(&self.root), "source outside workspace");
        ensure!(
            fs::metadata(&path)?.is_file(),
            "source is not a regular file"
        );
        let mut file = fs::File::open(path)?;
        ensure!(file.metadata()?.is_file(), "source is not a regular file");
        use std::io::Read;
        let mut bytes = Vec::new();
        std::io::Read::by_ref(&mut file)
            .take(self.max_bytes as u64 + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() <= self.max_bytes,
            "source exceeds adapter limit"
        );
        Ok(String::from_utf8(bytes)?)
    }
}
impl SourceAdapter for Files {
    fn head(&mut self, source: &Source) -> Result<Head> {
        if source.kind != "saved_file" {
            return Ok(Head::Unavailable);
        }
        match self.read(&source.locator) {
            Ok(text) => Ok(Head::Present {
                source: Source {
                    revision: digest(text.as_bytes()),
                    ..source.clone()
                },
                text,
            }),
            Err(e)
                if e.downcast_ref::<std::io::Error>()
                    .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
            {
                Ok(Head::Deleted)
            }
            Err(_) => Ok(Head::Unavailable),
        }
    }
}
