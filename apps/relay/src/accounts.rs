//! Relay accounts: email/password, and optionally GitHub.
//!
//! This is the one thing the relay persists. Everything else — tunnels,
//! streams, quotas — dies with the process on purpose; accounts cannot,
//! because an account is what a quota is counted against and what an abuse
//! report names.
//!
//! **SQLite, one table, created at boot.** The relay is a single process
//! forwarding bytes; giving it a Postgres to operate would be a bigger
//! commitment than the feature is worth. The schema is one `CREATE TABLE IF
//! NOT EXISTS`, which is the honest form of "migrations" for a table with no
//! second version yet — when there is one, this grows a proper migration
//! directory.
//!
//! ## Email/password is weaker attribution than GitHub, and that is a choice
//!
//! A GitHub account is one somebody else already vouched for. An email address
//! typed into a signup form is not — and this relay does not send mail, so it
//! cannot even prove the address exists. What that means concretely: password
//! accounts are cheap to mint, so the quota (three tunnels, eight hours) is
//! doing more of the work, and a serious abuse problem would be answered by
//! requiring verification or requiring GitHub. Stated here rather than
//! discovered.

use anyhow::{Context as _, Result, bail};
use hickory_identity::{
    check_password_strength, email_looks_valid, hash_password, normalize_email, verify_password,
};
use serde::Serialize;
use sqlx::SqlitePool;

/// One account, as anything outside this module sees it.
#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub id: String,
    /// How the account is named to a human: an email, or `@login`.
    pub label: String,
}

/// Open (or create) the relay's account store.
pub async fn open(database_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(database_url)
        .await
        .with_context(|| format!("opening the relay's account store at {database_url}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS accounts (
             id            TEXT PRIMARY KEY,
             email         TEXT UNIQUE,
             password_hash TEXT,
             github_login  TEXT UNIQUE,
             created_at    TEXT NOT NULL DEFAULT (datetime('now'))
         )",
    )
    .execute(&pool)
    .await
    .context("creating the accounts table")?;

    Ok(pool)
}

fn new_id() -> String {
    // Not a uuid crate dependency for one field: 128 bits of hex from the
    // clock and the process is unique enough for a primary key that is never
    // guessed at, only looked up.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in now
        .to_le_bytes()
        .iter()
        .chain(&std::process::id().to_le_bytes())
    {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    format!("{now:032x}{hash:016x}")
}

/// Create an account from an email and a password.
///
/// Errors are what the person typing them can act on; the caller turns them
/// into a 400 verbatim.
pub async fn sign_up(pool: &SqlitePool, email: &str, password: &str) -> Result<Account> {
    let email = normalize_email(email);
    if !email_looks_valid(&email) {
        bail!("{email:?} does not look like an email address");
    }
    if let Err(message) = check_password_strength(password) {
        bail!("{message}");
    }

    let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM accounts WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await?;
    if existing.is_some() {
        // Deliberately explicit. Hiding whether an address is registered is
        // the right call on a *login* form, where it leaks the user list to a
        // stranger; on signup it only leaves the person who owns the address
        // unable to work out why nothing happened.
        bail!("an account already exists for {email} — sign in instead");
    }

    let id = new_id();
    let hash = hash_password(password)?;
    sqlx::query("INSERT INTO accounts (id, email, password_hash) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&email)
        .bind(&hash)
        .execute(pool)
        .await
        .context("storing the new account")?;

    Ok(Account { id, label: email })
}

/// Check an email and password.
///
/// One message for both "no such account" and "wrong password", because the
/// difference is exactly what tells a stranger which addresses are registered.
pub async fn sign_in(pool: &SqlitePool, email: &str, password: &str) -> Result<Account> {
    const REFUSAL: &str = "that email and password do not match an account";

    let email = normalize_email(email);
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT id, password_hash FROM accounts WHERE email = ?")
            .bind(&email)
            .fetch_optional(pool)
            .await?;

    let Some((id, Some(hash))) = row else {
        // Hash anyway. Returning immediately makes "no such account" faster
        // than "wrong password", which is a timing oracle for the user list.
        let _ = hash_password(password);
        bail!(REFUSAL);
    };
    if !verify_password(password, &hash) {
        bail!(REFUSAL);
    }
    Ok(Account { id, label: email })
}

/// Find or create the account behind a GitHub login.
pub async fn upsert_github(pool: &SqlitePool, login: &str) -> Result<Account> {
    let label = format!("@{login}");
    if let Some((id,)) =
        sqlx::query_as::<_, (String,)>("SELECT id FROM accounts WHERE github_login = ?")
            .bind(login)
            .fetch_optional(pool)
            .await?
    {
        return Ok(Account { id, label });
    }

    let id = new_id();
    sqlx::query("INSERT INTO accounts (id, github_login) VALUES (?, ?)")
        .bind(&id)
        .bind(login)
        .execute(pool)
        .await
        .context("storing the new account")?;
    Ok(Account { id, label })
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> SqlitePool {
        // In-memory: the schema is created the same way it is in production,
        // which is the part worth testing.
        open("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn an_account_signs_up_and_back_in() {
        let pool = store().await;
        let account = sign_up(&pool, "  Nate@Example.COM ", "a long enough passphrase")
            .await
            .unwrap();
        assert_eq!(account.label, "nate@example.com");

        // Case and whitespace must not create a second account or block entry.
        let again = sign_in(&pool, "NATE@example.com", "a long enough passphrase")
            .await
            .unwrap();
        assert_eq!(again.id, account.id);
    }

    #[tokio::test]
    async fn a_wrong_password_and_an_unknown_address_are_indistinguishable() {
        let pool = store().await;
        sign_up(&pool, "nate@example.com", "a long enough passphrase")
            .await
            .unwrap();

        let wrong = sign_in(&pool, "nate@example.com", "not the passphrase")
            .await
            .unwrap_err()
            .to_string();
        let unknown = sign_in(&pool, "stranger@example.com", "not the passphrase")
            .await
            .unwrap_err()
            .to_string();
        // Different messages here would tell a stranger which addresses have
        // accounts.
        assert_eq!(wrong, unknown);
    }

    #[tokio::test]
    async fn signup_refuses_what_it_cannot_store_and_says_why() {
        let pool = store().await;
        let err = sign_up(&pool, "not-an-address", "a long enough passphrase")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not look like an email"), "{err}");

        let err = sign_up(&pool, "nate@example.com", "short")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("12"), "{err}");

        sign_up(&pool, "nate@example.com", "a long enough passphrase")
            .await
            .unwrap();
        let err = sign_up(&pool, "nate@example.com", "another long passphrase")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("sign in instead"), "{err}");
    }

    #[tokio::test]
    async fn a_github_login_maps_to_one_stable_account() {
        let pool = store().await;
        let first = upsert_github(&pool, "nate").await.unwrap();
        let second = upsert_github(&pool, "nate").await.unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(first.label, "@nate");

        let other = upsert_github(&pool, "someone-else").await.unwrap();
        assert_ne!(other.id, first.id);
    }

    #[tokio::test]
    async fn ids_do_not_collide() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            assert!(seen.insert(new_id()), "id collision");
        }
    }
}
