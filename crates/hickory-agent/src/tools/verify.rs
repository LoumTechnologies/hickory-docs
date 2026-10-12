//! Execution and complete project publication for the same edit closure.
use super::*;
impl EditSession {
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
        let upstream_report = if declared.is_empty() {
            self.verify_upstream(executor.clone()).await
        } else {
            UpstreamReport {
                failures: vec![],
                note: String::new(),
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

        // Write outputs to disk (like `hick run`), so document and
        // committed outputs stay in sync.
        let mut written = Vec::new();
        if let Some(project) = &result.evaluated_project {
            match hick_project_env::write_tree(&project_dir, &project.new) {
                Ok(paths) => written.extend(paths.into_iter().map(|p| {
                    p.strip_prefix(&project_dir)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .into_owned()
                })),
                Err(e) => failures.push(format!("Could not publish complete project: {e}")),
            }
        } else {
            for (rel_path, content) in &result.files {
                let full = project_dir.join(rel_path);
                if let Some(parent) = full.parent()
                    && !parent.as_os_str().is_empty()
                    && let Err(e) = std::fs::create_dir_all(parent)
                {
                    failures.push(format!("could not create {}: {e}", parent.display()));
                    continue;
                }
                let write_result = match content {
                    FileContent::Text(s) => std::fs::write(&full, s),
                    FileContent::Binary(data) => match data.to_bytes() {
                        Ok(bytes) => std::fs::write(&full, bytes),
                        Err(e) => {
                            failures.push(format!("could not encode {rel_path}: {e}"));
                            continue;
                        }
                    },
                };
                match write_result {
                    Ok(()) => written.push(rel_path.clone()),
                    Err(e) => failures.push(format!("could not write {}: {e}", full.display())),
                }
            }
        }
        written.sort();

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
        failures.extend(upstream_report.failures);
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

    /// Re-run every upstream document and write its outputs.
    ///
    /// Each is executed on its own, in its own directory, because that is
    /// where its outputs are committed — an upstream document woven into the
    /// primary's directory would create files nobody checks and leave the
    /// real ones stale.
    async fn verify_upstream(&self, executor: Arc<dyn Executor>) -> UpstreamReport {
        let mut failures = Vec::new();
        let mut verified = 0usize;
        for (path, _) in self.upstream.iter() {
            let Ok(source) = std::fs::read_to_string(path) else {
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
            for (rel_path, content) in &result.files {
                let full = dir.join(rel_path);
                if let Some(parent) = full.parent()
                    && !parent.as_os_str().is_empty()
                    && let Err(e) = std::fs::create_dir_all(parent)
                {
                    failures.push(format!("upstream {name}: could not create {parent:?}: {e}"));
                    continue;
                }
                let write = match content {
                    FileContent::Text(t) => std::fs::write(&full, t),
                    FileContent::Binary(d) => match d.to_bytes() {
                        Ok(b) => std::fs::write(&full, b),
                        Err(e) => {
                            failures
                                .push(format!("upstream {name}: could not encode {rel_path}: {e}"));
                            continue;
                        }
                    },
                };
                if let Err(e) = write {
                    failures.push(format!(
                        "upstream {name}: could not write {}: {e}",
                        full.display()
                    ));
                }
            }
            verified += 1;
        }
        let note = if verified == 0 {
            String::new()
        } else {
            format!("; also re-wove {verified} upstream document(s)")
        };
        UpstreamReport { failures, note }
    }
}
