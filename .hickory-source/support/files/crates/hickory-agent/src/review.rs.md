<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-agent/src/review.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `faf3ebad24d274cce3b6abb894ee97006b04548f4f671b1b20b23ea54355c49f`.

<book:entry path="crates/hickory-agent/src/review.rs">
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
</book:entry>

</book:doc>
