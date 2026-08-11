//! Envelope encryption for the provider API keys accounts bring themselves.
//!
//! One key-encryption key per deployment (`KEY_ENCRYPTION_KEY`, 32 bytes,
//! base64) seals every row of `user_llm_keys` with XChaCha20-Poly1305. The
//! design constraints, in the order they mattered:
//!
//! - **A stolen database is not a bag of API keys.** The sealing key lives in
//!   the process environment, not in Postgres, so a dump — a backup, a
//!   read-replica, an errant `pg_dump` in a support ticket — carries only
//!   ciphertext.
//! - **A ciphertext is bound to the row it came from.** `user_id:provider` is
//!   the AEAD's associated data, so a row copied onto another account fails to
//!   open rather than quietly handing one user another user's credential. This
//!   is the failure that would otherwise be invisible: the agent would run, the
//!   bill would land somewhere else, and nothing would look wrong.
//! - **Rotation stays possible.** Every row records the `key_version` that
//!   sealed it, so a future second key can decrypt old rows while writing new
//!   ones — rather than a flag day that invalidates every stored key at once.
//!
//! Absent configuration this module is simply not constructed: the BYOK
//! endpoints answer 503 and say which variable to set, the same graceful
//! degradation Stripe, SendGrid, and PostHog already get.

use anyhow::{Context as _, Result, bail};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore as _;
use uuid::Uuid;

/// The key version written by this build. Bump when a second key is
/// introduced, never reuse a version for different key material.
pub const CURRENT_KEY_VERSION: i32 = 1;

/// Bytes of a raw key-encryption key.
const KEY_LEN: usize = 32;
/// XChaCha20-Poly1305 nonce length.
const NONCE_LEN: usize = 24;

/// A sealed credential, exactly as the three `user_llm_keys` columns hold it.
#[derive(Debug, Clone)]
pub struct Sealed {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub key_version: i32,
}

/// The deployment's key-encryption key.
#[derive(Clone)]
pub struct KeyVault {
    cipher: XChaCha20Poly1305,
}

impl std::fmt::Debug for KeyVault {
    /// Never let the key reach a log line through a derived `Debug` on some
    /// enclosing struct.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KeyVault(<redacted>)")
    }
}

impl KeyVault {
    /// Build from the base64 `KEY_ENCRYPTION_KEY` value.
    ///
    /// The error names the variable, the expected shape, and how to mint a
    /// valid one — a deployment blocked here has no other way to find out
    /// that "32 bytes" means "before base64, not after".
    pub fn from_base64(raw: &str) -> Result<Self> {
        let bytes = B64
            .decode(raw.trim())
            .context("KEY_ENCRYPTION_KEY is not valid base64; generate one with `just gen-key`")?;
        if bytes.len() != KEY_LEN {
            bail!(
                "KEY_ENCRYPTION_KEY decodes to {} bytes, expected {KEY_LEN} \
                 (generate one with `just gen-key`)",
                bytes.len()
            );
        }
        let cipher = XChaCha20Poly1305::new_from_slice(&bytes)
            .map_err(|e| anyhow::anyhow!("invalid KEY_ENCRYPTION_KEY: {e}"))?;
        Ok(Self { cipher })
    }

    /// Mint a fresh key in the exact form `KEY_ENCRYPTION_KEY` accepts.
    pub fn generate_base64() -> String {
        let mut bytes = [0u8; KEY_LEN];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        B64.encode(bytes)
    }

    /// Associated data binding a ciphertext to one account's one provider.
    fn aad(user_id: Uuid, provider: &str) -> Vec<u8> {
        format!("{user_id}:{provider}").into_bytes()
    }

    /// Seal `plaintext` for `(user_id, provider)`.
    pub fn seal(&self, user_id: Uuid, provider: &str, plaintext: &str) -> Result<Sealed> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);
        let aad = Self::aad(user_id, provider);
        let ciphertext = self
            .cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext.as_bytes(),
                    aad: &aad,
                },
            )
            // The error carries no plaintext, and must not: this string can
            // reach a log.
            .map_err(|_| anyhow::anyhow!("failed to encrypt the provider key"))?;
        Ok(Sealed {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            key_version: CURRENT_KEY_VERSION,
        })
    }

    /// Open a sealed credential for `(user_id, provider)`.
    ///
    /// Fails when the key-encryption key changed, when the row was written by
    /// a key version this build does not hold, or when the ciphertext does not
    /// belong to this account and provider.
    pub fn open(&self, user_id: Uuid, provider: &str, sealed: &Sealed) -> Result<String> {
        if sealed.key_version != CURRENT_KEY_VERSION {
            bail!(
                "stored key was sealed with key version {} but this deployment holds version \
                 {CURRENT_KEY_VERSION}; the account must re-enter its {provider} key",
                sealed.key_version
            );
        }
        if sealed.nonce.len() != NONCE_LEN {
            bail!("stored key has a malformed nonce; the account must re-enter its key");
        }
        let nonce = XNonce::from_slice(&sealed.nonce);
        let aad = Self::aad(user_id, provider);
        let plaintext = self
            .cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &sealed.ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| {
                anyhow::anyhow!(
                    "stored {provider} key could not be decrypted — KEY_ENCRYPTION_KEY has \
                     changed since it was saved, or the row does not belong to this account. \
                     Re-enter the key in Settings → API keys."
                )
            })?;
        String::from_utf8(plaintext).context("stored key is not valid UTF-8")
    }
}

/// The last four characters of a key, for display. Short keys show as `••••`
/// rather than revealing most of themselves.
pub fn last4(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() < 8 {
        return "••••".to_string();
    }
    chars[chars.len() - 4..].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vault() -> KeyVault {
        KeyVault::from_base64(&KeyVault::generate_base64()).unwrap()
    }

    #[test]
    fn a_sealed_key_round_trips() {
        let v = vault();
        let user = Uuid::new_v4();
        let sealed = v.seal(user, "anthropic", "sk-ant-secret").unwrap();
        assert_ne!(sealed.ciphertext, b"sk-ant-secret");
        assert_eq!(v.open(user, "anthropic", &sealed).unwrap(), "sk-ant-secret");
    }

    /// Guarantee: docs/guarantees/agent/byok-key-at-rest.md — a ciphertext
    /// lifted onto another account or another provider does not open.
    #[test]
    fn a_ciphertext_does_not_open_for_a_different_row() {
        let v = vault();
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();
        let sealed = v.seal(alice, "anthropic", "sk-ant-alice").unwrap();

        assert!(
            v.open(bob, "anthropic", &sealed).is_err(),
            "another account must not be able to open Alice's key"
        );
        assert!(
            v.open(alice, "openai", &sealed).is_err(),
            "the same account's other provider row must not open it either"
        );
    }

    #[test]
    fn a_different_deployment_key_cannot_open_it() {
        let user = Uuid::new_v4();
        let sealed = vault().seal(user, "anthropic", "sk-ant-secret").unwrap();
        let err = vault()
            .open(user, "anthropic", &sealed)
            .expect_err("a different KEY_ENCRYPTION_KEY must not decrypt");
        // The user's next step has to be in the message; "decryption failed"
        // sends them nowhere.
        let msg = err.to_string();
        assert!(msg.contains("KEY_ENCRYPTION_KEY"), "{msg}");
        assert!(msg.contains("Re-enter"), "{msg}");
    }

    #[test]
    fn every_write_uses_a_fresh_nonce() {
        let v = vault();
        let user = Uuid::new_v4();
        let a = v.seal(user, "anthropic", "same-key").unwrap();
        let b = v.seal(user, "anthropic", "same-key").unwrap();
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(
            a.ciphertext, b.ciphertext,
            "identical plaintext must not seal to identical ciphertext"
        );
    }

    #[test]
    fn a_malformed_configuration_names_the_variable() {
        let err = KeyVault::from_base64("not base64!!!").unwrap_err();
        assert!(err.to_string().contains("KEY_ENCRYPTION_KEY"));
        let short = B64.encode([0u8; 16]);
        let err = KeyVault::from_base64(&short).unwrap_err();
        assert!(err.to_string().contains("16 bytes"), "{err}");
    }

    #[test]
    fn last4_never_reveals_a_short_key() {
        assert_eq!(last4("sk-ant-abcd1234"), "1234");
        assert_eq!(last4("short"), "••••");
    }
}
