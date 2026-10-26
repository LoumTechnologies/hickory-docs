<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/debug_sources.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9c1c3fe2ca3c"></a>
## `crates/hickory-cli/src/serve/debug_sources.rs`

<a id="f-9c1c3fe2ca3c-document"></a>
### `document`

<storycode:copy id="f-9c1c3fe2ca3c-document">
use super::*;
use hick_dap::Breakpoint;
use std::path::Path;

// Validate the client identity before asking the session's exact mapping.
// A captured URI must belong to that session; a workspace path cannot escape
// the open folder through '..' or a symlink. Neither route substitutes bytes.
fn document(root: &Path, source: &str) -> anyhow::Result<PathBuf> {
    if source.starts_with("hick-project:///") {
        return Ok(PathBuf::from(source));
    }
    let path = resolve(root, source).canonicalize()?;
    anyhow::ensure!(
        path.starts_with(root.canonicalize()?),
        "breakpoint source is outside the open folder"
    );
    Ok(path)
}

</storycode:copy>
<a id="f-9c1c3fe2ca3c-initial"></a>
### `initial`

<storycode:copy id="f-9c1c3fe2ca3c-initial">
pub(super) fn initial(
    root: &Path,
    sources: std::collections::HashMap<String, Vec<Breakpoint>>,
) -> anyhow::Result<Vec<hick_dap::SourceBreakpoints>> {
    sources
        .into_iter()
        .map(|(source, breakpoints)| {
            Ok(hick_dap::SourceBreakpoints {
                document: document(root, &source)?,
                source,
                breakpoints,
            })
        })
        .collect()
}

</storycode:copy>
<a id="f-9c1c3fe2ca3c-set"></a>
### `set`

<storycode:copy id="f-9c1c3fe2ca3c-set">
pub(super) async fn set(
    registry: &Arc<Registry>,
    root: &Path,
    session: String,
    source: String,
    breakpoints: Vec<Breakpoint>,
) -> Response {
    let result = async {
        let live = registry.get(&session).await?;
        let document = document(root, &source)?;
        live.session
            .set_source_breakpoints(&source, &document, &breakpoints)
            .await
    }
    .await;
    let breakpoints = match result {
        Ok(breakpoints) => breakpoints,
        Err(error) => {
            return Response::Failed {
                session: Some(session),
                doc: Some(source),
                message: format!("{error:#}"),
                about: Some("source_breakpoints".into()),
                lines: breakpoints.iter().map(|bp| bp.line).collect(),
                offer_install: None,
            };
        }
    };
    Response::SourceBreakpoints {
        session,
        source,
        breakpoints,
    }
}

</storycode:copy>
<a id="f-9c1c3fe2ca3c-statuses"></a>
### `statuses`

<storycode:copy id="f-9c1c3fe2ca3c-statuses">
pub(super) fn statuses(session: &str, live: &Arc<crate::debug_sessions::Live>) -> Vec<Response> {
    live.session
        .source_breakpoint_statuses()
        .into_iter()
        .map(|(source, breakpoints)| Response::SourceBreakpoints {
            session: session.to_string(),
            source,
            breakpoints,
        })
        .collect()
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/debug_sources.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/debug_sources.rs"><storycode:paste select="#f-9c1c3fe2ca3c-document"/><storycode:paste select="#f-9c1c3fe2ca3c-initial"/><storycode:paste select="#f-9c1c3fe2ca3c-set"/><storycode:paste select="#f-9c1c3fe2ca3c-statuses"/></storycode:file>

</storycode:doc>
