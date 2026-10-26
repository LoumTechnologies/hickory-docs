<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/dap_install.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d008b50ec937"></a>
## `crates/hick-plugins/src/languages/dap_install.rs`

<a id="f-d008b50ec937-PYTHON"></a>
### `PYTHON`

<storycode:copy id="f-d008b50ec937-PYTHON">
use crate::{Asset, Installer};
pub static PYTHON: Installer = Installer {
    language: "python",
    tool: "uv",
    package: "debugpy",
    // A real virtualenv, for the same reason the language-server
    // installer uses one: `pyvenv.cfg` is how discovery recognises it,
    // and the interpreter that has debugpy must be the interpreter that
    // runs the cell — a machine python paired with a venv's debugpy
    // fails in a way that reads as our bug.
    command: "uv venv --allow-existing {prefix}/python && uv pip install --python {prefix}/python/bin/python debugpy",
    assets: &[],
    reason: "Microsoft's Python debugger, and the reference DAP implementation",
};

</storycode:copy>
<a id="f-d008b50ec937-TYPESCRIPT"></a>
### `TYPESCRIPT`

<storycode:copy id="f-d008b50ec937-TYPESCRIPT">
pub static TYPESCRIPT: Installer = Installer {
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
};

</storycode:copy>
<a id="f-d008b50ec937-JAVA"></a>
### `JAVA`

<storycode:copy id="f-d008b50ec937-JAVA">
pub static JAVA: Installer = Installer {
    language: "java",
    tool: "curl",
    package: "java-debug",
    command: "",
    assets: JAVA_DEBUG,
    // EPL-1.0, and allowed: it is a jar loaded by a separate JVM — the
    // language server's — never linked into anything hick builds. That is
    // the same relationship this product already has with eclipse.jdt.ls
    // itself. See docs/specs/freeform/debugging-the-jvm.md, where the
    // decision is recorded rather than left implicit.
    reason: "the Java debugger, which runs as a plugin inside the language server",
};

</storycode:copy>
<a id="f-d008b50ec937-RUST"></a>
### `RUST`

<storycode:copy id="f-d008b50ec937-RUST">
pub static RUST: Installer = Installer {
    language: "rust",
    tool: "curl",
    package: "codelldb",
    command: "",
    assets: CODELLDB,
    // MIT, ships its own lldb, and speaks DAP over stdio when started
    // with no arguments. `lldb-dap` from LLVM is the other adapter
    // discovery looks for, and one the user installed is still preferred.
    reason: "the LLDB-based debugger VS Code's Rust users run, usable outside it",
};

</storycode:copy>
<a id="f-d008b50ec937-JAVA_DEBUG"></a>
### `JAVA_DEBUG`

<storycode:copy id="f-d008b50ec937-JAVA_DEBUG">
const JAVA_DEBUG: &[Asset] = &[
    Asset {
        os: "linux",
        arch: "x86_64",
        url: JAVA_DEBUG_URL,
        sha256: JAVA_DEBUG_SHA256,
        unpack: "unzip",
        into: "java-debug",
    },
    Asset {
        os: "linux",
        arch: "aarch64",
        url: JAVA_DEBUG_URL,
        sha256: JAVA_DEBUG_SHA256,
        unpack: "unzip",
        into: "java-debug",
    },
    Asset {
        os: "macos",
        arch: "aarch64",
        url: JAVA_DEBUG_URL,
        sha256: JAVA_DEBUG_SHA256,
        unpack: "unzip",
        into: "java-debug",
    },
    Asset {
        os: "macos",
        arch: "x86_64",
        url: JAVA_DEBUG_URL,
        sha256: JAVA_DEBUG_SHA256,
        unpack: "unzip",
        into: "java-debug",
    },
    Asset {
        os: "windows",
        arch: "x86_64",
        url: JAVA_DEBUG_URL,
        sha256: JAVA_DEBUG_SHA256,
        unpack: "unzip",
        into: "java-debug",
    },
];

</storycode:copy>
<a id="f-d008b50ec937-JAVA_DEBUG_URL"></a>
### `JAVA_DEBUG_URL`

<storycode:copy id="f-d008b50ec937-JAVA_DEBUG_URL">
const JAVA_DEBUG_URL: &str = "https://open-vsx.org/api/vscjava/vscode-java-debug/0.59.0/file/vscjava.vscode-java-debug-0.59.0.vsix";
</storycode:copy>
<a id="f-d008b50ec937-JAVA_DEBUG_SHA256"></a>
### `JAVA_DEBUG_SHA256`

<storycode:copy id="f-d008b50ec937-JAVA_DEBUG_SHA256">
const JAVA_DEBUG_SHA256: &str = "87627e24dbb5b01137decc0265f043cb08adad22af3c195f1ba39898dafb1588";

</storycode:copy>
<a id="f-d008b50ec937-JS_DEBUG"></a>
### `JS_DEBUG`

js-debug v1.117.0.
One tarball for every platform: it is JavaScript, so the bytes really are
identical everywhere and the same checksum is the honest answer for each
row rather than a copy-paste. The archive carries its own `js-debug/`
top-level directory, so it unpacks into the prefix itself.
The checksum was taken on 2026-09-04 by fetching the asset. Only
`linux-x86_64` has been unpacked and run.

<storycode:copy id="f-d008b50ec937-JS_DEBUG">
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

</storycode:copy>
<a id="f-d008b50ec937-JS_DEBUG_URL"></a>
### `JS_DEBUG_URL`

<storycode:copy id="f-d008b50ec937-JS_DEBUG_URL">
const JS_DEBUG_URL: &str = "https://github.com/microsoft/vscode-js-debug/releases/download/v1.117.0/js-debug-dap-v1.117.0.tar.gz";
</storycode:copy>
<a id="f-d008b50ec937-JS_DEBUG_SHA256"></a>
### `JS_DEBUG_SHA256`

<storycode:copy id="f-d008b50ec937-JS_DEBUG_SHA256">
const JS_DEBUG_SHA256: &str = "ad8d04ede9d4b75cc290fd5438a65047a06f786d04f604b6112485b36f090772";

</storycode:copy>
<a id="f-d008b50ec937-CODELLDB"></a>
### `CODELLDB`

codelldb v1.12.3, pinned per platform.
Published as a VS Code extension archive (`.vsix`, which is a zip of
`extension/…`), so each one unpacks INTO `codelldb/` and the adapter is
`.hick-cache/adapters/codelldb/extension/adapter/codelldb`, with the lldb
it bundles beside it. The checksums were taken from the five assets on
2026-09-02 by fetching each one. Only `linux-x86_64` has been unpacked and
run; the other four are pinned bytes nobody here has executed.

<storycode:copy id="f-d008b50ec937-CODELLDB">
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
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/dap_install.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/dap_install.rs"><storycode:paste select="#f-d008b50ec937-PYTHON"/><storycode:paste select="#f-d008b50ec937-TYPESCRIPT"/><storycode:paste select="#f-d008b50ec937-JAVA"/><storycode:paste select="#f-d008b50ec937-RUST"/><storycode:paste select="#f-d008b50ec937-JAVA_DEBUG"/><storycode:paste select="#f-d008b50ec937-JAVA_DEBUG_URL"/><storycode:paste select="#f-d008b50ec937-JAVA_DEBUG_SHA256"/><storycode:paste select="#f-d008b50ec937-JS_DEBUG"/><storycode:paste select="#f-d008b50ec937-JS_DEBUG_URL"/><storycode:paste select="#f-d008b50ec937-JS_DEBUG_SHA256"/><storycode:paste select="#f-d008b50ec937-CODELLDB"/></storycode:file>

</storycode:doc>
