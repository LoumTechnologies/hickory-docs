<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/selected_run.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-208e04c23a9d"></a>
## `crates/hickory-cli/src/serve/selected_run.rs`

<a id="f-208e04c23a9d-impl-LocalState"></a>
### `impl-LocalState`

<storycode:copy id="f-208e04c23a9d-impl-LocalState">
use super::*;

impl LocalState {
    /// Execute a document, publishing transcript events and the final status
    /// on the run channel. Returns the run id immediately.
    pub async fn start_run(&self, id: &str, check_only: bool) -> ApiResult<String> {
        self.start_run_expected(id, check_only, None).await
    }

    pub async fn start_run_expected(
        &self,
        id: &str,
        check_only: bool,
        expected: Option<String>,
    ) -> ApiResult<String> {
        self.start_run_selected(id, check_only, expected, None)
            .await
    }

    pub async fn start_run_selected(
        &self,
        id: &str,
        check_only: bool,
        expected: Option<String>,
        cells: Option<Vec<String>>,
    ) -> ApiResult<String> {
        let partial = cells.as_ref().is_some_and(|ids| !ids.is_empty());
        if partial && check_only {
            return Err(ApiError::unprocessable(
                "Selected-cell verification is unsupported; check the whole document.",
            ));
        }
        if partial && self.private_prediction.is_some() {
            return Err(ApiError::unprocessable(
                "Selected-cell runs are unsupported for private prediction documents.",
            ));
        }
        if let Some(bound) = &self.private_prediction {
            return bound.start(self, id, expected);
        }
        let path = self
            .index
            .absolute(id)
            .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;
        let run_id = format!("{:016x}", rand_id());
        let started_at = now_rfc3339();

        self.runs.lock().unwrap().insert(
            run_id.clone(),
            RunRecord {
                document: id.to_string(),
                check_only,
                requested_source_hash: expected.clone(),
                error: None,
                source_hash: None,
                status: "running".into(),
                started_at: started_at.clone(),
                blocks: json!([]),
            },
        );

        let state = self.clone();
        let doc_key = id.to_string();
        let run_key = run_id.clone();
        tokio::spawn(async move {
            let _write = state.writes.lock().await;
            let mode = if check_only {
                RunMode::Verify
            } else {
                RunMode::Execute
            };
            let before = match &state.engine_woven {
                Some(woven) => Some(woven.lock().await.snapshot()),
                None => None,
            };
            let outcome = crate::selected_run::run_doc_selected(
                &path,
                &state.params,
                mode,
                state.executor,
                expected.as_deref(),
                cells,
            )
            .await;

            let failure = outcome
                .as_ref()
                .err()
                .map(crate::selected_run::failure_message);
            let failed_source = outcome
                .as_ref()
                .err()
                .and_then(|error| error.downcast_ref::<crate::CapturedRootError>())
                .map(|error| error.source_hash.clone());
            let (status, blocks, source_hash) = match outcome {
                Ok(run) => {
                    // Replay each cell's transcript on the run channel, the
                    // same shape the hosted server publishes.
                    for (exec_id, entries) in &run.result.transcripts {
                        for event in entries.iter().flat_map(|entry| &entry.events) {
                            state
                                .rooms
                                .publish_run_event(
                                    &doc_key,
                                    &json!({
                                        "run_id": run_key,
                                        "exec_id": exec_id,
                                        "event": event,
                                    }),
                                )
                                .await;
                        }
                    }
                    if !check_only && let Err(e) = crate::write_outputs(&run, None) {
                        log::error!("writing outputs failed: {e:#}");
                    }
                    let output_failed = !check_only
                        && crate::output::refresh_outputs(&run)
                            .inspect_err(|e| {
                                log::error!("saving cell outputs failed: {e:#}");
                            })
                            .is_err();
                    let failed = output_failed || run.result.expectations.iter().any(|o| !o.passed);
                    let blocks = crate::block_model_json(&run)
                        .map(|v| v.get("blocks").cloned().unwrap_or(json!([])))
                        .unwrap_or(json!([]));
                    (
                        if failed { "failed" } else { "ok" },
                        blocks,
                        Some(hick_context_protocol::digest(run.source.as_bytes())),
                    )
                }
                Err(e) => {
                    log::warn!("run {run_key} failed: {e:#}");
                    let blocks = e
                        .downcast_ref::<crate::CapturedRootError>()
                        .and_then(|captured| {
                            captured
                                .error
                                .downcast_ref::<crate::failed_run::FailedRun>()
                        })
                        .map(|failed| json!(failed.blocks))
                        .unwrap_or_else(|| json!([]));
                    ("failed", blocks, failed_source)
                }
            };

            if let (Some(woven), Some(before)) = (&state.engine_woven, before) {
                let docs = woven.lock().await.adopt_changed(&before);
                if let Some(tx) = state.up_commands.lock().unwrap().as_ref() {
                    for doc in docs {
                        let _ = tx.send(crate::up::UpCommand::Reweave(doc));
                    }
                }
                let history = woven.lock().await;
                crate::serve::watch::reconcile_rooms(
                    &state,
                    &state.index.root().to_path_buf(),
                    &history,
                )
                .await;
            }
            if let Some(record) = state.runs.lock().unwrap().get_mut(&run_key) {
                record.status = status.to_string();
                record.source_hash = source_hash;
                record.error = failure;
                record.blocks = blocks;
            }
            state
                .rooms
                .publish_run_event(&doc_key, &json!({ "run_id": run_key, "status": status }))
                .await;
        });

        Ok(run_id)
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/selected_run.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/selected_run.rs"><storycode:paste select="#f-208e04c23a9d-impl-LocalState"/></storycode:file>

</storycode:doc>
