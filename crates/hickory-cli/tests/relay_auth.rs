//! Signing in to a relay, both ways — and the relay's right to offer only one.
//!
//! Protects docs/guarantees/collaboration/the-relay-offers-only-what-it-has.md.
//!
//! The interesting property is negative: a relay with no GitHub OAuth app must
//! not advertise GitHub, and the CLI must not offer it. A sign-in option that
//! appears and then fails at the last step is worse than one that was never
//! shown, because the person has already committed to it.

use std::sync::Arc;

use hickory_cli::login;
use hickory_relay::{Quota, TunnelCensus};
use hickory_relay_server::github::StubIdentifier;
use hickory_relay_server::tunnel::TunnelRegistry;
use hickory_relay_server::{RelayState, router as relay_router};
use tokio::sync::Mutex;

const SECRET: &str = "a-test-secret-that-is-long-enough-to-sign";

struct Relay {
    base: String,
    identifier: Arc<StubIdentifier>,
}

async fn start_relay(github: Option<&str>, with_accounts: bool) -> Relay {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let identifier = Arc::new(StubIdentifier::new("nate"));

    let accounts = if with_accounts {
        Some(
            hickory_relay_server::accounts::open("sqlite::memory:")
                .await
                .unwrap(),
        )
    } else {
        None
    };

    let state = RelayState {
        tunnels: Arc::new(TunnelRegistry::default()),
        census: Arc::new(Mutex::new(TunnelCensus::default())),
        quota: Quota::default(),
        identifier: identifier.clone(),
        accounts,
        token_secret: SECRET.to_string(),
        github_client_id: github.map(str::to_string),
        apex: format!("localhost:{port}"),
        scheme: "http".into(),
    };

    tokio::spawn(async move {
        axum::serve(listener, relay_router(state)).await.unwrap();
    });

    Relay {
        base: format!("http://127.0.0.1:{port}"),
        identifier,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_relay_without_an_oauth_app_does_not_advertise_github() {
    let relay = start_relay(None, true).await;
    let methods = login::methods(&relay.base).await.unwrap();
    assert!(methods.password, "email and password are always available");
    assert!(
        methods.github.is_none(),
        "an unconfigured relay must not offer GitHub"
    );

    // And the exchange endpoint refuses rather than half-working, for anyone
    // who calls it directly.
    let resp = reqwest::Client::new()
        .post(format!("{}/_relay/auth/github", relay.base))
        .json(&serde_json::json!({ "access_token": "gho_whatever" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
    let body = resp.text().await.unwrap();
    assert!(body.contains("email and password"), "{body}");
    assert_eq!(
        relay.identifier.calls(),
        0,
        "an unconfigured relay must not call GitHub at all"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_configured_relay_advertises_the_client_id_the_cli_needs() {
    let relay = start_relay(Some("Iv1.example"), true).await;
    let methods = login::methods(&relay.base).await.unwrap();
    let github = methods.github.expect("GitHub is offered");
    // The CLI takes the client id from the relay rather than compiling one in,
    // so a self-hosted relay works with its own OAuth app and no rebuild.
    assert_eq!(github.client_id, "Iv1.example");

    let credentials = login::exchange_github(&relay.base, "gho_valid")
        .await
        .unwrap();
    assert_eq!(credentials.login, "@nate");
    // What is stored is the RELAY's token, not GitHub's.
    assert_ne!(credentials.access_token, "gho_valid");
    let claims = hickory_identity::verify_token(SECRET, &credentials.access_token).unwrap();
    assert_eq!(claims.label, "@nate");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_account_signs_up_signs_in_and_gets_a_relay_token() {
    let relay = start_relay(None, true).await;

    let created = login::with_password(&relay.base, "nate@example.com", "a long passphrase", true)
        .await
        .unwrap();
    assert_eq!(created.login, "nate@example.com");
    hickory_identity::verify_token(SECRET, &created.access_token).unwrap();

    let signed_in =
        login::with_password(&relay.base, "NATE@example.com", "a long passphrase", false)
            .await
            .unwrap();
    assert_eq!(signed_in.login, "nate@example.com");

    // Signing in never touches GitHub.
    assert_eq!(relay.identifier.calls(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn refusals_reach_the_person_who_typed_them() {
    let relay = start_relay(None, true).await;

    let err = login::with_password(&relay.base, "nate@example.com", "short", true)
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("12"), "{err}");

    let err = login::with_password(&relay.base, "not-an-address", "a long passphrase", true)
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("does not look like an email"), "{err}");

    login::with_password(&relay.base, "nate@example.com", "a long passphrase", true)
        .await
        .unwrap();

    // Wrong password and unknown address must be indistinguishable from
    // outside, or the relay leaks who has an account.
    let wrong = login::with_password(&relay.base, "nate@example.com", "wrong passphrase", false)
        .await
        .unwrap_err()
        .to_string();
    let unknown = login::with_password(
        &relay.base,
        "stranger@example.com",
        "wrong passphrase",
        false,
    )
    .await
    .unwrap_err()
    .to_string();
    assert_eq!(wrong, unknown);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_relay_with_no_account_store_says_so_instead_of_failing_obscurely() {
    let relay = start_relay(None, false).await;
    let err = login::with_password(&relay.base, "nate@example.com", "a long passphrase", true)
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("RELAY_DATABASE_URL"), "{err}");
}
