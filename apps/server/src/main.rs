use anyhow::Result;
use hickory_server::{Config, build_router, build_state, init_db};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = Config::from_env()?;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    // No port here on purpose — with PORT=0 it is not known until after bind.
    log::info!(
        "hickory-server booting: env={:?} executor={}",
        config.app_env,
        config.executor.as_str()
    );

    let db = init_db(&config.database_url).await?;
    let state = build_state(config, db)?;
    let router = build_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    // Log what we ACTUALLY bound, not what we asked for: dev runs with PORT=0
    // so the kernel picks the port, and "listening on 0.0.0.0:0" would tell
    // nobody anything. Port Zero discovers this port and publishes it at a
    // stable name, but the log is still the ground truth when it doesn't.
    log::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;
    Ok(())
}
