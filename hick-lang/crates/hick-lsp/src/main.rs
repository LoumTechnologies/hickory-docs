use tower_lsp::{LspService, Server};
use tracing_subscriber::EnvFilter;

mod backend;
mod child_lsp;
mod dispatcher;
mod document;
mod lang_detect;
mod position_map;
mod virtual_file;

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
