# Crucible Requirements

Crucible is a cross-platform runtime that uses a **personality** approach to run Linux, macOS, and Windows programs on any host operating system. It can run as a runtime on a host OS or as a bare-metal operating system.

---

## Core Requirements

### CR-1: Hickory Docs Integration (Primary)

**Crucible must be designed from the outset to serve as an execution backend for Hickory docs.**

The Crucible runtime, when running on a host OS alongside Hickory docs, must be able to:

1. **Execute `hick:exec` elements** — Accept commands from Hickory docs and run them inside Crucible-managed environments, returning stdout/stderr for capture into document outputs.

2. **Support multiple guest operating systems** — Run Linux, macOS, and Windows programs. Hickory docs currently uses container2wasm for Linux-only execution; Crucible must extend this to macOS and Windows guests.

3. **Support headless execution** — Provide stdin/stdout/stderr I/O for non-interactive, scripted execution (the primary use case for Hickory docs pipelines).

4. **Support visible UI automation** (stretch) — When needed, support display output and UI automation (e.g., for GUI testing, screenshots, or interactive demos). This includes:
   - Virtual display for headless hosts
   - Optional forwarding to host display
   - Automation APIs (accessibility, input injection)

5. **Expose a programmatic execution API** — Hickory docs invokes execution via a `ContainerExecutor`-like abstraction. Crucible must provide an API (library, CLI, or IPC) that Hickory docs can call to:
   - Start an environment from an image/personality
   - Execute commands and capture output
   - Manage lifecycle (start, pause, resume, shutdown)

6. **Run on Linux, macOS, and Windows hosts** — The Crucible runtime must run on all major host operating systems so that Hickory docs users can use it regardless of their development machine.

### CR-2: Personality-Based Execution

Crucible uses **personality layers** (syscall translation, compatibility layers) to run foreign binaries. Each guest OS (Linux, macOS, Windows) has a personality that translates its system calls or APIs to the host (or Crucible OS kernel).

### CR-3: Dual Deployment Modes

- **Crucible runtime** — Runs on a host OS (Linux, macOS, Windows), providing personality-based execution. Hickory docs runs on the same host and invokes Crucible for `hick:exec`.
- **Crucible OS bare metal** — When Crucible is the operating system, personalities run natively. Hickory docs can run on Crucible OS and use Crucible for execution.

---

## Hickory Docs Context

Hickory docs (hick) processes `.hick` files that define:

- **Containers** — Isolated execution environments with capability-based security
- **`<hick:exec>`** — Commands to run inside containers; output is captured for file generation
- **Volumes** — Shared filesystem between containers
- **DAG ordering** — Execs are sequenced by document order and dependencies

Today, Hickory docs uses container2wasm (c2w) for execution: OCI Linux images are converted to WASM and run via wasmtime. Crucible would provide an alternative or complementary backend for multi-OS and UI automation scenarios.

---

## Non-Goals (Out of Scope for Core Requirements)

- Browser execution (unlike container2wasm’s WASM output)
- Replacing container2wasm for Linux-only use cases where WASM portability is preferred
