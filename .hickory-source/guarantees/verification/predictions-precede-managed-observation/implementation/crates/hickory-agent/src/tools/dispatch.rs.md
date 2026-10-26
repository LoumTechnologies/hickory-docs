<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/tools/dispatch.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7798872b38ba"></a>
## `crates/hickory-agent/src/tools/dispatch.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../contract.md).

<a id="f-7798872b38ba-execute_tool"></a>
### `execute_tool`

Shared tool dispatch and host-selected private policy.

<storycode:copy id="f-7798872b38ba-execute_tool">
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
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/dispatch.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/dispatch.rs"><storycode:paste select="#f-7798872b38ba-execute_tool"/></storycode:file>

</storycode:doc>
