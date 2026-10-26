<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain editors/zed-hick/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-57638c9b1d6b"></a>
## `editors/zed-hick/src/lib.rs`

<a id="f-57638c9b1d6b-HickExtension"></a>
### `HickExtension`

<storycode:copy id="f-57638c9b1d6b-HickExtension">
use zed_extension_api::{self as zed, LanguageServerId};

struct HickExtension;

</storycode:copy>
<a id="f-57638c9b1d6b-impl-zed-Extension-for-HickExtension"></a>
### `impl-zed::Extension for HickExtension`

<storycode:copy id="f-57638c9b1d6b-impl-zed-Extension-for-HickExtension">
impl zed::Extension for HickExtension {
    fn new() -> Self {
        HickExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed_extension_api::Result<zed::Command> {
        let path = worktree
            .which("hick-lsp")
            .ok_or_else(|| "hick-lsp not found on PATH. Install it or add it to your PATH.".to_string())?;
        Ok(zed::Command {
            command: path,
            args: vec![],
            env: worktree.shell_env(),
        })
    }
}

</storycode:copy>
<a id="f-57638c9b1d6b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-57638c9b1d6b-expression_statement">
zed::register_extension!(HickExtension);
</storycode:copy>

### Assemble `editors/zed-hick/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="editors/zed-hick/src/lib.rs"><storycode:paste select="#f-57638c9b1d6b-HickExtension"/><storycode:paste select="#f-57638c9b1d6b-impl-zed-Extension-for-HickExtension"/><storycode:paste select="#f-57638c9b1d6b-expression_statement"/></storycode:file>

</storycode:doc>
