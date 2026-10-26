<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/selected_run.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d71989681217"></a>
## `crates/hickory-cli/src/selected_run.rs`

<a id="f-d71989681217-run_doc_expected"></a>
### `run_doc_expected`

<storycode:copy id="f-d71989681217-run_doc_expected">
use super::*;

pub async fn run_doc_expected(
    doc_path: &Path,
    params: &[(String, String)],
    mode: RunMode,
    executor_choice: ExecutorChoice,
    expected: &str,
) -> Result<DocRun> {
    run_doc_selected(
        doc_path,
        params,
        mode,
        executor_choice,
        Some(expected),
        None,
    )
    .await
}

</storycode:copy>
<a id="f-d71989681217-run_doc_selected"></a>
### `run_doc_selected`

<storycode:copy id="f-d71989681217-run_doc_selected">
pub(crate) async fn run_doc_selected(
    doc_path: &Path,
    params: &[(String, String)],
    mode: RunMode,
    executor_choice: ExecutorChoice,
    expected: Option<&str>,
    cells: Option<Vec<String>>,
) -> Result<DocRun> {
    let source = std::fs::read_to_string(doc_path)
        .with_context(|| format!("failed to read {}", doc_path.display()))?;
    let actual = hick_context_protocol::digest(source.as_bytes());
    if let Some(expected) = expected {
        if actual.trim_start_matches("sha256:") != expected.trim_start_matches("sha256:") {
            bail!(
                "Source revision changed before execution; expected {expected}, observed {actual}. No root document operation ran."
            );
        }
    }
    let subset = cells
        .filter(|ids| !ids.is_empty())
        .map(|ids| {
            let doc = hick_lang::parse(&source)?;
            if !doc.find_tags("project").is_empty() {
                bail!(
                    "Selected-cell runs are unsupported for project documents; no cells executed"
                );
            }
            let dag = hick_exec::dag::build_dag(&doc)?;
            let mut selected = std::collections::HashSet::new();
            let mut stack = Vec::new();
            for id in ids {
                let exec = dag
                    .execs
                    .iter()
                    .find(|exec| {
                        let container = if exec.is_script || exec.agent.is_some() {
                            ""
                        } else {
                            &exec.container
                        };
                        format!("{}:{}", container, exec.source_line) == id
                    })
                    .with_context(|| format!("Unknown requested cell {id}; no cells executed"))?;
                stack.push(exec.id);
            }
            while let Some(id) = stack.pop() {
                if selected.insert(id) {
                    stack.extend(dag.predecessors(id));
                }
            }
            Ok::<_, anyhow::Error>(selected)
        })
        .transpose()
        .map_err(|error| CapturedRootError {
            source_hash: actual.clone(),
            error,
        })?;
    run_source(
        doc_path,
        params,
        mode,
        executor_choice,
        CacheMode::Off,
        subset,
        source,
    )
    .await
    .map_err(|error| {
        CapturedRootError {
            source_hash: actual,
            error,
        }
        .into()
    })
}

</storycode:copy>
<a id="f-d71989681217-failure_message"></a>
### `failure_message`

The source receipt is metadata, not a second copy of the execution error.
Format the original anyhow chain intact without the transparent receipt wrapper.

<storycode:copy id="f-d71989681217-failure_message">
/// The source receipt is metadata, not a second copy of the execution error.
/// Format the original anyhow chain intact without the transparent receipt wrapper.
pub(crate) fn failure_message(error: &anyhow::Error) -> String {
    let original = error
        .downcast_ref::<CapturedRootError>()
        .map(|captured| &captured.error)
        .unwrap_or(error);
    let original = original
        .downcast_ref::<crate::failed_run::FailedRun>()
        .map(|failed| &failed.error)
        .unwrap_or(original);
    format!("{original:#}")
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/selected_run.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/selected_run.rs"><storycode:paste select="#f-d71989681217-run_doc_expected"/><storycode:paste select="#f-d71989681217-run_doc_selected"/><storycode:paste select="#f-d71989681217-failure_message"/></storycode:file>

</storycode:doc>
