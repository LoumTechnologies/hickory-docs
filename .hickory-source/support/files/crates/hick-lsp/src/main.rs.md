<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lsp/src/main.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c6e31a14f720d0f93c1b215cecdae806600e8c9102f512bfd983218db0ace458`.

<book:entry path="crates/hick-lsp/src/main.rs">
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
</book:entry>

</book:doc>
