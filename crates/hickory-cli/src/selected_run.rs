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

/// The source receipt is metadata, not a second copy of the execution error.
/// Format the original anyhow chain intact without the transparent receipt wrapper.
pub(crate) fn failure_message(error: &anyhow::Error) -> String {
    let original = error
        .downcast_ref::<CapturedRootError>()
        .map(|captured| &captured.error)
        .unwrap_or(error);
    format!("{original:#}")
}
