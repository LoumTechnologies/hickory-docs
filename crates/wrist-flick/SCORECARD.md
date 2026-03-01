# Waymark Framework Scorecard: wrist-flick

**App:** wrist-flick (AI assistant — conversations, tasks, memories, capabilities)
**Date:** 2026-02-28
**Evaluator:** _pending_

## Grades

| Grade | Meaning | Action |
|-------|---------|--------|
| **PASS** | Works correctly, meets expectations | None |
| **FRICTION** | Works but UX is rough (confusing API, slow, needs workaround) | Should fix |
| **BLOCKER** | Prevents completion or breaks the app | Must fix |
| **N/A** | Feature not applicable to this app | Skip |
| **SKIP** | Not tested (missing infrastructure, time, etc.) | Document reason |
| **FIXED** | Previously BLOCKER/FRICTION, now resolved | Verify with retest |
| **RETEST** | Fixed upstream, needs verification in this app | Re-run after fix |

---

## 1. Project Setup & Scaffolding

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 1.1 | `waymark new` | Generate a new project with `waymark new` | N/A | wrist-flick predates scaffolding; uses manual project structure |
| 1.2 | Generated project compiles | `cargo check` on freshly generated project | N/A | See 1.1 |
| 1.3 | Generated project runs | `cargo run` on freshly generated project | N/A | See 1.1 |
| 1.4 | `waymark generate entity` | Add a new entity to an existing project | N/A | Uses manual entity definitions via drift-signal-core |
| 1.5 | Hick feature flags | Scaffold with different `--features` combinations | N/A | See 1.1 |
| 1.6 | `hick generate-matrix` | All feature combinations pass verification | N/A | See 1.1 |

## 2. Entity Declaration (Derive Macros)

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 2.1 | `#[waymark(sub_app)]` | Declare a sub-app with the derive macro | N/A | Uses manual SubApp impl via drift-signal-core |
| 2.2 | `#[derive(CrdtEntity)]` | Declare a CRDT entity struct | N/A | Uses manual entity definitions |
| 2.3 | String field | LWW string register on an entity | SKIP | String fields used (title, description, content) but via manual impl |
| 2.4 | i64 field | LWW integer register | N/A | No integer fields in wrist-flick entities |
| 2.5 | f64 field | LWW float register | N/A | No float fields in wrist-flick entities |
| 2.6 | bool field | LWW boolean register | N/A | No boolean fields in wrist-flick entities |
| 2.7 | `Option<T>` field | Optional/nullable field | SKIP | Optional fields used (due_date, conversation_id) but via manual impl |
| 2.8 | Counter field | Monotonic counter CRDT | N/A | No Counter fields |
| 2.9 | PnCounter field | Bidirectional (positive-negative) counter | N/A | No PnCounter fields |
| 2.10 | GrowSet field | Grow-only set CRDT | N/A | No GrowSet fields |
| 2.11 | CrdtSet field | Add/remove set CRDT | N/A | No CrdtSet fields |
| 2.12 | CrdtList field | Ordered list CRDT | N/A | No CrdtList fields |
| 2.13 | CrdtMap field | Key-value map CRDT | N/A | No CrdtMap fields |
| 2.14 | StateMachine field | State machine with transition graph | N/A | Status is plain string ("open"/"done"), not typed state machine |
| 2.15 | RichText field | ProseMirror JSON rich text | N/A | No RichText fields |
| 2.16 | DateTime field | Date/time value | SKIP | Timestamps used but via manual impl |
| 2.17 | EntityId field | UUID v7 with entity prefix | SKIP | IDs used but via manual impl |
| 2.18 | BelongsTo field | Foreign key association | N/A | No BelongsTo associations |
| 2.19 | LinkTo field | Many-to-many association | N/A | No LinkTo associations |

## 3. Field Attributes

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 3.1 | `#[sync(required)]` | Mark a field as required | N/A | No derive macros used |
| 3.2 | `#[sync(searchable)]` | Enable FTS5 indexing on a field | N/A | FTS5 configured manually, not via attribute |
| 3.3 | `#[sync(immutable)]` | Prevent field from being modified after creation | N/A | No derive macros used |
| 3.4 | `#[sync(unique)]` | Enforce uniqueness across the collection | N/A | No derive macros used |
| 3.5 | `#[sync(unique, scope = "...")]` | Enforce uniqueness scoped to a parent | N/A | No derive macros used |
| 3.6 | `#[sync(encrypted)]` | Per-field selective E2EE | N/A | No encryption in wrist-flick |
| 3.7 | `#[sync(skip)]` | Exclude field from sync | N/A | No derive macros used |
| 3.8 | `#[sync(subdocument)]` | Nested subdocument sync | N/A | No derive macros used |
| 3.9 | `#[sync(cross_entity)]` | Cross-entity field constraints | N/A | No derive macros used |
| 3.10 | `#[sync(range = "...")]` | Numeric range constraint | N/A | No derive macros used |
| 3.11 | `#[sync(one_of = "...")]` | Enum/one-of constraint | N/A | No derive macros used |
| 3.12 | `#[sync(pattern = "...")]` | Regex pattern constraint | N/A | No derive macros used |
| 3.13 | `#[sync(min_length)]` | Minimum string length | N/A | No derive macros used |
| 3.14 | `#[sync(max_length)]` | Maximum string length | N/A | No derive macros used |
| 3.15 | `#[sync(hidden)]` | Hide from default UI rendering | N/A | No derive macros used |
| 3.16 | `#[sync(label = "...")]` | Custom display label for UI | N/A | No derive macros used |
| 3.17 | `#[sync(states = "...")]` | State machine transition definition | N/A | No derive macros used |

## 4. Entity-Level Attributes

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 4.1 | `#[entity(icon = "...")]` | Icon for UI rendering | N/A | No derive macros used |
| 4.2 | `#[entity(display = "...")]` | Display template for list views | N/A | No derive macros used |
| 4.3 | `#[entity(crud = [...])]` | Selective CRUD operations | N/A | No derive macros used |
| 4.4 | `#[entity(searchable = [...])]` | Entity-level FTS configuration | N/A | No derive macros used |

## 5. CRUD Operations

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 5.1 | Create entity | `insert()` a new entity | SKIP | CRUD via execute_command(), needs evaluation |
| 5.2 | Read entity | `get()` a single entity by ID | SKIP | Read via SQL projection, needs evaluation |
| 5.3 | List entities | `list()` entities in a collection | SKIP | List via SQL query, needs evaluation |
| 5.4 | Update entity | `mutate()` fields on an existing entity | SKIP | Update via execute_command(), needs evaluation |
| 5.5 | Delete entity | `delete()` an entity | SKIP | Delete via execute_command(), needs evaluation |
| 5.6 | SQL projection | Events project to SQL read model automatically | SKIP | SQL projections configured manually, needs evaluation |
| 5.7 | Projection rebuild | Rebuild SQL read model from event replay | SKIP | Needs evaluation |
| 5.8 | Full-text search | Query FTS5 index on searchable fields | SKIP | FTS5 used for memory search, needs evaluation |

## 6. Validation & Constraints

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 6.1 | Range validation | Reject values outside `#[sync(range)]` bounds | N/A | No range constraints |
| 6.2 | OneOf validation | Reject values not in `#[sync(one_of)]` set | N/A | No one-of constraints |
| 6.3 | Regex validation | Reject values not matching `#[sync(pattern)]` | N/A | No pattern constraints |
| 6.4 | State machine transitions | Reject invalid state transitions | N/A | No typed state machines |
| 6.5 | Required field enforcement | Reject null/empty on `#[sync(required)]` fields | N/A | No derive macros used |
| 6.6 | Immutable field enforcement | Reject mutation of `#[sync(immutable)]` fields | N/A | No immutable fields |
| 6.7 | Uniqueness tracking | Detect and report duplicate values | N/A | No uniqueness constraints |
| 6.8 | Scoped uniqueness | Detect duplicates within parent scope | N/A | No scoped uniqueness |
| 6.9 | Cross-entity invariants | Budget/capacity rules across entities | N/A | No cross-entity rules |
| 6.10 | Orphan detection | Detect entities whose parent was deleted | N/A | No parent-child hierarchy |
| 6.11 | Rules engine (Rete) | Deterministic violation resolution via Rete | N/A | No rules engine |
| 6.12 | Post-merge validation | Validate after CRDT merge, report violations | N/A | No validation rules configured |
| 6.13 | Resolution pipeline | Multi-violation deterministic resolution | N/A | No resolution pipeline |
| 6.14 | Custom `CommandValidator` | App-specific validation plugin | N/A | No custom validators |

## 7. Encryption & Cryptography

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 7.1 | Password key derivation | `with_password()` Argon2id key derivation | N/A | No encryption in wrist-flick |
| 7.2 | KEK/DEK hierarchy | Key encryption key wraps data encryption keys | N/A | No encryption |
| 7.3 | Selective encryption | `#[sync(encrypted)]` fields encrypted, others searchable | N/A | No encryption |
| 7.4 | Full E2EE | Entire entity encrypted as opaque blob | N/A | No encryption |
| 7.5 | BIP39 recovery mnemonic | 24-word backup phrase for key recovery | N/A | No encryption |
| 7.6 | Salt persistence | Salt file stored alongside database | N/A | No encryption |
| 7.7 | Database re-open with password | Close and re-open encrypted database | N/A | No encryption |
| 7.8 | Cross-device decryption | Same KEK decrypts on another device | N/A | No encryption |

## 8. Sync

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 8.1 | yrs state vector exchange | Diff-based CRDT sync via state vectors | SKIP | drift-signal-core uses yrs underneath, needs evaluation |
| 8.2 | P2P gossip (Layer 1) | Peer discovery and flood-fill dissemination | SKIP | P2P sync wired but not fully tested |
| 8.3 | P2P QUIC reconciliation (Layer 2) | 5-step QUIC sync protocol | SKIP | P2P sync wired but not fully tested |
| 8.4 | Server HTTP push | Push local changes to server | SKIP | Sync start/stop endpoints exist, needs evaluation |
| 8.5 | Server HTTP pull | Pull remote changes from server | SKIP | Sync start/stop endpoints exist, needs evaluation |
| 8.6 | Incremental sync | Only sync deltas since last sync | SKIP | Needs evaluation |
| 8.7 | Concurrent merge | Multiple simultaneous edits merge correctly | SKIP | Needs evaluation |
| 8.8 | Cross-sub-app sync | Each sub-app syncs independently | SKIP | 4 sub-apps registered, needs evaluation |
| 8.9 | Multi-document sync | SyncEngine manages multiple yrs Docs | SKIP | Needs evaluation |
| 8.10 | Three-way convergence | Client-client-server all converge | SKIP | Needs evaluation |
| 8.11 | SyncConfig | Configure relay URL, S3, budgets, intervals | SKIP | Relay URL configurable via CLI, needs evaluation |

## 9. Device & Identity

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 9.1 | Device linking (typestate) | Initiated -> Claimed -> Verified flow | SKIP | Link token endpoint exists, needs evaluation |
| 9.2 | KEK transfer over QUIC | Transfer encryption key to new device | N/A | No encryption |
| 9.3 | ShareSession | Per-document DEK exchange for sharing | N/A | No encryption |
| 9.4 | OIDC authentication | Authenticate via Apple/Google/custom OIDC | N/A | No OIDC integration |
| 9.5 | SyncToken | Time-limited JWT for sync sessions | SKIP | Needs evaluation |
| 9.6 | Biscuit capability tokens | Attenuation-based authorization | N/A | Not used |
| 9.7 | Tag-share tokens | `tsh4-` offline sharing with snapshot | N/A | Not used |
| 9.8 | Team management | Team boundaries, scoped shared data | N/A | Single-user app |
| 9.9 | Device management | Multi-device support, revocation | SKIP | Device ID generation exists, needs evaluation |
| 9.10 | Role-based access control | User/Team/Device hierarchy with roles | N/A | Single-user app |

## 10. Structured Messages & Documents

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 10.1 | XML structured messages | Send XML with semantic annotations | N/A | Uses custom action parsing (<mem:fact>, <task:create>) but not framework XML messages |
| 10.2 | Multi-namespace dispatch | Route XML commands to different sub-apps | N/A | No namespace dispatch |
| 10.3 | Chat-as-commit-log | All mutations flow through chat timeline | N/A | Chat is separate from entity mutations |
| 10.4 | Structured documents | Persistent yrs XmlFragment CRDT tree | N/A | No structured documents |
| 10.5 | Binary diff blobs | Incremental yrs diffs wrapped in BlobRef | N/A | No binary diffs |
| 10.6 | Document -> entity lifecycle | Documents create entities with independent streams | N/A | No document lifecycle |
| 10.7 | Per-entity event streams | Each entity has its own yrs Doc | SKIP | drift-signal-core manages event streams, needs evaluation |
| 10.8 | HTML allowlist sanitizer | Strip disallowed tags, prevent XSS | N/A | No HTML content |
| 10.9 | Rich threaded comments | Polymorphic targets, resolution tracking | N/A | No threaded comments |

## 11. Storage & Persistence

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 11.1 | SQLite read model | SQL tables per collection, auto-created | SKIP | SQLite used via drift-signal-core, needs evaluation |
| 11.2 | SQL migrations | Versioned, per-sub-app, auto-applied | SKIP | Needs evaluation |
| 11.3 | Full-text search (FTS5) | Query across searchable fields | SKIP | FTS5 used for memory search, needs evaluation |
| 11.4 | Event store | Immutable append-only yrs XmlFragment | SKIP | Event sourcing via drift-signal-core, needs evaluation |
| 11.5 | Ownership tracking | `_entity_ownership` metadata table | SKIP | Needs evaluation |
| 11.6 | Delete policies | Cascade/Orphan/Prevent per collection | N/A | No parent-child cascade configured |
| 11.7 | Retention policy | Max age + max event count, auto-pruning | N/A | No retention policies configured |
| 11.8 | Blob storage (BLAKE3) | Content-addressed blob management | N/A | No blob storage |
| 11.9 | Blob cache (LRU) | Eviction + pin/unpin, configurable budget | N/A | No blob cache |
| 11.10 | Content-addressed attachments | viking-block + prolly tree file storage | N/A | No attachments |

## 12. Cross-Platform & Code Generation

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 12.1 | UniFFI (Swift) | Generate Swift bindings | N/A | REST API app, no native bindings |
| 12.2 | UniFFI (Kotlin) | Generate Kotlin bindings | N/A | REST API app |
| 12.3 | UniFFI (C#) | Generate C# bindings | N/A | REST API app |
| 12.4 | WASM target | Compile to wasm32-unknown-unknown | N/A | Server-side app |
| 12.5 | Web app scaffolding | Vite + React + TypeScript + Tailwind | N/A | REST API only (has web/ dir but no frontend framework) |
| 12.6 | iOS app scaffolding | SwiftUI + Combine | N/A | Server-side app |
| 12.7 | Android app scaffolding | Jetpack Compose + Kotlin | N/A | Server-side app |
| 12.8 | Metadata JSON export | `SyncApp::export_metadata_json()` | N/A | No metadata export configured |
| 12.9 | Metadata dump CLI | `dump-metadata` binary | N/A | No metadata dump binary |
| 12.10 | Metadata-driven codegen | metadata.json -> platform-specific DTOs | N/A | No metadata-driven codegen |
| 12.11 | Pre-commit metadata hook | Auto-regenerate metadata.json | N/A | No pre-commit hook |

## 13. Developer Experience

| # | Feature | Interaction | Grade | Notes |
|---|---------|-------------|-------|-------|
| 13.1 | Compile time | How long does a clean build take? | SKIP | Not benchmarked |
| 13.2 | Incremental compile time | How long does a change-and-rebuild take? | SKIP | Not benchmarked |
| 13.3 | Test suite | `cargo test` passes with no failures | SKIP | Integration tests exist, needs evaluation |
| 13.4 | Error messages | Are compile/runtime errors actionable? | SKIP | Not evaluated |
| 13.5 | API discoverability | Can you find the right API without reading source? | SKIP | Not evaluated |
| 13.6 | Documentation | Are the docs sufficient to build with? | SKIP | Not evaluated |
| 13.7 | Boilerplate | How much ceremony to add a new entity/sub-app? | SKIP | Manual SubApp impl is more verbose than derive macro |
| 13.8 | Defaults | Do sensible defaults reduce configuration? | SKIP | Not evaluated |

---

## Bugs Found

_No bugs documented yet. Run the scorecard evaluation to populate._

---

## Summary

| Grade | Count | Items |
|-------|------:|-------|
| PASS | 0 | |
| FRICTION | 0 | |
| BLOCKER | 0 | |
| N/A | 95 | 1.1-1.6, 2.1-2.2, 2.4-2.6, 2.8-2.15, 2.18-2.19, 3.1-3.17, 4.1-4.4, 6.1-6.14, 7.1-7.8, 9.2-9.4, 9.6-9.8, 9.10, 10.1-10.6, 10.8-10.9, 11.6-11.10, 12.1-12.11 |
| SKIP | 40 | 2.3, 2.7, 2.16-2.17, 5.1-5.8, 8.1-8.11, 9.1, 9.5, 9.9, 10.7, 11.1-11.5, 13.1-13.8 |
| FIXED | 0 | |
| RETEST | 0 | |
| **Total** | **135** | |

### Coverage

- **0% PASS** (0/40 applicable) — no items evaluated yet
- **70% N/A** (95/135) — wrist-flick uses manual SubApp impl, not derive macros
- **30% SKIP** (40/135) — applicable items awaiting evaluation

### Profile

wrist-flick is a **REST API wrapper** around `drift-signal-core` (the older manual
SubApp API). It demonstrates event sourcing, SQL projections, and P2P sync
infrastructure, but does not use the derive macro system, encryption, state
machines, constraints, or most of the advanced framework features. The large N/A
count reflects the intentional use of the lower-level API rather than a framework
gap.
