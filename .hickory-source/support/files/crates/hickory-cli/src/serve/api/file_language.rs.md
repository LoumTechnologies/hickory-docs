<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/api/file_language.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f6acf355debe28b679d69d0e4f2847ab7dbdbefff34bb8c5de6eb05ff39eaf2f`.

<book:entry path="crates/hickory-cli/src/serve/api/file_language.rs">
/// Use the same plugin-owned routing as plain files, LSP, debugging and codegen.
pub(in crate::serve) fn language_of(path: &str) -> String {
    hick_plugins::routing::language_id(path)
        .unwrap_or_else(|| std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or(""))
        .to_string()
}
</book:entry>

</book:doc>
