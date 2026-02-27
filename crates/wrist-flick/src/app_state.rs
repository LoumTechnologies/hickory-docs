use std::sync::Arc;

use tokio::sync::Mutex;

use crate::llm::LlmClient;
use drift_signal_core::AppDriftSignal;

#[derive(Clone)]
pub struct AppState {
    pub core: Arc<Mutex<AppDriftSignal>>,
    pub llm: Arc<dyn LlmClient>,
}
