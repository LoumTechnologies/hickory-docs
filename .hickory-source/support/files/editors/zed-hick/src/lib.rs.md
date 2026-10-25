<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# editors/zed-hick/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0b95cb8bd65bd332badea176bfb7db326427b68e8543d6d8974e92deb520b648`.

<book:entry path="editors/zed-hick/src/lib.rs">
use zed_extension_api::{self as zed, LanguageServerId};

struct HickExtension;

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

zed::register_extension!(HickExtension);
</book:entry>

</book:doc>
