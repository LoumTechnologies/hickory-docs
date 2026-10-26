<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-broker/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b5a517cec2a0"></a>
## `crates/hickory-broker/src/lib.rs`

<a id="f-b5a517cec2a0-policy"></a>
### `policy`

The sealed machine and the broker: an agent with no keys and one road out.
See `docs/specs/freeform/the-broker-and-the-sealed-machine.md`. An engineer
buys a machine to run an agent on. The agent has a shell, a checkout and a
model behind an API — which is to say it has everything needed to read the
repository and post it somewhere, and prompt injection means it can be told
to. The mitigation implemented here is the one that actually holds:
> **The agent never has a credential worth stealing, and has exactly one
> road out, with a toll booth on it.**
**It is not airgapped.** An agent that can reach a model is on a network,
and calling it airgapped is the kind of nearly-true sentence this product
goes out of its way to refuse. What is true is the sentence above.
Two properties, deliberately separable because each is useful without the
other:
1. **Key quarantine** — no real provider key is ever present in the sealed
machine's environment, key store, memory, transcripts or session files.
2. **Egress mediation** — every outbound request is allowed, denied, or
held for a human, per host, by policy the engineer wrote.
What is built here is the seal (step 1) and the broker with `allow`/`deny`
(step 2). Steps 1 and 2 stand alone, which is the test: on its own, a
sealed machine with no broker is a machine that cannot use a model at all,
which is a coherent and testable state.

<storycode:copy id="f-b5a517cec2a0-policy">
//! The sealed machine and the broker: an agent with no keys and one road out.
//!
//! See `docs/specs/freeform/the-broker-and-the-sealed-machine.md`. An engineer
//! buys a machine to run an agent on. The agent has a shell, a checkout and a
//! model behind an API — which is to say it has everything needed to read the
//! repository and post it somewhere, and prompt injection means it can be told
//! to. The mitigation implemented here is the one that actually holds:
//!
//! > **The agent never has a credential worth stealing, and has exactly one
//! > road out, with a toll booth on it.**
//!
//! **It is not airgapped.** An agent that can reach a model is on a network,
//! and calling it airgapped is the kind of nearly-true sentence this product
//! goes out of its way to refuse. What is true is the sentence above.
//!
//! Two properties, deliberately separable because each is useful without the
//! other:
//!
//! 1. **Key quarantine** — no real provider key is ever present in the sealed
//!    machine's environment, key store, memory, transcripts or session files.
//! 2. **Egress mediation** — every outbound request is allowed, denied, or
//!    held for a human, per host, by policy the engineer wrote.
//!
//! What is built here is the seal (step 1) and the broker with `allow`/`deny`
//! (step 2). Steps 1 and 2 stand alone, which is the test: on its own, a
//! sealed machine with no broker is a machine that cannot use a model at all,
//! which is a coherent and testable state.

pub mod policy;
</storycode:copy>
<a id="f-b5a517cec2a0-proxy"></a>
### `proxy`

<storycode:copy id="f-b5a517cec2a0-proxy">
pub mod proxy;
</storycode:copy>
<a id="f-b5a517cec2a0-seal"></a>
### `seal`

<storycode:copy id="f-b5a517cec2a0-seal">
pub mod seal;

</storycode:copy>
<a id="f-b5a517cec2a0-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-b5a517cec2a0-use_declaration">
pub use policy::{BrokerLog, LogEntry, Policy, Verb, denial};
</storycode:copy>
<a id="f-b5a517cec2a0-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-b5a517cec2a0-use_declaration-2">
pub use proxy::{Broker, is_local, parse_connect};
</storycode:copy>
<a id="f-b5a517cec2a0-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-b5a517cec2a0-use_declaration-3">
pub use seal::{SealCheck, SealFinding, check_seal};
</storycode:copy>

### Assemble `crates/hickory-broker/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-broker/src/lib.rs"><storycode:paste select="#f-b5a517cec2a0-policy"/><storycode:paste select="#f-b5a517cec2a0-proxy"/><storycode:paste select="#f-b5a517cec2a0-seal"/><storycode:paste select="#f-b5a517cec2a0-use_declaration"/><storycode:paste select="#f-b5a517cec2a0-use_declaration-2"/><storycode:paste select="#f-b5a517cec2a0-use_declaration-3"/></storycode:file>

</storycode:doc>
