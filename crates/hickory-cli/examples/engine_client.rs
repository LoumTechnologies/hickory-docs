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
    if let Some(profile) = std::env::var_os("HICKORY_TEST_PRIVATE_PROFILE") {
        anyhow::ensure!(
            std::env::var_os("HICKORY_STATE_DIR").is_some(),
            "Private fixture requires isolated engine state"
        );
        let mut profile: hickory_cli::serve::PrivatePredictionStartup =
            serde_json::from_slice(&std::fs::read(profile)?)?;
        if profile.seal.is_empty()
            && let Some(runtime) = &profile.runtime
        {
            profile.identity.environment = runtime.capture().await?;
        }
        opts.prediction = Some(profile);
        if std::env::var_os("HICKORY_TEST_LIVE_PROVIDER").is_some() {
            opts.key_store_path = hickory_agent::KeyStore::desktop_path();
        }
        opts.ui_settings_path = std::env::var_os("HICKORY_AGENT_TEST_UI_SETTINGS").map(Into::into);
        opts.python_runtime_path = std::env::var_os("HICKORY_AGENT_TEST_PYTHON").map(Into::into);
    }
    opts.callback = Some(format!("http://{}", listener.local_addr()?));
    opts.callback_token = engine::window_token();
    let token = opts.callback_token.clone();
    let picked = root.join(name);
    let shell = hickory_cli::serve::Shell {
        window: std::sync::Arc::new(|_| anyhow::bail!("This browser fixture has no native window")),
        pick_folder: std::sync::Arc::new(move |_| Ok(Some(picked.clone()))),
        save_file: std::sync::Arc::new(|_, _| Ok(None)),
        open_folder: std::sync::Arc::new(|_, _| Ok(())),
        close_window: std::sync::Arc::new(|| Ok(())),
        cancel_quit: std::sync::Arc::new(|| {}),
    };
    let connection = engine::connect(opts).await?;
    let _guard = connection.guard();
    let router = engine::client_router(connection, Some(shell), token, args.get(2).cloned());
    println!("http://{}", listener.local_addr()?);
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
