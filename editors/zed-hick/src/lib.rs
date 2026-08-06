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
