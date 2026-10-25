<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/index_install.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3972cf768502ca6c06a8056c622f92f443335c91d8c9b5f3a9e11d93b08fb89e`.

<book:entry path="crates/hick-plugins/src/languages/index_install.rs">
use crate::Installer;
pub static TYPESCRIPT: Installer = Installer {
    language: "typescript",
    tool: "npm",
    package: "@sourcegraph/scip-typescript",
    command: "npm install --no-fund --no-audit --prefix {prefix}/node @sourcegraph/scip-typescript",
    assets: &[],
    reason: "Sourcegraph's TypeScript indexer, and the reference SCIP producer",
};

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
</book:entry>

</book:doc>
