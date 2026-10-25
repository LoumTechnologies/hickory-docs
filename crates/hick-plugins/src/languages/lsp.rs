use crate::LspCandidate as Candidate;
const fn c(bin: &'static str, args: &'static [&'static str]) -> Candidate {
    Candidate { bin, args }
}
pub const C_RUST: &[Candidate] = &[c("rust-analyzer", &[])];
pub const C_PYTHON: &[Candidate] = &[
    c("basedpyright-langserver", &["--stdio"]),
    c("pyright-langserver", &["--stdio"]),
    c("pylsp", &[]),
    c("jedi-language-server", &[]),
    c("ruff", &["server"]),
];
pub const C_TYPESCRIPT: &[Candidate] = &[
    c("typescript-language-server", &["--stdio"]),
    c("vtsls", &["--stdio"]),
    c("deno", &["lsp"]),
];
pub const C_GO: &[Candidate] = &[c("gopls", &[])];
pub const C_C: &[Candidate] = &[c("clangd", &[]), c("ccls", &[])];
pub const C_JAVA: &[Candidate] = &[c("jdtls", &[])];
pub const C_RUBY: &[Candidate] = &[c("ruby-lsp", &[]), c("solargraph", &["stdio"])];
pub const C_PHP: &[Candidate] = &[
    c("intelephense", &["--stdio"]),
    c("phpactor", &["language-server"]),
];
pub const C_KOTLIN: &[Candidate] = &[c("kotlin-language-server", &[])];
pub const C_SWIFT: &[Candidate] = &[c("sourcekit-lsp", &[])];
pub const C_SCALA: &[Candidate] = &[c("metals", &[])];
pub const C_ELIXIR: &[Candidate] = &[c("elixir-ls", &[]), c("lexical", &[])];
pub const C_HASKELL: &[Candidate] = &[c("haskell-language-server-wrapper", &["--lsp"])];
pub const C_LUA: &[Candidate] = &[c("lua-language-server", &[])];
pub const C_ZIG: &[Candidate] = &[c("zls", &[])];
pub const C_NIX: &[Candidate] = &[c("nil", &[]), c("nixd", &[])];
pub const C_JSON: &[Candidate] = &[
    c("vscode-json-language-server", &["--stdio"]),
    c("biome", &["lsp-proxy"]),
];
pub const C_YAML: &[Candidate] = &[c("yaml-language-server", &["--stdio"])];
pub const C_TOML: &[Candidate] = &[c("taplo", &["lsp", "stdio"])];
pub const C_HTML: &[Candidate] = &[c("vscode-html-language-server", &["--stdio"])];
pub const C_CSS: &[Candidate] = &[c("vscode-css-language-server", &["--stdio"])];
pub const C_MARKDOWN: &[Candidate] = &[c("marksman", &["server"]), c("harper-ls", &["--stdio"])];
pub const C_SQL: &[Candidate] = &[c("sqls", &[]), c("postgrestools", &["lsp-proxy"])];
pub const C_TERRAFORM: &[Candidate] = &[c("terraform-ls", &["serve"])];
pub const C_DOCKERFILE: &[Candidate] = &[c("docker-langserver", &["--stdio"])];
pub const C_SHELLSCRIPT: &[Candidate] = &[c("bash-language-server", &["start"])];
pub const C_R: &[Candidate] = &[c("air", &["language-server"])];
pub const C_DART: &[Candidate] = &[c("dart", &["language-server"])];
