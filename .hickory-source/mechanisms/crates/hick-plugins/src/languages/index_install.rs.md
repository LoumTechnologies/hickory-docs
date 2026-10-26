<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/index_install.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-aa3b33ad0f30"></a>
## `crates/hick-plugins/src/languages/index_install.rs`

<a id="f-aa3b33ad0f30-TYPESCRIPT"></a>
### `TYPESCRIPT`

<storycode:copy id="f-aa3b33ad0f30-TYPESCRIPT">
use crate::Installer;
pub static TYPESCRIPT: Installer = Installer {
    language: "typescript",
    tool: "npm",
    package: "@sourcegraph/scip-typescript",
    command: "npm install --no-fund --no-audit --prefix {prefix}/node @sourcegraph/scip-typescript",
    assets: &[],
    reason: "Sourcegraph's TypeScript indexer, and the reference SCIP producer",
};

</storycode:copy>
<a id="f-aa3b33ad0f30-PYTHON"></a>
### `PYTHON`

<storycode:copy id="f-aa3b33ad0f30-PYTHON">
pub static PYTHON: Installer = Installer {
    language: "python",
    tool: "npm",
    package: "@sourcegraph/scip-python",
    // npm, despite indexing Python: Sourcegraph publishes it there, and
    // the alternative is a pip install whose interpreter has to match the
    // project's — a mismatch that reads as our bug.
    command: "npm install --no-fund --no-audit --prefix {prefix}/node @sourcegraph/scip-python",
    assets: &[],
    reason: "Sourcegraph's Python indexer, published on npm",
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/index_install.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/index_install.rs"><storycode:paste select="#f-aa3b33ad0f30-TYPESCRIPT"/><storycode:paste select="#f-aa3b33ad0f30-PYTHON"/></storycode:file>

</storycode:doc>
