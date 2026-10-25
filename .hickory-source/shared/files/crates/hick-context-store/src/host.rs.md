<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-context-store/src/host.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Host context observations are immutable and changes are source-backed](../../../../../guarantees/agent/immutable-context-lineage/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `d03c0330dbef16802b4ef3488bb828440151d4e54b564fc264877551690d549c`.

<book:entry path="crates/hick-context-store/src/host.rs">
//! Synchronous capability service for the existing request/response channel.
use super::*;
use serde_json::Value;

/// Editor adapter state is host-fed. Missing editor buffers are unavailable, never disk reads.
#[derive(Default)]
pub struct Editors(pub BTreeMap<String, String>);
struct Adapters<'a> {
    files: &'a mut Files,
    editors: &'a Editors,
    snapshots_only: bool,
}
impl SourceAdapter for Adapters<'_> {
    fn head(&mut self, source: &Source) -> Result<Head> {
        if self.snapshots_only {
            return Ok(Head::Unavailable);
        }
        if source.kind == "editor_buffer" {
            return Ok(match self.editors.0.get(&source.id) {
                Some(text) => Head::Present {
                    source: Source {
                        revision: digest(text.as_bytes()),
                        ..source.clone()
                    },
                    text: text.clone(),
                },
                None => Head::Unavailable,
            });
        }
        self.files.head(source)
    }
}
pub struct Service {
    pub store: Store,
    pub editors: Editors,
    files: Files,
    branch: String,
    negotiated: Option<Negotiated>,
    snapshots_only: bool,
}
impl Service {
    pub fn open(root: &Path, path: &Path, branch: &str) -> Result<Self> {
        Ok(Self {
            store: Store::open(path, 64 * 1024 * 1024, 1024)?,
            files: Files::new(root, 8 * 1024 * 1024)?,
            editors: Editors::default(),
            branch: branch.into(),
            negotiated: None,
            snapshots_only: false,
        })
    }
    /// A fresh host-fed context with no live file/editor adapters or imported history.
    /// Callers must import only receipts already approved by their private policy.
    pub fn open_snapshots(root: &Path, path: &Path, branch: &str) -> Result<Self> {
        let mut service = Self::open(root, path, branch)?;
        ensure!(
            service.store.sources().is_empty()
                && service.store.nodes().is_empty()
                && service.store.productions().is_empty(),
            "private context requires a fresh store"
        );
        service.snapshots_only = true;
        Ok(service)
    }
    pub fn offer() -> Value {
        json!({"versions":[1],"capabilities":["context.capture","context.derive","context.changes","context.compare"],
            "transport":"cursor_polling","max_frame_bytes":MAX_FRAME,"max_page_items":MAX_PAGE})
    }
    pub fn poll(&mut self) -> Result<()> {
        self.store.reconcile(&mut Adapters {
            files: &mut self.files,
            editors: &self.editors,
            snapshots_only: self.snapshots_only,
        })
    }
    /// Host discovery supplies source identities; untrusted workers cannot mint adapter identities.
    pub fn saved(&mut self, locator: &str) -> Result<Source> {
        ensure!(
            !self.snapshots_only,
            "capability_missing: live files withheld"
        );
        let text = self.files.read(locator)?;
        let mut source = self.store.register("saved_file", locator)?;
        source.revision = digest(text.as_bytes());
        Ok(source)
    }
    pub fn editor(&mut self, identity: &str, text: &str) -> Result<Source> {
        ensure!(
            !self.snapshots_only,
            "capability_missing: live editors withheld"
        );
        let mut source = self.store.register("editor_buffer", identity)?;
        source.revision = digest(text.as_bytes());
        self.editors.0.insert(source.id.clone(), text.into());
        Ok(source)
    }
    /// Existing exact read receipts can be imported using their original full snapshot.
    /// Never reconstruct a source revision by rereading the live path later.
    pub fn snapshots(
        &mut self,
        kind: &str,
        locator: &str,
        text: &str,
        key: &str,
    ) -> Result<Vec<Node>> {
        let mut source = self.store.register(kind, locator)?;
        source.revision = digest(text.as_bytes());
        self.store
            .import_chunks(source, span(0, text.len()), text, key)
    }
    pub fn snapshot(&mut self, kind: &str, locator: &str, text: &str, key: &str) -> Result<Node> {
        let mut source = self.store.register(kind, locator)?;
        source.revision = digest(text.as_bytes());
        Ok(self
            .store
            .capture(
                &CaptureRequest {
                    idempotency_key: key.into(),
                    source,
                    span: None,
                    max_bytes: text.len() as u64,
                },
                text,
            )?
            .0)
    }
    pub fn read_receipt(
        &mut self,
        source: Source,
        snapshot: &str,
        first: usize,
        last: usize,
        returned: &str,
        key: &str,
    ) -> Result<Node> {
        ensure!(first > 0 && last >= first, "invalid returned line range");
        let start: usize = snapshot
            .split_inclusive('\n')
            .take(first - 1)
            .map(str::len)
            .sum();
        let end = start
            + snapshot
                .split_inclusive('\n')
                .skip(first - 1)
                .take(last - first + 1)
                .map(str::len)
                .sum::<usize>();
        ensure!(
            snapshot.get(start..end) == Some(returned) && !returned.is_empty(),
            "receipt differs from snapshot"
        );
        Ok(self
            .store
            .capture(
                &CaptureRequest {
                    idempotency_key: key.into(),
                    source,
                    span: Some(span(start, end)),
                    max_bytes: returned.len() as u64,
                },
                snapshot,
            )?
            .0)
    }
    pub fn call(&mut self, method: &str, params: &Value) -> Result<Value> {
        let result = self.call_inner(method, params)?;
        let limit = self
            .negotiated
            .as_ref()
            .map_or(MAX_FRAME, |n| n.max_frame_bytes as usize);
        ensure!(
            serde_json::to_vec(&result)?.len() + 128 <= limit,
            "frame_limit: request smaller captures/pages"
        );
        Ok(result)
    }
    fn call_inner(&mut self, method: &str, params: &Value) -> Result<Value> {
        if method == "context.hello" {
            let negotiated = negotiate(&Self::offer(), params)?;
            self.negotiated = Some(negotiated.clone());
            self.poll()?;
            let mut sources = self.store.sources();
            let source_count = sources.len();
            sources.truncate(negotiated.max_page_items as usize);
            return Ok(
                json!({"negotiated":negotiated,"cursor":self.store.cursor(&self.branch),
                "sources":sources,"source_count":source_count}),
            );
        }
        let operation = match method {
            "context.capture" => Operation::ContextCapture,
            "context.derive" => Operation::ContextDerive,
            "context.changes" | "context.reconcile" => Operation::ContextChanges,
            "context.compare" => Operation::ContextCompare,
            _ => bail!("capability_missing"),
        };
        let negotiated = self
            .negotiated
            .as_ref()
            .context("capability_missing: negotiate context.hello first")?;
        ensure!(
            negotiated.capabilities.contains(&operation),
            "capability_missing"
        );
        ensure!(
            serde_json::to_vec(params)?.len() < negotiated.max_frame_bytes as usize,
            "frame_limit"
        );
        if method == "context.reconcile" {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Reconcile {
                #[serde(default)]
                references: Vec<String>,
            }
            let request: Reconcile = serde_json::from_value(params.clone())?;
            ensure!(
                request.references.len() <= negotiated.max_page_items as usize,
                "invalid_page"
            );
            return self.recovery_references(&request.references);
        }
        match operation {
            Operation::ContextCapture => {
                validate(params, "CaptureRequest")?;
                let request: CaptureRequest = serde_json::from_value(params.clone())?;
                // Replay before rereading: a deleted/changed live source does not alter a retry.
                let fingerprint = digest(&serde_json::to_vec(&request)?);
                if let Some(node) = self.store.replay(&request.idempotency_key, &fingerprint)? {
                    let receipt = self
                        .store
                        .captures()
                        .iter()
                        .find(|r| r.node == node.id)
                        .unwrap();
                    return Ok(json!({"node":node,"receipt":receipt}));
                }
                ensure!(
                    !self.snapshots_only,
                    "capability_missing: live capture withheld; use approved receipts"
                );
                let known = self
                    .store
                    .data
                    .sources
                    .get(&request.source.id)
                    .context("missing_reference")?
                    .clone();
                let snapshot = match known.kind.as_str() {
                    "saved_file" => self.files.read(&known.locator)?,
                    "editor_buffer" => self
                        .editors
                        .0
                        .get(&known.id)
                        .context("editor unavailable")?
                        .clone(),
                    _ => bail!("capability_missing"),
                };
                let (node, receipt) = self.store.capture(&request, &snapshot)?;
                Ok(json!({"node":node,"receipt":receipt}))
            }
            Operation::ContextDerive => {
                validate(params, "DeriveRequest")?;
                Ok(serde_json::to_value(
                    self.store
                        .derive(&serde_json::from_value(params.clone())?)?,
                )?)
            }
            Operation::ContextCompare => {
                validate(params, "CompareRequest")?;
                Ok(serde_json::to_value(
                    self.store
                        .compare(&serde_json::from_value(params.clone())?)?,
                )?)
            }
            Operation::ContextChanges => {
                validate(params, "ChangesRequest")?;
                let request: ChangesRequest = serde_json::from_value(params.clone())?;
                ensure!(
                    request.limit <= self.negotiated.as_ref().unwrap().max_page_items,
                    "invalid_page"
                );
                self.poll()?;
                Ok(serde_json::to_value(
                    self.store.changes(&request, &self.branch)?,
                )?)
            }
            _ => bail!("capability_missing"),
        }
    }
    /// An expired cursor requires explicit reconciliation, never an empty "unchanged" page.
    pub fn recovery(&mut self) -> Result<Value> {
        let references: Vec<_> = self
            .store
            .nodes()
            .iter()
            .take(MAX_PAGE as usize)
            .map(|n| n.id.clone())
            .collect();
        self.recovery_references(&references)
    }
    pub fn recovery_references(&mut self, references: &[String]) -> Result<Value> {
        ensure!(references.len() <= MAX_PAGE as usize, "invalid_page");
        self.poll()?;
        let states: BTreeMap<_, _> = references
            .iter()
            .map(|r| Ok((r.clone(), self.store.currentness(r)?)))
            .collect::<Result<_>>()?;
        let mut source_ids = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut pending = references.to_vec();
        while let Some(reference) = pending.pop() {
            if !visited.insert(reference.clone()) {
                continue;
            }
            let node = self.store.node(&reference)?;
            if let Some(source) = &node.source {
                source_ids.insert(source.id.clone());
            }
            pending.extend(node.parents.iter().cloned());
        }
        let mut sources: Vec<_> = self
            .store
            .sources()
            .into_iter()
            .filter(|s| source_ids.contains(&s.id))
            .collect();
        let source_count = sources.len();
        sources.truncate(MAX_PAGE as usize);
        Ok(
            json!({"cursor":self.store.cursor(&self.branch),"currentness":states,"sources":sources,"source_count":source_count}),
        )
    }
}

pub fn protocol_error(error: &anyhow::Error) -> ProtocolError {
    let message = format!("{error:#}");
    let code = if let Some(failure) = error.downcast_ref::<Failure>() {
        failure.code
    } else if message.contains("invalid UTF-8 range")
        || message.contains("expected UTF-8 bytes")
        || message.contains("invalid read span")
    {
        ErrorCode::InvalidOffset
    } else if message.contains("source revision mismatch") {
        ErrorCode::DigestMismatch
    } else if message.contains("cursor_expired") {
        ErrorCode::CursorExpired
    } else if message.contains("idempotency_conflict") {
        ErrorCode::IdempotencyConflict
    } else if message.contains("missing_reference") || message.contains("unknown source identity") {
        ErrorCode::MissingReference
    } else if message.contains("capability_missing") {
        ErrorCode::CapabilityMissing
    } else if message.contains("invalid_page") {
        ErrorCode::InvalidPage
    } else if message.contains("frame_limit") {
        ErrorCode::FrameLimit
    } else {
        ErrorCode::InvalidSchema
    };
    ProtocolError {
        code,
        message,
        request_id: None,
        retryable: false,
    }
}
</book:entry>

</book:doc>
