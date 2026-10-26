<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# One Spoken Phrase Enrols Both Machines, With No Server And Nobody Trusted

For: Engineers using local sessions or pairing their own machines.

Given two machines that have never met, when a phrase generated on one is typed
on the other, then each ends up holding the other's public key — **both
directions in one exchange** — and no server, directory or third party is
involved in deciding whose key is whose.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/a-phrase-pairs-both-machines.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-902bf51e0ac1"></a>
## `crates/hickory-peer/src/pairing.rs`

Related contracts: [One Spoken Phrase Enrols Both Machines, With No Server And Nobody Trusted](contract.md).

<a id="f-902bf51e0ac1-PAIR_ALPN"></a>
### `PAIR_ALPN`

Pairing by a spoken phrase: no invitation to copy, and no server.
`one-engineer-many-machines.md` sketched enrolment as a short one-time code
— `7 QUAIL DRIFT 2`, expiring in sixty seconds. When the fleet was first
built that was set aside with the reasoning that *a code that short cannot
carry a public key, so it assumes a rendezvous, and the only rendezvous
available to two machines that cannot reach each other is one we would run*
— which `local-only.md` deletes.
**That reasoning was wrong, and this is the correction.** The code does not
have to *carry* a key. It can *be* one:
> Both machines derive the same keypair from the phrase. One binds an
> endpoint under it and waits; the other computes the same public half,
> which is the address, and dials. Over that connection they trade their
> real keys.
The phrase is the rendezvous. No directory, no account, nobody to trust for
identity — which is the property an account version could not have had.
## What it is worth, stated plainly
Anyone who knows the phrase can complete this exchange, and either side of
it. So the phrase is a **one-time secret with a deadline**, and it is
treated like one: generated rather than chosen, single-use, expiring, and
with both fingerprints printed at the end so a person can compare them.
This is not a PAKE. A phrase an attacker guesses during the window gets
them enrolled, and the guard is entropy plus a short window plus a slow
derivation — not a cryptographic protocol that survives a weak secret. Say
"a one-time secret", never "secure pairing", which implies a stronger claim
than the construction supports.
## Mutual in one exchange
Today a fleet takes two ceremonies: each machine invites and the other
accepts, in both directions, because a fleet is a mutual list. Here both
keys cross in one round trip, so one phrase finishes the job — which is the
friction the question was actually about.

<storycode:copy id="f-902bf51e0ac1-PAIR_ALPN">
//! Pairing by a spoken phrase: no invitation to copy, and no server.
//!
//! `one-engineer-many-machines.md` sketched enrolment as a short one-time code
//! — `7 QUAIL DRIFT 2`, expiring in sixty seconds. When the fleet was first
//! built that was set aside with the reasoning that *a code that short cannot
//! carry a public key, so it assumes a rendezvous, and the only rendezvous
//! available to two machines that cannot reach each other is one we would run*
//! — which `local-only.md` deletes.
//!
//! **That reasoning was wrong, and this is the correction.** The code does not
//! have to *carry* a key. It can *be* one:
//!
//! > Both machines derive the same keypair from the phrase. One binds an
//! > endpoint under it and waits; the other computes the same public half,
//! > which is the address, and dials. Over that connection they trade their
//! > real keys.
//!
//! The phrase is the rendezvous. No directory, no account, nobody to trust for
//! identity — which is the property an account version could not have had.
//!
//! ## What it is worth, stated plainly
//!
//! Anyone who knows the phrase can complete this exchange, and either side of
//! it. So the phrase is a **one-time secret with a deadline**, and it is
//! treated like one: generated rather than chosen, single-use, expiring, and
//! with both fingerprints printed at the end so a person can compare them.
//!
//! This is not a PAKE. A phrase an attacker guesses during the window gets
//! them enrolled, and the guard is entropy plus a short window plus a slow
//! derivation — not a cryptographic protocol that survives a weak secret. Say
//! "a one-time secret", never "secure pairing", which implies a stronger claim
//! than the construction supports.
//!
//! ## Mutual in one exchange
//!
//! Today a fleet takes two ceremonies: each machine invites and the other
//! accepts, in both directions, because a fleet is a mutual list. Here both
//! keys cross in one round trip, so one phrase finishes the job — which is the
//! friction the question was actually about.

use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use hickory_fleet::{Fleet, Identity, Invitation, Kind, Machine};

use crate::tunnel::{Reach, builder_for};

/// The protocol spoken over a rendezvous connection.
pub const PAIR_ALPN: &[u8] = b"hickory/pair/0";

</storycode:copy>
<a id="f-902bf51e0ac1-WINDOW"></a>
### `WINDOW`

How long a phrase is good for. Short on purpose: the whole security of this
is that the window is small and the secret is used once.

<storycode:copy id="f-902bf51e0ac1-WINDOW">
/// How long a phrase is good for. Short on purpose: the whole security of this
/// is that the window is small and the secret is used once.
pub const WINDOW: Duration = Duration::from_secs(120);

</storycode:copy>
<a id="f-902bf51e0ac1-WORDS"></a>
### `WORDS`

Words chosen to be short, common, and hard to mishear — no homophones, no
plurals of each other, nothing that sounds like a letter. A phrase gets
read aloud across a room, which is the case that decides the list.

<storycode:copy id="f-902bf51e0ac1-WORDS">
/// Words chosen to be short, common, and hard to mishear — no homophones, no
/// plurals of each other, nothing that sounds like a letter. A phrase gets
/// read aloud across a room, which is the case that decides the list.
const WORDS: &[&str] = &[
    "amber", "anchor", "apple", "arrow", "autumn", "bacon", "badge", "bagel", "banjo", "barley",
    "basin", "beacon", "bison", "blanket", "bottle", "boulder", "bramble", "bridge", "bucket",
    "buffalo", "cabin", "cactus", "camera", "candle", "canyon", "carbon", "cargo", "carpet",
    "castle", "cedar", "cellar", "chalk", "cherry", "chimney", "cinder", "clover", "cobalt",
    "compass", "copper", "coral", "cotton", "crater", "crimson", "crystal", "cutlass", "dagger",
    "daisy", "damson", "denim", "diamond", "dolphin", "domino", "dragon", "drift", "eagle",
    "ember", "engine", "falcon", "fathom", "feather", "fennel", "ferry", "fiddle", "flannel",
    "flint", "forest", "fossil", "fountain", "fox", "gadget", "gallon", "garden", "garnet",
    "gazelle", "ginger", "glacier", "granite", "gravel", "grotto", "guitar", "hammer", "harbor",
    "harvest", "hazel", "helmet", "hickory", "hollow", "honey", "hornet", "hunter", "indigo",
    "island", "ivory", "jacket", "jasmine", "jigsaw", "junction", "juniper", "kernel", "kettle",
    "keystone", "lantern", "lattice", "lemon", "lily", "linen", "lobster", "locket", "lumber",
    "magnet", "mahogany", "mallet", "mango", "maple", "marble", "marlin", "meadow", "mercury",
    "meteor", "mineral", "mitten", "monsoon", "mosaic", "muffin", "mustard", "nectar", "nickel",
    "nutmeg",
];

</storycode:copy>
<a id="f-902bf51e0ac1-Phrase"></a>
### `Phrase`

<storycode:copy id="f-902bf51e0ac1-Phrase">
/// A one-time pairing phrase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phrase(String);

</storycode:copy>
<a id="f-902bf51e0ac1-impl-Phrase"></a>
### `impl-Phrase`

<storycode:copy id="f-902bf51e0ac1-impl-Phrase">
impl Phrase {
    /// Generate one. Four words and two digits: about 34 bits, which is far
    /// more than a two-minute online-only guessing window can chew through
    /// when each attempt costs an Argon2 derivation.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("the OS has randomness");
        let mut parts: Vec<String> = Vec::new();
        for chunk in bytes.chunks(2).take(4) {
            let n = (u16::from(chunk[0]) << 8 | u16::from(chunk[1])) as usize;
            parts.push(WORDS[n % WORDS.len()].to_string());
        }
        let digits = (u16::from(bytes[8]) << 8 | u16::from(bytes[9])) % 100;
        parts.push(format!("{digits:02}"));
        Self(parts.join("-"))
    }

    /// Read a phrase somebody typed or was told.
    ///
    /// Forgiving about the things people get wrong when copying by ear —
    /// case, spaces instead of hyphens, a trailing full stop — and unforgiving
    /// about anything else, because a phrase that nearly matches derives a
    /// completely different key and the failure would look like a network
    /// problem.
    pub fn parse(raw: &str) -> Result<Self> {
        let cleaned: String = raw
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase()
            .chars()
            .map(|c| {
                if c.is_whitespace() || c == '_' {
                    '-'
                } else {
                    c
                }
            })
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        let parts: Vec<&str> = cleaned.split('-').filter(|p| !p.is_empty()).collect();
        if parts.len() < 2 {
            bail!(
                "{raw:?} does not look like a pairing phrase. One looks like \
                 `anchor-drift-maple-harbor-47` — four words and two digits, \
                 printed by `hick fleet pair --new` on the other machine.\n  \
                 Phrases expire after two minutes and work once; if it has been \
                 sitting a while, generate a new one."
            );
        }
        Ok(Self(parts.join("-")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The keypair this phrase names. Both machines compute it.
    ///
    /// Argon2id, with a fixed domain-separating salt. A fixed salt is normally
    /// a mistake and is deliberate here: both ends must arrive at the same key
    /// from the phrase alone, and the phrase is high-entropy and used once, so
    /// the rainbow-table argument a random salt defends against does not
    /// apply. What the slow derivation buys is making each guess in the window
    /// expensive.
    pub fn rendezvous_key(&self) -> Result<iroh::SecretKey> {
        use argon2::{Algorithm, Argon2, Params, Version};
        let argon = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(19 * 1024, 2, 1, Some(32)).expect("valid argon2 params"),
        );
        let mut out = [0u8; 32];
        argon
            .hash_password_into(self.0.as_bytes(), b"hickory-fleet-pairing-v1", &mut out)
            .map_err(|e| anyhow::anyhow!("could not derive the rendezvous key: {e}"))?;
        Ok(iroh::SecretKey::from_bytes(&out))
    }
}

</storycode:copy>
<a id="f-902bf51e0ac1-impl-std-fmt-Display-for-Phrase"></a>
### `impl-std::fmt::Display for Phrase`

<storycode:copy id="f-902bf51e0ac1-impl-std-fmt-Display-for-Phrase">
impl std::fmt::Display for Phrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

</storycode:copy>
<a id="f-902bf51e0ac1-Paired"></a>
### `Paired`

<storycode:copy id="f-902bf51e0ac1-Paired">
/// What one side learned.
#[derive(Debug, Clone)]
pub struct Paired {
    /// The machine now enrolled here.
    pub machine: Machine,
    /// This machine's own fingerprint, so both ends can be compared by a
    /// person — the only check that catches somebody who guessed the phrase.
    pub ours: String,
}

</storycode:copy>
<a id="f-902bf51e0ac1-trade"></a>
### `trade`

Send our invitation and read theirs, on an open stream.

<storycode:copy id="f-902bf51e0ac1-trade">
/// Send our invitation and read theirs, on an open stream.
async fn trade(
    identity: &Identity,
    kind: Kind,
    send: &mut iroh::endpoint::SendStream,
    recv: &mut iroh::endpoint::RecvStream,
) -> Result<Invitation> {
    let mine = identity.invitation(kind).encode();
    send.write_all(mine.as_bytes()).await?;
    send.write_all(b"\n").await?;
    send.finish()?;
    let raw = recv
        .read_to_end(8 * 1024)
        .await
        .context("the other machine hung up before sending its key")?;
    Invitation::decode(String::from_utf8_lossy(&raw).trim())
}

</storycode:copy>
<a id="f-902bf51e0ac1-PairCancelled"></a>
### `PairCancelled`

<storycode:copy id="f-902bf51e0ac1-PairCancelled">
#[derive(Debug)]
pub struct PairCancelled;
</storycode:copy>
<a id="f-902bf51e0ac1-impl-std-fmt-Display-for-PairCancelled"></a>
### `impl-std::fmt::Display for PairCancelled`

<storycode:copy id="f-902bf51e0ac1-impl-std-fmt-Display-for-PairCancelled">
impl std::fmt::Display for PairCancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pairing cancelled before local enrollment; the remote machine may have received an invitation. Inspect both rosters before retrying."
        )
    }
}
</storycode:copy>
<a id="f-902bf51e0ac1-impl-std-error-Error-for-PairCancelled"></a>
### `impl-std::error::Error for PairCancelled`

<storycode:copy id="f-902bf51e0ac1-impl-std-error-Error-for-PairCancelled">
impl std::error::Error for PairCancelled {}

</storycode:copy>
<a id="f-902bf51e0ac1-PairPhase"></a>
### `PairPhase`

<storycode:copy id="f-902bf51e0ac1-PairPhase">
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PairPhase {
    Preparing,
    Binding,
    WaitingForPeer,
    ExchangingKeys,
    Enrolled,
    Closing,
    Closed,
}

</storycode:copy>
<a id="f-902bf51e0ac1-phase"></a>
### `phase`

<storycode:copy id="f-902bf51e0ac1-phase">
fn phase(progress: &Option<tokio::sync::watch::Sender<PairPhase>>, value: PairPhase) {
    if let Some(progress) = progress {
        progress.send_replace(value);
    }
}
</storycode:copy>
<a id="f-902bf51e0ac1-close_endpoint"></a>
### `close_endpoint`

<storycode:copy id="f-902bf51e0ac1-close_endpoint">
async fn close_endpoint(
    endpoint: &iroh::Endpoint,
    progress: &Option<tokio::sync::watch::Sender<PairPhase>>,
) {
    phase(progress, PairPhase::Closing);
    endpoint.close().await;
    phase(progress, PairPhase::Closed);
}
</storycode:copy>
<a id="f-902bf51e0ac1-cancelled"></a>
### `cancelled`

<storycode:copy id="f-902bf51e0ac1-cancelled">
async fn cancelled(cancel: &mut tokio::sync::watch::Receiver<bool>) {
    loop {
        if *cancel.borrow() {
            return;
        }
        if cancel.changed().await.is_err() {
            std::future::pending::<()>().await;
        }
    }
}

</storycode:copy>
<a id="f-902bf51e0ac1-host"></a>
### `host`

Wait for the other machine to dial this phrase, then trade keys.

<storycode:copy id="f-902bf51e0ac1-host">
/// Wait for the other machine to dial this phrase, then trade keys.
pub async fn host(
    identity: &Identity,
    fleet: &Fleet,
    phrase: &Phrase,
    kind: Kind,
    reach: &Reach,
    today: &str,
) -> Result<Paired> {
    let (keep, cancel) = tokio::sync::watch::channel(false);
    let result = host_cancellable(identity, fleet, phrase, kind, reach, today, cancel, None).await;
    drop(keep);
    result
}

</storycode:copy>
<a id="f-902bf51e0ac1-host_cancellable"></a>
### `host_cancellable`

<storycode:copy id="f-902bf51e0ac1-host_cancellable">
pub async fn host_cancellable(
    identity: &Identity,
    fleet: &Fleet,
    phrase: &Phrase,
    kind: Kind,
    reach: &Reach,
    today: &str,
    mut cancel: tokio::sync::watch::Receiver<bool>,
    progress: Option<tokio::sync::watch::Sender<PairPhase>>,
) -> Result<Paired> {
    if *cancel.borrow() {
        return Err(PairCancelled.into());
    }
    phase(&progress, PairPhase::Binding);
    let endpoint = builder_for(reach)?
        .secret_key(phrase.rendezvous_key()?)
        .alpns(vec![PAIR_ALPN.to_vec()])
        .bind()
        .await
        .context("could not bind the rendezvous endpoint")?;

    phase(&progress, PairPhase::WaitingForPeer);
    let exchange = async {
        let accepted = tokio::time::timeout(WINDOW, endpoint.accept())
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "nobody dialled that phrase within {} seconds, so it has expired.\n  \
                 A phrase is a one-time secret with a deadline, which is most of \
                 what makes this safe.\n  \
                 Next step: run `hick fleet pair --new` again for a fresh one, and \
                 have the other machine ready before you read it out.",
                    WINDOW.as_secs()
                )
            })?;
        let Some(incoming) = accepted else {
            bail!("the rendezvous endpoint closed before anyone arrived");
        };
        let connection = incoming.await.context("the rendezvous handshake failed")?;
        let (mut send, mut recv) = connection
            .accept_bi()
            .await
            .context("the other machine opened no stream")?;

        phase(&progress, PairPhase::ExchangingKeys);
        let theirs = trade(identity, kind, &mut send, &mut recv).await?;
        Ok::<_, anyhow::Error>((connection, theirs))
    };
    let outcome = tokio::select! {
        biased;
        _ = cancelled(&mut cancel) => Err(PairCancelled.into()),
        outcome = tokio::time::timeout(WINDOW, exchange) => outcome.unwrap_or_else(|_| Err(anyhow::anyhow!("Pairing exchange expired before local enrollment"))),
    };
    let (connection, theirs) = match outcome {
        Ok(value) if !*cancel.borrow() => value,
        Ok(_) => {
            close_endpoint(&endpoint, &progress).await;
            return Err(PairCancelled.into());
        }
        Err(error) => {
            close_endpoint(&endpoint, &progress).await;
            return Err(error);
        }
    };
    let machine = match fleet.add(&theirs, today) {
        Ok(machine) => machine,
        Err(error) => {
            close_endpoint(&endpoint, &progress).await;
            return Err(error);
        }
    };
    phase(&progress, PairPhase::Enrolled);

    // Wait for the other side to hang up before tearing the endpoint down.
    // `finish()` marks a stream complete; it does not mean the bytes have
    // arrived, and closing the endpoint underneath them loses the last write —
    // which the other machine reports as "hung up before sending its key",
    // pointing at the wrong side entirely.
    // Once enrolled, cancellation cannot undo the write or cut off delivery of
    // the invitation already sent. Report the actual paired result after grace.
    let _ = tokio::time::timeout(Duration::from_secs(10), connection.closed()).await;
    // Single use: the phrase dies with this connection, so a second dialler
    // finds nothing.
    close_endpoint(&endpoint, &progress).await;
    Ok(Paired {
        machine,
        ours: identity.fingerprint(),
    })
}

</storycode:copy>
<a id="f-902bf51e0ac1-join"></a>
### `join`

Dial a phrase somebody read out, and trade keys.

<storycode:copy id="f-902bf51e0ac1-join">
/// Dial a phrase somebody read out, and trade keys.
pub async fn join(
    identity: &Identity,
    fleet: &Fleet,
    phrase: &Phrase,
    kind: Kind,
    reach: &Reach,
    today: &str,
) -> Result<Paired> {
    let (keep, cancel) = tokio::sync::watch::channel(false);
    let result = join_cancellable(identity, fleet, phrase, kind, reach, today, cancel, None).await;
    drop(keep);
    result
}

</storycode:copy>
<a id="f-902bf51e0ac1-join_cancellable"></a>
### `join_cancellable`

<storycode:copy id="f-902bf51e0ac1-join_cancellable">
pub async fn join_cancellable(
    identity: &Identity,
    fleet: &Fleet,
    phrase: &Phrase,
    kind: Kind,
    reach: &Reach,
    today: &str,
    mut cancel: tokio::sync::watch::Receiver<bool>,
    progress: Option<tokio::sync::watch::Sender<PairPhase>>,
) -> Result<Paired> {
    if *cancel.borrow() {
        return Err(PairCancelled.into());
    }
    phase(&progress, PairPhase::Binding);
    let rendezvous = phrase.rendezvous_key()?;
    let target: iroh::EndpointId = rendezvous.public();

    let endpoint = builder_for(reach)?
        .bind()
        .await
        .context("could not bind an endpoint to dial from")?;
    phase(&progress, PairPhase::WaitingForPeer);
    let exchange = async {
        let connection = tokio::time::timeout(
            WINDOW,
            endpoint.connect(iroh::EndpointAddr::from(target), PAIR_ALPN),
        )
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "could not reach that phrase within {} seconds.\n  \
             Either it has expired, or nobody is hosting it, or the two \
             machines cannot find each other on this network.\n  \
             Next steps: check `hick fleet pair --new` is still running on the \
             other machine, and that the phrase was typed exactly as read.",
                WINDOW.as_secs()
            )
        })?
        .context("that phrase is not being hosted right now")?;

        let (mut send, mut recv) = connection
            .open_bi()
            .await
            .context("could not open a stream to the other machine")?;
        phase(&progress, PairPhase::ExchangingKeys);
        let theirs = trade(identity, kind, &mut send, &mut recv).await?;
        Ok::<_, anyhow::Error>((connection, theirs))
    };
    let outcome = tokio::select! {
        biased;
        _ = cancelled(&mut cancel) => Err(PairCancelled.into()),
        outcome = tokio::time::timeout(WINDOW, exchange) => outcome.unwrap_or_else(|_| Err(anyhow::anyhow!("Pairing exchange expired before local enrollment"))),
    };
    let (connection, theirs) = match outcome {
        Ok(value) if !*cancel.borrow() => value,
        Ok(_) => {
            close_endpoint(&endpoint, &progress).await;
            return Err(PairCancelled.into());
        }
        Err(error) => {
            close_endpoint(&endpoint, &progress).await;
            return Err(error);
        }
    };
    let machine = match fleet.add(&theirs, today) {
        Ok(machine) => machine,
        Err(error) => {
            close_endpoint(&endpoint, &progress).await;
            return Err(error);
        }
    };
    phase(&progress, PairPhase::Enrolled);

    connection.close(0u32.into(), b"paired");
    close_endpoint(&endpoint, &progress).await;
    Ok(Paired {
        machine,
        ours: identity.fingerprint(),
    })
}

</storycode:copy>
<a id="f-902bf51e0ac1-tests"></a>
### `tests`

<storycode:copy id="f-902bf51e0ac1-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_phrase_is_readable_and_has_the_shape_people_expect() {
        let phrase = Phrase::generate();
        let parts: Vec<&str> = phrase.as_str().split('-').collect();
        assert_eq!(parts.len(), 5, "{phrase}");
        assert!(parts[4].chars().all(|c| c.is_ascii_digit()), "{phrase}");
        assert!(parts[..4].iter().all(|w| WORDS.contains(w)), "{phrase}");
    }

    #[test]
    fn two_generated_phrases_differ() {
        assert_ne!(Phrase::generate(), Phrase::generate());
    }

    #[test]
    fn parsing_forgives_what_people_get_wrong_by_ear() {
        // Case, spaces for hyphens, a trailing full stop.
        let want = Phrase::parse("anchor-drift-maple-harbor-47").unwrap();
        for spelling in [
            "ANCHOR-DRIFT-MAPLE-HARBOR-47",
            "anchor drift maple harbor 47",
            "  anchor-drift-maple-harbor-47.  ",
            "anchor_drift_maple_harbor_47",
        ] {
            assert_eq!(Phrase::parse(spelling).unwrap(), want, "{spelling}");
        }
    }

    #[test]
    fn something_that_is_not_a_phrase_says_what_one_looks_like() {
        let err = Phrase::parse("hello").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("anchor-drift-maple-harbor-47"), "{msg}");
        assert!(msg.contains("hick fleet pair --new"), "{msg}");
        // And that it expires, since a stale phrase is the common failure.
        assert!(msg.contains("expire"), "{msg}");
    }

    #[test]
    fn the_same_phrase_derives_the_same_key_and_a_different_one_does_not() {
        // This is the whole mechanism: the phrase IS the rendezvous address.
        let a = Phrase::parse("anchor-drift-maple-harbor-47").unwrap();
        let b = Phrase::parse("ANCHOR DRIFT MAPLE HARBOR 47").unwrap();
        let c = Phrase::parse("anchor-drift-maple-harbor-48").unwrap();
        assert_eq!(
            a.rendezvous_key().unwrap().public(),
            b.rendezvous_key().unwrap().public()
        );
        assert_ne!(
            a.rendezvous_key().unwrap().public(),
            c.rendezvous_key().unwrap().public()
        );
    }

    #[test]
    fn the_wordlist_has_no_duplicates() {
        // A duplicate would quietly halve the entropy of that slot.
        let mut seen = WORDS.to_vec();
        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), before);
        assert!(
            WORDS.len() >= 128,
            "the list is smaller than the entropy claim"
        );
    }
}
</storycode:copy>

### Assemble `crates/hickory-peer/src/pairing.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-peer/src/pairing.rs"><storycode:paste select="#f-902bf51e0ac1-PAIR_ALPN"/><storycode:paste select="#f-902bf51e0ac1-WINDOW"/><storycode:paste select="#f-902bf51e0ac1-WORDS"/><storycode:paste select="#f-902bf51e0ac1-Phrase"/><storycode:paste select="#f-902bf51e0ac1-impl-Phrase"/><storycode:paste select="#f-902bf51e0ac1-impl-std-fmt-Display-for-Phrase"/><storycode:paste select="#f-902bf51e0ac1-Paired"/><storycode:paste select="#f-902bf51e0ac1-trade"/><storycode:paste select="#f-902bf51e0ac1-PairCancelled"/><storycode:paste select="#f-902bf51e0ac1-impl-std-fmt-Display-for-PairCancelled"/><storycode:paste select="#f-902bf51e0ac1-impl-std-error-Error-for-PairCancelled"/><storycode:paste select="#f-902bf51e0ac1-PairPhase"/><storycode:paste select="#f-902bf51e0ac1-phase"/><storycode:paste select="#f-902bf51e0ac1-close_endpoint"/><storycode:paste select="#f-902bf51e0ac1-cancelled"/><storycode:paste select="#f-902bf51e0ac1-host"/><storycode:paste select="#f-902bf51e0ac1-host_cancellable"/><storycode:paste select="#f-902bf51e0ac1-join"/><storycode:paste select="#f-902bf51e0ac1-join_cancellable"/><storycode:paste select="#f-902bf51e0ac1-tests"/></storycode:file>

<a id="output-25d0eba2b153"></a>
## `crates/hickory-peer/tests/pairing_cancel.rs`

Related contracts: [One Spoken Phrase Enrols Both Machines, With No Server And Nobody Trusted](contract.md).

<a id="f-25d0eba2b153-cancelling_a_bound_pairing_host_closes_it_before_local_enrollment"></a>
### `cancelling_a_bound_pairing_host_closes_it_before_local_enrollment`

<storycode:copy id="f-25d0eba2b153-cancelling_a_bound_pairing_host_closes_it_before_local_enrollment">
use hickory_fleet::{Fleet, Identity, Kind};
use hickory_peer::pairing::{PairCancelled, PairPhase, host_cancellable, join_cancellable};
use hickory_peer::{Phrase, Reach};
use std::time::Duration;

#[tokio::test]
async fn cancelling_a_bound_pairing_host_closes_it_before_local_enrollment() {
    let root = tempfile::tempdir().unwrap();
    let identity = Identity::load_or_create(root.path(), "private-cancel-fixture").unwrap();
    let roster = Fleet::at(identity.dir());
    let roster_path = identity.dir().to_path_buf();
    let phrase = Phrase::generate();
    let (cancel, receiver) = tokio::sync::watch::channel(false);
    let (progress, mut observed) = tokio::sync::watch::channel(PairPhase::Preparing);
    let task = tokio::spawn(async move {
        host_cancellable(
            &identity,
            &roster,
            &phrase,
            Kind::Desktop,
            &Reach::Direct,
            "2026-10-20",
            receiver,
            Some(progress),
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if *observed.borrow() == PairPhase::WaitingForPeer {
                break;
            }
            observed
                .changed()
                .await
                .expect("Host exited before binding its local fixture endpoint");
        }
    })
    .await
    .expect("Host did not bind in time");
    cancel.send(true).unwrap();
    let error = tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(error.downcast_ref::<PairCancelled>().is_some());
    assert_eq!(*observed.borrow(), PairPhase::Closed);
    assert!(Fleet::at(&roster_path).machines().is_empty());
}

</storycode:copy>
<a id="f-25d0eba2b153-an_already_cancelled_dial_never_enrolls_or_binds"></a>
### `an_already_cancelled_dial_never_enrolls_or_binds`

<storycode:copy id="f-25d0eba2b153-an_already_cancelled_dial_never_enrolls_or_binds">
#[tokio::test]
async fn an_already_cancelled_dial_never_enrolls_or_binds() {
    let root = tempfile::tempdir().unwrap();
    let identity = Identity::load_or_create(root.path(), "private-dial-fixture").unwrap();
    let roster = Fleet::at(identity.dir());
    let (_cancel, receiver) = tokio::sync::watch::channel(true);
    let (progress, observed) = tokio::sync::watch::channel(PairPhase::Preparing);
    let error = join_cancellable(
        &identity,
        &roster,
        &Phrase::generate(),
        Kind::Desktop,
        &Reach::Direct,
        "2026-10-20",
        receiver,
        Some(progress),
    )
    .await
    .unwrap_err();
    assert!(error.downcast_ref::<PairCancelled>().is_some());
    assert_eq!(*observed.borrow(), PairPhase::Preparing);
    assert!(roster.machines().is_empty());
}
</storycode:copy>

### Assemble `crates/hickory-peer/tests/pairing_cancel.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-peer/tests/pairing_cancel.rs"><storycode:paste select="#f-25d0eba2b153-cancelling_a_bound_pairing_host_closes_it_before_local_enrollment"/><storycode:paste select="#f-25d0eba2b153-an_already_cancelled_dial_never_enrolls_or_binds"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-peer/src/pairing.rs](contract.md#output-902bf51e0ac1)
- [crates/hickory-peer/src/tunnel.rs](../../../mechanisms/crates/hickory-peer/src/tunnel.rs.md#output-17fc8ed55ce5)

### Tests and fixtures

- [apps/web/src/views/FleetPane.test.tsx](../../../mechanisms/apps/web/src/views/FleetPane.tsx.md#output-ef08e5ce40d1)
- [crates/hickory-peer/tests/pairing_cancel.rs](contract.md#output-25d0eba2b153)
- [crates/hickory-peer/tests/peer_channel.rs](../../../mechanisms/crates/hickory-peer/tests/peer_channel.rs.md#output-669e41b69a09)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
