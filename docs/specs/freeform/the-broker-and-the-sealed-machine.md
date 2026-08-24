# The sealed machine and the broker: an agent with no keys and one road out

*Status: design of record for sealed machines and mediated egress. Adopted
2026-08-23. **Sequence steps 1 and 2 are built** (2026-08-23): the seal and
its check, and the broker with `allow`/`deny` — a CONNECT proxy with a host
policy and a log, no TLS termination, no keys, no CA. `ask` and `substitute`
are configurable and **deny with the reason** rather than hanging or
pretending. See
`docs/guarantees/collaboration/one-road-out-with-a-toll-booth.md`. It is the third of the sharing designs
— `one-engineer-many-machines.md` is how you reach the box, this is what the
box is allowed to do, and `machine-scoped-edits.md` is what an edit made on it
means. It obeys `local-only.md` without amendment: the broker is a subcommand
of the program you already downloaded, run by the engineer on hardware they
own. Nothing here is a service we operate, and nothing here requires buying
anything.*

An engineer buys a machine to run an agent on. The agent has a shell, a
checkout, and a model behind an API — which is to say it has everything needed
to read the repository and post it somewhere, and prompt injection means it can
be told to. The mitigation this design implements is the one that actually
holds: **the agent never has a credential worth stealing, and has exactly one
road out, with a toll booth on it.**

## What "airgapped" means here, stated precisely

It is not airgapped. An agent that can reach a model is on a network, and
calling it airgapped is the kind of nearly-true sentence `local-only.md` and
`notes-ide.md` both go out of their way to refuse. What is true:

> **The agent's machine has no route to the internet except through one broker
> the engineer controls, and holds no credential that is worth anything
> anywhere else.**

Two properties, deliberately separable, because each is useful without the
other:

1. **Key quarantine.** No real provider key is ever present in the sealed
   machine's environment, key store, memory, transcripts or session files. It
   holds a stub that only the broker accepts, and the broker substitutes the
   real key on its way past.
2. **Egress mediation.** Every outbound request from that machine is allowed,
   denied, or held for a human, per host, by policy the engineer wrote.

## What already exists

More than it looks, and the design is mostly composition:

| Piece | Already does |
|---|---|
| `*_BASE_URL` per provider | every client — `llm_anthropic.rs`, `llm_openai.rs` for OpenAI, DeepSeek, xAI, OpenRouter, Gab — already takes its endpoint from the environment. Pointing a machine at the broker needs no new code |
| `hick-token` | `NetworkRule::Allow { host, port }` and `DenyAll` as macaroon caveats, attenuable and never escalatable |
| `hickory-executor-sandbox` | confines a cell to its own workdir, reaches the network only if declared, and **refuses to run when it cannot confine** rather than degrading — the exact posture this needs |
| `hick-sink` | typed egress with per-slot classification policies, recipient allow-lists, and per-sink call ceilings |
| `hick-secrets` | age-encrypted secrets behind a `SecretsProvider` trait, with a zeroizing cache |
| `key_store.rs` | one private file, `0600`, `Debug` that cannot print a key |
| the peer channel | `one-engineer-many-machines.md` — somewhere to send a prompt that needs a human |

The new parts are a **seal** on a machine and a **broker** process. Everything
else is wiring.

## The seal

Sealing is a property a machine asserts about itself, set by the person
standing at it, recorded in that machine's own config. It is deliberately
**not** a grant another machine can hand out or take away: a fleet peer that
could unseal a box would mean a compromised laptop unseals the box, which
inverts the whole point.

On a sealed machine:

- **`KeyStore::set` refuses.** The Settings key page is replaced by a line
  saying this machine is sealed and keys live on the broker, naming it.
- **A provider variable in the environment is a boot failure, not a
  fallback.** `config-and-environments` already requires failing at boot with a
  message naming the variable, what was wrong with it, and what a valid value
  looks like. Here what was wrong is that it exists at all: *`ANTHROPIC_API_KEY`
  is set on a sealed machine. Unset it; this machine reaches the model through
  the broker at `http://broker.lan:7749`, using the stub credential in
  `~/.config/hick/sealed.toml`.* Silently ignoring it would leave a real key
  sitting in a process that is not supposed to hold one.
- **Every provider base URL must resolve to the broker**, and a client
  configured otherwise refuses to start rather than reaching the vendor
  directly.
- **The default route is denied at the OS**, so a process that ignores the
  proxy settings gets nothing rather than getting out. The proxy is the only
  permitted destination, and this is firewall configuration on the engineer's
  machine that `hick sealed --check` verifies rather than performs — we do not
  reconfigure somebody's networking for them.
- **The fleet channel carries no key material at all**, sealed or not. That is
  a property of the protocol rather than a rule with an exception, so there is
  nothing to enforce and nothing to get wrong.

`hick sealed --check` is the whole of it as one command, with the exit code and
the next steps `user-facing-errors` requires: no provider variable set, no key
file present, base URLs pointed at the broker, broker reachable and
authenticating this machine, default route denied.

## The broker

One process, `hick broker`, on **a different machine** than the sealed one —
the engineer's laptop, a NAS, a Raspberry Pi. It is a first-party HTTP proxy
and nothing else: it does not execute documents, does not store them, and never
accepts a connection from outside the local network.

```
sealed machine ──proxy──► hick broker ──► api.anthropic.com
  (no keys)                (holds keys,    (real key,
                            policy, log)    substituted here)
                                │
                                └──ask──► your laptop's IDE, via the fleet channel
```

Policy is per host, and there are four verbs:

| Verb | Meaning | Sees |
|---|---|---|
| `allow` | forward it | hostname only |
| `deny` | refuse it | hostname only |
| `ask` | hold it, and ask a human | hostname only |
| `substitute` | forward it, replacing the stub credential with the real one | **the whole request and response** |

**`substitute` requires terminating TLS, and that is a real property with a
real cost.** To put a header into a request the broker must be inside it, which
means the sealed machine trusts a CA generated on the broker, and the broker
can read every byte of the agent's model traffic. That is acceptable — both
machines belong to the same person and the traffic is that person's own — but
it must be *visible*, because "we can read your model traffic" is exactly the
sentence that must never be discovered rather than stated. So the two modes are
distinguished in the config, in `hick broker status`, and in the log, and a host
is never silently upgraded from `allow` to `substitute`.

**The stub is worthless everywhere else.** It is minted by the broker, bound to
the sealed machine's key, and rejected by the vendor because it was never a
vendor credential. An agent that leaks it leaks nothing; a human who copies it
onto another machine finds it does not work. That is the property that makes
quarantine better than "be careful with the key".

**`ask` needs somewhere to ask, and the fleet is it.** A prompt arrives in the
IDE session on whichever machine the engineer is sitting at, naming the host,
the method, the size, and the sealed machine and agent turn that caused it.
Answers are `allow once`, `allow for this turn`, `always allow`, `deny`. With
nobody to answer, the default is deny after a timeout — and the request is
recorded so it can be replayed if the engineer approves it later, because the
alternative is an agent that hangs for an hour and a half-finished turn nobody
can explain.

**A denial must be legible to the agent, not a hang.** The broker answers with
a structured 403 whose body the agent surfaces verbatim: *the broker denied a
request to `raw.githubusercontent.com`; ask the engineer to allow that host*. An
agent that receives a timeout invents a reason; an agent that receives a
sentence repeats it.

**Everything the broker did is a log the engineer can read** — append-only,
one line per decision, with the host, the verb, the rule that fired, the sealed
machine, the agent turn, and byte counts. Never a credential, never a body. The
log is the answer to *what did the machine in the basement do last night*, and
it is the reason the design is worth more than a firewall rule.

## Two things this does not defend against

Stated here rather than left to be discovered, because the design is worthless
if it is trusted for more than it does.

- **Exfiltration through the allowed channel.** An agent that may talk to a
  model may encode the repository into a prompt. The broker cannot tell that
  from work. What it can do is bound it — request size ceilings, per-turn call
  ceilings (`hick-sink` already models both), classification slots that refuse
  content labelled sensitive, and a log that makes an unusual volume visible
  after the fact. That is mitigation, not prevention, and no amount of
  proxy makes it prevention.
- **A compromised broker or a hostile provider.** The broker holds the real
  keys and, on `substitute` hosts, the plaintext. It is the most valuable
  machine in the fleet and should be the most boring one.

What it *does* defend against is worth restating so the trade is clear: an
agent that exfiltrates a key it was never given, an agent that reaches a host
the engineer never intended, and a prompt-injected agent that tries to post
somewhere new — the last of which becomes a prompt on the engineer's screen
naming a host they have never heard of, which is the single most useful signal
in the design.

## The hardware question

A Raspberry Pi is fine. A smart switch is the wrong shape, and the reasoning
matters more than the conclusion:

- **Transparent interception loses identity.** An inline device sees packets,
  not "the agent turn that caused this", so `ask` cannot say what asked and the
  log cannot say who did it.
- **It cannot substitute without the same CA anyway.** The hard part of
  `substitute` is TLS, and being inline does not help with it.
- **It is a second thing to keep running.** A brokered proxy is a process that
  can be restarted; a switch is a network that stops working.

And the product rule settles the default: **nothing may require buying
hardware.** The honest default is the broker on a machine the engineer already
has, with the Pi as one deployment of it rather than a component of the design.
A single-machine degenerate case must also work — broker and sealed machine as
two processes on one box, weaker (a root process can read the broker's keys)
and useful for trying the thing before buying a machine for it.

## Sequence

1. ~~**The seal, without a broker.**~~ **Built 2026-08-23.**
2. ~~**The broker, `allow`/`deny` only.**~~ **Built 2026-08-23.** One thing
   fell out of building it: a verb that is configured but not implemented must
   **deny with the reason**, never hang — an agent that receives a timeout
   invents a reason, and that argument applies to our own unbuilt verbs as much
   as to a denied host.
3. **`substitute`** — the CA, the stub credential, key storage on the broker
   via `hick-secrets`.
4. **`ask`**, over the fleet channel, with replay-after-approval.
5. **Ceilings and classification** on top of `hick-sink`.

Steps 1 and 2 stand alone, which is the test.

## Open edges

- **Git is egress too.** A sealed machine still needs the repository, and
  `notes-ide.md` puts sync on the user's own git remote. That is either an
  `allow` for one host or a LAN-only fetch from a peer, and which one is the
  default is not decided. It is also the widest exfiltration channel in the
  design: an agent that can push can push anything.
- **The broker's log wants to be a note.** A `.hick` document ingesting the
  decision log would make claims about the agent's egress checkable by `hick
  test` — *no host outside this list was reached this week* as an expectation
  that fails the day it stops being true. That is the product's own idea
  applied to itself and it is left as a second step, because a log format that
  is also a document format is a decision that should follow the log existing.
- **MCP servers and tools are egress with a different shape.** A tool that
  reaches the network from inside the agent's process is not obviously covered
  by a proxy setting, and `agent-cells.md`'s rule that the tool surface is the
  only write path has a sibling here that has not been written.
- **Whether a sealed machine may ever be unsealed remotely.** The answer here
  is no, and the cost is a trip to the basement. If that proves intolerable the
  temptation will be a fleet grant, which is the inversion this document
  refuses; the honest alternative is a physical gesture on the machine itself.
