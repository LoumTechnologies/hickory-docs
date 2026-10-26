<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-peer/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1e5c254940f7"></a>
## `crates/hickory-peer/src/lib.rs`

<a id="f-1e5c254940f7-grants"></a>
### `grants`

The peer channel: reaching another of your machines' sessions.
See `docs/specs/freeform/one-engineer-many-machines.md`. Every session on
every machine you own, visible and drivable from whichever one you are
sitting at. It is not pair programming: there is one engineer, one
identity, and no second human anywhere in the design, which is what makes
it tractable without the relay, the accounts, and the abuse surface that
`local-collaboration.md` needed and `local-only.md` deleted.
## The transport is QUIC dialled by public key
The design sketched Noise over a WebSocket. `iroh` is that property with
an implementation nobody here had to write — and its endpoint identity **is
an ed25519 public key**, which is what a machine already is in this
product. So the fleet's mutual key list is the allowlist directly: a
connection arrives already authenticated as a key, and either that key is
in the list or the connection is closed.
## Three channels, still never confused
**Durable state never travels over this.** Two machines editing the same
document do not sync files to each other; they each commit and push, and
git reconciles. What crosses here is the *live* room for a document open in
both places, which is the same second-writer problem the CRDT already
exists for — the third writer is simply on another box.
The consequence is worth stating because it will feel like a bug the first
time: **close both sessions without committing and the machines diverge.**
That is correct. This is a window onto a session, not a replication
protocol.
## Reachability, and the relay
Resolution is iroh's: a direct path where one exists — the LAN case, and
the engineer's-own-overlay case — and a relay to coordinate hole-punching
and to carry traffic when no direct path can be found.
**The default relay is number0's, and that is a decision recorded rather
than a default inherited** (2026-08-24). It is not a server *we* run, so
`local-only.md`'s refusal is intact and the sentence "nothing talks to a
server we run" stays true — but a third party in the data path is a
posture this product takes nowhere else, so the product **says** when a
connection is relayed instead of leaving it to be discovered, and
[`Reach`] can name the engineer's own relay or refuse relays entirely.

<storycode:copy id="f-1e5c254940f7-grants">
//! The peer channel: reaching another of your machines' sessions.
//!
//! See `docs/specs/freeform/one-engineer-many-machines.md`. Every session on
//! every machine you own, visible and drivable from whichever one you are
//! sitting at. It is not pair programming: there is one engineer, one
//! identity, and no second human anywhere in the design, which is what makes
//! it tractable without the relay, the accounts, and the abuse surface that
//! `local-collaboration.md` needed and `local-only.md` deleted.
//!
//! ## The transport is QUIC dialled by public key
//!
//! The design sketched Noise over a WebSocket. `iroh` is that property with
//! an implementation nobody here had to write — and its endpoint identity **is
//! an ed25519 public key**, which is what a machine already is in this
//! product. So the fleet's mutual key list is the allowlist directly: a
//! connection arrives already authenticated as a key, and either that key is
//! in the list or the connection is closed.
//!
//! ## Three channels, still never confused
//!
//! **Durable state never travels over this.** Two machines editing the same
//! document do not sync files to each other; they each commit and push, and
//! git reconciles. What crosses here is the *live* room for a document open in
//! both places, which is the same second-writer problem the CRDT already
//! exists for — the third writer is simply on another box.
//!
//! The consequence is worth stating because it will feel like a bug the first
//! time: **close both sessions without committing and the machines diverge.**
//! That is correct. This is a window onto a session, not a replication
//! protocol.
//!
//! ## Reachability, and the relay
//!
//! Resolution is iroh's: a direct path where one exists — the LAN case, and
//! the engineer's-own-overlay case — and a relay to coordinate hole-punching
//! and to carry traffic when no direct path can be found.
//!
//! **The default relay is number0's, and that is a decision recorded rather
//! than a default inherited** (2026-08-24). It is not a server *we* run, so
//! `local-only.md`'s refusal is intact and the sentence "nothing talks to a
//! server we run" stays true — but a third party in the data path is a
//! posture this product takes nowhere else, so the product **says** when a
//! connection is relayed instead of leaving it to be discovered, and
//! [`Reach`] can name the engineer's own relay or refuse relays entirely.

pub mod grants;
</storycode:copy>
<a id="f-1e5c254940f7-pairing"></a>
### `pairing`

<storycode:copy id="f-1e5c254940f7-pairing">
pub mod pairing;
</storycode:copy>
<a id="f-1e5c254940f7-tunnel"></a>
### `tunnel`

<storycode:copy id="f-1e5c254940f7-tunnel">
pub mod tunnel;

</storycode:copy>
<a id="f-1e5c254940f7-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-1e5c254940f7-use_declaration">
pub use grants::{Denial, permitted};
</storycode:copy>
<a id="f-1e5c254940f7-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-1e5c254940f7-use_declaration-2">
pub use pairing::{Paired, Phrase, host, join};
</storycode:copy>
<a id="f-1e5c254940f7-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-1e5c254940f7-use_declaration-3">
pub use tunnel::{ALPN, Attached, PeerServer, Reach, Reachability};
</storycode:copy>

### Assemble `crates/hickory-peer/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-peer/src/lib.rs"><storycode:paste select="#f-1e5c254940f7-grants"/><storycode:paste select="#f-1e5c254940f7-pairing"/><storycode:paste select="#f-1e5c254940f7-tunnel"/><storycode:paste select="#f-1e5c254940f7-use_declaration"/><storycode:paste select="#f-1e5c254940f7-use_declaration-2"/><storycode:paste select="#f-1e5c254940f7-use_declaration-3"/></storycode:file>

</storycode:doc>
