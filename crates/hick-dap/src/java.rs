//! Where eclipse.jdt.ls and the java-debug bundle live, once installed.
//!
//! Java's adapter is not a program: `java-debug` is a plugin that runs inside
//! the Java language server, and a debug session is obtained by ASKING that
//! server for one. The asking needs LSP, which this crate must not know
//! about — but *whether the pieces are here* is an ordinary filesystem
//! question, and discovery has to answer it to say honestly whether Java is
//! debuggable at all.
//!
//! So the lookup lives here, at the layer that reports capability, and
//! `hickory_cli::java_debug` — which can speak LSP — uses the same answer.
//! Two copies of these paths would be the bug this repository has shipped
//! six times.

use std::path::{Path, PathBuf};

/// Where the pieces live once `hick lsp install java` and
/// `hick dap install java` have run.
pub struct Installed {
    /// `…/extension/server/plugins/org.eclipse.equinox.launcher_*.jar`
    pub launcher: PathBuf,
    /// `…/extension/server/config_linux`, per platform.
    pub configuration: PathBuf,
    /// The java-debug plugin jar jdt.ls is told to load.
    pub bundle: PathBuf,
}

/// jdt.ls ships a `config_` directory per platform and picks none for you.
pub fn configuration_dir() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "config_mac_arm",
        ("macos", _) => "config_mac",
        ("windows", _) => "config_win",
        (_, "aarch64") => "config_linux_arm",
        _ => "config_linux",
    }
}

/// The newest file in `dir` whose name starts with `prefix` and ends `.jar`.
///
/// The launcher jar carries a build stamp in its name, so it cannot be named
/// literally; and several similarly-named ones sit beside it — the
/// platform-specific `org.eclipse.equinox.launcher.gtk.linux.x86_64_*` is NOT
/// the one to run, which is why the match is anchored on the underscore.
pub fn newest_jar(dir: &Path, prefix: &str) -> Option<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|e| e == "jar")
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(prefix))
        })
        .collect();
    found.sort();
    found.pop()
}

/// Find jdt.ls and the java-debug bundle, walking up from `root`.
///
/// The same rule the adapter cache uses: an install at the top of a project
/// has to serve a document three folders down.
pub fn installed(root: &Path) -> Option<Installed> {
    let mut dir = Some(root);
    while let Some(here) = dir {
        let server = here.join(".hick-cache/servers/java/extension/server");
        let bundles = here.join(".hick-cache/adapters/java-debug/extension/server");
        let launcher = newest_jar(&server.join("plugins"), "org.eclipse.equinox.launcher_");
        let bundle = newest_jar(&bundles, "com.microsoft.java.debug.plugin");
        if let (Some(launcher), Some(bundle)) = (launcher, bundle) {
            let configuration = server.join(configuration_dir());
            if configuration.is_dir() {
                return Some(Installed {
                    launcher,
                    configuration,
                    bundle,
                });
            }
        }
        dir = here.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_platforms_configuration_directory_is_named() {
        // jdt.ls ships one per platform and picks none for you.
        assert!(configuration_dir().starts_with("config_"));
    }

    /// The launcher jar sits beside several similarly-named platform
    /// fragments, and running one of those starts nothing.
    #[test]
    fn the_launcher_is_the_plain_one_not_a_platform_fragment() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "org.eclipse.equinox.launcher_1.8.0.v20260804.jar",
            "org.eclipse.equinox.launcher.gtk.linux.x86_64_1.2.1600.jar",
        ] {
            std::fs::write(dir.path().join(name), "").unwrap();
        }
        let found = newest_jar(dir.path(), "org.eclipse.equinox.launcher_").unwrap();
        assert_eq!(
            found.file_name().unwrap(),
            "org.eclipse.equinox.launcher_1.8.0.v20260804.jar",
            "the platform fragment is not the launcher"
        );
    }

    /// Both halves or nothing: the debugger runs inside the server.
    #[test]
    fn nothing_installed_is_none_rather_than_a_guess() {
        let dir = tempfile::tempdir().unwrap();
        assert!(installed(dir.path()).is_none());
    }
}
