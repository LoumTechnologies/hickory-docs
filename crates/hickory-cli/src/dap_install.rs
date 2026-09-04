//! Which debug adapters `hick dap install` can fetch.
//!
//! The machinery — the sandbox, the prefix, the refusals — is shared with
//! `hick lsp install`; see [`crate::tool_install`]. This file is the
//! catalogue and nothing else.

use crate::tool_install::{Asset, Catalogue, Installer};

/// Where a debug adapter is installed, relative to the project root.
///
/// Beside the language servers rather than among them: the two are found by
/// different code and a mixed directory would make "which of these is which"
/// a question somebody has to answer at a shell prompt.
pub const ADAPTERS_DIR: &str = ".hick-cache/adapters";

pub static CATALOGUE: Catalogue = Catalogue {
    what: "debug adapter",
    prefix: ADAPTERS_DIR,
    command: "hick dap install",
    installers: INSTALLERS,
};

/// The installable set.
///
/// Shorter than the language-server catalogue, because debug adapters are
/// less uniformly packaged: several ship only inside an editor's extension,
/// which is not something to fetch on a user's behalf. Where an ecosystem has
/// no installable adapter, discovery still finds one the user installed
/// themselves — that path is unaffected by anything here.
const INSTALLERS: &[Installer] = &[
    Installer {
        language: "python",
        tool: "uv",
        package: "debugpy",
        // A real virtualenv, for the same reason the language-server
        // installer uses one: `pyvenv.cfg` is how discovery recognises it,
        // and the interpreter that has debugpy must be the interpreter that
        // runs the cell — a machine python paired with a venv's debugpy
        // fails in a way that reads as our bug.
        command: "uv venv {prefix}/python && uv pip install --python {prefix}/python/bin/python debugpy",
        assets: &[],
        reason: "Microsoft's Python debugger, and the reference DAP implementation",
    },
    Installer {
        language: "typescript",
        tool: "curl",
        package: "js-debug",
        // Nothing to run: an archive install builds its command from the
        // matching asset below.
        //
        // This was `npm install @vscode/js-debug`, which 404s and always
        // has: the package is not published to npm. `hick dap install
        // typescript` therefore failed on every machine, for as long as it
        // existed, while `hick lang` reported JavaScript and TypeScript as
        // having a debugger one command away.
        command: "",
        assets: JS_DEBUG,
        reason: "the Node debugger VS Code ships, usable outside it",
    },
    Installer {
        language: "csharp",
        tool: "curl",
        package: "netcoredbg",
        // Nothing to run: an archive install has no setup script, so the
        // command is built from the matching asset below.
        command: "",
        assets: NETCOREDBG,
        // Samsung's, MIT, and the only option. Microsoft's `vsdbg` is
        // licensed for use only with Visual Studio and VS Code, which makes
        // it unavailable to this product rather than merely unchosen.
        reason: "the only MIT-licensed .NET debugger, and the one hick's C# support was built against",
    },
    Installer {
        language: "rust",
        tool: "curl",
        package: "codelldb",
        command: "",
        assets: CODELLDB,
        // MIT, ships its own lldb, and speaks DAP over stdio when started
        // with no arguments. `lldb-dap` from LLVM is the other adapter
        // discovery looks for, and one the user installed is still preferred.
        reason: "the LLDB-based debugger VS Code's Rust users run, usable outside it",
    },
];

/// js-debug v1.117.0.
///
/// One tarball for every platform: it is JavaScript, so the bytes really are
/// identical everywhere and the same checksum is the honest answer for each
/// row rather than a copy-paste. The archive carries its own `js-debug/`
/// top-level directory, so it unpacks into the prefix itself.
///
/// The checksum was taken on 2026-09-04 by fetching the asset. Only
/// `linux-x86_64` has been unpacked and run.
const JS_DEBUG: &[Asset] = &[
    Asset {
        os: "linux",
        arch: "x86_64",
        url: JS_DEBUG_URL,
        sha256: JS_DEBUG_SHA256,
        unpack: "tar",
        into: "",
    },
    Asset {
        os: "linux",
        arch: "aarch64",
        url: JS_DEBUG_URL,
        sha256: JS_DEBUG_SHA256,
        unpack: "tar",
        into: "",
    },
    Asset {
        os: "macos",
        arch: "aarch64",
        url: JS_DEBUG_URL,
        sha256: JS_DEBUG_SHA256,
        unpack: "tar",
        into: "",
    },
    Asset {
        os: "macos",
        arch: "x86_64",
        url: JS_DEBUG_URL,
        sha256: JS_DEBUG_SHA256,
        unpack: "tar",
        into: "",
    },
    Asset {
        os: "windows",
        arch: "x86_64",
        url: JS_DEBUG_URL,
        sha256: JS_DEBUG_SHA256,
        unpack: "tar",
        into: "",
    },
];

const JS_DEBUG_URL: &str = "https://github.com/microsoft/vscode-js-debug/releases/download/v1.117.0/js-debug-dap-v1.117.0.tar.gz";
const JS_DEBUG_SHA256: &str = "ad8d04ede9d4b75cc290fd5438a65047a06f786d04f604b6112485b36f090772";

/// codelldb v1.12.3, pinned per platform.
///
/// Published as a VS Code extension archive (`.vsix`, which is a zip of
/// `extension/…`), so each one unpacks INTO `codelldb/` and the adapter is
/// `.hick-cache/adapters/codelldb/extension/adapter/codelldb`, with the lldb
/// it bundles beside it. The checksums were taken from the five assets on
/// 2026-09-02 by fetching each one. Only `linux-x86_64` has been unpacked and
/// run; the other four are pinned bytes nobody here has executed.
const CODELLDB: &[Asset] = &[
    Asset {
        os: "linux",
        arch: "x86_64",
        url: "https://github.com/vadimcn/codelldb/releases/download/v1.12.3/codelldb-linux-x64.vsix",
        sha256: "1cd7f386598022b51a5b93b9ffa23e812b23f519cfe1833384ec4bef4bfd1be1",
        unpack: "unzip",
        into: "codelldb",
    },
    Asset {
        os: "linux",
        arch: "aarch64",
        url: "https://github.com/vadimcn/codelldb/releases/download/v1.12.3/codelldb-linux-arm64.vsix",
        sha256: "0887f67d440554617894266f80706b700907c36b95e6e49d23b95a0e05318101",
        unpack: "unzip",
        into: "codelldb",
    },
    Asset {
        os: "macos",
        arch: "aarch64",
        url: "https://github.com/vadimcn/codelldb/releases/download/v1.12.3/codelldb-darwin-arm64.vsix",
        sha256: "2f114a990e1b368dd1dbd33c80c0e719767af2d228391ec0df0571c957f9ac91",
        unpack: "unzip",
        into: "codelldb",
    },
    Asset {
        os: "macos",
        arch: "x86_64",
        url: "https://github.com/vadimcn/codelldb/releases/download/v1.12.3/codelldb-darwin-x64.vsix",
        sha256: "e25cc716b94c62c07fec268ff2785d2b797245b160502baef8b9c970a0c4d8e8",
        unpack: "unzip",
        into: "codelldb",
    },
    Asset {
        os: "windows",
        arch: "x86_64",
        url: "https://github.com/vadimcn/codelldb/releases/download/v1.12.3/codelldb-win32-x64.vsix",
        sha256: "a916e509308dac817732f63ca604a8b93ed29cd16f38a2fa9f0b64ed58e8f51a",
        unpack: "unzip",
        into: "codelldb",
    },
];

/// netcoredbg 3.2.0-1092, pinned per platform.
///
/// Every archive unpacks to a `netcoredbg/` directory holding the binary and
/// its managed DLLs, which is why discovery looks for
/// `.hick-cache/adapters/netcoredbg/netcoredbg` rather than a `bin/`.
///
/// The checksums were taken from the four assets on 2026-08-27 by fetching
/// each one. Only `linux-x86_64` has been unpacked and run; the other three
/// are pinned bytes nobody here has executed, and the guarantee says so.
/// Bumping the version means replacing all four together — a mixed set would
/// install one release's binary against another's checksum and fail the
/// verification rather than do anything dangerous, which is the failure mode
/// worth having.
const NETCOREDBG: &[Asset] = &[
    Asset {
        os: "linux",
        arch: "x86_64",
        url: "https://github.com/Samsung/netcoredbg/releases/download/3.2.0-1092/netcoredbg-linux-amd64.tar.gz",
        sha256: "080eb3b2d2152465f599d3b33d1ee6e747794e11cc0a3773ec689f5e5f2c5afa",
        unpack: "tar",
        into: "",
    },
    Asset {
        os: "linux",
        arch: "aarch64",
        url: "https://github.com/Samsung/netcoredbg/releases/download/3.2.0-1092/netcoredbg-linux-arm64.tar.gz",
        sha256: "065ff49badec8a695dbea2de6ab6a330c774a191e426a217ab8cc05250627ccb",
        unpack: "tar",
        into: "",
    },
    Asset {
        os: "macos",
        arch: "aarch64",
        url: "https://github.com/Samsung/netcoredbg/releases/download/3.2.0-1092/netcoredbg-osx-arm64.zip",
        sha256: "f4fa33b3ff874910cc184b4bb3b9c56d0abdf5c6521cee0b144d7c6e4a6e59ea",
        unpack: "unzip",
        into: "",
    },
    Asset {
        os: "windows",
        arch: "x86_64",
        url: "https://github.com/Samsung/netcoredbg/releases/download/3.2.0-1092/netcoredbg-win64.zip",
        sha256: "3c410a45fa502415203a94fcb88654af65bf8e3dac158a5527a722e7a6b9274a",
        unpack: "unzip",
        into: "",
    },
];

/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<crate::tool_install::InstallPlan> {
    crate::tool_install::plans(&CATALOGUE)
}

/// Install one language's adapter, confined.
pub fn install(root: &std::path::Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    crate::tool_install::install(&CATALOGUE, root, language)
}

#[cfg(test)]
mod tests {
    /// The two halves of "how do I get an adapter" must agree.
    ///
    /// `hick_dap::how_to_get` writes the sentence a person reads when
    /// discovery finds nothing, and this file decides what `hick dap install`
    /// can actually do. They live in different crates because one is adapter
    /// knowledge and the other is a catalogue — so nothing but this test
    /// stops them drifting into a message that names a command that does not
    /// exist.
    #[test]
    fn nothing_is_offered_that_cannot_be_installed_and_nothing_installable_is_hidden() {
        for language in hick_dap::known_languages() {
            let installable = super::INSTALLERS.iter().any(|i| i.language == language)
                // The catalogue is keyed by ecosystem, not by language id:
                // one `typescript` installer serves every JavaScript flavour
                // discovery routes to the same adapter.
                || matches!(
                    language,
                    "javascript" | "typescriptreact" | "javascriptreact"
                );
            assert_eq!(
                hick_dap::suggests_hick_install(language),
                installable,
                "`{language}`: the message and the catalogue disagree about whether \
                 `hick dap install {language}` exists"
            );
        }
    }
}
