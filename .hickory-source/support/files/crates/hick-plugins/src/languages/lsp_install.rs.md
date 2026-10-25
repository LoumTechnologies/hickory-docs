<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/lsp_install.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e0eebb37d5e9f6ccf189b60a464e605ee4c835379182c424b5f35ca581843c25`.

<book:entry path="crates/hick-plugins/src/languages/lsp_install.rs">
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

pub static JSON: Installer = Installer {
    language: "json",
    tool: "npm",
    package: "vscode-langservers-extracted",
    command: "npm install --no-fund --no-audit --prefix {prefix}/node vscode-langservers-extracted",
    assets: &[],
    reason: "VS Code's own JSON, HTML and CSS servers, extracted; one install covers all three",
};

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

const JDT_LS_URL: &str =
    "https://open-vsx.org/api/redhat/java/1.57.2026090408/file/redhat.java-1.57.2026090408.vsix";
const JDT_LS_SHA256: &str = "2249b3669443f88e82243ad4fae8b1247741bafe341f52d1e6d15cd2e720ed6e";
</book:entry>

</book:doc>
