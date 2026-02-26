//! Environment variable secrets provider.
//!
//! Reads secrets from host environment variables by name.

use crate::{SecretsError, SecretsProvider};

/// Secrets provider that reads from environment variables.
pub struct EnvSecretsProvider;

#[async_trait::async_trait]
impl SecretsProvider for EnvSecretsProvider {
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError> {
        std::env::var(name).map_err(|_| SecretsError::NotFound(name.to_string()))
    }
}

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
