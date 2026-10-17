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
                let doc = self.resolve_doc(args).map_err(anyhow::Error::msg)?;
                self.changeset = Some(crate::changeset::Contract::capture(
                    &doc,
                    format!("{:016x}", crate::serve::rand_id()),
                )?);
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
