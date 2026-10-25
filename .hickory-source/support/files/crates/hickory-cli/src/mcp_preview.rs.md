<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/mcp_preview.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Conversation edits use the client's review policy](../../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md)

Captured SHA-256: `3dd99afac0bc49845e48133bb191c146cd807950018262c6e803d9728b80203e`.

<book:entry path="crates/hickory-cli/src/mcp_preview.rs">
//! Read-only edit previews for client-owned review.
use super::*;

impl Server {
    pub(crate) fn agent_bridge(
        root: PathBuf,
        doc: Option<PathBuf>,
        executor: Arc<dyn Executor>,
        session: PathBuf,
    ) -> Self {
        Self {
            root,
            changeset: None,
            sessions: HashMap::new(),
            executor,
            params: Vec::new(),
            default_doc: doc,
            session_log: Some(session),
            debuggers: crate::debug_sessions::Registry::new(),
            context_reads: Vec::new(),
        }
    }
    pub(crate) fn agent_set_changeset(&mut self, contract: crate::changeset::Contract) {
        self.changeset = Some(contract);
    }
    pub(crate) fn agent_begin_changeset(&mut self) -> Result<()> {
        let (scope, starting) =
            crate::workspace_capture::WorkspaceCapture::freeze(Some(self.root.clone()), &[])?;
        let mut contract = crate::changeset::Contract::new(
            self.root.clone(),
            hick_lang::ProjectInventory::default(),
            starting,
            format!("{:016x}", crate::serve::rand_id()),
        );
        contract.workspace = Some((scope, Arc::new(crate::serve::acp::edits::Edits::default())));
        self.changeset = Some(contract);
        Ok(())
    }
    pub(crate) fn agent_completion(&mut self) -> Result<crate::changeset::Artifact> {
        self.changeset
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Missing changeset boundary"))?
            .completion()
    }
    pub(crate) async fn agent_revision(
        &mut self,
        args: &Value,
    ) -> Result<(String, Vec<Value>), String> {
        let doc = self.resolve_doc(args)?;
        let session = self.session_for(&doc).await?;
        session.refresh_source().await?;
        Ok((session.revision(), session.source_snapshots()))
    }

    pub(crate) fn set_default_doc(&mut self, doc: PathBuf) {
        self.default_doc = Some(doc);
    }

    pub(crate) async fn preview_edit(
        &mut self,
        name: &str,
        args: &Value,
    ) -> Result<(String, String, String), String> {
        if name == "create_doc" {
            // Match application normalization before requesting approval.
            let path = args["path"]
                .as_str()
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .ok_or("Pass a document path.")?;
            let mut path = PathBuf::from(path);
            if path.extension().is_none_or(|e| e != "md") {
                path.set_extension("md");
            }
            if self.root.join(&path).exists() {
                return Err("This document already exists; use edit_doc.".into());
            }
            let input = args["input"].as_str().unwrap_or("");
            let input = if input.is_empty() || input.ends_with('\n') {
                input.into()
            } else {
                format!("{input}\n")
            };
            return Ok((path.display().to_string(), String::new(), input));
        }
        let doc = self.resolve_doc(args)?;
        let inv = Self::invocation(name, args)?;
        self.session_for(&doc).await?.preview_edit(&inv).await
    }
}
</book:entry>

</book:doc>
