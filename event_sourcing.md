---
name: Event Sourcing Architecture
trigger: auto
description: Event sourcing architectural framework description
---

# Event Sourcing Architecture

Sliding scale between heavy servers/thin clients and thin servers/heavy clients. All domain changes are event-driven.

**Benefits:** Reduce cloud costs, maximize offline functionality, minimize frontend code duplication.

## Core Design

- Event sourcing for all domain objects
- E2E encryption for sensitive entities
- Server only for: auth, ferrying/storing encrypted data, push notifications
- Clients use SQLite for local data subset
- File contents separate from event stream (still encrypted)

## File Uploads

Command event requests signed URL → upload happens async → out of band from events. Works with any object storage.

## Snapshots

Events organized by aggregate root. Snapshot events compress history for new clients. Unencrypted `isSnapshot` flag lets server discard older events (optionally keep for audit).

## Conflict Resolution

Each event has `previousEventId`. Concurrent offline edits produce events with same previous ID → resolve via CRDT (default: last-write-wins). Some entities surface conflicts in UI for manual resolution. Events resolving conflicts have multiple previous IDs.

## Branching

Optional `branch` property enables dry runs, review/approve workflows, merge operations. Merge events are unencrypted (server needs to know which events to send).

## Local Storage

SQLite for data, device filesystem for cached files. Assume users have more files than fit on device—cloud storage with local cache. Files always encrypted before upload.

## Bulk Import/Export

Parquet files for admin bulk operations on all entities.

## Server Responsibilities

Server ferries/stores encrypted data between devices and users (based on org membership). Provides events since last snapshot by entity ID.

## End-to-End Encryption

**Password = Private Key:** Client decrypts server's challenge (encrypted with user's public key) to prove identity. Key never leaves device.

**New Device Login:** Enter password; save for long period. Re-auth via passwordless (email/SMS links).

**User ID ↔ Public Key:** Mapped but not identical (allows key rotation).

### Device-to-Device Sync

Same-network devices communicate directly:
1. Exchange public keys
2. Verify identity via challenge/response
3. Send events encrypted with recipient's public key

### Partial Encryption

Not all entities E2E encrypted—allows password reset with partial data loss. E2E only for very sensitive data where loss on password reset is acceptable.

## Event Properties

| Property | Notes |
|----------|-------|
| Timestamp | |
| Site ID | Part of event ID (with Lamport timestamp) |
| Author IDs | User(s) who authored; if LLM-generated: [human, LLM] |
| Lamport Timestamp | Sequence number |
| Previous Event ID | For conflict detection |
| Branch | Optional |
| Is Snapshot | Boolean |
