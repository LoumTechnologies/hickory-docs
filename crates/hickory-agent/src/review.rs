//! Guard for complete replacements approved by the shared tool review UI.
pub(crate) fn whole_edit(
    index: &crate::tools::hashline::LineIndex,
    inv: &crate::ToolInvocation,
) -> Option<Result<(usize, String), String>> {
    if inv.arg("whole") != Some("true") {
        return None;
    }
    Some(if inv.arg("snapshot") != Some(index.snapshot.as_str()) {
        Err("A complete reviewed replacement requires the exact surface snapshot.".into())
    } else {
        Ok((
            index.line_end(index.lines.len() - 1),
            inv.input.clone().unwrap_or_default(),
        ))
    })
}
