use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("scratch folder"));
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: Some(root.join("keys.json")),
        ui_settings_path: Some(root.join("ui.json")),
    })
    .await?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    println!("http://{}", listener.local_addr()?);
    axum::serve(listener, prepared.router).await?;
    Ok(())
}
