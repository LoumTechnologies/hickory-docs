use tower_lsp::Server;
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

    let (service, socket) = hick_lsp::service(backend::HickBackend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
