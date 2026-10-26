<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-secrets/src/env.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d28c4fe8ac11"></a>
## `crates/hick-secrets/src/env.rs`

<a id="f-d28c4fe8ac11-EnvSecretsProvider"></a>
### `EnvSecretsProvider`

Environment variable secrets provider.
Reads secrets from host environment variables by name.

<storycode:copy id="f-d28c4fe8ac11-EnvSecretsProvider">
//! Environment variable secrets provider.
//!
//! Reads secrets from host environment variables by name.

use crate::{SecretsError, SecretsProvider};

/// Secrets provider that reads from environment variables.
pub struct EnvSecretsProvider;

</storycode:copy>
<a id="f-d28c4fe8ac11-impl-SecretsProvider-for-EnvSecretsProvider"></a>
### `impl-SecretsProvider for EnvSecretsProvider`

<storycode:copy id="f-d28c4fe8ac11-impl-SecretsProvider-for-EnvSecretsProvider">
#[async_trait::async_trait]
impl SecretsProvider for EnvSecretsProvider {
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError> {
        std::env::var(name).map_err(|_| SecretsError::NotFound(name.to_string()))
    }
}

</storycode:copy>
<a id="f-d28c4fe8ac11-tests"></a>
### `tests`

<storycode:copy id="f-d28c4fe8ac11-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reads_existing_env_var() {
        // SAFETY: test runs with --test-threads=1, no concurrent env access.
        unsafe { std::env::set_var("HICK_TEST_SECRET_ENV", "test-value-123") };
        let provider = EnvSecretsProvider;
        let result = provider.get_secret("HICK_TEST_SECRET_ENV").await.unwrap();
        assert_eq!(result, "test-value-123");
        unsafe { std::env::remove_var("HICK_TEST_SECRET_ENV") };
    }

    #[tokio::test]
    async fn returns_not_found_for_missing() {
        let provider = EnvSecretsProvider;
        let result = provider.get_secret("HICK_NONEXISTENT_VAR_XYZ").await;
        assert!(matches!(result, Err(SecretsError::NotFound(_))));
    }
}
</storycode:copy>

### Assemble `crates/hick-secrets/src/env.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-secrets/src/env.rs"><storycode:paste select="#f-d28c4fe8ac11-EnvSecretsProvider"/><storycode:paste select="#f-d28c4fe8ac11-impl-SecretsProvider-for-EnvSecretsProvider"/><storycode:paste select="#f-d28c4fe8ac11-tests"/></storycode:file>

</storycode:doc>
