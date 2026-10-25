<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/metadata.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6604460b1330a364e8f245f2afea8c3c0ca9bf026d5b615b432c163f2c2d839a`.

<book:entry path="crates/hick-plugins/src/metadata.rs">
//! Ecosystem advice is distinct from implemented provider capabilities.
pub fn index_advice(language: &str) -> String {
    match language {
        // One indexer serves every JavaScript flavour, and the catalogue is
        // keyed by ecosystem — so the command to name is `typescript`, not
        // whichever id the file happened to have. Naming the id would print a
        // command that fails with "no installer for 'javascript'".
        "typescript" | "javascript" | "typescriptreact" | "javascriptreact" => {
            "Install one with `hick index install typescript` — it indexes JavaScript too."
                .to_string()
        }
        "python" => "Install one with `hick index install python`.".to_string(),
        // rust-analyzer emits SCIP itself, so anybody writing Rust already has
        // the indexer and does not need a second one.
        "rust" => "rust-analyzer emits SCIP itself: `rust-analyzer scip .`. If you have \
                   rust-analyzer, you have the indexer."
            .to_string(),
        "csharp" => "scip-dotnet is a `dotnet tool`: `dotnet tool install --global \
                     scip-dotnet`. hick will find it on PATH."
            .to_string(),
        "java" | "kotlin" | "scala" => "Sourcegraph's scip-java covers these; it installs \
                                        through Coursier rather than through hick."
            .to_string(),
        _ => format!("hick has no indexer for {language}."),
    }
}

pub fn is_data_language(language: &str) -> bool {
    matches!(
        language,
        "json" | "yaml" | "toml" | "markdown" | "html" | "css" | "sql" | "xml"
    )
}
pub fn indexed_by_the_ecosystem(language: &str) -> bool {
    matches!(
        language,
        "rust" | "csharp" | "java" | "scala" | "kotlin" | "go" | "c" | "cpp" | "ruby"
    )
}

pub fn fence_language(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "rust",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "cs" => "csharp",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "sh" | "bash" => "bash",
        // `.csproj` and friends are XML with a different name on them — a
        // document that ingests `dotnet new` gets one whether or not it asked.
        // Kept in step with `ALIASES` in apps/web/src/editor/languages.ts, so
        // the fence in the woven markdown and the colouring in the app agree
        // about what a file is.
        "xml" | "hick" | "csproj" | "props" | "targets" | "xaml" | "xsd" => "xml",
        "html" | "htm" => "html",
        "css" => "css",
        "go" => "go",
        "java" => "java",
        "sql" => "sql",
        "md" | "markdown" => "markdown",
        "rb" => "ruby",
        "php" => "php",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "zig" => "zig",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        "gradle" => "groovy",
        _ => crate::routing::language_id(path).unwrap_or(""),
    }
}
pub fn lineage_language(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or_default();
    match ext.to_ascii_lowercase().as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        "sh" | "bash" => "shell",
        "html" | "htm" => "html",
        "css" => "css",
        "sql" => "sql",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "hpp" | "cxx" => "cpp",
        "rb" => "ruby",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "hick" => "hick",
        "xml" => "xml",
        "txt" => "text",
        _ => crate::routing::language_id(path).unwrap_or("text"),
    }
}
</book:entry>

</book:doc>
