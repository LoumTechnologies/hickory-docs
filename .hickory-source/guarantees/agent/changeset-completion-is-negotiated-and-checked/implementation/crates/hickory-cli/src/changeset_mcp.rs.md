<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/changeset_mcp.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fd880fd957c2"></a>
## `crates/hickory-cli/src/changeset_mcp.rs`

Related contracts: [Finish an agent turn against its captured project boundary](../../../../contract.md).

<a id="f-fd880fd957c2-impl-Server"></a>
### `impl-Server`

Standalone MCP task boundaries; the dock bridge binds them to a turn instead.

<storycode:copy id="f-fd880fd957c2-impl-Server">
//! Standalone MCP task boundaries; the dock bridge binds them to a turn instead.
use super::*;

impl Server {
    pub(super) fn call_changeset(&mut self, name: &str, args: &Value) -> Value {
        if let Some(path) = &self.session_log
            && let Err(error) = crate::doc_tools::record_intent(path, name, args)
        {
            return text_result(&format!("Recording failed; no tool ran: {error:#}"), true);
        }
        let result = (|| -> anyhow::Result<Value> {
            if name == "begin_changeset" {
                anyhow::ensure!(
                    self.changeset.is_none(),
                    "A starting boundary is already frozen; use changeset_status. Restart the MCP server for a new task."
                );
                if let Some(workspace) = args["workspace"].as_str() {
                    let path = std::path::Path::new(workspace);
                    let root = if path.is_absolute() {
                        path.to_path_buf()
                    } else {
                        self.root.join(path)
                    }
                    .canonicalize()?;
                    let (scope, starting) = crate::workspace_capture::WorkspaceCapture::freeze(
                        Some(root.clone()),
                        &[],
                    )?;
                    let mut contract = crate::changeset::Contract::new(
                        root,
                        hick_lang::ProjectInventory::default(),
                        starting,
                        format!("{:016x}", crate::serve::rand_id()),
                    );
                    contract.workspace = Some((
                        scope,
                        std::sync::Arc::new(crate::serve::acp::edits::Edits::default()),
                    ));
                    self.changeset = Some(contract);
                } else {
                    // The engine's cwd can belong to a different client.
                    let doc = self
                        .root
                        .join(self.resolve_doc(args).map_err(anyhow::Error::msg)?);
                    self.changeset = Some(crate::changeset::Contract::capture(
                        &doc,
                        format!("{:016x}", crate::serve::rand_id()),
                    )?);
                }
            }
            self.changeset
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("Call begin_changeset before making changes."))?
                .call(
                    if name == "begin_changeset" {
                        "changeset_status"
                    } else {
                        name
                    },
                    args,
                )
        })();
        let newly_accepted =
            name == "submit_changeset" && result.as_ref().is_ok_and(|v| v["accepted"] == true);
        let reply = self.record_activity(
            name,
            result
                .and_then(|v| {
                    if name == "submit_changeset" && v["accepted"] == false {
                        anyhow::bail!(v.to_string());
                    }
                    Ok(v.to_string())
                })
                .map_err(|e| format!("{e:#}")),
        );
        if newly_accepted
            && reply["isError"] == true
            && let Some(contract) = &mut self.changeset
        {
            contract.accepted = None;
        }
        reply
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/changeset_mcp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/changeset_mcp.rs"><storycode:paste select="#f-fd880fd957c2-impl-Server"/></storycode:file>

</storycode:doc>
