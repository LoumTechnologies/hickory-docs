/// Language tag for an output path, matching the hosted server's mapping.
/// `pub(super)` because plain files (serve/plain_file.rs) tag with the same
/// mapping — a `.py` file must highlight the same whether woven or plain.
pub(in crate::serve) fn language_of(path: &str) -> String {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "rust",
        "py" => "python",
        "ts" => "typescript",
        "tsx" => "typescript",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "javascript",
        "go" => "go",
        "cs" => "csharp",
        "sh" | "bash" => "shell",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        other => other,
    }
    .to_string()
}
