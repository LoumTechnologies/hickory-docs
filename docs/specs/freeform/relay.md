# The relay: making a laptop reachable

*Status: design of record for `hickory serve --public` and `apps/relay`.
Adopted 2026-08-11. The one cloud service `local-first.md` keeps.*

A session runs on someone's machine. `--share` binds it to the local network,
which covers two people in a room. Everyone else needs an address, and a laptop
behind NAT does not have one. The relay is the thing that forwards bytes, and
**nothing else**: it does not execute documents and does not store them. Its
only durable state is a table of accounts, because a quota has to be counted
against something that outlives a process.

```
guest browser ──HTTPS──►  relay.hickorydocs.com  ──existing outbound WS──►  hickory serve
                          (public, ours)                                     (laptop, theirs)
```

The direction of the arrow on the right is the whole trick: the laptop dials
*out* and the relay answers inbound requests by writing back down that same
connection. Nothing needs to be forwarded, opened, or configured on the host's
network.

## Identity: an account, obtained one of two ways

Opening a tunnel requires an account. Not because the product needs to know who
you are — the tool works with no account at all — but because a relay is a
general-purpose byte forwarder pointed at the internet, and an anonymous one
becomes a free tunnel service for whatever a stranger wants to expose. The
account is what makes a quota enforceable and abuse attributable.

**The relay says which ways in it has**, and the CLI asks before offering
anything:

```
GET /_relay/auth/methods → {"password": true, "github": {"client_id": "Iv1.…"}}
```

A relay with no OAuth app omits the `github` field entirely, and `hickory
login` then never mentions GitHub. That is the difference between an option
hidden because it does not exist and an option offered that fails at the last
step — which is the worst possible place to learn a feature is unconfigured.

**Email and password** is always available:

```
$ hickory login --signup
  Email: nate@example.com
  Choose a password: ␣
  Signed in as nate@example.com.
```

**GitHub, device flow**, when configured — not the web redirect flow. A CLI has
no browser to redirect back to, and a developer may be on a remote machine over
SSH. Device flow is designed for that: the CLI polls, the human authenticates
wherever their browser already is. No callback server, no localhost port, no
client secret anywhere. Scope requested: **none** — the default grants a token
that can read a public profile and nothing else.

### Whichever way in, the relay issues its own token

The GitHub token is exchanged, once, at sign-in; what the CLI stores is a token
*this relay* signed. Three consequences, all of them the reason it works this
way:

- a tunnel handshake verifies a signature locally and calls nobody, so a page
  load through the relay costs no third-party round trip;
- GitHub being down does not stop an already-signed-in session;
- a self-hosted relay works with its own OAuth app and no CLI rebuild, because
  the client id comes from `/methods`.

Credentials land in `~/.config/hickory/credentials.json`, mode `0600`, never in
the repository. `hickory logout` deletes that file and nothing else — the
token the relay issued stays valid until it expires, and we say so rather than
implying otherwise.

### The relay does persist accounts, and that is a change

The first version of this design said "no user table". Email/password made that
false: an account has to outlive the process it signed in from. So the relay
now keeps **one SQLite table** — id, email, password hash, GitHub login — and
nothing else. Tunnels, streams, and quotas are still memory-only and still die
with the process.

**Email/password is weaker attribution than GitHub.** A GitHub account is one
somebody else already vouched for; an address typed into a form is not, and
this relay sends no mail, so it cannot even prove the address exists. Password
accounts are therefore cheap to mint, which puts more weight on the quota. If
abuse becomes real, the answers are email verification or requiring GitHub —
both deliberately deferred rather than pre-built.

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

No *tunnel* state is persisted. A relay restart drops every tunnel and every
client reconnects, which is the entire disaster-recovery plan for the
forwarding half — sufficient because it holds nothing anyone would miss. The
accounts table is the exception, and the only thing worth backing up.

## The abuse surface, stated plainly

A byte forwarder pointed at the internet will be misused. What limits the
damage:

- **Every tunnel is attributable** to an account — a GitHub identity, or an
  email address that at least had to be typed and kept.
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
- **A GitHub OAuth app** — *optional*. Without one, the relay offers email and
  password only and the CLI hides the GitHub option. With one, set
  `GH_OAUTH_CLIENT_ID`; the client id is public by construction in device flow
  and the secret is never needed.
- **`RELAY_TOKEN_SECRET`** (≥32 bytes), which signs the tokens the relay
  issues. Required, and stable: a value that changed between restarts would
  sign everyone out.
- **`RELAY_DATABASE_URL`**, e.g. `sqlite:///data/relay.db` on a Fly volume.
  Absent, the relay still forwards for anyone holding a token it signed
  earlier, but nobody new can sign in.
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
