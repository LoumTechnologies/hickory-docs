//! Pluggable secrets storage for hick containers.
//!
//! Provides a `SecretsProvider` trait with an age-based implementation
//! using the `age` crate (Rust port of the age encryption tool).

pub mod cache;
pub mod chain;
pub mod env;

/// Thread-safe TTL cache for secrets, backed by zeroize-on-drop [`secrecy::SecretString`] values.
pub use cache::SecretCache;
/// Composite provider that tries multiple [`SecretsProvider`] backends in order,
/// falling through on [`SecretsError::NotFound`].
pub use chain::SecretsProviderChain;
/// [`SecretsProvider`] that reads secrets from host environment variables.
pub use env::EnvSecretsProvider;

use std::io::Read;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// SecretsProvider trait
// ---------------------------------------------------------------------------

/// Trait for retrieving secrets by name.
///
/// Implementations can back this with age-encrypted files, Vault, AWS SSM, etc.
#[async_trait::async_trait]
pub trait SecretsProvider: Send + Sync {
    /// Retrieve a secret value by name.
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError>;
}

/// Errors from secrets operations.
#[derive(Debug, thiserror::Error)]
pub enum SecretsError {
    #[error("secret not found: {0}")]
    NotFound(String),

    #[error("decryption failed: {0}")]
    Decryption(String),

    #[error("identity file error: {0}")]
    Identity(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid UTF-8 in secret: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

// ---------------------------------------------------------------------------
// AgeSecretsProvider
// ---------------------------------------------------------------------------

/// Secrets provider that decrypts age-encrypted files from a directory.
///
/// Secret files are stored as `<secrets_dir>/<name>.age`. Each file is
/// decrypted using the identity (private key) from the configured key file.
pub struct AgeSecretsProvider {
    /// Path to the age identity (private key) file.
    key_path: PathBuf,
    /// Directory containing `<name>.age` encrypted secret files.
    secrets_dir: PathBuf,
}

impl AgeSecretsProvider {
    /// Create a new provider with the given key file and secrets directory.
    pub fn new(key_path: impl Into<PathBuf>, secrets_dir: impl Into<PathBuf>) -> Self {
        Self {
            key_path: key_path.into(),
            secrets_dir: secrets_dir.into(),
        }
    }

    /// Decrypt an age-encrypted file using the configured identity.
    fn decrypt_file(&self, path: &Path) -> Result<String, SecretsError> {
        let ciphertext = std::fs::read(path)?;

        let identity_file =
            age::IdentityFile::from_file(self.key_path.to_string_lossy().into_owned())
                .map_err(|e| SecretsError::Identity(format!("{e}")))?;

        let identities = identity_file
            .into_identities()
            .map_err(|e| SecretsError::Identity(format!("{e}")))?;

        let identity_refs: Vec<&dyn age::Identity> = identities
            .iter()
            .map(|i| i.as_ref() as &dyn age::Identity)
            .collect();

        let decryptor = age::Decryptor::new_buffered(&ciphertext[..])
            .map_err(|e| SecretsError::Decryption(format!("{e}")))?;

        let mut reader = decryptor
            .decrypt(identity_refs.into_iter())
            .map_err(|e| SecretsError::Decryption(format!("{e}")))?;

        let mut plaintext = Vec::new();
        reader.read_to_end(&mut plaintext)?;

        Ok(String::from_utf8(plaintext)?)
    }
}

#[async_trait::async_trait]
impl SecretsProvider for AgeSecretsProvider {
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError> {
        let path = self.secrets_dir.join(format!("{name}.age"));
        if !path.exists() {
            return Err(SecretsError::NotFound(name.to_string()));
        }
        // Decryption is CPU-bound; run on blocking thread pool
        let provider_key = self.key_path.clone();
        let provider_dir = self.secrets_dir.clone();
        let name = name.to_string();

        tokio::task::spawn_blocking(move || {
            let provider = AgeSecretsProvider::new(provider_key, provider_dir);
            let path = provider.secrets_dir.join(format!("{name}.age"));
            provider.decrypt_file(&path)
        })
        .await
        .map_err(|e| SecretsError::Decryption(format!("task join error: {e}")))?
    }
}

// ---------------------------------------------------------------------------
// InMemorySecretsProvider (for testing)
// ---------------------------------------------------------------------------

/// Simple in-memory secrets provider for testing.
pub struct InMemorySecretsProvider {
    secrets: std::collections::HashMap<String, String>,
}

impl InMemorySecretsProvider {
    pub fn new() -> Self {
        Self {
            secrets: std::collections::HashMap::new(),
        }
    }

    pub fn add_secret(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.secrets.insert(name.into(), value.into());
    }
}

impl Default for InMemorySecretsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SecretsProvider for InMemorySecretsProvider {
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError> {
        self.secrets
            .get(name)
            .cloned()
            .ok_or_else(|| SecretsError::NotFound(name.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::ExposeSecret;

    #[tokio::test]
    async fn in_memory_provider_returns_secret() {
        let mut provider = InMemorySecretsProvider::new();
        provider.add_secret("api-key", "sk-12345");

        let result = provider.get_secret("api-key").await.unwrap();
        assert_eq!(result, "sk-12345");
    }

    #[tokio::test]
    async fn in_memory_provider_returns_not_found() {
        let provider = InMemorySecretsProvider::new();
        let result = provider.get_secret("nonexistent").await;
        assert!(matches!(result, Err(SecretsError::NotFound(_))));
    }

    #[tokio::test]
    async fn age_provider_returns_not_found_for_missing_file() {
        let provider = AgeSecretsProvider::new("/nonexistent/key.txt", "/nonexistent/secrets");
        let result = provider.get_secret("missing").await;
        assert!(matches!(result, Err(SecretsError::NotFound(_))));
    }

    #[test]
    fn age_provider_roundtrip() {
        // Generate a key pair
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public();

        // Encrypt a secret
        let plaintext = b"super-secret-value";
        let encrypted = {
            let recipients: Vec<Box<dyn age::Recipient + Send>> = vec![Box::new(recipient)];
            let recipient_refs: Vec<&dyn age::Recipient> = recipients
                .iter()
                .map(|r| r.as_ref() as &dyn age::Recipient)
                .collect();
            let encryptor =
                age::Encryptor::with_recipients(recipient_refs.into_iter()).expect("recipients");
            let mut output = vec![];
            let mut writer = encryptor.wrap_output(&mut output).expect("wrap_output");
            std::io::Write::write_all(&mut writer, plaintext).expect("write");
            writer.finish().expect("finish");
            output
        };

        // Write identity to a temp file
        let dir = tempdir();
        let key_path = dir.join("key.txt");
        std::fs::write(&key_path, identity.to_string().expose_secret()).unwrap();

        // Write encrypted secret
        let secrets_dir = dir.join("secrets");
        std::fs::create_dir_all(&secrets_dir).unwrap();
        std::fs::write(secrets_dir.join("my-token.age"), &encrypted).unwrap();

        // Decrypt
        let provider = AgeSecretsProvider::new(&key_path, &secrets_dir);
        let result = provider
            .decrypt_file(&secrets_dir.join("my-token.age"))
            .unwrap();
        assert_eq!(result, "super-secret-value");
    }

    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hick-secrets-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
