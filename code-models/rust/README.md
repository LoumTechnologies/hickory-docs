# `hick-model-rust` — the Rust code model server

The second half of Rust's **Gold** rung (`hick lang`,
`docs/specs/freeform/language-tiers.md`). It answers questions about a Cargo
workspace in **Rust's own terms** — packages, crates, modules, items,
`impl` blocks, derives, `cfg` gates — over GraphQL on a pipe, so a generator
can query the type system instead of parsing it.

```
cd code-models/rust && cargo build --release
cp target/release/hick-model-rust <project>/.hick-cache/models/
hick code-model rust <project>                 # the schema, introspected
hick code-model rust <project> --query '{ types(deriving: "Serialize") { name } }'
```

## What it is built on, and what it is deliberately not

| | |
|---|---|
| parses with | `syn` (`full`), `proc-macro2` with `span-locations` |
| enumerates with | `cargo metadata --no-deps` |
| needs at runtime | `cargo` on `PATH` |
| does **not** need | a nightly toolchain, rustdoc JSON, `rustc`, or a successful build |

**No rustdoc JSON.** It is nightly-only and it requires the crate to compile.
A model server is wanted most while a workspace is half-written, and one that
answers only after a green build is a tool you cannot reach for at the moment
you want it. The cost is stated rather than hidden: this is a **lexical**
resolver. It follows `mod` and `use`; it does not typecheck, does not expand
macros, and does not evaluate `cfg`.

**`cfg` is followed and recorded, never evaluated.** There is no feature
resolution and no target triple here, so deciding that `#[cfg(unix)] mod imp;`
is out of scope would be inventing an answer. The module is read, and every
item under it carries `cfg: ["unix"]` — including the ones whose own
attributes say nothing, because an item inside `#[cfg(test)] mod tests` is
conditional and a generator emitting it into a release build would be wrong.

**`mod` decides what exists, not the directory.** A `.rs` file nobody declares
is not part of the crate and is never read. `#[path]` is honoured, and it
resolves against the directory of the **file the declaration is written in**
while a bare `mod foo;` resolves against the **module** directory — two
different bases, and collapsing them reported 183 real test modules as missing
in the first workspace this was pointed at.

## The schema's top-level types

`Query` has seven fields:

| field | answers |
|---|---|
| `version` | `ServerInfo` — name, version, language, root, file count |
| `unresolved` | `[UnresolvedReference]` — every crate outside the workspace, by name |
| `packages` | `[Package]` — each `Cargo.toml` and its crate roots |
| `modules` | `[Module]` — the module tree, with the file each was reached through |
| `items` | `[Item]` — every declaration, filtered nine ways |
| `types` | `[Item]` — `struct`, `enum`, `union` only; the derive query |
| `impls` | `[ImplBlock]` — inherent and trait impls |
| `functions` | `[FnDecl]` — free functions and, by default, methods |

`Item` is a GraphQL **interface** with ten variants — `StructDecl`,
`EnumDecl`, `UnionDecl`, `TraitDecl`, `ImplBlock`, `FnDecl`,
`TypeAliasDecl`, `ConstDecl`, `MacroDecl`, `UseDecl` — sharing nine fields
(`name`, `module`, `package`, `crateName`, `visibility`, `docs`,
`attributes`, `cfg`, `declaration`). Supporting types: `Visibility`,
`Attribute`, `AttributeArg`, `TypeRef`, `Generics`, `GenericParam`, `Field`,
`EnumVariant`, `Param`, `Receiver`, `AssociatedType`, `AssociatedConst`,
`CrateRoot`, `Span`, `ServerInfo`, `UnresolvedReference`.

**Introspection is the documentation.** `hick code-model rust <root>` with no
`--query` prints it; `--sdl` prints it in the form every ecosystem's GraphQL
codegen reads.

### Where this schema refuses to lower

* **An `impl` block is an item, not a bag of methods on a type.** Which trait
  a method came from is the most useful fact about it, and folding is what
  loses it. `impls(ofTrait: "Display")` is one query; reconstructing it from a
  type's method list is not possible at all.
* **A tuple struct stays a tuple struct.** `shape` is `NAMED`, `TUPLE` or
  `UNIT`, and a tuple field's `name` is its index — which is what Rust calls
  it, since `s.0` is a field access.
* **An enum variant carries data.** `EnumVariant.fields` and its own `shape`,
  because a Rust enum is a sum type and every other server in `code-models/`
  has an enum that is a list of names.
* **`Visibility` is an object, not an enum.** `pub(in crate::api)` carries a
  path, and dropping it would make "is this visible to the api layer"
  unanswerable.
* **Bounds and `where` clauses are text.** A structured bound is a
  type-system question and this server does not typecheck; the text is what a
  generator writing an `impl` needs to emit back, and pretending to a
  structure that was never resolved would be the lowering this schema forbids.

## The four queries a generator asks

Run against this repository's own `code-models/rust`, or any Cargo workspace.

**1 — every type deriving a trait.** The first question a Rust generator
asks, and the reason `derives` is parsed rather than left in the attribute's
raw tokens: string-matching `#[derive(Describe)]` also matches
`#[derive(NotDescribe)]`.

```graphql
{
  types(deriving: "Describe") {
    name
    module
    __typename
    ... on StructDecl { shape fields { name } }
  }
}
```

**2 — a type's fields, with their types and attributes.**

```graphql
{
  types(named: "Order") {
    ... on StructDecl {
      fields {
        name
        visibility { kind }
        type { text name kind isReference arguments { text } }
        attributes { name tokens args { key value } }
        docs
      }
    }
  }
}
```

**3 — every impl of a trait**, with what each block actually contains.
`ofTrait` matches on the last segment, so `Display` finds
`std::fmt::Display` and `fmt::Display` alike.

```graphql
{
  impls(ofTrait: "Display") {
    name
    ofTrait
    selfType { text }
    generics { text whereClause }
    methods { name signature }
    declaration { file startLine }
  }
}
```

**4 — every public fn in a module.** `inModule` matches on module-path
boundaries, so `crate::api` selects `crate::api::routes` and never
`crate::apiary`. Methods are included by default, because "every public fn in
this module" means what a reader of the module would say it means.

```graphql
{
  functions(inModule: "crate::api", publicOnly: true) {
    name
    signature
    isAsync
    params { name type { text } }
    returnType { text }
    owner
  }
}
```

Two more worth knowing:

```graphql
# What did you not see?  Never empty for a real workspace.
{ unresolved { message at { file startLine } } }

# Every serde field rename in the workspace, structured.
{ types(deriving: "Serialize") {
    name
    ... on StructDecl {
      fields { name attributes { name args { key value } } }
    }
} }
```

## Measured

Against `pilockdb`, a 224k-line Cargo workspace of 17 packages:

| | |
|---|---|
| load (cargo metadata + parse 398 files) | ~0.9 s, once |
| a derive query over the whole workspace | sub-millisecond after that |
| modules reached | 569 |
| `unresolved` | 58 — every one an external crate, named |

Held open across a generation pass for the same reason every other server is:
loading is the whole cost, and a server per question pays it per question.

## Protocol

One JSON object per line in, one per line out. Diagnostics to stderr; stdout
carries the protocol and nothing else.

```
→ {"query": "{ types(deriving: \"Describe\") { name } }"}
← {"data":{"types":[{"name":"Order"}]}}
```

A malformed request is answered rather than fatal: the caller is a generator
being written, and being written means getting it wrong.

## Writing the generator in Rust

`hick code-model rust <root> --client queries.graphql --target rust` emits
serde structs plus the query text, and `--runtime --target rust -o <dir>`
writes `hick_model.rs` — the transport. A worked end-to-end example, a Cargo
workspace whose generator queries its own type system and writes a file back
into it, is `examples/rust-generator.hick`.
