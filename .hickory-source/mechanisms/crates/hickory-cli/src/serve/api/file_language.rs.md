<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/api/file_language.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5afd7de24b27"></a>
## `crates/hickory-cli/src/serve/api/file_language.rs`

<a id="f-5afd7de24b27-language_of"></a>
### `language_of`

Use the same plugin-owned routing as plain files, LSP, debugging and codegen.

<storycode:copy id="f-5afd7de24b27-language_of">
/// Use the same plugin-owned routing as plain files, LSP, debugging and codegen.
pub(in crate::serve) fn language_of(path: &str) -> String {
    hick_plugins::routing::language_id(path)
        .unwrap_or_else(|| std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or(""))
        .to_string()
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/api/file_language.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/api/file_language.rs"><storycode:paste select="#f-5afd7de24b27-language_of"/></storycode:file>

</storycode:doc>
