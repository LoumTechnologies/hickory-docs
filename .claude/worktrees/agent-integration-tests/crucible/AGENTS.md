# Crucible — Agent Instructions

Crucible is a cross-platform runtime that uses personality layers to run Linux, macOS, and Windows programs on any host OS. When assisting with Crucible development, keep the following in mind.

---

## Core Requirement: Hickory Docs Integration

**Hickory docs integration is a primary design requirement for Crucible.** All architectural decisions should consider whether they support or hinder this use case.

### What This Means

1. **Execution backend** — Crucible must be usable as an execution backend for Hickory docs `hick:exec` elements. Hickory docs invokes a `ContainerExecutor`-like abstraction; Crucible must expose an API that can fulfill this role.

2. **Multi-OS guests** — Crucible must support Linux, macOS, and Windows guests. Hickory docs currently uses container2wasm for Linux only; Crucible extends this to macOS and Windows.

3. **Headless first** — The primary Hickory docs use case is headless execution: stdin/stdout/stderr I/O, no display. Crucible’s core API must support this. Visible UI automation is a stretch goal.

4. **Programmatic API** — Crucible must provide a programmatic execution API (library, CLI, or IPC) so Hickory docs can:
   - Start environments from images/personalities
   - Execute commands and capture output
   - Manage lifecycle (start, pause, resume, shutdown)

5. **Host portability** — Crucible must run on Linux, macOS, and Windows hosts so Hickory docs users can use it on any development machine.

### Design Implications

When designing or implementing Crucible:

- **Avoid** APIs that assume interactive use only; ensure headless, scripted execution is first-class.
- **Consider** how Hickory docs would discover and invoke Crucible (e.g., configurable backend, `--executor=crucible`).
- **Preserve** the ability to capture stdout/stderr and return exit codes — Hickory docs needs these for transcript rendering.
- **Document** integration points so Hickory docs maintainers can add a Crucible backend when ready.

---

## Reference

See [requirements.md](./requirements.md) for the full requirements document.
