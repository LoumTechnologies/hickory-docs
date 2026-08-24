# Third-party libraries and tools that would need porting to C#

An inventory of every non-first-party dependency Hickory Docs relies on, and
what a C# port would have to do with it. "Port" here means one of four things:

| Verdict | Meaning |
|---|---|
| **Swap** | A mature .NET equivalent exists; mechanical substitution. |
| **Adapt** | A .NET equivalent exists but differs enough to change design or lose a feature. |
| **Port** | No usable .NET library; the functionality must be written or a Rust/native library must be bound via P/Invoke. |
| **Keep native** | Not portable and not worth porting; keep the existing artifact and call it out of process. |

Scope: the 34 workspace crates (`Cargo.toml`), the Tauri shells under `apps/`,
and the frontend in `apps/web`. The frontend is React/TypeScript and survives a
backend port untouched — it is listed only where a dependency is really a
backend contract in disguise.

---

## 1. The load-bearing problems (port these first, or don't port at all)

These five decide whether the port is feasible. Everything in section 2 is
routine by comparison.

### `iroh` — the peer channel (`hickory-peer`, `hickory-cli`)

QUIC dialled by ed25519 public key, where the endpoint identity *is* the key.
`one-engineer-many-machines.md` builds the whole fleet allowlist on that
property: the public-key list is the ACL directly, with no extra handshake.

There is no .NET equivalent. `System.Net.Quic` (.NET 7+, over MsQuic) gives you
QUIC but not key-addressed endpoints, not hole punching, not relay fallback,
and not number0's address publishing. Options, worst to best:

- Rebuild key-addressed QUIC on `System.Net.Quic` + a custom TLS certificate
  scheme + your own NAT traversal. This is a project, not a task.
- P/Invoke a Rust `cdylib` wrapping `iroh` — keeps the semantics exactly,
  which matters because the fleet grants and the deny-by-default path table
  are written against them.
- Drop liveness sharing and let git carry everything. This deletes a
  shipped feature.

Note the pin: `iroh` is `default-features = false, features = ["tls-ring"]`
because the default `portmapper` feature pulls MPL-2.0 `attohttpc`. Any port
inherits that licence constraint — .NET P2P/UPnP packages need the same audit.

### `yrs` / Yjs CRDTs (`hick-grove`, `hickory-collab`, `apps/web`)

The Rust `yrs` and the browser's `yjs` are two ends of one wire protocol, so
this is not a library swap — it is a protocol compatibility requirement. The
frontend keeps `yjs`, `y-protocols`, `y-codemirror.next`, and `lib0`; the
backend must speak the same update format.

`Ycs` is a C# port of Yjs and is the only real candidate. It is
community-maintained and lags upstream `yrs`. Verify update-format and
state-vector compatibility against the exact `yjs` version in
`apps/web/package.json` before committing — a subtle encoding mismatch here
corrupts documents rather than failing loudly. The alternative is again
P/Invoke to `yrs`, which is what the compatibility requirement argues for.

**Verdict: Adapt (with a real risk of Port).**

### `tree-sitter` + grammars (`hick-structure`)

Used with the Rust, Python, JavaScript, and TypeScript grammars for structural
parsing. `tree-sitter` is a C library, so this is the friendliest of the hard
cases: bind the C API from C# directly. `TreeSitterSharp` and similar bindings
exist but are thin and partially maintained; expect to own the binding.
Grammars stay as compiled C — nothing to port there, but each grammar becomes a
native asset that must be built per target platform, which complicates the
"download runs with no toolchain" promise in
`.instructions/continuous-delivery-downloadable.md`.

**Verdict: Adapt — bind the C library, own the P/Invoke layer.**

### `portable-pty` + `vt100` (`hick-term`)

Terminal capture: spawn a process under a pty, parse the escape-sequence stream
into a screen. Two separate problems.

- pty: `Pty.Net` (from the VS Code / .NET tooling world) covers Windows
  ConPTY, macOS, and Linux. **Swap, with care** — its Windows behaviour is the
  part to test.
- VT parsing: there is no maintained C# equivalent of `vt100`. The nearest
  things are inside terminal *applications*, not reusable libraries. This is a
  genuine **Port** — a few thousand lines of state machine, and the transcript
  format's fidelity depends on getting it right.

### Tauri v2 (`apps/desktop`, `apps/web` as its UI)

Not a library so much as the whole shipping story: one React/TS package, three
entry points, desktop `.dmg`/`AppImage`/`.msi`, plus iOS and Android shells.

.NET has no equivalent that preserves all of it. The candidates:

- **Photino** — closest in spirit (native webview, tiny, .NET host). No mobile.
- **Avalonia** — real cross-platform including iOS/Android, but it is not a
  webview shell; the React frontend would have to be abandoned or embedded in
  `WebView` controls, which is not how Avalonia wants to be used.
- **.NET MAUI** — has the mobile story and a `BlazorWebView`/`WebView`, but the
  desktop Linux target does not exist, and `AppImage` is an advertised target.

There is no clean answer. Whichever is chosen, `tauri-plugin-dialog`,
`tauri-build`, and `rust-embed` (embedding the built frontend in the binary)
all get rewritten against the new shell's conventions.

**Verdict: Port — this is the largest single item in the whole inventory.**

---

## 2. Everything else, by crate

### Core language, documents, and weave

| Rust crate | Used by | C# path | Verdict |
|---|---|---|---|
| `quick-xml` | `hick-grove` | `System.Xml.XmlReader` — but see the caveat below | Adapt |
| `roxmltree` | `hick-xml` | `System.Xml.Linq` / `XmlReader` | Adapt |
| `pulldown-cmark` | `hick-literate` | `Markdig` | Swap |
| `serde` / `serde_json` | everywhere | `System.Text.Json` | Swap |
| `serde_yaml` | frontmatter | `YamlDotNet` | Swap |
| `indexmap` | ordered maps | `OrderedDictionary` (.NET 9) or `List` + `Dictionary` | Swap |
| `similar` | `hick-merge`, `hickory-cli` | `DiffPlex` | Adapt — `similar`'s three-way merge and Patience/Myers options are richer than `DiffPlex`; the merge driver in `hick-merge` depends on the three-way behaviour, so expect to write that half. |
| `regex` | many | `System.Text.RegularExpressions` | Adapt — .NET regex is backtracking and can blow up where Rust's linear-time engine cannot. Audit every pattern that touches user documents. |

**XML caveat, and it is the important one.** The hick parser's no-escaping
invariant (`AGENTS.md`) says only namespace-prefixed tags are structured and
everything else is raw, byte-for-byte — no CDATA, no entity escaping. That is
not what any conformant XML reader does. The existing crates are used as
tokenizers under a hand-written parser, not as document parsers, so the port is
of *our* parser; `System.Xml` is at best a lexer underneath it, and more likely
the port hand-rolls the scanner. Do not let "C# has XML built in" hide this.

### Execution and the executor trait

| Rust crate | Used by | C# path | Verdict |
|---|---|---|---|
| `tokio`, `tokio-stream`, `tokio-util`, `futures`, `async-stream` | everywhere | `Task` / `IAsyncEnumerable` / `System.Threading.Channels` | Swap — idiomatically different, mechanically fine |
| `async-trait` | traits | native in C# | Swap (delete it) |
| `tonic`, `prost`, `tonic-build`, `protoc-bin-vendored` | `hickory-executor-canopy` | `Grpc.Net.Client` + `Google.Protobuf` + `Grpc.Tools` | Swap — the most comfortable substitution in this document |
| `libc`, `windows` | platform calls | `System.Runtime.InteropServices` + P/Invoke | Adapt |
| `xattr` | `hickory-cli` | no .NET library; P/Invoke `setxattr`/`getxattr` per platform | Port (small) |
| `fs2` (file locking) | `hickory-cli` | `FileStream.Lock` / `FileShare` — semantics differ from `flock` | Adapt |
| `sysinfo` | process/system info | `System.Diagnostics.Process` covers some; cross-platform memory/CPU does not have one good package | Adapt |
| `notify` (fs watching) | `hick-literate`, `hickory-cli` | `FileSystemWatcher` | Adapt — `FileSystemWatcher` is notoriously unreliable on macOS and network paths; `notify` papers over exactly those differences. Budget debugging time. |
| `tempfile` | tests, exec | `Path.GetTempPath` + own cleanup | Port (trivial, but `tempfile`'s delete-on-drop has no direct analogue) |
| `walkdir` | traversal | `Directory.EnumerateFiles` | Swap |
| `ignore` | `hickory-cli`, `hick-search` | **no .NET equivalent** — gitignore semantics (nested files, negation, precedence) are subtle and the ingest gitignore filter depends on them | Port |
| `glob` | patterns | `Microsoft.Extensions.FileSystemGlobbing` | Swap |
| `tar`, `zip` | archives | `System.Formats.Tar`, `System.IO.Compression` | Swap |
| `mime_guess` | content types | `Microsoft.AspNetCore.StaticFiles.FileExtensionContentTypeProvider` | Swap |

### Server and HTTP

| Rust crate | C# path | Verdict |
|---|---|---|
| `axum`, `tower`, `tower-http`, `hyper-util` | ASP.NET Core minimal APIs + middleware | Swap |
| `reqwest` | `HttpClient` | Swap |
| `tokio-tungstenite` | ASP.NET Core WebSockets / `ClientWebSocket` | Swap |
| `bytes` | `Memory<byte>` / `ReadOnlySequence<byte>` | Swap |
| `wiremock` (dev) | `WireMock.Net` | Swap |

Worth stating once: this is a local server bound to the user's machine, not a
service. ASP.NET Core's default templates assume the opposite — hosting model,
logging sinks, and the health-check/telemetry defaults all have to be turned
off to keep the "nothing phones home" rule in
`.instructions/continuous-delivery-downloadable.md`.

### Crypto, secrets, identity

| Rust crate | Used by | C# path | Verdict |
|---|---|---|---|
| `ed25519-dalek` | `hickory-fleet`, `hickory-peer` | `System.Security.Cryptography` (Ed25519 arrived in .NET 10; before that, BouncyCastle or NSec) | Adapt |
| `argon2` | `hickory-peer` | `Konscious.Security.Cryptography.Argon2` or `libsodium` binding | Adapt — verify parameter compatibility, or existing invitations stop verifying |
| `age` | `hick-secrets` | **no maintained .NET implementation of the age format** | Port — or shell out to the `age` binary, which contradicts "no third-party CLIs" |
| `macaroon` | `hick-token` | `Macaroons.Net` exists and is effectively unmaintained | Port |
| `sha2`, `hex`, `base64` | everywhere | `SHA256`, `Convert.ToHexString`, `Convert.ToBase64String` | Swap |
| `rand`, `rand_core`, `getrandom` | everywhere | `RandomNumberGenerator` | Swap |
| `secrecy`, `rpassword` | secret handling | `SecureString` is discouraged on non-Windows; hand-roll zeroing + `Console.ReadKey` | Port (small) |
| `dashmap` | concurrent maps | `ConcurrentDictionary` | Swap |

`age` and `macaroon` are the two that force a decision rather than a
substitution. Both are format specifications with test vectors, so a port is
verifiable — but a hand-rolled crypto format is exactly the kind of thing that
should be bound to a reference implementation rather than reimplemented.

### Search and classification

| Rust crate | Used by | C# path | Verdict |
|---|---|---|---|
| `model2vec-rs` | `hick-search` | no .NET port; see below | Port |
| `arrow` 54 | `hick-classify` (optional) | `Apache.Arrow` (official, Apache-2.0) | Swap |
| `parquet` | `hick-classify` (optional) | `Parquet.Net` (MIT) | Swap |
| `blake3` | `hick-classify` (optional) | `Blake3.NET` binding | Adapt |

`model2vec-rs` runs static embeddings locally with no network, which is the
whole point of it under the local-only rule. It is *static* embeddings — the
model is distilled to a lookup table, so there is no transformer forward pass
and no need for ONNX Runtime. What has to be ported is the safetensors load,
the HuggingFace tokenizer path (the crate is pinned to the `fancy-regex`
pre-tokenizer flavour, and a pre-tokenizer mismatch silently changes every
vector), and model2vec's own pooling and token weighting.

**`host-mask` is not a dependency.** It is a *feature flag* on `hick-classify`
(`host-mask = ["arrow", "parquet", "blake3"]`) gating a first-party module.
An earlier revision of this document listed it as a library to port; it is
not, and there is nothing to port. The three crates above are what the
feature actually pulls in, and all three are ordinary substitutions. Any
future pass over `Cargo.toml` files should read `[features]` and
`[dependencies.x]` sections deliberately — a naive scan conflates all three.

### Storage

| Rust crate | Used by | C# path | Verdict |
|---|---|---|---|
| `aws-config`, `aws-sdk-s3` | `hick-store` | `AWSSDK.S3` | Swap |
| `hick-grove`'s `sqlite-example` feature | dev/example only | `Microsoft.Data.Sqlite` | Swap |

### Editor protocol support

| Rust crate | Used by | C# path | Verdict |
|---|---|---|---|
| `tower-lsp` | `hick-lsp` | `OmniSharp.Extensions.LanguageServer` | Swap |
| (DAP, hand-rolled) | `hick-dap` | `OmniSharp.Extensions.DebugAdapter` | Swap |

Both OmniSharp packages are mature and are what the C# tooling ecosystem
itself uses. This section is the easiest win in the port.

### Diagnostics, CLI, misc

| Rust crate | C# path | Verdict |
|---|---|---|
| `clap` | `System.CommandLine` | Adapt — `System.CommandLine`'s API has churned; pin a version |
| `anyhow`, `thiserror` | exceptions + custom types | Adapt — the error-message quality bar in `.instructions/user-facing-errors.md` is easier to hit with typed errors than with exception messages; do not let this regress |
| `tracing`, `tracing-subscriber`, `log`, `env_logger` | `Microsoft.Extensions.Logging` | Swap |
| `chrono` | `DateTimeOffset` / `TimeProvider` | Swap |
| `uuid` | `System.Guid` | Adapt — `Guid` is UUIDv4; UUIDv7 needs .NET 9's `Guid.CreateVersion7` |
| `dirs` | `Environment.GetFolderPath` + XDG handling by hand | Adapt — .NET does not implement the XDG base-directory spec on Linux |
| `envconfig` (per `config-and-environments`) | `Microsoft.Extensions.Configuration` + options binding with validation | Swap |

---

## 3. Frontend and build tooling (mostly unaffected)

`apps/web` is React + TypeScript + Vite and does not care what the backend is
written in. Two exceptions are backend contracts:

- **`openapi-typescript`** — generates the API client from the server's OpenAPI
  spec. ASP.NET Core emits OpenAPI natively (`Microsoft.AspNetCore.OpenApi`),
  so this keeps working; the spec's *shape* changes, which regenerates the
  client. Per `.instructions/implement-dev-environment`, this client is
  generated and never hand-merged — keep it that way through the port.
- **`yjs` / `y-protocols` / `y-codemirror.next` / `lib0`** — the other end of
  the `yrs` problem above.

The rest — CodeMirror 6 and its language packs, `@xterm/xterm`, `katex`,
`mermaid`, `vitest`, `jsdom`, `@testing-library/react` — is untouched.

Build tooling that changes: `cargo` → `dotnet`, `cargo-nextest`/`cargo test` →
`dotnet test`, `rust-toolchain.toml` → `global.json`. `just` stays as the only
task-runner entry point (`.instructions/just.md`), and the recipes' *contents*
change while their names do not. `flake.nix` needs .NET SDK inputs instead of
the Rust toolchain.

---

## 4. Licence check the port must repeat

`AGENTS.md` says MIT only, no copyleft linked into the product. The .NET
ecosystem is mostly MIT/Apache-2.0 and this is easier than in Rust, but three
things need looking at rather than assuming:

- **`Macaroons.Net`** and other small unmaintained packages — check the actual
  package licence, not the repo's README.
- **`Pty.Net`** and native pty helpers — some ship GPL'd native components.
- Anything wrapping **libsodium** or other native crypto — the wrapper's
  licence and the native library's licence are two separate answers, the same
  way `grit-lib` (MIT) and `grit-cli` (GPL-2.0) are.

Note that this workspace's own `[workspace.package]` declares
`license = "GPL-3.0-or-later"` — that is the licence *we* publish under and is
unaffected; the MIT-only rule is about what gets linked *in*.

---

## 5. Summary — where the work actually is

**Must be written from scratch or bound natively (6):**
`iroh` (key-addressed QUIC), `vt100` (VT parser), `ignore` (gitignore
semantics), `age` (encryption format), `macaroon` (token format),
`model2vec-rs` (static embeddings). Plus `tauri` as an entire shipping layer.

All six have narrower consumed surfaces than the crate name suggests — `ignore`
is used only for `WalkBuilder`, `model2vec-rs` only for `StaticModel`, and
`age` only for X25519 identities and decryption. Port the surface, not the
upstream crate.

**Substitutable but with a design or fidelity cost (~12):**
`yrs`/Yjs wire compatibility, `tree-sitter` bindings, `similar`'s three-way
merge, `regex` engine semantics, `notify`, `fs2` locking, `argon2`
parameters, `dirs`/XDG, `clap`, error modelling, `sysinfo`, `xattr`.

**Mechanical (the rest, ~48):**
web stack, serialization, hashing, archives, gRPC, S3, LSP/DAP, logging,
async — all have first-class .NET answers.

The honest reading: the mechanical majority is a large but boring job, and the
port's real cost is concentrated in `iroh`, the Yjs wire format, the VT parser,
and the Tauri replacement. The first three all have the same escape hatch —
keep the Rust crate as a native library behind P/Invoke — which turns "port to
C#" into "rewrite the application layer in C# over a Rust core". Whether that
counts as the port that was asked for is a decision, not a detail.
