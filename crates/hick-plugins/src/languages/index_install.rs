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
