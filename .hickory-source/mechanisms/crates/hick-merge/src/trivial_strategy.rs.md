<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-merge/src/trivial_strategy.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-29fa88e2e539"></a>
## `crates/hick-merge/src/trivial_strategy.rs`

<a id="f-29fa88e2e539-TakeGenerated"></a>
### `TakeGenerated`

Trivial merge strategies for common cases.

<storycode:copy id="f-29fa88e2e539-TakeGenerated">
//! Trivial merge strategies for common cases.

use async_trait::async_trait;

use crate::error::MergeError;
use crate::merge_strategy::MergeStrategy;

/// Always use the generated (pipeline) version, discarding user edits.
pub struct TakeGenerated;

</storycode:copy>
<a id="f-29fa88e2e539-impl-MergeStrategy-for-TakeGenerated"></a>
### `impl-MergeStrategy for TakeGenerated`

<storycode:copy id="f-29fa88e2e539-impl-MergeStrategy-for-TakeGenerated">
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

</storycode:copy>
<a id="f-29fa88e2e539-KeepEdited"></a>
### `KeepEdited`

Always keep the user's edited version, ignoring pipeline changes.

<storycode:copy id="f-29fa88e2e539-KeepEdited">
/// Always keep the user's edited version, ignoring pipeline changes.
pub struct KeepEdited;

</storycode:copy>
<a id="f-29fa88e2e539-impl-MergeStrategy-for-KeepEdited"></a>
### `impl-MergeStrategy for KeepEdited`

<storycode:copy id="f-29fa88e2e539-impl-MergeStrategy-for-KeepEdited">
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

</storycode:copy>
<a id="f-29fa88e2e539-FailOnConflict"></a>
### `FailOnConflict`

Fail with an error on any true three-way conflict.

<storycode:copy id="f-29fa88e2e539-FailOnConflict">
/// Fail with an error on any true three-way conflict.
pub struct FailOnConflict;

</storycode:copy>
<a id="f-29fa88e2e539-impl-MergeStrategy-for-FailOnConflict"></a>
### `impl-MergeStrategy for FailOnConflict`

<storycode:copy id="f-29fa88e2e539-impl-MergeStrategy-for-FailOnConflict">
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

</storycode:copy>
<a id="f-29fa88e2e539-tests"></a>
### `tests`

<storycode:copy id="f-29fa88e2e539-tests">
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
</storycode:copy>

### Assemble `crates/hick-merge/src/trivial_strategy.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-merge/src/trivial_strategy.rs"><storycode:paste select="#f-29fa88e2e539-TakeGenerated"/><storycode:paste select="#f-29fa88e2e539-impl-MergeStrategy-for-TakeGenerated"/><storycode:paste select="#f-29fa88e2e539-KeepEdited"/><storycode:paste select="#f-29fa88e2e539-impl-MergeStrategy-for-KeepEdited"/><storycode:paste select="#f-29fa88e2e539-FailOnConflict"/><storycode:paste select="#f-29fa88e2e539-impl-MergeStrategy-for-FailOnConflict"/><storycode:paste select="#f-29fa88e2e539-tests"/></storycode:file>

</storycode:doc>
