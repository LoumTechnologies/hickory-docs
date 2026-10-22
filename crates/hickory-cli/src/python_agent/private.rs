//! Private mode reuses the engine's captured EditSession and the existing human review.
use super::*;

impl Host<'_> {
    async fn private_proposal(&mut self, args: &Value) -> Result<Proposal> {
        let value = self
            .private
            .as_ref()
            .context("Private binding missing")?
            .private_request(
                reqwest::Method::POST,
                "preview",
                &json!({"name":"edit_doc","arguments":args}),
            )
            .await?;
        Ok(Proposal {
            tool: "edit_doc".into(),
            arguments: args.clone(),
            surface: value["surface"]
                .as_str()
                .context("Missing captured surface")?
                .into(),
            before: value["before"]
                .as_str()
                .context("Missing captured source")?
                .into(),
            after: value["after"]
                .as_str()
                .context("Missing proposed source")?
                .into(),
            revision: Some(
                value["revision"]
                    .as_str()
                    .context("Missing captured revision")?
                    .into(),
            ),
            sources: serde_json::from_value(value["sources"].clone())?,
        })
    }
    pub(super) async fn private_tool(&mut self, params: &Value) -> Result<Value> {
        self.tools += 1;
        ensure!(
            self.tools <= self.config.max_turns * 64,
            "Tool budget exhausted"
        );
        let name = params["name"].as_str().context("Tool name required")?;
        ensure!(
            ["read_doc", "edit_doc", "seal_prediction", "verify"].contains(&name),
            "Tool withheld in private prediction mode"
        );
        let mut args = params.get("arguments").cloned().unwrap_or(json!({}));
        ensure!(args.is_object(), "Tool arguments must be an object");
        if name == "edit_doc" {
            let proposal = self.private_proposal(&args).await?;
            metadata(
                &self.session,
                "python-edit-proposal",
                &serde_json::to_value(&proposal)?,
            )?;
            let approved = self.review.approve(&proposal).await?;
            metadata(
                &self.session,
                "python-edit-review",
                &json!({"actor":"user-declared","approved":approved,"proposal":proposal}),
            )?;
            if !approved {
                return Ok(
                    json!({"isError":true,"content":[{"type":"text","text":"Human declined the proposed edit; no edit ran."}]}),
                );
            }
            ensure!(
                proposal == self.private_proposal(&args).await?,
                "Proposal changed during approval; no edit ran"
            );
            if let Some(after) = self.review.replacement() {
                args["input"] = json!(after);
                args["whole"] = json!("true");
                args["snapshot"] =
                    json!(hickory_agent::hashline::LineIndex::new(&proposal.before).snapshot);
                self.private_proposal(&args).await?;
                metadata(
                    &self.session,
                    "python-reviewed-replacement",
                    &json!({"actor":"user-declared","arguments":args}),
                )?;
            }
        }
        let result = self
            .private
            .as_ref()
            .unwrap()
            .call(&json!({"name":name,"arguments":args}))
            .await?;
        metadata(
            &self.session,
            "python-private-tool",
            &json!({"name":name,"result":result}),
        )?;
        // These are bounded, policy-filtered receipts, never live saved_file adapters.
        let references = self
            .context_store
            .snapshots(
                "private_tool_receipt",
                name,
                &result.to_string(),
                &hick_context_store::fresh_id(),
            )?
            .into_iter()
            .map(|node| node.id)
            .collect::<Vec<_>>();
        let mut result = result;
        result["context_references"] = json!(references);
        Ok(result)
    }
    pub(super) async fn private_complete(&mut self, summary: &str) -> Result<Value> {
        let history = self
            .private
            .as_ref()
            .unwrap()
            .private_request(reqwest::Method::GET, "history", &json!({}))
            .await?;
        metadata(
            &self.session,
            "python-private-completion",
            &json!({"model_summary":summary,
            "history":history,"current_certification":false,"ordinary_changeset":false}),
        )?;
        self.summary = format!(
            "Private prediction investigation finished; no current certification or ordinary changeset acceptance is claimed. Model summary:\n\n{summary}"
        );
        append(
            &self.session,
            SessionEvent::Assistant {
                prose: &self.summary,
                action: None,
                reasoning: None,
            },
        )?;
        (self.on_event)(AgentEvent::ResponseComplete {
            text: self.summary.clone(),
        });
        (self.on_event)(AgentEvent::Done {
            summary: self.summary.clone(),
        });
        self.finished = true;
        Ok(json!({"accepted":true,"current_certification":false,"ordinary_changeset":false}))
    }
}
