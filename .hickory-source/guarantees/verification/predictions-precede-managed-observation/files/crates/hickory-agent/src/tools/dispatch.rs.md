<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-agent/src/tools/dispatch.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `88af58c61baea751c60facb669b22df0eb07458736aa362f4851fb78bcfac12a`.

<book:entry path="crates/hickory-agent/src/tools/dispatch.rs">
//! Shared tool dispatch and host-selected private policy.
use super::*;

/// Execute one tool invocation against the session. Failures are tool
/// results (observations), never process errors.
pub async fn execute_tool(
    session: &mut EditSession,
    executor: Arc<dyn Executor>,
    inv: &ToolInvocation,
) -> ToolOutcome {
    if session.prediction.is_some()
        && !matches!(inv.name.as_str(), "read_doc" | "edit_doc" | "verify")
    {
        return ToolOutcome::refused(&inv.name, "Private prediction policy withholds generated outputs, files and alternate execution; use source investigation and sealed Verify".into());
    }
    if matches!(inv.name.as_str(), "read_doc" | "read_output" | "verify")
        && let Err(error) = session.sync_with_disk().await
    {
        return ToolOutcome::err(&inv.name, error);
    }
    match inv.name.as_str() {
        "read_doc" => session.read_doc(inv),
        "read_output" => session.read_output(inv),
        "read_file" => read_file(
            &session.root,
            session.doc_path.parent().unwrap_or(Path::new(".")),
            inv,
        ),
        "edit_output" => session.edit_output(inv).await,
        "edit_doc" => session.edit_doc(inv).await,
        "verify" => {
            let before = session.revision();
            let mut result = if session.prediction.is_some() {
                session.verify_prediction(executor).await
            } else {
                session.verify(executor).await
            };
            if before != session.revision() {
                result.ok = false;
                result.text.push_str("\nSource closure changed during verification; this result does not establish a pass for the current sources.");
            }
            result
        }
        other => ToolOutcome::err(
            other,
            format!(
                "unknown tool '{other}' — available: read_doc, read_output, read_file, \
                 edit_output, edit_doc, verify"
            ),
        ),
    }
}
</book:entry>

</book:doc>
