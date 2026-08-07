//! Single-use, expiring tokens for email verification and password reset.
//!
//! The token is generated, mailed, and never stored. Only its SHA-256 goes to
//! the database, so a database read cannot be replayed as a verification or a
//! password reset — the same reasoning as `password_hash`, and the reason
//! redemption looks the token up *by hash* rather than fetching a row and
//! comparing.
//!
//! SHA-256 rather than argon2 here on purpose: these are 256 bits of CSPRNG
//! output, not a human-chosen password, so there is nothing to brute-force
//! and the deliberate slowness argon2 buys would only make redemption slow.

use anyhow::Result;
use base64::Engine as _;
use chrono::{DateTime, Duration, Utc};
use rand::RngCore as _;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// What a token authorises. A verification link must not be redeemable as a
/// password reset, so the purpose is part of the lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    Verify,
    Reset,
}

impl Purpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Purpose::Verify => "verify",
            Purpose::Reset => "reset",
        }
    }

    /// How long a link stays live.
    ///
    /// A reset is short because possession of a mailbox becomes possession of
    /// an account; verification is longer because it is routine and a
    /// too-short window just means people ask for another one.
    pub fn ttl(self) -> Duration {
        match self {
            Purpose::Verify => Duration::hours(24),
            Purpose::Reset => Duration::hours(1),
        }
    }
}

/// SHA-256 of a token, hex-encoded. The only form that reaches the database.
pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Issue a token for `user_id`, returning the plaintext to be mailed.
///
/// Any outstanding token for the same user and purpose is expired first: two
/// live reset links doubles the window in which a leaked mailbox is an
/// account takeover, and "I clicked the old one" is a support burden with no
/// upside.
pub async fn issue(db: &PgPool, user_id: Uuid, purpose: Purpose) -> Result<String> {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    // URL-safe and unpadded: this goes in a link, and padding invites
    // mail clients to mangle it.
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

    let mut tx = db.begin().await?;
    sqlx::query(
        "UPDATE email_tokens SET used_at = now()
         WHERE user_id = $1 AND purpose = $2 AND used_at IS NULL",
    )
    .bind(user_id)
    .bind(purpose.as_str())
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO email_tokens (token_hash, user_id, purpose, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(hash_token(&token))
    .bind(user_id)
    .bind(purpose.as_str())
    .bind(Utc::now() + purpose.ttl())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(token)
}

/// Redeem `token` for `purpose`, returning the user it belongs to.
///
/// Atomic: the UPDATE sets `used_at` and returns the row only if it was still
/// unused and unexpired, so two concurrent redemptions of one link cannot
/// both succeed. Doing this as SELECT-then-UPDATE would leave exactly that
/// race, which for a reset link means two people setting a password.
///
/// `None` covers every failure — unknown, wrong purpose, expired, already
/// used — because the caller must not tell them apart to the outside world.
pub async fn redeem(db: &PgPool, token: &str, purpose: Purpose) -> Result<Option<Uuid>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE email_tokens SET used_at = now()
         WHERE token_hash = $1 AND purpose = $2
           AND used_at IS NULL AND expires_at > now()
         RETURNING user_id",
    )
    .bind(hash_token(token))
    .bind(purpose.as_str())
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// How many tokens of this purpose were issued to `user_id` since `since`.
///
/// The rate limit: without one, the send endpoint is a free mail cannon
/// pointed at any address an attacker names, which is how a sending domain
/// gets a reputation problem.
pub async fn issued_since(
    db: &PgPool,
    user_id: Uuid,
    purpose: Purpose,
    since: DateTime<Utc>,
) -> Result<i64> {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM email_tokens
         WHERE user_id = $1 AND purpose = $2 AND created_at > $3",
    )
    .bind(user_id)
    .bind(purpose.as_str())
    .bind(since)
    .fetch_one(db)
    .await?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashing_is_stable_and_hides_the_token() {
        let h = hash_token("abc");
        assert_eq!(h, hash_token("abc"), "hashing must be deterministic");
        assert_ne!(h, hash_token("abd"));
        assert_eq!(h.len(), 64, "sha-256 hex is 64 chars");
        assert!(!h.contains("abc"), "the token must not survive in its hash");
    }

    #[test]
    fn a_reset_link_expires_sooner_than_a_verification_link() {
        // Possession of a mailbox becomes possession of an account, so the
        // reset window is the one that must stay small.
        assert!(Purpose::Reset.ttl() < Purpose::Verify.ttl());
        assert_eq!(Purpose::Reset.ttl(), Duration::hours(1));
    }

    #[test]
    fn purposes_are_distinct_strings() {
        // They key the CHECK constraint and the redemption lookup; if these
        // ever collided a verification link would reset a password.
        assert_ne!(Purpose::Verify.as_str(), Purpose::Reset.as_str());
    }
}
