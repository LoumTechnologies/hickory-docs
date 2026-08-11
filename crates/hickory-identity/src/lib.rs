//! Password hashing and bearer tokens, shared by anything in this repo that
//! has accounts.
//!
//! Extracted from the hosted server so the relay does not grow a second
//! implementation of the two primitives that must never be got wrong: how a
//! password is stored, and what a token proves. Both are small; both are the
//! kind of small that is quietly wrong in a copy.
//!
//! Deliberately free of any web framework. The server maps a failure onto its
//! `ApiError` and the relay onto its own; neither concern belongs here.

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Hash a password for storage (argon2id, random salt).
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("argon2 hashing failed: {e}"))?;
    Ok(hash.to_string())
}

/// Check a password against a stored hash.
///
/// A malformed stored hash verifies as `false` rather than erroring: a
/// corrupted row must not become a way in.
pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Why a password was refused, in words the person who typed it can act on.
///
/// The rule is length and nothing else. Composition rules ("one capital, one
/// symbol") push people toward `Password1!` and are worse than a longer
/// minimum; this is the one place to state that, rather than in each caller.
pub const MIN_PASSWORD_LENGTH: usize = 12;

pub fn check_password_strength(password: &str) -> Result<(), String> {
    let length = password.chars().count();
    if length < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "that password is {length} characters; {MIN_PASSWORD_LENGTH} is the minimum. \
             A short phrase you can remember beats a short password you cannot."
        ));
    }
    Ok(())
}

/// What a token proves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Account id.
    pub sub: String,
    /// How the account is named to a human — an email address, or `@login`
    /// for an account that arrived through GitHub. Carried in the token so a
    /// holder can be identified without a database round trip.
    pub label: String,
    /// Expiry, seconds since the epoch.
    pub exp: i64,
}

/// Mint a token for an account.
pub fn issue_token(secret: &str, id: &str, label: &str, valid_days: i64) -> anyhow::Result<String> {
    let claims = Claims {
        sub: id.to_string(),
        label: label.to_string(),
        exp: (Utc::now() + chrono::Duration::days(valid_days)).timestamp(),
    };
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

/// Read a token, checking its signature and expiry.
pub fn verify_token(secret: &str, token: &str) -> Result<Claims, TokenError> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
        _ => TokenError::Invalid,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenError {
    Expired,
    Invalid,
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Distinguished on purpose: "sign in again" is a different
            // instruction from "that is not a token of ours", and a person
            // stuck on the second should not be told to retry the first.
            TokenError::Expired => f.write_str("this sign-in has expired — sign in again"),
            TokenError::Invalid => f.write_str("this token is not valid"),
        }
    }
}

impl std::error::Error for TokenError {}

/// Normalise an email for storage and comparison.
///
/// Lowercased and trimmed, so `Nate@Example.com ` and `nate@example.com` are
/// one account rather than two. The local part is technically case-sensitive
/// per RFC 5321; no mail provider anyone uses treats it that way, and two
/// accounts differing only in case would be a support problem, not a feature.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

/// The shape check an address must pass. Deliberately loose: the only real
/// test of an address is sending mail to it, and rejecting valid-but-unusual
/// addresses is a way to lose users, not to gain safety.
pub fn email_looks_valid(email: &str) -> bool {
    let email = email.trim();
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.contains(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_round_trips_and_a_wrong_one_does_not() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &hash));
        assert!(!verify_password("Correct horse battery staple", &hash));
        assert!(!verify_password("", &hash));
    }

    #[test]
    fn the_same_password_hashes_differently_every_time() {
        // A shared salt would let one rainbow table cover every account.
        let a = hash_password("correct horse battery staple").unwrap();
        let b = hash_password("correct horse battery staple").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn a_corrupted_hash_is_a_refusal_not_a_way_in() {
        assert!(!verify_password("anything", "not-a-hash"));
        assert!(!verify_password("anything", ""));
    }

    #[test]
    fn password_length_is_the_only_rule_and_it_says_the_number() {
        let err = check_password_strength("short").unwrap_err();
        assert!(err.contains("12"), "{err}");
        check_password_strength("a passphrase that is long enough").unwrap();
    }

    #[test]
    fn a_token_round_trips_and_carries_who_it_is_for() {
        let token = issue_token("secret", "acc-1", "nate@example.com", 30).unwrap();
        let claims = verify_token("secret", &token).unwrap();
        assert_eq!(claims.sub, "acc-1");
        assert_eq!(claims.label, "nate@example.com");
    }

    #[test]
    fn another_secret_cannot_mint_a_token_this_one_accepts() {
        let token = issue_token("secret-a", "acc-1", "nate@example.com", 30).unwrap();
        assert_eq!(
            verify_token("secret-b", &token).unwrap_err(),
            TokenError::Invalid
        );
    }

    #[test]
    fn an_expired_token_says_so_rather_than_looking_forged() {
        let token = issue_token("secret", "acc-1", "nate@example.com", -1).unwrap();
        let err = verify_token("secret", &token).unwrap_err();
        assert_eq!(err, TokenError::Expired);
        assert!(err.to_string().contains("sign in again"));
    }

    #[test]
    fn emails_are_one_account_however_they_are_typed() {
        assert_eq!(normalize_email("  Nate@Example.COM "), "nate@example.com");
    }

    #[test]
    fn address_shape_rejects_the_obviously_broken_and_allows_the_unusual() {
        assert!(email_looks_valid("nate@example.com"));
        assert!(email_looks_valid("nate+hickory@sub.example.co.uk"));
        // Unusual but legal — refusing these would be losing users for nothing.
        assert!(email_looks_valid("a_b'c@example.com"));

        assert!(!email_looks_valid("nate"));
        assert!(!email_looks_valid("nate@localhost"));
        assert!(!email_looks_valid("@example.com"));
        // Surrounding whitespace is trimmed, not rejected: it is what pasting
        // an address into a terminal produces, and `normalize_email` removes
        // it before anything is stored.
        assert!(email_looks_valid("nate@example.com "));
        assert!(!email_looks_valid("na te@example.com"));
    }
}
