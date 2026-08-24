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
pub mod proxy;
pub mod seal;

pub use policy::{BrokerLog, LogEntry, Policy, Verb, denial};
pub use proxy::{Broker, is_local, parse_connect};
pub use seal::{SealCheck, SealFinding, check_seal};
