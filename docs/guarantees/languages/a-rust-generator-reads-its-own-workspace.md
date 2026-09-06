# A Rust Generator Reads Its Own Workspace

Given a Cargo workspace, when `hick-model-rust` is installed, then a **Rust**
program in that workspace can query **Rust's own type system** — packages,
modules, items, `impl` blocks, derives, `cfg` gates, spans — and generate
source from the answers, without a nightly toolchain, without rustdoc JSON,
and without the workspace compiling first.

The model is **read only**. A generator emits text; the document owns the
bytes through `hick:file`, where lineage and the drift gate already work.

Everything the model could not bind is **named**. A lexical resolver follows
`mod` and `use`; it does not typecheck. So every crate outside the workspace
appears in `unresolved` with its name and an example path, and `cfg` is
recorded on every item it gates — including items whose own attributes say
nothing, because an item inside `#[cfg(test)] mod tests` is conditional and a
generator emitting it into a release build would be wrong.

`hick lang` reports Rust as **GOLD** by measurement: the MODEL column is
`code_model::discover("rust", root)` finding a binary, and the CLIENT column
is `typed_client::emit::Target::parse("rust")` returning one. Neither is a
row in a table.

## Why not rustdoc JSON

It is nightly-only and it requires the crate to compile. A model server is
wanted most while a workspace is half-written — that is the moment somebody
reaches for a generator — and one that answers only after a green build is a
tool you cannot use when you want it. The cost of `syn` instead is stated
rather than hidden, and it is the next section.

## What this does not promise

**There is no `references` query, and there will not be one.** The other three
servers have a binder — Go's `Info.Uses`, the TypeScript checker, Roslyn's
semantic model. This one does not. A references answer built on a parser
would be an identifier match: `parse` in two crates as one symbol, a call
through a trait object missed entirely, a shadowed local reported as a use of
the type. That is the confidently-wrong answer `unresolved` exists to forbid,
so it is refused by name. rust-analyzer emits SCIP itself, which is a real
binder's reference graph; that is the tool for the job, and `hick index list`
already says so.

**Macros are named, not expanded.** A `macro_rules!` definition is reported by
name and by its arms' patterns. What it expands to is not modelled, because
modelling it would be inventing.

**`cfg` is followed, never evaluated.** There is no feature resolution and no
target triple here, so `#[cfg(unix)] mod imp;` is read and recorded rather
than judged. The caller knows what it is generating for; the server does not.

---

Last LLM verification:
- Date: 2026-09-06
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `code-models/rust/` — `hick-model-rust`, its own Cargo workspace, built
    with `hick code-model rust --install`. `load.rs` enumerates with
    `cargo metadata --no-deps` and walks the module tree from each crate
    root; `lower.rs` turns `syn` items into the schema; `schema.rs` is the
    seven-field `Query`; `model.rs` is the schema itself, ten item kinds
    behind one GraphQL interface.
  - Run against `pilockdb`, a 224k-line workspace of 17 packages: 398 files,
    569 modules, ~0.9 s to load, sub-millisecond per query after that, and
    58 `unresolved` entries — **every one of them an external crate, named**.
  - `crates/hickory-cli/src/typed_client/emit_rust.rs` emits serde structs
    plus the query text; `code-models/clients/rust/hick_model.rs` is the
    transport, `include_str!`'d by `emit::Target::Rust::runtime`.
  - `crates/hickory-cli/src/code_model/install.rs` builds a server confined
    and copies it into `.hick-cache/models`, and its tests hold the catalogue
    to the name discovery looks for.
  - End to end: `HICKORY_EXECUTOR=local hick run examples/rust-generator.hick`
    writes `examples/rust-generator/catalogue/src/describe.rs` from a Rust
    generator's stdout and then compiles and runs the workspace with it,
    against a byte-exact `hick:expect`. `hick test examples/rust-generator.hick`
    passes.
- Test coverage:
  - `code-models/rust/src/main.rs::tests` — a derive is findable and a tuple
    struct keeps its shape and its index-named fields; an `impl` block says
    which trait it is for; a `.rs` file no `mod` declares is **not** part of
    the crate and an inherited `cfg` is on the item; an external crate is
    named in `unresolved`; a malformed request is answered rather than fatal.
  - `code-models/rust/src/syntax.rs::tests` — a type reads back as Rust and
    not as tokens (that text is copied into generated source), a single colon
    keeps its space where `::` does not, a `serde` argument with a comma
    inside a string is one argument, a doc comment is prose.
  - `crates/hickory-cli/src/typed_client/emit_rust.rs::tests` — a keyword
    field is a raw identifier keeping its wire name, a `camelCase` field is
    renamed rather than silently missing, `__typename` inside a variant
    defaults rather than failing.
  - `crates/hickory-cli/src/code_model/install.rs::tests` — every catalogue
    entry is named the way discovery looks for it, is a surveyed language,
    names a command that exists, and writes only into its own source tree.
  - `crates/hickory-cli/src/language_tier.rs::tests::a_tier_needs_every_rung_below_it`
    still holds: Gold is unreachable in a project with no server installed,
    which is what keeps the claim measured.
- Caveat requiring LLM review:
  - **`syn` sees text, and text is not the program.** A workspace whose types
    come from a code-generating build script, an `include!`, or a
    function-like macro at item position is modelled as what is written, and
    what is written there is a macro call. `lower.rs` returns nothing for
    those rather than guessing; nothing yet reports them in `unresolved`, and
    that is the honest next thing to add.
  - **Uniform paths are resolved by name, not by binding.** `use backend::X`
    is bound to a sibling module when one exists with that name. A crate and
    a module sharing a name would be resolved to the module, which is what
    rustc does — but this arrives at the answer by string match rather than
    by resolution, and a case where rustc disagrees has not been constructed.
  - **Nothing checks that the emitted Rust client compiles, in CI.** It
    compiles in `examples/rust-generator`, which is the same standard the C#
    runtime is held to by the warehouse, and it is a demo rather than a gate.
