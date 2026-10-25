<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/debug_sources.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6d2ecf43d7cb0a1839c1040778e576ef08fdbabf40f72f6d1ec53b0aa80f1fba`.

<book:entry path="crates/hickory-cli/src/serve/debug_sources.rs">
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
</book:entry>

</book:doc>
