//! Trivial merge strategies for common cases.

use async_trait::async_trait;

use crate::error::MergeError;
use crate::merge_strategy::MergeStrategy;

/// Always use the generated (pipeline) version, discarding user edits.
pub struct TakeGenerated;

#[async_trait]
impl MergeStrategy for TakeGenerated {
    async fn merge(
        &self,
        _path: &str,
        _base: &[u8],
        generated: &[u8],
        _edited: &[u8],
    ) -> Result<Vec<u8>, MergeError> {
        Ok(generated.to_vec())
    }
}

/// Always keep the user's edited version, ignoring pipeline changes.
pub struct KeepEdited;

#[async_trait]
impl MergeStrategy for KeepEdited {
    async fn merge(
        &self,
        _path: &str,
        _base: &[u8],
        _generated: &[u8],
        edited: &[u8],
    ) -> Result<Vec<u8>, MergeError> {
        Ok(edited.to_vec())
    }
}

/// Fail with an error on any true three-way conflict.
pub struct FailOnConflict;

#[async_trait]
impl MergeStrategy for FailOnConflict {
    async fn merge(
        &self,
        path: &str,
        _base: &[u8],
        _generated: &[u8],
        _edited: &[u8],
    ) -> Result<Vec<u8>, MergeError> {
        Err(MergeError::Conflict {
            path: path.to_string(),
            reason: "true three-way conflict (no auto-resolve strategy)".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn take_generated_returns_generated() {
        let strategy = TakeGenerated;
        let result = strategy
            .merge("file.txt", b"base", b"generated", b"edited")
            .await
            .unwrap();
        assert_eq!(result, b"generated");
    }

    #[tokio::test]
    async fn keep_edited_returns_edited() {
        let strategy = KeepEdited;
        let result = strategy
            .merge("file.txt", b"base", b"generated", b"edited")
            .await
            .unwrap();
        assert_eq!(result, b"edited");
    }

    #[tokio::test]
    async fn fail_on_conflict_errors() {
        let strategy = FailOnConflict;
        let result = strategy
            .merge("file.txt", b"base", b"generated", b"edited")
            .await;
        assert!(result.is_err());
        assert!(matches!(result, Err(MergeError::Conflict { .. })));
    }
}
