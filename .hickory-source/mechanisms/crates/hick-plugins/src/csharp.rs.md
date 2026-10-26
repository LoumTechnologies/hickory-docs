<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/csharp.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e6a9a65f387a"></a>
## `crates/hick-plugins/src/csharp.rs`

<a id="f-e6a9a65f387a-PLUGIN"></a>
### `PLUGIN`

The C# plugin: declarations only. The host owns each protocol and process.

<storycode:copy id="f-e6a9a65f387a-PLUGIN">
//! The C# plugin: declarations only. The host owns each protocol and process.
use crate::*;

pub static PLUGIN: Plugin = Plugin {
    id: "csharp",
    languages: &["csharp"],
    interfaces: &[
        Interface::Scaffold {
            catalog: crate::scaffold::catalog,
        },
        Interface::Project {
            markers: &["*.csproj", "*.sln", "*.slnx"],
        },
        Interface::Build {
            recipe: crate::build::Build::Command {
                project: "*.csproj",
                argv: &["dotnet", "build", "--nologo", "--configuration", "Debug"],
                artifact: "bin/Debug/*/*.dll",
                needs: "a `hick:file path=\"app/app.csproj\"` block, or `hick ingest` the one \
                    `dotnet new` writes",
            },
        },
        Interface::TestRunner { plan: test_command },
        Interface::TypedClient {
            languages: &["csharp"],
            target: crate::typed_client::emit::Target::CSharp,
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("cs", "csharp", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            // Keep existing machine discovery preference; csharp-ls is the
            // tool the explicit installer provides. Both speak stdio LSP.
            candidates: &[
                LspCandidate {
                    bin: "omnisharp",
                    args: &["-lsp"],
                },
                LspCandidate {
                    bin: "csharp-ls",
                    args: &[],
                },
            ],
            project_dirs: &[".hick-cache/servers/dotnet"],
            installer: Some(&LSP_INSTALLER),
        },
        // C# launch/build handling stays in hick-dap; the plugin declares
        // how to find and speak to its adapter.
        Interface::Dap {
            candidates: &[DebugCandidate {
                adapter: "netcoredbg",
                recipe: DebugRecipe::Binary {
                    bin: "netcoredbg",
                    args: &["--interpreter=vscode"],
                },
                transport: Transport::Stdio,
                hosted: false,
                launch: "{}",
                multi_session: false,
            }],
            project_dirs: &[".hick-cache/adapters/netcoredbg"],
            installer: Some(&DAP_INSTALLER),
        },
        Interface::CodeModel {
            bin: "hick-model-csharp",
        },
        Interface::Runtime {
            environment: &[
                Environment {
                    name: "DOTNET_CLI_HOME",
                    value: EnvValue::StatePath("dotnet"),
                },
                Environment {
                    name: "NUGET_PACKAGES",
                    value: EnvValue::StatePath("nuget/packages"),
                },
                Environment {
                    name: "DOTNET_CLI_TELEMETRY_OPTOUT",
                    value: EnvValue::Literal("1"),
                },
                Environment {
                    name: "DOTNET_NOLOGO",
                    value: EnvValue::Literal("1"),
                },
                Environment {
                    name: "DOTNET_GENERATE_ASPNET_CERTIFICATE",
                    value: EnvValue::Literal("false"),
                },
            ],
        },
    ],
};

</storycode:copy>
<a id="f-e6a9a65f387a-LSP_INSTALLER"></a>
### `LSP_INSTALLER`

<storycode:copy id="f-e6a9a65f387a-LSP_INSTALLER">
const LSP_INSTALLER: Installer = Installer {
    language: "csharp",
    tool: "dotnet",
    package: "csharp-ls",
    // Three environment variables, and every one of them is what makes
    // this installable at all: `dotnet tool install` writes to
    // `$HOME/.nuget/packages` and `$HOME/.dotnet` by default, and the
    // installer sandbox makes ONLY the prefix writable. Redirected, the
    // whole install — tool, package cache, CLI state — lands in one
    // directory `hick init` already ignores.
    //
    // The telemetry opt-out is not incidental either. This product says
    // nothing to anyone; a tool it spawns on the user's behalf must not
    // be the exception, and the .NET CLI phones home on first run unless
    // told not to.
    command: "DOTNET_CLI_HOME={prefix}/dotnet DOTNET_CLI_TELEMETRY_OPTOUT=1                   NUGET_PACKAGES={prefix}/dotnet/nuget                   dotnet tool install --tool-path {prefix}/dotnet csharp-ls",
    // Why this and not OmniSharp, which discovery has always looked for:
    // OmniSharp ships as a per-platform release archive, so installing it
    // means picking a URL and a checksum, while csharp-ls is a `dotnet
    // tool` — one command, the SDK's own package path, and the SDK is
    // already there if you are writing C#. Both are MIT; a machine that
    // has OmniSharp keeps using it, because discovery prefers it.
    //
    // Not Microsoft's own Roslyn language server: it ships inside the C#
    // extension under a licence that permits use only with Microsoft's
    // editors, which is not something to install on someone's behalf.
    assets: &[],
    reason: "a Roslyn-based C# server that installs as a dotnet tool",
};

</storycode:copy>
<a id="f-e6a9a65f387a-DAP_INSTALLER"></a>
### `DAP_INSTALLER`

<storycode:copy id="f-e6a9a65f387a-DAP_INSTALLER">
const DAP_INSTALLER: Installer = Installer {
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
};

</storycode:copy>
<a id="f-e6a9a65f387a-NETCOREDBG"></a>
### `NETCOREDBG`

netcoredbg 3.2.0-1092, pinned per platform.
Every archive unpacks to a `netcoredbg/` directory holding the binary and
its managed DLLs, which is why discovery looks for
`.hick-cache/adapters/netcoredbg/netcoredbg` rather than a `bin/`.
The checksums were taken from the four assets on 2026-08-27 by fetching
each one. Only `linux-x86_64` has been unpacked and run; the other three
are pinned bytes nobody here has executed, and the guarantee says so.
Bumping the version means replacing all four together — a mixed set would
install one release's binary against another's checksum and fail the
verification rather than do anything dangerous, which is the failure mode
worth having.

<storycode:copy id="f-e6a9a65f387a-NETCOREDBG">
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

</storycode:copy>
<a id="f-e6a9a65f387a-test_command"></a>
### `test_command`

<storycode:copy id="f-e6a9a65f387a-test_command">
fn test_command(
    root: &std::path::Path,
    file: &std::path::Path,
    name: &str,
) -> Result<crate::testing::TestCommand, String> {
    use crate::testing::{TestCommand, nearest_glob};
    let rel = file
        .strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    let name = name.trim();
    let title = if name.is_empty() {
        format!("test: {rel}")
    } else {
        format!("test: {name}")
    };
    let s = |text: &str| text.to_string();

    let dir = nearest_glob(root, file, ".csproj")
        .ok_or_else(|| format!("{rel} is not inside a project: no .csproj above it"))?;
    let mut argv = vec![s("dotnet"), s("test")];
    if !name.is_empty() {
        argv.push(s("--filter"));
        argv.push(format!("FullyQualifiedName~{name}"));
    }
    Ok(TestCommand {
        title,
        cwd: dir,
        argv,
    })
}
</storycode:copy>

### Assemble `crates/hick-plugins/src/csharp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/csharp.rs"><storycode:paste select="#f-e6a9a65f387a-PLUGIN"/><storycode:paste select="#f-e6a9a65f387a-LSP_INSTALLER"/><storycode:paste select="#f-e6a9a65f387a-DAP_INSTALLER"/><storycode:paste select="#f-e6a9a65f387a-NETCOREDBG"/><storycode:paste select="#f-e6a9a65f387a-test_command"/></storycode:file>

</storycode:doc>
