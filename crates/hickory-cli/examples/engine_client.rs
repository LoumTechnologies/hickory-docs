//! Real process fixture: the same client proxy and lifecycle as a desktop.
use hickory_cli::{ExecutorChoice, engine};
fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Some(result) = engine::run_argv(&args) {
        return result;
    }
    tokio::runtime::Runtime::new()?.block_on(run(args))
}

async fn run(args: Vec<String>) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let root = std::path::PathBuf::from(&args[0]);
    let name = args.get(1).cloned().unwrap_or_else(|| "window".into());
    let mut opts = engine::Attach::new(root.clone(), ExecutorChoice::Local);
    opts.callback = Some(format!("http://{}", listener.local_addr()?));
    opts.callback_token = engine::window_token();
    let token = opts.callback_token.clone();
    let picked = root.join(name);
    let shell = hickory_cli::serve::Shell {
        pick_folder: std::sync::Arc::new(move |_| Ok(Some(picked.clone()))),
        save_file: std::sync::Arc::new(|_, _| Ok(None)),
        open_folder: std::sync::Arc::new(|_, _| Ok(())),
        close_window: std::sync::Arc::new(|| Ok(())),
    };
    let connection = engine::connect(opts).await?;
    let _guard = connection.guard();
    let router = engine::client_router(connection, Some(shell), token, args.get(2).cloned());
    println!("http://{}", listener.local_addr()?);
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
