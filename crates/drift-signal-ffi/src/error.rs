use std::sync::PoisonError;

use waymark_core::SyncError;

/// FFI-safe error for crossing the UniFFI boundary.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum DriftSignalError {
    #[error("{message}")]
    ValidationError {
        message: String,
        field: String,
        suggestion: String,
    },

    #[error("not found: {entity_id} in {collection}")]
    NotFound {
        collection: String,
        entity_id: String,
    },

    #[error("{message}")]
    StorageError { message: String },

    #[error("{message}")]
    EngineError { message: String },
}

impl From<anyhow::Error> for DriftSignalError {
    fn from(err: anyhow::Error) -> Self {
        DriftSignalError::EngineError {
            message: err.to_string(),
        }
    }
}

impl From<SyncError> for DriftSignalError {
    fn from(err: SyncError) -> Self {
        match err {
            SyncError::Validation { message } => DriftSignalError::ValidationError {
                message,
                field: String::new(),
                suggestion: String::new(),
            },
            SyncError::EntityNotFound {
                ref collection,
                ref id,
            } => DriftSignalError::NotFound {
                collection: collection.clone(),
                entity_id: id.clone(),
            },
            SyncError::Storage { .. } => DriftSignalError::StorageError {
                message: err.to_string(),
            },
            _ => DriftSignalError::EngineError {
                message: err.to_string(),
            },
        }
    }
}

impl<T> From<PoisonError<T>> for DriftSignalError {
    fn from(_err: PoisonError<T>) -> Self {
        DriftSignalError::EngineError {
            message: "Internal engine error — please restart the app.".to_string(),
        }
    }
}
