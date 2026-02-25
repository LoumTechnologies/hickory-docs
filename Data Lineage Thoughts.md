### Summary of Explored Ideas

Throughout our conversation, we've designed a secure system for running AI-generated code that processes both trusted (e.g., sensitive email addresses from internal docs) and untrusted/tainted (e.g., user summaries or web data) inputs, ensuring no tainted data leaks into trusted actions like email recipients. The core goal is "information-flow control" with oblivious computation—e.g., sending emails without the sender knowing the body contents unless sanitized. We drew inspiration from a white paper on a custom Python interpreter with data-flow analysis and evolved it into Rust/WASM-focused architectures, with extensions to other languages.

Key ideas include:
- **Taint Tracking with Phantom Types (Rust)**: Wrap data in zero-cost structs like `Trusted<T>` and `Tainted<T>`. The borrow checker enforces one-way flows (tainted can't become trusted without host sanitization). Extended to opaque wrappers for blinding.
- **WASM Sandboxing and FFI Control**: AI-generated Rust compiles to WASM, run in Wasmtime. Host controls imports/exports—e.g., `send_email` only accepts `Trusted<String>` for recipients. No syscalls or leaks outside whitelisted functions.
- **Cryptographic Lineage and DAGs**: Every value gets a hash-chained `LineageId` (using BLAKE3 + host secret salt). Host builds/maintains a DAG of flows; verifies paths from sources to sinks. Prevents forging clean lineages.
- **Module Separation**: Split into trusted (AI-generated from clean contexts, full access) and untrusted (tainted contexts, sandboxed) WASM modules. Host orchestrates, bridging via sanitized outputs.
- **Static Analysis and Slicing**: Use Miri for UB checks, custom lints for macro taints. Automatically slice programs at closures/async points into trusted/tainted runs; dual-execution with cfg flags mocks inputs to isolate flows.
- **Custom Interpreters**: Alternatives like taint-aware Python (runtime tracking) or WASM interpreters (memory tagging across languages). Less Rust-specific but more flexible for dynamic flows.
- **Lazy Flow Builders**: AI builds declarative plans (e.g., via Rust API with operator overloads, Polars/Ibis in Python) that output a verifiable DAG. Host checks before executing real data.
- **Provenance-Aware Codegen**: AI context taint determines generated code's "genesis" (trusted vs. tainted access). Extended to custom XML DSL with embedded code, full lineage, and XSLT for transformations/provenance.

These build a multi-layer pipeline: AI generates → static checks (compile/Miri/lints) → host verifies (lineage/DAG) → isolated execution (WASM splits) → audited outputs.

### Tradeoffs

- **Security vs. Performance/Complexity**: Phantom types and WASM add zero runtime cost but require careful API design (e.g., no exposed sanitizers). Custom interpreters or lineage hashing add 2-20x slowdown but catch dynamic leaks static tools miss. Slicing simplifies code but risks mis-slices (fails safe via panics/rejects).
- **Ease for AI vs. Restrictiveness**: Lazy builders (Polars/Ibis/XML) feel idiomatic but limit free-form code—AI must use APIs, not arbitrary strings. Full Rust allows expressive code but needs more checks (Miri/lints). Custom XML requires fine-tuning but offers perfect lineage.
- **Language Flexibility vs. Control**: Rust gives compile-time guarantees; Python/others easier for data ops but weaker safety. WASM unifies but slows non-Rust langs. XML/XSLT adds provenance but introduces parsing overhead.
- **Static vs. Runtime Verification**: Static (types/lints) prevents bad code from running; runtime (interpreters/lineage) audits subtle flows but risks side-channels (timing). Host-only execution (lazy plans) minimizes leaks but reduces AI autonomy.
- **Implementation Effort**: Basic Rust taints are quick; full DAG + interpreters are complex (fork Wasmtime). Polars/Ibis reuse existing libs; custom XML needs tuning but scales for declarative flows.
- **Failure Modes**: Systems prefer "loud fails" (compile errors, panics, rejects) over silent leaks—good for security, but could frustrate if AI generates invalid plans often.

Overall, higher security (e.g., 9.5-10/10) trades off with developer/AI usability; we leaned toward paranoia without killing dynamism.

### Core Set of Requirements

**Hard Requirements** (Non-negotiable for security):
- No tainted data flows to trusted sinks (e.g., email recipients must trace only to trusted sources; bodies can be tainted post-sanitization).
- Host controls all sensitive ops (e.g., FFI imports, sanitizers, sends)—AI can't overwrite or forge.
- Cryptographic verification (hashes + secret salts) for lineages to prevent faking clean provenance.
- Sandbox isolation (WASM or equivalent) with no direct access to OS/resources.
- Fail-safe on errors: Wrong flows must panic/reject/compile-fail, not proceed silently.
- Support oblivious modes: Code can't inspect/ log trusted payloads without explicit host approval.
- Multi-layer checks: Compile-time (types/borrow checker) + runtime (DAG validation) + static (Miri/lints).

**Soft Requirements** (Desirable, flexible based on tradeoffs):
- Idiomatic API for AI: Overloads/builders to mimic normal code (e.g., `a + b` builds DAG, not direct ops).
- Language agnosticism: Core in Rust/WASM, but extendable to Python (Polars/Ibis) or custom DSLs (XML with embeds).
- Performance tolerance: Accept 2-10x slowdown for interpreters/lineage if it boosts verification.
- Reusability: Generated code/plans should be auditable/post-mortem via logged DAGs.
- Fine-tuning friendliness: For custom formats (XML), support prompt engineering or small datasets.
- Scalability: Handle complex ops (e.g., casing detection, summaries) without manual taint annotations.
- Cross-tool integration: Compatible with existing libs (Polars for data frames, XSLT for XML provenance).