<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/lsp.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8c0ceaf40bf9"></a>
## `crates/hick-plugins/src/languages/lsp.rs`

<a id="f-8c0ceaf40bf9-c"></a>
### `c`

<storycode:copy id="f-8c0ceaf40bf9-c">
use crate::LspCandidate as Candidate;
const fn c(bin: &'static str, args: &'static [&'static str]) -> Candidate {
    Candidate { bin, args }
}
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_RUST"></a>
### `C_RUST`

<storycode:copy id="f-8c0ceaf40bf9-C_RUST">
pub const C_RUST: &[Candidate] = &[c("rust-analyzer", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_PYTHON"></a>
### `C_PYTHON`

<storycode:copy id="f-8c0ceaf40bf9-C_PYTHON">
pub const C_PYTHON: &[Candidate] = &[
    c("basedpyright-langserver", &["--stdio"]),
    c("pyright-langserver", &["--stdio"]),
    c("pylsp", &[]),
    c("jedi-language-server", &[]),
    c("ruff", &["server"]),
];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_TYPESCRIPT"></a>
### `C_TYPESCRIPT`

<storycode:copy id="f-8c0ceaf40bf9-C_TYPESCRIPT">
pub const C_TYPESCRIPT: &[Candidate] = &[
    c("typescript-language-server", &["--stdio"]),
    c("vtsls", &["--stdio"]),
    c("deno", &["lsp"]),
];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_GO"></a>
### `C_GO`

<storycode:copy id="f-8c0ceaf40bf9-C_GO">
pub const C_GO: &[Candidate] = &[c("gopls", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_C"></a>
### `C_C`

<storycode:copy id="f-8c0ceaf40bf9-C_C">
pub const C_C: &[Candidate] = &[c("clangd", &[]), c("ccls", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_JAVA"></a>
### `C_JAVA`

<storycode:copy id="f-8c0ceaf40bf9-C_JAVA">
pub const C_JAVA: &[Candidate] = &[c("jdtls", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_RUBY"></a>
### `C_RUBY`

<storycode:copy id="f-8c0ceaf40bf9-C_RUBY">
pub const C_RUBY: &[Candidate] = &[c("ruby-lsp", &[]), c("solargraph", &["stdio"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_PHP"></a>
### `C_PHP`

<storycode:copy id="f-8c0ceaf40bf9-C_PHP">
pub const C_PHP: &[Candidate] = &[
    c("intelephense", &["--stdio"]),
    c("phpactor", &["language-server"]),
];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_KOTLIN"></a>
### `C_KOTLIN`

<storycode:copy id="f-8c0ceaf40bf9-C_KOTLIN">
pub const C_KOTLIN: &[Candidate] = &[c("kotlin-language-server", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_SWIFT"></a>
### `C_SWIFT`

<storycode:copy id="f-8c0ceaf40bf9-C_SWIFT">
pub const C_SWIFT: &[Candidate] = &[c("sourcekit-lsp", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_SCALA"></a>
### `C_SCALA`

<storycode:copy id="f-8c0ceaf40bf9-C_SCALA">
pub const C_SCALA: &[Candidate] = &[c("metals", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_ELIXIR"></a>
### `C_ELIXIR`

<storycode:copy id="f-8c0ceaf40bf9-C_ELIXIR">
pub const C_ELIXIR: &[Candidate] = &[c("elixir-ls", &[]), c("lexical", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_HASKELL"></a>
### `C_HASKELL`

<storycode:copy id="f-8c0ceaf40bf9-C_HASKELL">
pub const C_HASKELL: &[Candidate] = &[c("haskell-language-server-wrapper", &["--lsp"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_LUA"></a>
### `C_LUA`

<storycode:copy id="f-8c0ceaf40bf9-C_LUA">
pub const C_LUA: &[Candidate] = &[c("lua-language-server", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_ZIG"></a>
### `C_ZIG`

<storycode:copy id="f-8c0ceaf40bf9-C_ZIG">
pub const C_ZIG: &[Candidate] = &[c("zls", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_NIX"></a>
### `C_NIX`

<storycode:copy id="f-8c0ceaf40bf9-C_NIX">
pub const C_NIX: &[Candidate] = &[c("nil", &[]), c("nixd", &[])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_JSON"></a>
### `C_JSON`

<storycode:copy id="f-8c0ceaf40bf9-C_JSON">
pub const C_JSON: &[Candidate] = &[
    c("vscode-json-language-server", &["--stdio"]),
    c("biome", &["lsp-proxy"]),
];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_YAML"></a>
### `C_YAML`

<storycode:copy id="f-8c0ceaf40bf9-C_YAML">
pub const C_YAML: &[Candidate] = &[c("yaml-language-server", &["--stdio"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_TOML"></a>
### `C_TOML`

<storycode:copy id="f-8c0ceaf40bf9-C_TOML">
pub const C_TOML: &[Candidate] = &[c("taplo", &["lsp", "stdio"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_HTML"></a>
### `C_HTML`

<storycode:copy id="f-8c0ceaf40bf9-C_HTML">
pub const C_HTML: &[Candidate] = &[c("vscode-html-language-server", &["--stdio"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_CSS"></a>
### `C_CSS`

<storycode:copy id="f-8c0ceaf40bf9-C_CSS">
pub const C_CSS: &[Candidate] = &[c("vscode-css-language-server", &["--stdio"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_MARKDOWN"></a>
### `C_MARKDOWN`

<storycode:copy id="f-8c0ceaf40bf9-C_MARKDOWN">
pub const C_MARKDOWN: &[Candidate] = &[c("marksman", &["server"]), c("harper-ls", &["--stdio"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_SQL"></a>
### `C_SQL`

<storycode:copy id="f-8c0ceaf40bf9-C_SQL">
pub const C_SQL: &[Candidate] = &[c("sqls", &[]), c("postgrestools", &["lsp-proxy"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_TERRAFORM"></a>
### `C_TERRAFORM`

<storycode:copy id="f-8c0ceaf40bf9-C_TERRAFORM">
pub const C_TERRAFORM: &[Candidate] = &[c("terraform-ls", &["serve"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_DOCKERFILE"></a>
### `C_DOCKERFILE`

<storycode:copy id="f-8c0ceaf40bf9-C_DOCKERFILE">
pub const C_DOCKERFILE: &[Candidate] = &[c("docker-langserver", &["--stdio"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_SHELLSCRIPT"></a>
### `C_SHELLSCRIPT`

<storycode:copy id="f-8c0ceaf40bf9-C_SHELLSCRIPT">
pub const C_SHELLSCRIPT: &[Candidate] = &[c("bash-language-server", &["start"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_R"></a>
### `C_R`

<storycode:copy id="f-8c0ceaf40bf9-C_R">
pub const C_R: &[Candidate] = &[c("air", &["language-server"])];
</storycode:copy>
<a id="f-8c0ceaf40bf9-C_DART"></a>
### `C_DART`

<storycode:copy id="f-8c0ceaf40bf9-C_DART">
pub const C_DART: &[Candidate] = &[c("dart", &["language-server"])];
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/lsp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/lsp.rs"><storycode:paste select="#f-8c0ceaf40bf9-c"/><storycode:paste select="#f-8c0ceaf40bf9-C_RUST"/><storycode:paste select="#f-8c0ceaf40bf9-C_PYTHON"/><storycode:paste select="#f-8c0ceaf40bf9-C_TYPESCRIPT"/><storycode:paste select="#f-8c0ceaf40bf9-C_GO"/><storycode:paste select="#f-8c0ceaf40bf9-C_C"/><storycode:paste select="#f-8c0ceaf40bf9-C_JAVA"/><storycode:paste select="#f-8c0ceaf40bf9-C_RUBY"/><storycode:paste select="#f-8c0ceaf40bf9-C_PHP"/><storycode:paste select="#f-8c0ceaf40bf9-C_KOTLIN"/><storycode:paste select="#f-8c0ceaf40bf9-C_SWIFT"/><storycode:paste select="#f-8c0ceaf40bf9-C_SCALA"/><storycode:paste select="#f-8c0ceaf40bf9-C_ELIXIR"/><storycode:paste select="#f-8c0ceaf40bf9-C_HASKELL"/><storycode:paste select="#f-8c0ceaf40bf9-C_LUA"/><storycode:paste select="#f-8c0ceaf40bf9-C_ZIG"/><storycode:paste select="#f-8c0ceaf40bf9-C_NIX"/><storycode:paste select="#f-8c0ceaf40bf9-C_JSON"/><storycode:paste select="#f-8c0ceaf40bf9-C_YAML"/><storycode:paste select="#f-8c0ceaf40bf9-C_TOML"/><storycode:paste select="#f-8c0ceaf40bf9-C_HTML"/><storycode:paste select="#f-8c0ceaf40bf9-C_CSS"/><storycode:paste select="#f-8c0ceaf40bf9-C_MARKDOWN"/><storycode:paste select="#f-8c0ceaf40bf9-C_SQL"/><storycode:paste select="#f-8c0ceaf40bf9-C_TERRAFORM"/><storycode:paste select="#f-8c0ceaf40bf9-C_DOCKERFILE"/><storycode:paste select="#f-8c0ceaf40bf9-C_SHELLSCRIPT"/><storycode:paste select="#f-8c0ceaf40bf9-C_R"/><storycode:paste select="#f-8c0ceaf40bf9-C_DART"/></storycode:file>

</storycode:doc>
