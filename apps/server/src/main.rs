use anyhow::Result;
use hickory_server::{Config, build_router, build_state, init_db};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = Config::from_env()?;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    log::info!(
        "hickory-server booting: env={:?} executor={} port={}",
        config.app_env,
        config.executor.as_str(),
        config.port
    );

    let db = init_db(&config.database_url).await?;
    let state = build_state(config, db)?;
    let router = build_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("listening on {addr}");
    axum::serve(listener, router).await?;
    Ok(())
}
