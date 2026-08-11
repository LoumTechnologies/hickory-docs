# A Guest Outside The Network Reaches The Session, And Gains Nothing By It

Given `hickory serve --share --public` on a machine behind NAT, when someone
outside that network opens the share link, then they get the document, its
generated outputs, the byte-precise lineage, and a live collaborative room —
and they get **exactly** the access the session's own capability check grants
them, no more.

The relay forwards bytes. It is not a second door, and it does not vouch for
anyone:

- **The session's checks run unchanged.** A request arriving through the relay
  is served by the same router that serves a guest on the LAN, so a bad token
  is 403 and a read-only link cannot write — from anywhere in the world.
- **The relay identifies the *host*, not the guest.** Opening a tunnel costs an
  account — an email and password, or GitHub where a relay offers it — because
  a byte forwarder pointed at the internet has to be attributable and
  quota-able. Guests remain anonymous, as they are on the LAN.
- **No third party is called during a session.** The handshake verifies a token
  the relay itself signed at sign-in; GitHub, where used, is asked once at
  sign-in and never again. The test asserts the count is zero during a
  session — a relay that phoned a third party per page load would be rate
  limited into uselessness and would stop working whenever that party did.
- **The only thing stored is accounts.** No documents, no sessions, no tunnel
  state. A relay restart drops every tunnel and every client reconnects; that
  is the whole recovery plan for the forwarding half.
- **A dead session says so.** An address with no tunnel behind it answers 404
  with the reason a person would recognise — the host ended it — rather than a
  gateway error.

## The two races this cost, recorded because they will recur

Both were found by the end-to-end test and both had the same shape: a frame
arriving on the reader loop before the task meant to receive it had been
scheduled.

1. **Request bodies.** A body arrives in `Data` frames *after* its `Open`. The
   first version registered the body channel inside the spawned task, so every
   PUT and POST through the tunnel reached the local router with an empty body
   and came back 400.
2. **The document room.** Same bug, worse symptom: the guest's opening "send me
   the document" was dropped, so the room sat silent and the page never
   loaded — with no error anywhere, because nothing had failed.

Both are fixed by registering the receiving end **in the reader loop, before
spawning**. Anything added to this protocol that arrives after an `Open` must
do the same.

## Boundaries

- **Traffic is plaintext inside the relay process.** TLS terminates there and
  is re-established to the laptop; forwarding requires reading. This is not
  end-to-end encryption and must never be described as such.
- **A signed-in host keeps their access until the relay's token expires.**
  Signing out on a machine deletes the local copy; it does not invalidate the
  token. Nor does revoking the GitHub grant, since the relay stopped depending
  on it after the exchange.
- **Email/password is weaker attribution than GitHub.** The relay sends no
  mail, so it cannot prove an address exists; accounts are cheap to mint and
  the quota carries more of the weight.
- **Backpressure is bounded, not solved.** A guest faster than the host's
  uplink overflows a stream's queue and the stream is closed, which shows up as
  a dropped connection rather than a stall.
- **Quotas are per process.** Three tunnels per account and an eight-hour life,
  held in memory. Two relay instances would each allow three; that is fine at
  one instance and is the first thing to fix at two.

---

Last LLM verification:
- Date: 2026-08-11
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-relay` defines the frame protocol both ends share.
  `apps/relay/src/tunnel.rs::serve_agent` verifies a token the relay signed
  (`hickory_identity::verify_token`), admits the account against
  `TunnelCensus`, and registers the tunnel; `apps/relay/src/auth_routes.rs`
  issues those tokens for either sign-in method and omits `github` from
  `/methods` when no OAuth app is configured; `RelayState::slug_of`
  matches the host suffix so a crafted `Host` cannot borrow a tunnel.
  `crates/hickory-cli/src/serve/tunnel.rs` serves guest requests against the
  session's own `Router` and bridges guest WebSockets into the session's real
  listener, so `share::authorize` and the scope checks run exactly as they do
  on the LAN.
- Test coverage: `crates/hickory-cli/tests/relay_end_to_end.rs` runs a relay,
  a session, and a guest in one process:
  `a_guest_outside_the_network_reaches_the_document_through_the_relay`
  (document + lineage + the once-per-tunnel identity check),
  `the_document_room_works_through_the_relay` (a real Yjs handshake over the
  tunnel, asserting the document arrives uncorrupted),
  `the_relay_does_not_admit_anyone_the_session_would_refuse` (bad token, and a
  read-only link's write, both refused),
  `the_quota_stops_an_account_opening_tunnels_without_end`,
  `a_token_this_relay_did_not_sign_opens_nothing`,
  `an_address_with_no_session_behind_it_says_so_plainly`, and
  `a_closed_session_takes_its_address_with_it`.
  `crates/hickory-cli/tests/relay_auth.rs` covers sign-in: that an
  unconfigured relay neither advertises GitHub nor calls it, that a configured
  one hands the CLI the client id it needs, that a password sign-in yields a
  relay-signed token, and that a wrong password is indistinguishable from an
  unknown address. `crates/hickory-identity` unit-tests the primitives.
- Not covered by tests: TLS, wildcard DNS, and the real GitHub device flow
  against github.com — none of which exist yet (`docs/specs/freeform/relay.md` lists what must be
  provisioned). The relay has never run outside a test process.
