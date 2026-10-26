<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/lsp_install.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-867d2716fa76"></a>
## `crates/hick-plugins/src/languages/lsp_install.rs`

<a id="f-867d2716fa76-PYTHON"></a>
### `PYTHON`

<storycode:copy id="f-867d2716fa76-PYTHON">
use crate::{Asset, Installer};
pub static PYTHON: Installer = Installer {
    language: "python",
    tool: "uv",
    package: "basedpyright",
    // A real virtualenv rather than a --target install: the console
    // scripts land in bin/, which is where discovery already looks, and
    // `pyvenv.cfg` is how it recognises a virtualenv at all.
    command: "uv venv --allow-existing {prefix}/python && uv pip install --python {prefix}/python/bin/python basedpyright",
    // The answer to "why not pyright": pyright implements no semantic
    // tokens, no inlay hints and no folding ranges — those live in
    // Pylance, which is licensed for Microsoft's editors only.
    // basedpyright is the open-source fork that reimplements them, so
    // this is what makes Python code in a document actually coloured.
    assets: &[],
    reason: "pyright with the Pylance-only features (semantic tokens, inlay hints) added",
};

</storycode:copy>
<a id="f-867d2716fa76-TYPESCRIPT"></a>
### `TYPESCRIPT`

<storycode:copy id="f-867d2716fa76-TYPESCRIPT">
pub static TYPESCRIPT: Installer = Installer {
    language: "typescript",
    tool: "npm",
    package: "typescript-language-server typescript@5",
    // Pinned to 5, and that pin is load-bearing rather than caution:
    // `typescript` on npm is 7.x now, the native port, which ships no
    // `tsserver.js` at all. typescript-language-server is a wrapper
    // AROUND tsserver, so the obvious `npm install typescript
    // typescript-language-server` installs two packages that cannot work
    // together — the server starts and fails `initialize` with "Could not
    // find a valid tsserver".
    command: "npm install --no-fund --no-audit --prefix {prefix}/node typescript-language-server typescript@5",
    assets: &[],
    reason: "the server the TypeScript ecosystem treats as the default",
};

</storycode:copy>
<a id="f-867d2716fa76-JAVA"></a>
### `JAVA`

<storycode:copy id="f-867d2716fa76-JAVA">
pub static JAVA: Installer = Installer {
    language: "java",
    tool: "curl",
    package: "eclipse.jdt.ls",
    // Nothing to run: an archive install builds its command from the
    // matching asset below.
    command: "",
    assets: JDT_LS,
    // Taken from the `redhat.java` extension rather than from
    // download.eclipse.org, and the reason is the pin. jdt.ls publishes
    // `jdt-language-server-latest.tar.gz`, whose URL is stable and whose
    // BYTES change daily, so a SHA-256 against it would break within a
    // day; its milestone directories have no listing to resolve a version
    // from. The extension is published per version, so its bytes are
    // fixed — which is what an archive installer pins.
    reason: "the Java language server, and the host the debugger runs inside",
};

</storycode:copy>
<a id="f-867d2716fa76-JSON"></a>
### `JSON`

<storycode:copy id="f-867d2716fa76-JSON">
pub static JSON: Installer = Installer {
    language: "json",
    tool: "npm",
    package: "vscode-langservers-extracted",
    command: "npm install --no-fund --no-audit --prefix {prefix}/node vscode-langservers-extracted",
    assets: &[],
    reason: "VS Code's own JSON, HTML and CSS servers, extracted; one install covers all three",
};

</storycode:copy>
<a id="f-867d2716fa76-JDT_LS"></a>
### `JDT_LS`

<storycode:copy id="f-867d2716fa76-JDT_LS">
const JDT_LS: &[Asset] = &[
    Asset {
        os: "linux",
        arch: "x86_64",
        url: JDT_LS_URL,
        sha256: JDT_LS_SHA256,
        unpack: "unzip",
        into: "java",
    },
    Asset {
        os: "linux",
        arch: "aarch64",
        url: JDT_LS_URL,
        sha256: JDT_LS_SHA256,
        unpack: "unzip",
        into: "java",
    },
    Asset {
        os: "macos",
        arch: "aarch64",
        url: JDT_LS_URL,
        sha256: JDT_LS_SHA256,
        unpack: "unzip",
        into: "java",
    },
    Asset {
        os: "macos",
        arch: "x86_64",
        url: JDT_LS_URL,
        sha256: JDT_LS_SHA256,
        unpack: "unzip",
        into: "java",
    },
    Asset {
        os: "windows",
        arch: "x86_64",
        url: JDT_LS_URL,
        sha256: JDT_LS_SHA256,
        unpack: "unzip",
        into: "java",
    },
];

</storycode:copy>
<a id="f-867d2716fa76-JDT_LS_URL"></a>
### `JDT_LS_URL`

<storycode:copy id="f-867d2716fa76-JDT_LS_URL">
const JDT_LS_URL: &str =
    "https://open-vsx.org/api/redhat/java/1.57.2026090408/file/redhat.java-1.57.2026090408.vsix";
</storycode:copy>
<a id="f-867d2716fa76-JDT_LS_SHA256"></a>
### `JDT_LS_SHA256`

<storycode:copy id="f-867d2716fa76-JDT_LS_SHA256">
const JDT_LS_SHA256: &str = "2249b3669443f88e82243ad4fae8b1247741bafe341f52d1e6d15cd2e720ed6e";
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/lsp_install.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/lsp_install.rs"><storycode:paste select="#f-867d2716fa76-PYTHON"/><storycode:paste select="#f-867d2716fa76-TYPESCRIPT"/><storycode:paste select="#f-867d2716fa76-JAVA"/><storycode:paste select="#f-867d2716fa76-JSON"/><storycode:paste select="#f-867d2716fa76-JDT_LS"/><storycode:paste select="#f-867d2716fa76-JDT_LS_URL"/><storycode:paste select="#f-867d2716fa76-JDT_LS_SHA256"/></storycode:file>

</storycode:doc>
