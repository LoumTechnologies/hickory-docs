---
name: Structured Messages and Documents
trigger: auto
description: Description of the structured messages and structured documents architectural pattern
---

# Structured Messages and Documents

## Structured Messages

Messages with embedded semantic data (like structured logging). XML annotations link phrases to their meaning:

```xml
<message xmlns:task="https://example.com/vocab/task#"
         xmlns:user="https://example.com/vocab/user#">
  Hey <user:mention ref="joe-1">Joe</user:mention>,
  <task:create id="task-1" description="Change oil in the E-350">
    created and <task:assign to="joe-1">assigned this to you</task:assign>.
  </task:create>
  Can you get it done by <task:due iso="2025-01-15T17:00:00Z">tomorrow COB</task:due>?
</message>
```

- App receives message → updates DB → shows notification with message text (no separate notification)
- Limited HTML subset allowed (`<em>`, etc.)
- Binary blobs serialized alongside XML (for files, large data)
- Conversational UI: chat window shows changes; "Send" commits them

## Structured Documents

Persistent XML documents modified via structured message events. A service diffs document XML and emits entity change events.

**Key behaviors:**
- Documents can create/reference entities (which have independent event streams)
- When document deleted, created entities aren't auto-deleted (UI should prompt)
- XML schema is application-level (could be HTML subset, spreadsheet, word processor)

### Modifying Documents

Structured message = commit message (human text) + diff (binary blob):

```xml
<message xmlns:doc="https://example.com/vocab/doc#">
  <doc:update ref="project-plan-1">
    Updated the timeline to reflect the new deadline.
    <doc:diff encoding="binary" ref="blob-1"/>
  </doc:update>
</message>
```

### Binary Blobs for Diffs

Diffs stored as binary blobs referenced by XML because:
1. Diffs are large/complex; binary compresses well
2. Tree diffs don't interleave naturally with prose
3. Different diff algorithms produce different structures

### Conflict Handling

Framework must support conflict detection, resolution, and location (server/device/collaborative). Spec does NOT prescribe specific algorithms—use OT, CRDTs, three-way merge, or manual resolution.

**Example: Yjs XML CRDT**
1. Document stored as Yjs `XmlFragment`/`XmlElement`
2. Local edits captured as Yjs operations
3. Yjs update (binary) attached as blob in structured message
4. Remote changes applied via Yjs merge (concurrent edits merge cleanly)
5. Human-readable message text forms change history (like git log)
