<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-secrets/src/chain.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b71729cf5e27"></a>
## `crates/hick-secrets/src/chain.rs`

<a id="f-b71729cf5e27-SecretsProviderChain"></a>
### `SecretsProviderChain`

Provider chain that tries multiple secrets providers in order.

<storycode:copy id="f-b71729cf5e27-SecretsProviderChain">
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

</storycode:copy>
<a id="f-b71729cf5e27-impl-SecretsProviderChain"></a>
### `impl-SecretsProviderChain`

<storycode:copy id="f-b71729cf5e27-impl-SecretsProviderChain">
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

</storycode:copy>
<a id="f-b71729cf5e27-impl-Default-for-SecretsProviderChain"></a>
### `impl-Default for SecretsProviderChain`

<storycode:copy id="f-b71729cf5e27-impl-Default-for-SecretsProviderChain">
impl Default for SecretsProviderChain {
    fn default() -> Self {
        Self::new()
    }
}

</storycode:copy>
<a id="f-b71729cf5e27-impl-SecretsProvider-for-SecretsProviderChain"></a>
### `impl-SecretsProvider for SecretsProviderChain`

<storycode:copy id="f-b71729cf5e27-impl-SecretsProvider-for-SecretsProviderChain">
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

</storycode:copy>
<a id="f-b71729cf5e27-tests"></a>
### `tests`

<storycode:copy id="f-b71729cf5e27-tests">
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
</storycode:copy>

### Assemble `crates/hick-secrets/src/chain.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-secrets/src/chain.rs"><storycode:paste select="#f-b71729cf5e27-SecretsProviderChain"/><storycode:paste select="#f-b71729cf5e27-impl-SecretsProviderChain"/><storycode:paste select="#f-b71729cf5e27-impl-Default-for-SecretsProviderChain"/><storycode:paste select="#f-b71729cf5e27-impl-SecretsProvider-for-SecretsProviderChain"/><storycode:paste select="#f-b71729cf5e27-tests"/></storycode:file>

</storycode:doc>
