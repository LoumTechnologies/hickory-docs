use std::sync::Arc;

use clap::Parser;

use wrist_flick::config::WristFlickConfig;
use wrist_flick::llm::LlmClient;
use wrist_flick::llm::anthropic::AnthropicStreamingClient;
use wrist_flick::llm::openai::OpenAiStreamingClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = WristFlickConfig::parse();

    let llm: Arc<dyn LlmClient> = match config.llm.as_str() {
        "anthropic" => {
            let model = config
                .model
                .unwrap_or_else(|| "claude-sonnet-4-5-20250929".to_string());
            Arc::new(AnthropicStreamingClient::with_model(model))
        }
        _ => {
            let model = config.model.unwrap_or_else(|| "gpt-4o-mini".to_string());
            Arc::new(OpenAiStreamingClient::with_model(model))
        }
    };

    let state = wrist_flick::build_app_state(llm, config.db_path.as_deref()).await?;
    let app = wrist_flick::build_router(state);

    let addr = format!("0.0.0.0:{}", config.port);
    log::info!("wrist-flick listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
