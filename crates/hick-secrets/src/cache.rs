//! TTL-based secret cache with zeroize-on-drop values.

use std::time::{Duration, Instant};

use dashmap::DashMap;
use secrecy::SecretString;

use crate::{SecretsError, SecretsProvider};

struct CachedSecret {
    value: SecretString,
    cached_at: Instant,
}

/// Thread-safe secret cache with configurable TTL.
///
/// Wraps a [`SecretsProvider`] so that repeated lookups for the same secret
/// hit the provider at most once per TTL window. Cached values are stored
/// as [`SecretString`] (zeroized on drop).
pub struct SecretCache {
    entries: DashMap<String, CachedSecret>,
    ttl: Duration,
}

impl SecretCache {
    /// Create a new cache with the given TTL.
    ///
    /// A TTL of [`Duration::ZERO`] disables caching entirely — every call
    /// hits the underlying provider.
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: DashMap::new(),
            ttl,
        }
    }

    /// Get a secret by name, resolving through the provider on cache miss.
    pub async fn get_or_resolve(
        &self,
        name: &str,
        provider: &dyn SecretsProvider,
    ) -> Result<SecretString, SecretsError> {
        // Skip cache entirely when TTL is zero.
        if self.ttl == Duration::ZERO {
            let value = provider.get_secret(name).await?;
            return Ok(SecretString::from(value));
        }

        // Check for a valid cached entry.
        if let Some(entry) = self.entries.get(name) {
            if entry.cached_at.elapsed() < self.ttl {
                return Ok(entry.value.clone());
            }
        }

        // Cache miss or expired — resolve from provider.
        let value = provider.get_secret(name).await?;
        let secret = SecretString::from(value);
        self.entries.insert(
            name.to_string(),
            CachedSecret {
                value: secret.clone(),
                cached_at: Instant::now(),
            },
        );
        Ok(secret)
    }

    /// Remove a specific entry from the cache.
    pub fn evict(&self, name: &str) {
        self.entries.remove(name);
    }

    /// Remove all expired entries from the cache.
    pub fn evict_expired(&self) {
        self.entries
            .retain(|_, entry| entry.cached_at.elapsed() < self.ttl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemorySecretsProvider;
    use secrecy::ExposeSecret;

    #[tokio::test]
    async fn cache_returns_provider_value() {
        let mut provider = InMemorySecretsProvider::new();
        provider.add_secret("key", "secret-val");
        let cache = SecretCache::new(Duration::from_secs(300));
        let result = cache.get_or_resolve("key", &provider).await.unwrap();
        assert_eq!(result.expose_secret(), "secret-val");
    }

    #[tokio::test]
    async fn cache_returns_cached_value() {
        let mut provider = InMemorySecretsProvider::new();
        provider.add_secret("key", "first");
        let cache = SecretCache::new(Duration::from_secs(300));

        // First call populates cache.
        cache.get_or_resolve("key", &provider).await.unwrap();

        // Change the provider value — cache should still return "first".
        provider.add_secret("key", "second");
        let result = cache.get_or_resolve("key", &provider).await.unwrap();
        assert_eq!(result.expose_secret(), "first");
    }

    #[tokio::test]
    async fn zero_ttl_disables_caching() {
        let mut provider = InMemorySecretsProvider::new();
        provider.add_secret("key", "first");
        let cache = SecretCache::new(Duration::ZERO);

        cache.get_or_resolve("key", &provider).await.unwrap();

        provider.add_secret("key", "second");
        let result = cache.get_or_resolve("key", &provider).await.unwrap();
        assert_eq!(result.expose_secret(), "second");
    }

    #[tokio::test]
    async fn evict_removes_entry() {
        let mut provider = InMemorySecretsProvider::new();
        provider.add_secret("key", "first");
        let cache = SecretCache::new(Duration::from_secs(300));

        cache.get_or_resolve("key", &provider).await.unwrap();
        cache.evict("key");

        provider.add_secret("key", "second");
        let result = cache.get_or_resolve("key", &provider).await.unwrap();
        assert_eq!(result.expose_secret(), "second");
    }

    #[tokio::test]
    async fn not_found_propagates() {
        let provider = InMemorySecretsProvider::new();
        let cache = SecretCache::new(Duration::from_secs(300));
        let result = cache.get_or_resolve("missing", &provider).await;
        assert!(matches!(result, Err(SecretsError::NotFound(_))));
    }
}
