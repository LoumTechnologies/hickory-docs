//! Operator request logging for missing operators.

use serde::{Deserialize, Serialize};

/// A request for a missing operator encountered during plan processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorRequest {
    pub tag_name: String,
    pub attributes: Vec<(String, String)>,
    pub context_xml: String,
    pub source_line: usize,
}

/// Collects missing operator requests during plan processing.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OperatorRequestLog {
    pub requests: Vec<OperatorRequest>,
}

impl OperatorRequestLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&mut self, request: OperatorRequest) {
        self.requests.push(request);
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}
