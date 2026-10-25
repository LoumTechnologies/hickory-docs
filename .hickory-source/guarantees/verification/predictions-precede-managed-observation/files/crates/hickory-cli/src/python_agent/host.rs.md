<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/python_agent/host.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `dbc587a352a1b8690990df13f493ebe9cf1d7f72ac99e857ad3fc55dfa2367ab`.

<book:entry path="crates/hickory-cli/src/python_agent/host.rs">
use super::*;

impl Host<'_> {
    fn record_ui_verifications(&mut self) -> Result<()> {
        for record in self.review.take_ui_verifications() {
            metadata(&self.session, "verification", &record)?;
            self.ui_checks_recorded = true;
        }
        Ok(())
    }
    async fn proposal(&mut self, name: &str, args: &Value) -> Result<Proposal> {
        let (surface, before, after) = self
            .server
            .preview_edit(name, args)
            .await
            .map_err(anyhow::Error::msg)?;
        let (revision, sources) = if name == "create_doc" {
            (None, vec![])
        } else {
            let (revision, sources) = self
                .server
                .agent_revision(args)
                .await
                .map_err(anyhow::Error::msg)?;
            (Some(revision), sources)
        };
        let doc = args
            .get("doc")
            .and_then(Value::as_str)
            .map(|d| self.config.root.join(d))
            .or_else(|| self.config.doc.clone());
        if name == "create_doc" {
            let path = self.config.root.join(&surface);
            contained(
                &self.config.root,
                path.parent().context("No document directory")?,
            )?;
            validate_source(&self.config.root, &path, &after, &mut vec![])?;
        } else if name == "edit_doc" {
            let path = if args.get("upstream").is_some() {
                self.config.root.join(&surface)
            } else {
                doc.context("No document")?
            };
            validate_source(&self.config.root, &path, &after, &mut vec![])?;
        }
        Ok(Proposal {
            tool: name.into(),
            arguments: args.clone(),
            surface,
            before,
            after,
            revision,
            sources,
        })
    }
    async fn tool(&mut self, params: &Value) -> Result<Value> {
        if self.private.is_some()
            && !matches!(
                params["name"].as_str(),
                Some("ui_observe" | "ui_act" | "ui_cancel")
            )
        {
            return self.private_tool(params).await;
        }
        self.tools += 1;
        ensure!(
            self.tools <= self.config.max_turns * 64,
            "Tool budget exhausted"
        );
        let name = params["name"]
            .as_str()
            .context("Tool name must be a string")?;
        ensure!(
            allowed(name),
            "Tool {name} is not exposed by the Python agent"
        );
        if matches!(name, "ui_observe" | "ui_act" | "ui_cancel") {
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            metadata(
                &self.session,
                "python-ui-request",
                &json!({"actor":"agent","tool":name,"arguments":arguments}),
            )?;
            let mut result = self.review.ui_call(name, arguments).await?;
            let capture = hick_context_store::fresh_id();
            let mut image_references = vec![];
            let mut images = vec![];
            if let Some(content) = result["content"].as_array_mut() {
                ensure!(
                    content
                        .iter()
                        .filter(|item| item["type"] == "image")
                        .count()
                        <= 1,
                    "One captured image per UI read"
                );
                for image in content.iter().filter(|item| item["type"] == "image") {
                    let media = image["mimeType"]
                        .as_str()
                        .context("Image media type missing")?;
                    let data = image["data"].as_str().context("Image data missing")?;
                    let bytes = crate::serve::ui_images::decode(media, data)?;
                    let node = self.context_store.store.import_artifact(
                        "ui_image",
                        &capture,
                        media,
                        &bytes,
                        &format!("{capture}/image"),
                    )?;
                    image_references.push(node.id.clone());
                    images.push(json!({"reference":node.id,"artifact":node.artifact}));
                }
                content.retain(|item| item["type"] != "image");
            }
            if !images.is_empty() {
                result["images"] = json!(images);
            }
            let mut references = self
                .context_store
                .snapshots("ui_observation", &capture, &result.to_string(), &capture)?
                .into_iter()
                .map(|node| node.id)
                .collect::<Vec<_>>();
            references.extend(image_references);
            result["context_references"] = json!(references);
            return Ok(result);
        }
        if matches!(name, "changeset_status" | "submit_changeset") {
            self.review.refresh().await?;
        }
        let mut args = params.get("arguments").cloned().unwrap_or(json!({}));
        ensure!(args.is_object(), "Tool arguments must be an object");
        let doc = args
            .get("doc")
            .and_then(Value::as_str)
            .map(|d| self.config.root.join(d))
            .or_else(|| self.config.doc.clone());
        if matches!(
            name,
            "read_doc" | "read_output" | "read_file" | "edit_doc" | "edit_output"
        ) {
            let path = contained(
                &self.config.root,
                &doc.context("Pass doc or start with --doc")?,
            )?;
            validate_source(
                &self.config.root,
                &path,
                &std::fs::read_to_string(&path)?,
                &mut vec![],
            )?;
            if name == "read_file" {
                let file = args["path"].as_str().context("Pass a file path")?;
                contained(&self.config.root, &path.parent().unwrap().join(file))?;
            }
        }
        let mut created = None;
        if matches!(name, "edit_doc" | "edit_output" | "create_doc") {
            let proposal = self.proposal(name, &args).await?;
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
                proposal == self.proposal(name, &args).await?,
                "Proposal changed during approval; no edit ran"
            );
            if let Some(replacement) = self.review.replacement() {
                if name == "create_doc" {
                    args["input"] = json!(replacement);
                } else {
                    args["input"] = json!(replacement);
                    args["whole"] = json!("true");
                    args["snapshot"] =
                        json!(hickory_agent::hashline::LineIndex::new(&proposal.before).snapshot);
                }
                self.proposal(name, &args).await?;
                metadata(
                    &self.session,
                    "python-reviewed-replacement",
                    &json!({"actor":"user-declared","arguments":args}),
                )?;
            }
            self.unchecked_edits = true;
            if name == "create_doc" {
                let effective = self.proposal(name, &args).await?;
                created = Some((effective.surface, effective.after));
            }
        }
        let mut result = self
            .server
            .handle("tools/call", &json!({"name":name,"arguments":args}))
            .await
            .map_err(|(_, text)| anyhow::anyhow!(text))?;
        {
            let mut references = Vec::new();
            for read in self.server.take_context_reads() {
                let text = &read.content;
                let path = read
                    .source_path
                    .as_deref()
                    .unwrap_or_else(|| Path::new(&read.path));
                let absolute = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    self.config.root.join(path)
                };
                let locator = absolute.strip_prefix(&self.config.root)?.to_string_lossy();
                let kind = if name == "read_output" {
                    "generated_output"
                } else {
                    "saved_file"
                };
                let mut source = self.context_store.store.register(kind, &locator)?;
                source.revision = read.sha256;
                let start = read.byte_span.0;
                let end = read.byte_span.1;
                let nodes = self.context_store.store.import_chunks_anchored(
                    source,
                    hick_context_store::span(start, end),
                    text,
                    &hick_context_store::fresh_id(),
                    read.unique,
                )?;
                references.extend(nodes.into_iter().map(|node| node.id));
            }
            result["context_references"] = json!(references);
        }
        if result["isError"] != true
            && let Some((file, expected)) = created
        {
            let content = std::fs::read_to_string(contained(
                &self.config.root,
                &self.config.root.join(&file),
            )?)?;
            ensure!(
                content == expected,
                "Created document changed before its write receipt; actual state remains captured"
            );
            let index = hickory_agent::hashline::LineIndex::new(&content);
            let wrote = hickory_agent::Wrote {
                file,
                first_line: 1,
                last_line: index.lines.len(),
                hashes: index.hashes,
                fingerprints: index
                    .lines
                    .iter()
                    .map(|line| hickory_agent::hashline::line_fingerprint(line))
                    .collect(),
            };
            append(&self.session, SessionEvent::Wrote { wrote: &wrote })?;
        }
        Ok(result)
    }
    pub(super) async fn handle(&mut self, method: &str, params: &Value) -> Result<Value> {
        if let Some(remote) = &self.private {
            ensure!(
                remote.private_mode().await?,
                "Private prediction binding lost; no ordinary fallback"
            );
        } else {
            crate::engine::require_ordinary_agent_context_at(&self.config.root).await?;
        }
        ensure!(!self.finished, "The model already stopped");
        if method == "context.updates" && self.private.is_some() {
            return Ok(json!([]));
        }
        if method == "context.updates" {
            let branch = self.turn_id.as_deref().unwrap_or("standalone");
            let updates = if let Some(inbox) = self.review.context_inbox() {
                let mut inbox = inbox.lock().unwrap();
                let updates = inbox.take(branch);
                inbox.save()?;
                updates
            } else {
                vec![]
            };
            let mut result = vec![];
            for update in updates {
                let references = if let Some(text) = &update.text {
                    self.context_store
                        .snapshots(
                            &update.source.kind,
                            &update.source.locator,
                            text,
                            &update.id,
                        )?
                        .into_iter()
                        .map(|n| n.id)
                        .collect::<Vec<_>>()
                } else {
                    vec![update.old.clone()]
                };
                result.push(json!({"update":update,"references":references}));
            }
            return Ok(json!(result));
        }
        if method == "context.references" {
            let ids: Vec<String> = serde_json::from_value(params["references"].clone())?;
            ensure!(ids.len() <= 256, "Reference budget exhausted");
            let mut pending = ids;
            let mut found = std::collections::BTreeSet::new();
            while let Some(id) = pending.pop() {
                if found.insert(id.clone()) {
                    pending.extend(self.context_store.store.node(&id)?.parents.clone());
                    for receipt in self
                        .context_store
                        .store
                        .productions()
                        .iter()
                        .filter(|p| p.output == id)
                    {
                        pending.push(receipt.response.clone());
                    }
                }
            }
            return Ok(
                json!({"productions":self.context_store.store.productions().iter().filter(|p| found.contains(&p.output)).collect::<Vec<_>>(),"nodes":self.context_store.store.nodes().iter()
                .filter(|n| found.contains(&n.id)).collect::<Vec<_>>()}),
            );
        }
        if method.starts_with("context.") {
            if self.private.is_none()
                && matches!(
                    method,
                    "context.hello" | "context.capture" | "context.changes" | "context.reconcile"
                )
            {
                self.context_store.editors.0.clear();
                // A missing/timed-out editor feed cannot certify unchanged. File heads
                // remain independently checkable, and old editor captures remain intact.
                if let Ok(Some(heads)) = self.review.editor_heads().await {
                    let sources = self.context_store.store.sources();
                    for head in heads {
                        if let Some(source) = sources
                            .iter()
                            .find(|s| s.kind == "editor_buffer" && s.locator == head.identity)
                        {
                            self.context_store
                                .editors
                                .0
                                .insert(source.id.clone(), head.content);
                        }
                    }
                }
            }
            return self.context_store.call(method, params);
        }
        match method {
            "inference" => {
                // Transport/resource limits only. Never add prompts, replay history,
                // retry a provider request, parse an action, or choose the next step.
                ensure!(self.turns < 600, "Inference resource budget exhausted");
                let mut manifest = params["manifest"].clone();
                // Resolve against retained host nodes, never trust worker-supplied
                // snapshots or fabricated source identity in the manifest.
                manifest["nodes"] = serde_json::to_value(self.context_store.store.nodes())?;
                manifest["captures"] = serde_json::to_value(self.context_store.store.captures())?;
                ensure!(
                    manifest["deliveries"] == json!([]),
                    "Worker cannot attest delivery"
                );
                manifest["productions"] =
                    serde_json::to_value(self.context_store.store.productions())?;
                let bundle = hick_context_protocol::validate_bundle(&manifest)?;
                let request_value = serde_json::to_value(&bundle.request)?;
                let fingerprint = hick_context_protocol::assembly_digest(&request_value);
                if let Some((old, result)) = self.deliveries.get(&bundle.request.id) {
                    ensure!(old == &fingerprint, "idempotency_conflict");
                    return result
                        .clone()
                        .context("Request already attempted; delivery may be uncertain");
                }
                ensure!(
                    bundle.request.branch == self.turn_id.as_deref().unwrap_or("standalone"),
                    "Request branch mismatch"
                );
                let messages: Vec<Message> = bundle
                    .request
                    .parts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        ensure!(p.slot == i.to_string(), "Unsupported request slot");
                        if let Some(artifact) = &p.artifact {
                            ensure!(p.role == "user", "Image material must have user role");
                            let bytes = self.context_store.store.artifact_bytes(artifact)?;
                            use base64::Engine;
                            let data = base64::engine::general_purpose::STANDARD.encode(bytes);
                            crate::serve::ui_images::decode(&artifact.media_type, &data)?;
                            Ok(Message {
                                role: hickory_agent::Role::User,
                                content: String::new(),
                                images: vec![hickory_agent::Image {
                                    media_type: artifact.media_type.clone(),
                                    data,
                                }],
                            })
                        } else {
                            Ok(serde_json::from_value(
                                json!({"role":p.role,"content":p.text}),
                            )?)
                        }
                    })
                    .collect::<Result<_>>()?;
                self.deliveries
                    .insert(bundle.request.id.clone(), (fingerprint.clone(), None));
                let mut delivery = json!({"request":bundle.request.id,"attempt":"1",
                    "idempotency_key":bundle.request.id,"assembly_digest":fingerprint,"state":"prepared"});
                metadata(&self.session, "python-context-manifest", &manifest)?;
                metadata(&self.session, "python-context-delivery", &delivery)?;
                context_delivery(self.review.context_inbox(), &bundle.request, &delivery);
                ensure!(
                    serde_json::to_vec(&messages)?.len() <= 800_000,
                    "Inference request exceeds transport budget"
                );
                metadata(
                    &self.session,
                    "python-model-request",
                    &json!({"messages":messages,"backend":"python","turn":self.turn_id,
                    "iteration":self.turns,"model":self.client.model_name()}),
                )?;
                (self.on_event)(AgentEvent::Thinking);
                use futures::StreamExt;
                let request = async {
                    let mut stream = self.client.complete_stream(messages).await?;
                    let mut raw = String::new();
                    let mut usage = Usage::default();
                    let mut reasoning = String::new();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk?;
                        raw.push_str(&chunk.delta);
                        ensure!(
                            raw.len() <= MAX_FRAME / 2,
                            "Model response exceeds protocol budget"
                        );
                        if !chunk.reasoning.is_empty() {
                            reasoning.push_str(&chunk.reasoning);
                            ensure!(
                                reasoning.len() <= MAX_FRAME / 2,
                                "Reasoning exceeds protocol budget"
                            );
                            (self.on_event)(AgentEvent::Reasoning {
                                data: chunk.reasoning,
                            });
                        }
                        if !chunk.delta.is_empty() {
                            (self.on_event)(AgentEvent::Token { data: chunk.delta });
                        }
                        if let Some(value) = chunk.usage {
                            usage.add(&value);
                        }
                    }
                    Ok::<_, anyhow::Error>((raw, usage, reasoning))
                };
                delivery["state"] = json!("transport_started");
                metadata(&self.session, "python-context-delivery", &delivery)?;
                context_delivery(self.review.context_inbox(), &bundle.request, &delivery);
                self.active_delivery = Some(delivery.clone());
                let outcome = tokio::time::timeout(Duration::from_secs(180), request).await;
                let (raw, usage, reasoning) = match outcome {
                    Ok(Ok(value)) => {
                        self.private_failure = None;
                        value
                    }
                    result => {
                        delivery["state"] = json!(if result.is_err() {
                            "cancelled_uncertain"
                        } else {
                            "failed"
                        });
                        metadata(&self.session, "python-context-delivery", &delivery)?;
                        context_delivery(self.review.context_inbox(), &bundle.request, &delivery);
                        self.active_delivery = None;
                        return match result {
                            Ok(Err(error)) => {
                                if self.private.is_some() {
                                    self.private_failure =
                                        Some(PrivateFailure::provider(&error.to_string()));
                                }
                                Err(error)
                            }
                            Err(error) => {
                                if self.private.is_some() {
                                    self.private_failure =
                                        Some(PrivateFailure::ProviderUnavailable);
                                }
                                Err(error.into())
                            }
                            _ => unreachable!(),
                        };
                    }
                };
                // Receipt attests only that this host observed the completed stream;
                // the adapter exposes no provider-assigned ID or hidden context.
                let inputs = bundle
                    .request
                    .parts
                    .iter()
                    .flat_map(|p| p.segments.iter().map(|s| s.node.clone()))
                    .collect();
                let response_node =
                    self.context_store
                        .store
                        .model_response(&bundle.request.id, inputs, &raw)?;
                delivery["state"] = json!("provider_confirmed");
                delivery["provider_receipt"] =
                    json!(format!("host-observed-response:{}", response_node.id));
                metadata(&self.session, "python-context-delivery", &delivery)?;
                context_delivery(self.review.context_inbox(), &bundle.request, &delivery);
                if let Some(inbox) = self.review.context_inbox() {
                    let mut inbox = inbox.lock().unwrap();
                    let mut exposures = crate::context_updates::exposures(&manifest);
                    for exposure in &mut exposures {
                        exposure.unique =
                            self.context_store.store.unique_anchor(&exposure.reference);
                    }
                    inbox.publish(&bundle.request.branch, exposures);
                    inbox.save()?;
                }
                self.active_delivery = None;
                let result = json!({"text":raw,"reference":response_node.id});
                self.deliveries.get_mut(&bundle.request.id).unwrap().1 = Some(result.clone());
                metadata(
                    &self.session,
                    "python-model-response",
                    &json!({"actor":"model-declared","raw":raw,"reasoning":reasoning}),
                )?;
                if !reasoning.is_empty() {
                    append(
                        &self.session,
                        SessionEvent::Assistant {
                            prose: "",
                            action: None,
                            reasoning: Some(&reasoning),
                        },
                    )?;
                }
                self.usage.add(&usage);
                (self.on_event)(AgentEvent::TurnUsage {
                    turn: self.turns,
                    usage,
                    cost_usd: hickory_agent::cost_usd(self.client.model_name(), &usage),
                    total_usage: self.usage,
                    total_cost_usd: hickory_agent::cost_usd(self.client.model_name(), &self.usage),
                });
                append(
                    &self.session,
                    SessionEvent::Usage {
                        turn: Some(self.turns),
                        usage,
                        cost_usd: None,
                    },
                )?;
                self.turns += 1;
                Ok(result)
            }
            "record" => {
                let kind = params["kind"].as_str().context("Missing record kind")?;
                ensure!(
                    matches!(
                        kind,
                        "action" | "observation" | "model-error" | "context-prepared"
                    ),
                    "Unknown worker record kind"
                );
                metadata(
                    &self.session,
                    &format!("python-{kind}"),
                    &json!({"actor":"worker-reported","value":params["data"]}),
                )?;
                Ok(json!({}))
            }
            "tool" => {
                let name = params["name"].as_str().unwrap_or("unknown").to_string();
                (self.on_event)(AgentEvent::ToolStarted {
                    name: name.clone(),
                    data: params.to_string(),
                });
                let mut result = self.tool(params).await;
                let ok = result.as_ref().is_ok_and(|value| value["isError"] != true);
                if let Err(error) = self.review.completed(ok).await {
                    result = Err(error);
                }
                self.record_ui_verifications()?;
                metadata(
                    &self.session,
                    "python-tool-result",
                    &json!({
                        "actor":"host-observed", "name":name,
                        "result":result.as_ref().ok(), "error":result.as_ref().err().map(|e| format!("{e:#}"))
                    }),
                )?;
                (self.on_event)(AgentEvent::ToolFinished {
                    name,
                    ok: ok && result.is_ok(),
                    text: match &result {
                        Ok(value) => value.to_string(),
                        Err(error) => format!("{error:#}"),
                    },
                });
                result
            }
            "complete" => {
                let summary = params["summary"]
                    .as_str()
                    .context("Missing completion summary")?;
                if self.private.is_some() {
                    return self.private_complete(summary).await;
                }
                self.review.refresh().await?;
                self.record_ui_verifications()?;
                // Independent byte verification, never a model assertion.
                let artifact = self.server.agent_completion()?;
                metadata(
                    &self.session,
                    "python-verified-changeset",
                    &serde_json::to_value(artifact)?,
                )?;
                metadata(
                    &self.session,
                    "python-completion",
                    &json!({"actor":"harness","model_summary":summary,"model_stopped":true,"verification":"source-output-equality-only","runtime_checks":if self.ui_checks_recorded {"See separately recorded document check receipts; changeset-wide coverage is not established"} else {"No completed runtime check recorded"},"unchecked_edits":self.unchecked_edits,"assumption_coverage":"Not established"}),
                )?;
                let checks = if self.ui_checks_recorded {
                    "Document checks recorded separately; changeset-wide runtime coverage not established"
                } else {
                    "no completed runtime check recorded"
                };
                self.summary = format!(
                    "Changeset verified (source-output equality); {checks}. Model summary:\n\n{summary}"
                );
                append(
                    &self.session,
                    SessionEvent::Assistant {
                        prose: &self.summary,
                        action: None,
                        reasoning: None,
                    },
                )?;
                (self.on_event)(AgentEvent::Token {
                    data: self.summary.clone(),
                });
                (self.on_event)(AgentEvent::ResponseComplete {
                    text: self.summary.clone(),
                });
                (self.on_event)(AgentEvent::Done {
                    summary: self.summary.clone(),
                });
                self.finished = true;
                Ok(json!({"accepted":true}))
            }
            _ => bail!("Unknown agent protocol method {method}"),
        }
    }
}
</book:entry>

</book:doc>
