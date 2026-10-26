<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/tools/verify.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-913f8d4d5670"></a>
## `crates/hickory-agent/src/tools/verify.rs`

Related contracts: [Verify Covers the Whole Editable Set](../../../../../guarantees/agent/verify-covers-the-whole-editable-set/contract.md).

<a id="f-913f8d4d5670-UpstreamReport"></a>
### `UpstreamReport`

Execution and complete project publication for the same edit closure.

<storycode:copy id="f-913f8d4d5670-UpstreamReport">
//! Execution and complete project publication for the same edit closure.
use super::*;

struct UpstreamReport {
    failures: Vec<String>,
    note: String,
    pending: Vec<(PathBuf, hick_literate::PipelineResult)>,
}
</storycode:copy>
<a id="f-913f8d4d5670-impl-EditSession"></a>
### `impl-EditSession`

<storycode:copy id="f-913f8d4d5670-impl-EditSession">
impl EditSession {
    /// Opt into publishing generated files only after the entire editable scope
    /// passes expectations. This is a publication policy, NOT prediction strict
    /// mode: ordinary diagnostics and executor side effects remain observable.
    /// Hosts choose the policy; no model-supplied tool argument can disable it.
    pub fn require_verified_publication(mut self) -> Self {
        self.verified_publication = true;
        self
    }

    pub(super) async fn verify(&mut self, executor: Arc<dyn Executor>) -> ToolOutcome {
        const NAME: &str = "verify";
        if let Err(e) = self.sync_with_disk().await {
            return ToolOutcome::err(NAME, e);
        }
        // Verify the WHOLE editable set, not just the primary.
        //
        // The agent can edit any document in the closure, so a verify scoped
        // to the primary is a feedback loop that lies: edit a decision two
        // hops up, run verify, get PASS — while that document's own outputs
        // were never re-woven and `hick test` fails on every one of them.
        // Observed exactly once in a live run, where the agent then reported
        // success in good faith. Verification scope must equal edit scope.
        let declared = match weave::declared_sources(&self.doc_path, Some(&self.source)) {
            Ok(s) => s,
            Err(e) => return ToolOutcome::err(NAME, format!("Project closure failed: {e:#}")),
        };
        let mut captured_sources = if declared.is_empty() {
            self.upstream.clone()
        } else {
            declared.clone()
        };
        captured_sources.insert(self.doc_path.clone(), self.source.clone());
        let upstream_report = if declared.is_empty() {
            self.verify_upstream(executor.clone()).await
        } else {
            UpstreamReport {
                failures: vec![],
                note: String::new(),
                pending: vec![],
            }
        };
        let project_dir = self
            .doc_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();
        let sources = if declared.is_empty() {
            vec![(self.doc_name.as_str(), self.source.as_str())]
        } else {
            declared
                .iter()
                .map(|(name, text)| (name.to_str().unwrap(), text.as_str()))
                .collect()
        };
        let config = hick_literate::PipelineConfig {
            working_dir: Some(project_dir.clone()),
            max_rounds: 1,
            on_exec: None,
            ..Default::default()
        };
        let result =
            match hick_literate::run_pipeline_live(&sources, &config, &self.params, None, executor)
                .await
            {
                Ok(r) => r,
                Err(e) => return ToolOutcome::err(NAME, format!("execution failed: {e}")),
            };

        let mut failures = Vec::new();
        for outcome in &result.expectations {
            if !outcome.passed {
                failures.push(format!(
                    "expectation at {}:{} (container {}) failed: {}",
                    outcome.doc,
                    outcome.line,
                    outcome.container.as_deref().unwrap_or("agent cell"),
                    outcome.detail
                ));
            }
        }

        // Execution results remain provisional until the whole editable scope
        // passes. In particular, do not refresh session reads from a failed run.
        failures.extend(upstream_report.failures);
        if self.verified_publication {
            if captured_sources.iter().any(|(path, captured)| {
                std::fs::read_to_string(path).ok().as_ref() != Some(captured)
            }) || upstream_closure(&self.doc_path) != self.upstream
            {
                failures.push(
                    "Source closure changed during verification; rerun against current sources"
                        .into(),
                );
            }
            if std::iter::once(&result)
                .chain(upstream_report.pending.iter().map(|(_, result)| result))
                .flat_map(|result| &result.span_files)
                .any(|path| {
                    !captured_sources
                        .keys()
                        .any(|captured| normalize(captured) == normalize(Path::new(path)))
                })
            {
                failures
                    .push("Uncaptured source dependencies cannot publish verified outputs".into());
            }
            if !result.never_run.is_empty() || !result.stale.is_empty() {
                failures.push(
                    "Unavailable or stale execution evidence cannot publish verified outputs"
                        .into(),
                );
            }
        }
        if self.verified_publication && !failures.is_empty() {
            return ToolOutcome::err(
                NAME,
                format!(
                    "FAIL — {} problem(s); no verified outputs published:\n{}",
                    failures.len(),
                    failures.join("\n")
                ),
            );
        }

        // Only now publish upstream results and the primary result. Filesystem
        // errors may still leave partial writes; this is not a multi-root transaction.
        for (dir, result) in upstream_report.pending {
            if let Err(error) = super::verify_publish::publish(&dir, &result) {
                return ToolOutcome::err(
                    NAME,
                    format!("FAIL — upstream publication failed: {error}"),
                );
            }
        }
        // Write accepted outputs so document and committed outputs stay in sync.
        let written = match super::verify_publish::publish(&project_dir, &result) {
            Ok(paths) => paths,
            Err(error) => {
                return ToolOutcome::err(NAME, format!("FAIL — publication failed: {error}"));
            }
        };

        // Refresh the session's weave state from the executed result: real
        // transcripts give the freshest provenance.
        let mut files = HashMap::new();
        for (path, content) in &result.files {
            if let FileContent::Text(s) = content {
                files.insert(path.clone(), s.clone());
            }
        }
        let provenance = result
            .provenance_maps
            .iter()
            .map(|(p, m)| (p.clone(), hickory_lineage::from_provenance_map(m)))
            .collect();
        self.weave = WeaveState { files, provenance };

        let checked = result.expectations.len();
        if failures.is_empty() {
            ToolOutcome::ok(
                NAME,
                format!(
                    "PASS — checked {checked} expectation(s) {}; wrote {} output file(s): {}{}",
                    if result.evaluated_project.is_some() {
                        "against kept recordings; source construction verified, live runtime unrun"
                    } else {
                        "by executing the document"
                    },
                    written.len(),
                    if written.is_empty() {
                        "(none)".to_string()
                    } else {
                        written.join(", ")
                    },
                    upstream_report.note
                ),
            )
        } else {
            ToolOutcome::err(
                NAME,
                format!(
                    "FAIL — {} problem(s):\n{}",
                    failures.len(),
                    failures.join("\n")
                ),
            )
        }
    }

    /// Re-run every upstream document, retaining provisional outputs in memory.
    ///
    /// Each is executed on its own, in its own directory, because that is
    /// where its outputs are committed — an upstream document woven into the
    /// primary's directory would create files nobody checks and leave the
    /// real ones stale.
    async fn verify_upstream(&self, executor: Arc<dyn Executor>) -> UpstreamReport {
        let mut failures = Vec::new();
        let mut verified = 0usize;
        let mut pending = Vec::new();
        for (path, captured) in self.upstream.iter() {
            let Ok(source) = (if self.verified_publication {
                Ok(captured.clone())
            } else {
                std::fs::read_to_string(path)
            }) else {
                failures.push(format!("upstream {}: could not be read", path.display()));
                continue;
            };
            let name = path.display().to_string();
            let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let config = hick_literate::PipelineConfig {
                working_dir: Some(dir.clone()),
                max_rounds: 1,
                on_exec: None,
                ..Default::default()
            };
            let sources = vec![(name.as_str(), source.as_str())];
            let result = match hick_literate::run_pipeline_live(
                &sources,
                &config,
                &self.params,
                None,
                executor.clone(),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    failures.push(format!("upstream {name}: execution failed: {e}"));
                    continue;
                }
            };
            for outcome in &result.expectations {
                if !outcome.passed {
                    failures.push(format!(
                        "upstream {name}: expectation at line {} failed: {}",
                        outcome.line, outcome.detail
                    ));
                }
            }
            if self.verified_publication {
                if !result.never_run.is_empty() || !result.stale.is_empty() {
                    failures.push(format!(
                        "upstream {name}: unavailable or stale execution evidence"
                    ));
                }
                pending.push((dir, result));
            } else if let Err(error) = super::verify_publish::publish(&dir, &result) {
                failures.push(format!("upstream {name}: publication failed: {error}"));
            }
            verified += 1;
        }
        let note = if verified == 0 {
            String::new()
        } else {
            format!("; also re-wove {verified} upstream document(s)")
        };
        UpstreamReport {
            failures,
            note,
            pending,
        }
    }
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/verify.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/verify.rs"><storycode:paste select="#f-913f8d4d5670-UpstreamReport"/><storycode:paste select="#f-913f8d4d5670-impl-EditSession"/></storycode:file>

</storycode:doc>
