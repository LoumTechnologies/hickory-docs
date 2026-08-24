# One engineer, many machines: reaching a session from the chair you are in

*Status: design of record for the sharing mode. Adopted 2026-08-23.
**Sequence steps 1 and 2 are built** (2026-08-23 and 2026-08-24): machine
identity, the mutual key list, grants, `hick fleet`, and the authenticated
peer channel carrying the existing API under those grants. The transport is
**iroh 1.0 — QUIC dialled by public key** rather than the Noise-over-WebSocket
sketched below: its endpoint identity IS an ed25519 public key, so the fleet's
key list is the allowlist directly and nobody here wrote the crypto. Steps 3
to 6 (the fleet pane's attach, the `execute` grant's own surface, transports 2
and 3 behind the seam, follow) are not built.
The short one-time pairing code below is **built as designed** (2026-08-24),
after a first attempt got its reasoning wrong. That attempt said a code so
short cannot carry a public key, so it must assume a rendezvous, and the only
rendezvous for two unreachable machines is one we would run — which
`local-only.md` deletes — and it substituted a long self-contained invitation.
**The code does not have to carry a key; it can BE one.** Both machines derive
the same throwaway keypair from the phrase, one binds an endpoint under it and
the other dials it, and they trade real keys over that connection. The phrase
is the rendezvous. Nobody is trusted for identity and no server exists — and
because both keys cross in one exchange, one phrase finishes a job the
invitation flow needs two of. The invitation remains, for pairing where the two
machines cannot reach each other at all.
See `docs/guarantees/collaboration/a-machine-is-a-keypair.md`. It extends `local-only.md` and `notes-ide.md` without
reopening either: no server we operate, no account, no relay, no money. The
machines in this document are machines the engineer already owns, in the same
category as their Docker daemon, their Canopy node, and their git remote.
Pairs with `machine-scoped-edits.md` (what an edit made on one of them means),
`the-broker-and-the-sealed-machine.md` (a member with no keys and one road
out), and `changes-not-commits.md` (the open question underneath both).*

One person has a laptop, a desktop, a Windows box for the thing that only
builds on Windows, and — increasingly — a machine bought to run an agent. Each
of them is running the IDE on some worktree. Today each of those is an island:
the only way to see what the desktop is doing is to walk to the desktop.

This mode makes every session on every one of those machines **visible and
drivable from whichever one you are sitting at**. It is not pair programming.
There is one engineer, one identity, and no second human anywhere in the
design — which is what makes it tractable, and the reason it can arrive
without the relay, the accounts, and the abuse surface that
`local-collaboration.md` and `relay.md` needed and `local-only.md` deleted.

## What a session is, precisely

The word is already overloaded — an agent conversation is a session file
(`docs/guarantees/agent/a-session-is-the-conversation.md`). This document means
something else, so it gets its own word where it matters.

A **session** is one IDE process on one machine, rooted at one **workspace**: a
set of worktrees of one or more git repositories, plus the tabs, panes,
terminals, agent docks and drafts open over them. It already has durable
identity — `hickory-workspace` keys per-user state by the project's canonical
path — and it already has a server: the desktop app runs `serve::prepare`'s
router in-process on loopback and points its window at it.

That last fact is the whole mechanism. **The app is already a client of a
server it does not have to be co-located with.** Remote viewing is pointing
that client at another machine's router; remote control is the same, with
writes allowed.

A **fleet** is the set of machines that hold each other's keys. It is a
property of those machines and of nothing else — there is no registry, no
directory, no place a fleet exists apart from the machines in it.

## Three channels, and never confusing them

`notes-ide.md` already draws one of these lines and the design dies if the
other two blur into it.

| Channel | Carries | Reconciled by | Lives as long as |
|---|---|---|---|
| **git, to the user's own remote** | documents, outputs, history, everything durable | `hick-merge`, three-way | forever |
| **the peer channel** | liveness — editor buffers, cursors, run events, terminal bytes, the agent dock, broker prompts | `hickory-collab` / Yrs, in one room per open document | the two processes |
| **the local disk** | what this machine's checkout actually holds | the up-loop's `WovenState` and the directory lock | the machine |

**Durable state never travels over the peer channel.** Two sessions on two
machines editing the same document do not sync their files to each other; they
each commit and push, and git reconciles. What crosses the peer channel is the
*live* room for a document that is open in both places at once, which is the
same second-writer problem `local-only.md` already keeps the CRDT for — the
third writer is simply on another box.

The consequence is worth stating because it will feel like a bug the first
time: **close both sessions without committing and the machines diverge.** That
is correct. The peer channel is a window onto a session, not a replication
protocol, and a design where a laptop's unsaved buffer silently becomes the
desktop's file is a design that loses work when the window closes.

## Locking, restated for many machines

The rule that exists today — one process per directory, advisory lock, the
second refuses — generalises cleanly:

- **One session per worktree path per machine.** Unchanged; the lock is the
  same lock, and it is what stops the up-loop from fighting the app.
- **Many sessions across machines, on checkouts of the same repository.**
  Allowed, because they are different files on different disks and git is the
  reconciler. The fleet view shows that two machines are on the same branch,
  because that is a thing you want to know before you push.
- **Attaching to a remote session takes no lock at all.** You are not a second
  writer to that disk; you are a second writer to that session's *rooms*,
  which is what rooms are for.

## Identity: keys, not accounts

Enrolment is the one ceremony, and it is SSH's, not ours:

```
$ hick fleet pair                       # on the machine you are joining from
  Pairing code: 7 QUAIL DRIFT 2         # short, one-time, expires in 60s
$ hick fleet pair 7-QUAIL-DRIFT-2       # on the machine already in the fleet
  Added "windows-dev" (ed25519 SHA256:…), grants: view, edit
```

- **A machine is a keypair**, generated on first run, private half in the
  platform keychain, never leaving the machine, never in the repository.
- **A fleet is a mutual list of public keys**, one file per machine under the
  per-user state directory `hickory-workspace` already owns — deliberately not
  in the project, for the reason that module already gives: git would
  eventually commit it.
- **There is no account, no directory, and no server** to be signed in to. The
  pairing code is exchanged over the channel that already exists between two
  machines you own — a LAN, or you typing it.
- **Revocation is deleting a public key**, and it is complete, because no
  server holds a session you cannot reach. Contrast `local-collaboration.md`'s
  honest admission that a capability token could not be revoked without
  restarting: that limit came from tokens minted for strangers, and it goes
  away when the only holder is you.

**Grants are per machine, per verb**, and they are the trust model:

| Grant | Lets a peer | Default |
|---|---|---|
| `view` | see documents, outputs, ribbons, transcripts, the running terminal's bytes | on |
| `edit` | write into the room, and thereby into the file | on |
| `execute` | run a cell or the up-loop's pipeline, type into a shell, start or steer an agent turn | **off** |

*Revised 2026-08-23: `run`, `terminal` and `agent` were three separate grants.
They all mean "code runs on that machine as me", so three switches implied a
precision the security model does not have.* **What `execute` costs is a
property of the machine, not of the grant** — a cell under
`hickory-executor-sandbox` or Docker is confined to its own workdir and a shell
is not — so the fleet pane names the peer's executor beside it. A sealed agent
box (`the-broker-and-the-sealed-machine.md`) visibly grants less than a laptop
on `LocalExecutor`, without adding a switch nobody would reason about
correctly.

The dangerous sentence has not gone away, it has changed owner: *a remote
machine can cause code to run here*, and the remote machine is yours. So the
protection is not scope-for-strangers, it is blast radius when one of your own
machines is compromised. `execute` is off by default and granted per peer,
because "my laptop was stolen" should not read as "every machine I own now
executes whatever the thief types". A phone (`notes-ide.md`) enrols with `view`
and cannot be given `execute`, since it has no executor to grant.

Two things are deliberately **not** claimed. A peer with `edit` is not
harmless — it can write a document you later run, which is the same trust as
accepting a pull request and is stated so nobody assumes otherwise. And a
compromised machine with `execute` on an unsandboxed peer is a full compromise
of that peer; the grant model bounds it, it does not defeat it.

## Transport: the user's own, through a seam

`hick serve --public` already resolves a public URL through a provider seam
rather than a hard dependency (`HICKORY_PUBLIC_URL`, then PortZero). The fleet
channel takes the same shape and the same discipline — **with no first-party
relay, now or later.**

Resolution order, first that answers wins, each attempted only for peers whose
key we hold:

1. **Direct on the local network.** mDNS advertisement of `_hickory._tcp`,
   the peer's key as the TXT record's fingerprint. Covers the laptop and the
   desktop in one house, which is most of the actual use.
2. **The engineer's own overlay.** A Tailscale, Nebula, WireGuard or PortZero
   address, taken from configuration. This is the case that works from a café,
   and it works because the engineer already runs the overlay.
3. **An SSH reverse tunnel** to a host they already have.

With none of them available, `hick fleet` says the peer is unreachable and
names all three paths. It does not offer to make the machine reachable, and
this is the point at which the design refuses to grow the thing
`local-only.md` deleted:

> **A relay we operate is out of scope, not deferred.** It was the only
> meterable thing, it needed identity from day one, and an anonymous one is a
> free tunnel service. None of that changed because the two endpoints now
> belong to the same person — we would still be running the box.

Transport security is not the overlay's job even when there is one: every peer
connection is authenticated by the enrolled keys and encrypted end to end
(Noise over a WebSocket, one implementation, both ends). An overlay that
already encrypts is then belt and braces, which is what you want when the
alternative is a hostname resolving to the wrong machine.

## What the client actually does

Nothing in the document view changes, which is the test that this is the right
seam. `api.outputs(docId)`, `api.outputFile(…)`, `POST /docs/:id/run`,
`WS /api/ws?doc=…` are already the whole surface of a session; a remote session
answers them from its machine. What is new is above them:

- **A fleet pane** beside the folder tree: machines, each with its sessions,
  each session with its workspace, branch per worktree, dirty state, whether an
  agent turn is running, and whether it is reachable.
- **Attach** opens a peer's session as tabs in your window, marked — a persistent
  strip naming the machine, because "which box am I typing into" must never be
  a thing you infer from the font. The same reason the status bar shows the
  branch.
- **Awareness is you, twice.** The presence layer exists and is kept, but it
  labels machines rather than people: *you, on windows-dev*. One human with two
  cursors still needs to see both, and calling them by machine is the honest
  label.
- **Follow**, a viewer that pins to a remote session's active tab and cursor.
  This is the phone's whole story: watch the desktop run a document you cannot
  run here.

## Sequence

1. ~~**Machine identity and pairing**~~ **Built 2026-08-23**, and the short
   code added 2026-08-24 as `hick fleet pair` — the design's own ceremony,
   once the rendezvous problem turned out to be solvable without a server.
   `hick fleet invite` / `accept` remains for machines that cannot reach each
   other. The private half lives in a `0600` file rather than the platform
   keychain, which is weaker against a user-level compromise and is named as
   such rather than implied away.
2. ~~**The peer channel**~~ **Built 2026-08-24**, as QUIC-over-iroh rather
   than Noise-over-WebSocket. It carries the existing API gated per verb, and
   all three grants are enforced rather than `view`/`edit` only — the gate is
   one table, so leaving `execute` out of it would have meant leaving it
   *open*. Two things about the relay are decided rather than inherited: the
   default posture is number0's relays AND number0's address publishing
   (chosen 2026-08-24, because it is what makes a café work with no setup),
   and because neither is a server *we* run the sentence "nothing talks to a
   server we run" stays true while doing more work than it used to — so the
   product says on every connection whether it was direct or relayed, and
   `HICKORY_FLEET_RELAY` takes `direct` or your own relay.
3. **The fleet pane and attach**, with the machine strip.
4. **The `execute` grant**, an explicit act with its own refusal message, and
   the peer's executor shown beside it.
5. **Transport 2 and 3** — overlay address and SSH tunnel, behind the seam.
6. **Follow**, and the phone as a `view` member.

## Open edges

- **What happens to an attached session when the host machine sleeps.** The
  same question `local-collaboration.md` left open for a guest whose host
  vanished, with a better answer available: the durable copy is in git on the
  host, and the attaching machine can hold its unflushed room state and offer
  it back on reconnect. Not designed here.
- **Two machines, same repository, same branch, both editing.** Legal and
  unremarkable until both commit, at which point it is an ordinary git
  divergence — and `hick-merge` is registered as the merge driver on desktop.
  Whether the fleet pane should *warn* before you start typing on a branch a
  peer is also dirty on is a real question and probably yes.
- **The agent dock across machines is not obviously one conversation.** A turn
  run on the sealed box writes a session file on the sealed box. Attaching
  shows it live; hydrating it later requires the file, which arrives by git —
  so a conversation you drove from the laptop is not on the laptop until
  someone commits `sessions/`, which `hick init` gitignores on purpose. This
  is the sharpest collision in the design and `changes-not-commits.md` is
  where it gets picked up.
- **Nothing here phones home, and the fleet does not change that.** Two of your
  machines talking to each other is not telemetry, and no third party is
  involved in any of the three transports. Worth keeping stated, because
  "sharing mode" is exactly the phrase that makes a reader assume otherwise.
