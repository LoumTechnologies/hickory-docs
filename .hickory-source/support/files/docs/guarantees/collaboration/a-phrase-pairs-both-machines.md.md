<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/collaboration/a-phrase-pairs-both-machines.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `348b7e169691edc73ad14715cb86f9acd54613bfae14b22cb1a9087e79182ba1`.

<book:entry path="docs/guarantees/collaboration/a-phrase-pairs-both-machines.md">
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

## Cancellable peer exchange foundation (2026-10-20)

Peer host/dial helpers now accept cancellation and report actual preparing,
binding, waiting, key-exchange, enrolled and endpoint-closing/closed phases.
Cancellation before local enrollment closes the bound endpoint and does not add
a local roster entry. It cannot retract an invitation already sent to the remote
machine. Once local enrollment has happened, the actual paired result wins:
cancellation does not undo the roster or cut off the existing invitation-delivery
grace period. Connection/stream/key exchange has a bounded window, including
handshakes after acceptance. Errors close owned endpoints as well.

Two focused tests in `hickory-peer/tests/pairing_cancel.rs` passed: cancelling a
real bound direct/local endpoint reaches closed with an empty fixture roster,
and a pre-cancelled dial never binds/enrolls. The existing real offline two-way
pairing case also passed, preserving both keys and default grants. All identities
and rosters were temporary fixtures, with `Reach::Direct`, no relay and no change
to the user's fleet. File-length and diff checks passed. This is the peer-library
foundation; the shared UI's window-scoped job start/read/cancel wiring is still
required and is not established by these tests.

### Window-owned host requests (issue #72)

Versioned local pairing jobs bind to a live UI window. An exact request retry
returns the same host job; changing its arguments conflicts. Each window can
have one active request and eight retained results, with 64 total host slots.
Status, cancellation and forgetting require the originating live window. Pending
work cannot be forgotten; cancellation requests report the actual eventual
result without asserting rollback. Closing a window requests cancellation and
releases its records once work settles. IDs/results are in memory and do not
survive restart. Peer transport explicitly refuses this local authority surface.

Four focused host-state tests use deterministic fixture workers: idempotency and
window isolation, completed enrollment winning over cancellation, closed-window
cleanup, and bounded inspectable failures. They do not exercise native UI,
providers, actual fleet identities, or network pairing. The separate peer tests
above cover the real temporary-fixture network cancellation boundary. UI job
adoption remains the next integration step.

### Shared UI job adoption (issue #72)

The Fleet pane now starts versioned host/dial jobs using its actual UI scope;
it never falls back to the uncancellable blocking route. Both native controls
and registered agent actions inspect the same host job IDs and phases, request
cancellation, refresh retained requests and forget finished results. Full replies
remain behind an evidence disclosure. Only actual active jobs are polled, and
old pane/window responses cannot be adopted by a replacement controller.

An uncertain start retains its private request ID and exact arguments for retry.
Changed-argument retries require inspection rather than blindly starting again.
Definitive client refusals release the attempt. Cancellation before receiving a
job ID is explicitly pending and forwarded when that identity is recovered;
unmount also requests cancellation for known jobs and late start replies.
This does not promise that an uncertain transport failure means no work, that
remote invitations can be retracted, or that closing a pane rolls back enrollment.

Focused web validation uses mocked host replies: native phrase/dial adoption,
shared invitation/grant safety, actual job identity in agent cancellation,
uncertain retry identity, late-reply cancellation, pending-start cancellation
and later observed closed state. A focused API test verifies all versioned paths
and no legacy downgrade. Strict TypeScript and file-length checks apply to the
changed surface. These are fixture-backed integration witnesses, not a real
native/provider/remote-machine recording or released-build witness.
</book:entry>

</book:doc>
