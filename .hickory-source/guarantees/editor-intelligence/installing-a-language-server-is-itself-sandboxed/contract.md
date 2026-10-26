<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Installing A Language Server Is Itself Sandboxed

For: Authors using completion, diagnostics and refactoring inside literate source.

Given `hick lsp install <language>`, when the server is fetched, then the
installer runs **inside the same sandbox a document's cells run in**: it may
write only `.hick-cache/servers/`, its `HOME` is redirected inside that
directory, and it is the one and only place in this product that is granted
network access. Where no sandbox is available, the install is **refused**.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/installing-a-language-server-is-itself-sandboxed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-7c1a3943c2b0"></a>
## `crates/hickory-cli/src/lsp_install.rs`

Related contracts: [Installing A Language Server Is Itself Sandboxed](contract.md).

<a id="f-7c1a3943c2b0-SERVERS_DIR"></a>
### `SERVERS_DIR`

Which language servers `hick lsp install` can fetch.
The machinery — the sandbox, the prefix, the refusals — is shared with
`hick dap install`; see [`crate::tool_install`]. This file is the
catalogue and nothing else.

<storycode:copy id="f-7c1a3943c2b0-SERVERS_DIR">
//! Which language servers `hick lsp install` can fetch.
//!
//! The machinery — the sandbox, the prefix, the refusals — is shared with
//! `hick dap install`; see [`crate::tool_install`]. This file is the
//! catalogue and nothing else.

use crate::tool_install::{Catalogue, Installer};

/// Where a language server is installed, relative to the project root.
pub const SERVERS_DIR: &str = ".hick-cache/servers";

</storycode:copy>
<a id="f-7c1a3943c2b0-CATALOGUE"></a>
### `CATALOGUE`

<storycode:copy id="f-7c1a3943c2b0-CATALOGUE">
pub static CATALOGUE: std::sync::LazyLock<Catalogue> = std::sync::LazyLock::new(|| Catalogue {
    what: "language server",
    prefix: SERVERS_DIR,
    command: "hick lsp install",
    installers: &INSTALLERS,
});

</storycode:copy>
<a id="f-7c1a3943c2b0-plans"></a>
### `plans`

The installable set.
Deliberately short. Every entry is a package this project is willing to
point a network install at, chosen because it is the server that language's
own ecosystem treats as the default — not because it was easy to wire up.
Every language this can install, with whether the machine can.

<storycode:copy id="f-7c1a3943c2b0-plans">
/// The installable set.
///
/// Deliberately short. Every entry is a package this project is willing to
/// point a network install at, chosen because it is the server that language's
/// own ecosystem treats as the default — not because it was easy to wire up.

/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<crate::tool_install::InstallPlan> {
    crate::tool_install::plans(&CATALOGUE)
}

</storycode:copy>
<a id="f-7c1a3943c2b0-install"></a>
### `install`

<storycode:copy id="f-7c1a3943c2b0-install">
pub fn install(root: &std::path::Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    crate::tool_install::install(
        &CATALOGUE,
        root,
        hick_plugins::registry()
            .interface(language, hick_plugins::InterfaceKind::Lsp)
            .and_then(hick_plugins::Interface::installer)
            .map(|installer| installer.language)
            .unwrap_or(language),
    )
}

</storycode:copy>
<a id="f-7c1a3943c2b0-INSTALLERS"></a>
### `INSTALLERS`

Compatibility view; plugin declarations are owned by hick-plugins.

<storycode:copy id="f-7c1a3943c2b0-INSTALLERS">
// Compatibility view; plugin declarations are owned by hick-plugins.
static INSTALLERS: std::sync::LazyLock<Vec<Installer>> = std::sync::LazyLock::new(|| {
    hick_plugins::registry()
        .installers(hick_plugins::InterfaceKind::Lsp)
        .copied()
        .collect()
});
</storycode:copy>

### Assemble `crates/hickory-cli/src/lsp_install.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/lsp_install.rs"><storycode:paste select="#f-7c1a3943c2b0-SERVERS_DIR"/><storycode:paste select="#f-7c1a3943c2b0-CATALOGUE"/><storycode:paste select="#f-7c1a3943c2b0-plans"/><storycode:paste select="#f-7c1a3943c2b0-install"/><storycode:paste select="#f-7c1a3943c2b0-INSTALLERS"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-lsp/src/discovery.rs](../../../mechanisms/crates/hick-lsp/src/discovery.rs.md#output-9369ac0436fb)
- [crates/hick-plugins/src/lsp_discovery.rs](../../../mechanisms/crates/hick-plugins/src/lsp_discovery.rs.md#output-25da6e354bac)
- [crates/hickory-cli/src/lsp_install.rs](contract.md#output-7c1a3943c2b0)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-executor-sandbox/src/policy.rs](../../../mechanisms/crates/hickory-executor-sandbox/src/policy.rs.md#output-875ceba98dc8)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
