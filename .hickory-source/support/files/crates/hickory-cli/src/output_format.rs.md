<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/output_format.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7ab2e4e2cf25e3cc3f45b69837147978e5eea3d43e7c835515e39b03fedfc0f8`.

<book:entry path="crates/hickory-cli/src/output_format.rs">
pub(super) fn indent(s: &str) -> String {
    s.lines()
        .map(|l| format!("    | {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}
</book:entry>

</book:doc>
