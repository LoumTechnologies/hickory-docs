# One Spoken Phrase Enrols Both Machines, With No Server And Nobody Trusted

Given two machines that have never met, when a phrase generated on one is typed
on the other, then each ends up holding the other's public key — **both
directions in one exchange** — and no server, directory or third party is
involved in deciding whose key is whose.

**The phrase is the rendezvous.** Both machines derive the same throwaway
keypair from it (Argon2id over the phrase, with a fixed domain-separating
salt); one binds an endpoint under that key and waits, the other computes the
same public half — which is the address — and dials it. Over that connection
they trade their real keys.

That corrects a design note that had said this was impossible without a server:
a code short enough to read aloud cannot *carry* a public key, so it was
assumed to need a rendezvous somebody runs. It does not have to carry a key.
It can **be** one.

Corollaries that are part of the guarantee:

- **Mutual in one exchange.** A fleet is a mutual list, and the invitation flow
  needs two ceremonies to build one. This needs one — which is the friction
  the feature exists to remove.
- **Nobody is trusted for identity.** An account-backed directory could assert
  "this key is yours"; nothing here can. The trade-off is stated rather than
  hidden: **anyone who learns the phrase during the window can pair**, so it is
  generated rather than chosen, used once, and expires in two minutes.
- **Both fingerprints are printed on both ends**, because comparing them is the
  only check that catches somebody who guessed the phrase. The phrase proves
  nothing about who used it, and the tool says so in those words.
- **A phrase that is nearly right derives a completely different key**, so
  there is no partial success to misread — and the failure names expiry and
  single use, which are the usual causes.
- **Parsing forgives what people get wrong by ear** — case, spaces for hyphens,
  a trailing full stop — and nothing else.
- **Say "a one-time secret", never "secure pairing".** This is not a PAKE: a
  guessed phrase gets an attacker enrolled, and the guard is entropy (~34 bits)
  plus a short window plus a deliberately slow derivation, not a protocol that
  survives a weak secret.
- **Enrolment lands with `view` and `edit`, never `execute`**, exactly as the
  invitation flow does.
- **mDNS is on in every reachability posture**, which is what lets a phrase
  resolve on a LAN with no relay, no address publishing, and no third party at
  all. `Reach::Direct` therefore means "no third party", not "no discovery",
  and its summary says which.

What this does NOT replace: `hick fleet invite` / `accept` remains, for two
machines that cannot reach each other even to rendezvous.

---

Last LLM verification:
- Date: 2026-08-24
- Reviewer: Claude (Opus 5)
- Result: verified by tests, including two real endpoints pairing offline.
- Evidence:
  - `crates/hickory-peer/src/pairing.rs` — `Phrase::generate` (4 words + 2
    digits from a deduplicated 128-word list), `Phrase::parse` (forgiving by
    ear, strict otherwise), `Phrase::rendezvous_key` (Argon2id, fixed salt,
    justified), `host` / `join`, `trade`, `WINDOW`, and the flush wait before
    the endpoint closes.
  - `crates/hickory-peer/src/tunnel.rs` — `builder_for` adds
    `MdnsAddressLookup` on every posture; `Reach::Direct`'s summary says
    "mDNS" and "nothing published to anybody".
  - `crates/hickory-cli/src/main.rs` — `hick fleet pair [--new] [<phrase>]`
    and the fingerprint-comparison advice it prints.
  - Tests: `crates/hickory-peer/src/pairing.rs` unit tests (6, including the
    same-phrase-same-key property and the wordlist having no duplicates);
    `crates/hickory-peer/tests/peer_channel.rs` —
    `a_phrase_alone_enrols_both_machines_in_one_exchange` (two endpoints,
    `Reach::Direct`, offline) and `a_wrong_phrase_reaches_nothing`.
- Caveat requiring LLM review: the derivation uses a **fixed** salt, which is
  normally a mistake. It is required here — both ends must reach the same key
  from the phrase alone — and is defensible because the phrase is high-entropy
  and single-use, so the precomputation a random salt defends against buys an
  attacker nothing they could use inside a two-minute window. If phrases ever
  become user-chosen, that reasoning collapses and this must be revisited.
- Second caveat: an active attacker who knows the phrase can sit in the middle,
  because both sides derive the *same* secret rather than running a PAKE. The
  fingerprint comparison is what detects it, and it is a human step.
