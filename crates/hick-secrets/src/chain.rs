//! Provider chain that tries multiple secrets providers in order.

use crate::{SecretsError, SecretsProvider};

/// Chains multiple [`SecretsProvider`] implementations, trying each in order.
///
/// On [`SecretsError::NotFound`], falls through to the next provider. All other
/// errors are returned immediately. If all providers return `NotFound`, the
/// chain returns `NotFound`.
pub struct SecretsProviderChain {
    providers: Vec<Box<dyn SecretsProvider>>,
}

impl SecretsProviderChain {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Add a provider to the end of the chain (builder pattern).
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, provider: impl SecretsProvider + 'static) -> Self {
        self.providers.push(Box::new(provider));
        self
    }
}

impl Default for SecretsProviderChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SecretsProvider for SecretsProviderChain {
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError> {
        for provider in &self.providers {
            match provider.get_secret(name).await {
                Ok(value) => return Ok(value),
                Err(SecretsError::NotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(SecretsError::NotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemorySecretsProvider;

    #[tokio::test]
    async fn first_provider_wins() {
        let mut p1 = InMemorySecretsProvider::new();
        p1.add_secret("key", "from-p1");
        let mut p2 = InMemorySecretsProvider::new();
        p2.add_secret("key", "from-p2");

        let chain = SecretsProviderChain::new().add(p1).add(p2);
        let result = chain.get_secret("key").await.unwrap();
        assert_eq!(result, "from-p1");
    }

    #[tokio::test]
    async fn falls_through_on_not_found() {
        let p1 = InMemorySecretsProvider::new();
        let mut p2 = InMemorySecretsProvider::new();
        p2.add_secret("key", "from-p2");

        let chain = SecretsProviderChain::new().add(p1).add(p2);
        let result = chain.get_secret("key").await.unwrap();
        assert_eq!(result, "from-p2");
    }

    #[tokio::test]
    async fn all_not_found_returns_not_found() {
        let chain = SecretsProviderChain::new()
            .add(InMemorySecretsProvider::new())
            .add(InMemorySecretsProvider::new());
        let result = chain.get_secret("missing").await;
        assert!(matches!(result, Err(SecretsError::NotFound(_))));
    }

    #[tokio::test]
    async fn empty_chain_returns_not_found() {
        let chain = SecretsProviderChain::new();
        let result = chain.get_secret("any").await;
        assert!(matches!(result, Err(SecretsError::NotFound(_))));
    }
}
