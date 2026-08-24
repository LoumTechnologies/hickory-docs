# One Road Out, With A Toll Booth On It — Never An Airgap

Given a machine bought to run an agent on, when it is sealed and pointed at a
broker, then it holds no credential worth stealing and reaches the network only
through one process the engineer runs — and every host that process forwards to
is one the engineer allowed by name.

**It is not airgapped.** An agent that can reach a model is on a network, and
calling it airgapped is a nearly-true sentence this product refuses. What is
true is the heading, and it is the sentence the tool uses everywhere — in the
banner, in the refusal an agent receives, and in the check's output.

Two properties, deliberately separable because each is useful without the
other:

1. **Key quarantine.** No real provider key in the sealed machine's
   environment, key store, memory, transcripts or session files.
2. **Egress mediation.** Every outbound request allowed or denied per host, by
   policy the engineer wrote.

## The seal

- **Sealing is asserted by the person standing at the machine**, recorded in
  that machine's own config — deliberately **not** a grant another machine can
  hand out, because a fleet peer that could unseal the box would mean a
  compromised laptop unseals it, which inverts the whole point.
- **A provider variable is a failure, not a fallback.** The message names the
  variable, says to unset it, names the broker — and never prints the value it
  found. Silently ignoring it would leave a real key in a process that is not
  supposed to hold one.
- **A base URL that does not point at the broker is caught**, because that
  client would reach the vendor directly and go around the toll booth entirely.
- **`hick sealed --check` verifies and does not perform.** The default route
  being denied at the OS is firewall configuration on the engineer's own
  machine; an answer the check cannot get is *reported*, never assumed.
- **A sealed machine with no broker is a coherent state and is named as one** —
  it is a machine that cannot use a model at all, which is exactly what the
  seal alone buys.

## The broker

- **A CONNECT proxy and nothing else.** It does not execute documents, does not
  store them, and **never accepts a connection from outside the local network**
  — a broker exposed to the internet is an open proxy, which is the one thing
  it must never become. A non-local peer gets a bare 403 and learns nothing.
- **An unlisted host is denied.** A default of anything else would make the
  toll booth a sign rather than a booth.
- **A denial is a sentence the agent can repeat**, not a hang: an agent that
  receives a timeout invents a reason; an agent that receives a sentence
  repeats it.
- **The broker stays outside the connection.** `allow` forwards byte-for-byte,
  so the log holds a hostname, a port and a byte count — never a body, and the
  tests assert the body does not appear in it.
- **A host is never silently upgraded to `substitute`**, the one verb that
  would put the broker inside the connection. It requires terminating TLS, and
  "we can read your model traffic" is exactly the sentence that must be stated
  rather than discovered — so it takes an explicit flag, and `hick broker
  status` says which hosts it applies to.
- **An allowed host that cannot be reached says it is the network, not the
  policy**, so nobody debugs the wrong thing.

What this does NOT claim: `ask` and `substitute` are configurable but not
implemented, and both **deny with the reason** rather than hanging or pretending
— `ask` needs the fleet channel, which is not built, and `substitute` needs a CA
and TLS termination, which are not built. There is no stub-credential minting
and no key storage on the broker.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-broker/src/policy.rs` — `Verb`, `Policy` (deny by
    default, exact-beats-wildcard, longest-wildcard-wins, the
    never-silently-substitute refusal), `BrokerLog`, `denial`.
  - `crates/hickory-broker/src/proxy.rs` — `parse_connect` (CONNECT only),
    `is_local`, `Broker::handle` / `forward` / `refuse`, and the structured
    403.
  - `crates/hickory-broker/src/seal.rs` — `check_seal` and every finding.
  - `crates/hickory-cli/src/main.rs` — `hick broker serve|status|allow|deny|log`
    and `hick sealed`.
  - Tests: `crates/hickory-broker` unit tests (18) and
    `tests/proxy_e2e.rs` (5, over a real socket with a real upstream).
- Caveat requiring LLM review: `is_local` decides by address family. A machine
  whose LAN is bridged to the internet, or one reached through a NAT that
  rewrites the source address, would pass it. The design's own claim is that
  the broker never accepts a connection from outside the local network, and
  this implements the check an address can support — binding loopback by
  default is the stronger half of it.
