# The relay: making a laptop reachable

*Status: design of record for `hickory serve --public` and `apps/relay`.
Adopted 2026-08-11. The one cloud service `local-first.md` keeps.*

A session runs on someone's machine. `--share` binds it to the local network,
which covers two people in a room. Everyone else needs an address, and a laptop
behind NAT does not have one. The relay is the thing that forwards bytes, and
**nothing else**: it does not execute documents, does not store them, and holds
no state that outlives the session.

```
guest browser ──HTTPS──►  relay.hickorydocs.com  ──existing outbound WS──►  hickory serve
                          (public, ours)                                     (laptop, theirs)
```

The direction of the arrow on the right is the whole trick: the laptop dials
*out* and the relay answers inbound requests by writing back down that same
connection. Nothing needs to be forwarded, opened, or configured on the host's
network.

## Identity: GitHub OAuth, device flow

Opening a tunnel requires a GitHub account. Not because the product needs to
know who you are — the tool works with no account at all — but because a relay
is a general-purpose byte forwarder pointed at the internet, and an anonymous
one becomes a free tunnel service for whatever a stranger wants to expose. The
account is the thing that makes a quota enforceable and abuse attributable.

**Device flow**, not the web redirect flow:

```
$ hickory login
  Open https://github.com/login/device and enter:  WDJB-MJHT
  Waiting…
  Signed in as @nate
```

A CLI has no browser to redirect back to, and a developer may be on a remote
machine over SSH. Device flow is designed for exactly that: the CLI polls, the
human authenticates wherever their browser already is. No callback server, no
localhost port, no client secret on the user's machine.

Scope requested: **none**. The default (no scopes) grants a token that can read
a public profile and nothing else — no repositories, no email, no
organisations. The relay needs exactly one fact, "which GitHub account is
this", and asking for more would be asking for trust the feature does not need.

Credentials land in `~/.config/hickory/credentials.json`, mode `0600`, never in
the repository. `hickory logout` deletes the file and nothing else — GitHub
tokens are revoked by the user at GitHub, and we say so rather than implying we
can do it for them.

### What the relay verifies

On each tunnel open the relay calls `GET https://api.github.com/user` with the
presented token, once, and keeps the resulting login for the life of the
connection. No user table, no sessions, no password reset, no email. The
account exists at GitHub; we borrow it.

A revoked token therefore stops working at the *next* connection, not
mid-session. That is the right trade for a session measured in hours, and it is
recorded here rather than discovered.

## The tunnel protocol

One outbound WebSocket carries every guest's traffic, multiplexed by stream id.
Frames are small and explicit; the protocol lives in `crates/hickory-relay` and
is shared by both ends so they cannot disagree about it.

| Frame | Direction | Meaning |
|---|---|---|
| `Hello { token, requested_slug }` | agent → relay | Open a tunnel |
| `Ready { url, slug }` | relay → agent | The public address |
| `Open { stream, request }` | relay → agent | A guest request begins |
| `Data { stream, bytes }` | both | Body or WebSocket payload |
| `Response { stream, status, headers }` | agent → relay | Head of the reply |
| `Close { stream, reason }` | both | End of one stream |
| `Ping` / `Pong` | both | Liveness |

Guest WebSockets (the Yjs channel) are the reason `Data` is bidirectional and
the reason streams are not request/response pairs: a document room is a long
socket carrying frames both ways for as long as someone is editing.

**Backpressure is real and unsolved in v1.** A guest on a fast connection can
outrun a laptop on hotel wifi. The relay bounds each stream's queue and closes
the stream when it overflows, which is a blunt instrument that shows up as a
dropped connection rather than a stall. Stated because someone will hit it.

## Addressing

Each session gets `https://<slug>.relay.hickorydocs.com`, where the slug is
random and unguessable-ish (the capability token in the link is what actually
protects the session — the slug is an address, not a secret).

A subdomain rather than a path prefix, because the client is a single-page app
that loads `/assets/…` and opens `/api/ws`: path-prefixing every URL in a
bundle that is *also* served at the root by `hickory serve` means one build
cannot serve both. The cost is infrastructure — a wildcard DNS record and a
wildcard certificate — and that cost is paid once by us rather than by every
URL in the client.

## Quotas, and what they are for

Per authenticated account, held in the relay process only:

- **3 concurrent tunnels.** Enough for a laptop, a desktop, and a spare;
  nowhere near enough to run a tunnel service on top of ours.
- **8 hours per tunnel**, then it closes. A session is a working session, not
  a deployment.
- **A stream ceiling and a per-stream queue bound**, so one guest cannot
  exhaust the process.

These are deliberately small and deliberately not configurable per user: the
first version of a quota system that has exceptions is a billing system, and
pricing is not settled (`local-first.md`).

Nothing is persisted. A relay restart drops every tunnel, and every client
reconnects — which is also the entire disaster-recovery plan, and is
sufficient because the relay holds nothing anyone would miss.

## The abuse surface, stated plainly

A byte forwarder pointed at the internet will be misused. What limits the
damage:

- **Every tunnel is attributable** to a GitHub account.
- **Tunnels are short-lived** and few per account.
- **The relay does not fetch anything** on a guest's behalf: it only forwards
  to the one laptop that opened the tunnel. It is not an open proxy, and it
  cannot be turned into one by a crafted request.
- **The content is whatever that laptop serves.** We do not inspect it, and we
  cannot: that is the point of the design. What we can do is stop forwarding
  for an account when someone reports it, which requires knowing the account —
  which is why the OAuth is there.

Not claimed: that this is sufficient at scale. It is sufficient at the scale of
"a pre-launch tool with a handful of users", and the moment it is not, the
answer is rate limits and a block list, not architecture.

## What this needs that does not exist yet

- **DNS**: a wildcard `*.relay.hickorydocs.com` record.
- **TLS**: a wildcard certificate (Fly issues these with DNS validation).
- **A GitHub OAuth app**, whose client id ships in the CLI (public by
  construction in device flow) and whose secret the relay never needs — device
  flow for a public client does not use one.
- **Deployment**: the relay is a second Fly app, or a second process in the
  existing one. It shares nothing with the hosted workspace and should outlive
  it.

## Boundaries

- The relay never sees a document it is entitled to keep, and keeps none.
- The relay never executes anything.
- Guests still need the session's capability token; the relay forwards a
  request to the laptop, which enforces scope exactly as it does on the LAN
  (`local-collaboration.md`).
- Traffic is encrypted guest↔relay (TLS) and relay↔laptop (TLS on the control
  socket), and **plaintext inside the relay process**, which is what makes
  forwarding possible. Anyone who claims end-to-end encryption for this design
  would be wrong.
