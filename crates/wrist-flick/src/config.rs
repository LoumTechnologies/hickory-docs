use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "wrist-flick", about = "Personal AI assistant")]
pub struct WristFlickConfig {
    /// Port to listen on
    #[arg(long, default_value = "3000", env = "WRIST_FLICK_PORT")]
    pub port: u16,

    /// LLM provider: "openai" or "anthropic"
    #[arg(long, default_value = "openai", env = "WRIST_FLICK_LLM")]
    pub llm: String,

    /// LLM model name
    #[arg(long, env = "WRIST_FLICK_MODEL")]
    pub model: Option<String>,

    /// SQLite database path (default: in-memory)
    #[arg(long, env = "WRIST_FLICK_DB")]
    pub db_path: Option<String>,
}
