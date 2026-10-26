<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/main.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c5b2404f1380"></a>
## `crates/hick-lsp/src/main.rs`

<a id="f-c5b2404f1380-main"></a>
### `main`

<storycode:copy id="f-c5b2404f1380-main">
use tower_lsp::{LspService, Server};
use tracing_subscriber::EnvFilter;

use hick_lsp::backend;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(backend::HickBackend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/main.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/main.rs"><storycode:paste select="#f-c5b2404f1380-main"/></storycode:file>

</storycode:doc>
