//! Rust's own type system, as a GraphQL schema.
//!
//! Nothing here is shared with C#'s, TypeScript's or Go's schema except the
//! three core types — [`ServerInfo`], [`Span`] and [`UnresolvedReference`] —
//! which is the finding `code-models/README.md` records rather than an
//! accident. A Rust `enum` is not a C# enum, a Rust `struct` is not a Go
//! struct, and an `impl` block has no counterpart anywhere: it is a
//! declaration site that belongs to neither the type nor the trait, and
//! flattening it into "methods on a type" loses which trait a method came
//! from, which is the single most useful fact about it.
//!
//! The rules these types obey are the ones every server obeys:
//!
//! * **Nothing is lowered.** A tuple struct is not a named struct with
//!   numeric names — `shape` says which it is, and `Field.name` is the index
//!   only because the language has no other word for it.
//! * **Nothing the compiler invented.** Only what is written in the source.
//!   There is no `#[automatically_derived]` impl here because there is no
//!   `rustc` here.
//! * **Say what you could not resolve.** Every `use` of a crate outside the
//!   workspace is named in [`UnresolvedReference`], never dropped.

use async_graphql::{Enum, Interface, SimpleObject};

// ---------------------------------------------------------------------------
// The shared core: three types, identical in every model server.
// ---------------------------------------------------------------------------

/// Where something is written.
#[derive(Debug, Clone, SimpleObject)]
pub struct Span {
    /// Relative to the root the server was started on.
    pub file: String,
    pub start_line: i32,
    pub start_column: i32,
    pub end_line: i32,
    pub end_column: i32,
}

impl Span {
    pub fn empty(file: &str) -> Self {
        Self {
            file: file.to_string(),
            start_line: 0,
            start_column: 0,
            end_line: 0,
            end_column: 0,
        }
    }
}

/// What this server is and how much of the workspace it read.
#[derive(Debug, Clone, SimpleObject)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub language: String,
    pub root: String,
    pub file_count: i32,
}

/// Something the model could not bind.
///
/// Reported rather than swallowed: a generator that ran against a
/// half-resolved workspace and believed it saw everything emits confidently
/// wrong code. A lexical resolver is what this is — it follows `mod`
/// declarations and `use` paths, and it does not typecheck — so the honest
/// answer to `use serde::Serialize` is "named, outside the workspace",
/// which is a fact a generator can act on. Silence is not.
#[derive(Debug, Clone, SimpleObject)]
pub struct UnresolvedReference {
    pub message: String,
    pub at: Option<Span>,
}

// ---------------------------------------------------------------------------
// Visibility — a real keyword in Rust, unlike Go, and richer than C#'s.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum VisibilityKind {
    /// No keyword: visible to this module and its descendants.
    Inherited,
    Pub,
    /// `pub(crate)`.
    Crate,
    /// `pub(super)`.
    Super,
    /// `pub(in some::path)`.
    InPath,
}

/// `pub`, `pub(crate)`, `pub(in a::b)`, or nothing at all.
///
/// An object rather than an enum because `pub(in crate::api)` carries a path,
/// and an enum that dropped it would make "is this visible to the api layer"
/// unanswerable — which is exactly the kind of question a generator's
/// exceptions are written in.
#[derive(Debug, Clone, SimpleObject)]
pub struct Visibility {
    pub kind: VisibilityKind,
    /// The restriction path, for `PUB_IN`.
    pub path: Option<String>,
    /// As written, e.g. `pub(in crate::api)` — empty for inherited.
    pub text: String,
}

impl Visibility {
    pub fn inherited() -> Self {
        Self {
            kind: VisibilityKind::Inherited,
            path: None,
            text: String::new(),
        }
    }

    /// Whether this is reachable from outside its own crate.
    pub fn is_public(&self) -> bool {
        self.kind == VisibilityKind::Pub
    }
}

// ---------------------------------------------------------------------------
// Attributes — where a Rust generator's whole vocabulary lives.
// ---------------------------------------------------------------------------

/// One `key = value` inside an attribute's parentheses.
#[derive(Debug, Clone, SimpleObject)]
pub struct AttributeArg {
    /// `rename` in `#[serde(rename = "sku")]`; `default` in
    /// `#[serde(default)]`, whose value is null rather than empty.
    pub key: String,
    pub value: Option<String>,
}

/// One attribute, as written.
///
/// `tokens` is the truth and everything else is the convenience — the same
/// division Go's `StructTag` makes, and for the same reason: an attribute is
/// an arbitrary token tree that every macro parses for itself, so a model
/// that only offered its own parse would be lying about the ones it got
/// wrong.
#[derive(Debug, Clone, SimpleObject)]
pub struct Attribute {
    /// The last path segment: `derive`, `serde`, `cfg`.
    pub name: String,
    /// The whole path: `serde`, `tokio::main`.
    pub path: String,
    /// Everything inside the delimiters, verbatim.
    pub tokens: String,
    /// The parsed `key = value` pairs, for the attributes shaped that way.
    pub args: Vec<AttributeArg>,
    /// `#[derive(Clone, Debug)]` — the list, already split.
    pub derives: Vec<String>,
    /// An inner attribute: `#![allow(...)]` rather than `#[allow(...)]`.
    pub is_inner: bool,
    /// A doc comment, which Rust lowers to `#[doc = "…"]`. Excluded from
    /// `attributes` and served through `docs`, but marked here so a
    /// generator reading raw attributes is never surprised.
    pub is_doc: bool,
    pub declaration: Option<Span>,
}

// ---------------------------------------------------------------------------
// Types and generics, kept as Rust writes them.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TypeKind {
    Path,
    Reference,
    Pointer,
    Tuple,
    Slice,
    Array,
    ImplTrait,
    DynTrait,
    Fn,
    Never,
    Infer,
    Macro,
    Other,
}

/// A type as it appears in the source.
///
/// Text first, structure second, and deliberately no attempt at
/// normalisation: `&'a mut Vec<T>` is what the author wrote and what a
/// generator has to emit back. The structured half answers the two questions
/// a generator actually asks — is this a reference, and what is inside it.
#[derive(Debug, Clone, SimpleObject)]
pub struct TypeRef {
    /// As written.
    pub text: String,
    pub kind: TypeKind,
    pub is_reference: bool,
    /// `&mut T`, or `*mut T`.
    pub is_mutable: bool,
    /// `'a` in `&'a str`.
    pub lifetime: Option<String>,
    /// The leading path segments for `Path`: `std::collections::HashMap`.
    pub path: Option<String>,
    /// The last segment alone: `HashMap`. What a generator matches on.
    pub name: Option<String>,
    /// `T` and `U` in `HashMap<T, U>`; the referent of a reference; the
    /// element of a slice.
    pub arguments: Vec<TypeRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum GenericParamKind {
    Type,
    Lifetime,
    Const,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct GenericParam {
    pub name: String,
    pub kind: GenericParamKind,
    /// `Clone + Send` in `<T: Clone + Send>`, as text.
    pub bounds: Option<String>,
    pub default: Option<String>,
}

/// A declaration's generics.
///
/// Bounds and the `where` clause are **text**, on purpose. A structured bound
/// is a type-system question — `T: Iterator<Item = U>` binds an associated
/// type — and this server does not typecheck. Emitting the text back is
/// exactly what a generator writing an `impl` needs, and pretending to a
/// structure that is not resolved would be the lowering this schema forbids.
#[derive(Debug, Clone, SimpleObject)]
pub struct Generics {
    /// `<'a, T: Clone>` — empty when there are none.
    pub text: String,
    pub params: Vec<GenericParam>,
    /// `where T: Debug` — null when there is none.
    pub where_clause: Option<String>,
}

impl Generics {
    pub fn none() -> Self {
        Self {
            text: String::new(),
            params: Vec::new(),
            where_clause: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Fields, variants, parameters.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum StructShape {
    /// `struct S { a: i32 }`
    Named,
    /// `struct S(i32);`
    Tuple,
    /// `struct S;`
    Unit,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Field {
    /// The name, or the index for a tuple field — which is what Rust itself
    /// calls it: `s.0` is a field access.
    pub name: String,
    pub index: i32,
    #[graphql(name = "type")]
    pub ty: TypeRef,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub docs: String,
    pub declaration: Span,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct EnumVariant {
    pub name: String,
    /// A Rust enum variant carries data, which is the difference from every
    /// other language in `code-models/` and the reason `EnumDecl` could never
    /// have been hoisted into a shared schema.
    pub shape: StructShape,
    pub fields: Vec<Field>,
    /// `= 3` in `Red = 3`.
    pub discriminant: Option<String>,
    pub attributes: Vec<Attribute>,
    pub docs: String,
    pub declaration: Span,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Param {
    /// The binding, or the pattern's text when it is one.
    pub name: String,
    pub pattern: String,
    #[graphql(name = "type")]
    pub ty: TypeRef,
}

/// `self`, `&self`, `&mut self`, `self: Box<Self>`.
///
/// Its presence is what separates a method from a free function, and its
/// mutability is what a generator has to reproduce.
#[derive(Debug, Clone, SimpleObject)]
pub struct Receiver {
    pub text: String,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub lifetime: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct AssociatedType {
    pub name: String,
    pub bounds: Option<String>,
    /// The `= T` of an associated type in an `impl`, or a trait's default.
    pub value: Option<String>,
    pub docs: String,
    pub declaration: Span,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct AssociatedConst {
    pub name: String,
    #[graphql(name = "type")]
    pub ty: TypeRef,
    pub value: Option<String>,
    pub docs: String,
    pub declaration: Span,
}

// ---------------------------------------------------------------------------
// The items.
// ---------------------------------------------------------------------------

/// The nine fields every item has, and the struct around them.
///
/// A macro rather than nine repetitions, because the list drifting between
/// declaration kinds is exactly the hand-maintained-table failure this
/// product keeps writing down. `Item`'s interface names these same fields
/// once more, and that repetition is load-bearing: if one struct lost one,
/// the interface would stop compiling.
macro_rules! item {
    ($(#[$outer:meta])* $name:ident { $($extra:tt)* }) => {
        $(#[$outer])*
        #[derive(Debug, Clone, SimpleObject)]
        pub struct $name {
            /// The declared name. For an `impl` block, the self type as
            /// written — an `impl` has no name of its own, and an empty
            /// string would make it unfindable.
            pub name: String,
            /// `crate::api::routes`.
            pub module: String,
            /// The Cargo package this crate belongs to.
            pub package: String,
            /// The crate (target) name, hyphens already turned into
            /// underscores, which is how it is spelled in a `use`.
            pub crate_name: String,
            pub visibility: Visibility,
            /// Doc comments and `#[doc]`, joined, with each line's leading
            /// space removed.
            pub docs: String,
            /// Everything but doc comments.
            pub attributes: Vec<Attribute>,
            /// The `cfg(...)` predicates gating this item, outermost first,
            /// **including the ones inherited from the `mod` declarations
            /// above it** — an item inside `#[cfg(test)] mod tests` is
            /// conditional and nothing in its own attributes says so.
            pub cfg: Vec<String>,
            pub declaration: Span,
            $($extra)*
        }
    };
}

item!(StructDecl {
    pub shape: StructShape,
    pub fields: Vec<Field>,
    pub generics: Generics,
});

item!(EnumDecl {
    pub variants: Vec<EnumVariant>,
    pub generics: Generics,
});

item!(UnionDecl {
    pub fields: Vec<Field>,
    pub generics: Generics,
});

item!(TraitDecl {
    pub generics: Generics,
    /// `Clone + Send` in `trait T: Clone + Send`.
    pub supertraits: Vec<String>,
    pub is_unsafe: bool,
    pub is_auto: bool,
    /// Required and provided methods alike; `hasBody` tells them apart.
    pub methods: Vec<FnDecl>,
    pub associated_types: Vec<AssociatedType>,
    pub associated_consts: Vec<AssociatedConst>,
});

item!(
    /// An `impl` block: the declaration site with no counterpart in any other
    /// model server here.
    ///
    /// Its own item rather than folded into the type's methods, because
    /// *which trait a method came from* is the fact a generator needs and
    /// folding is what loses it. An inherent block has `ofTrait: null`.
    ImplBlock {
        /// The trait implemented, as written — null for an inherent `impl`.
        pub of_trait: Option<String>,
        /// The last segment of `ofTrait`, for matching.
        pub trait_name: Option<String>,
        pub self_type: TypeRef,
        pub generics: Generics,
        pub is_unsafe: bool,
        /// `impl !Send for T` — a different statement from having no impl.
        pub is_negative: bool,
        pub methods: Vec<FnDecl>,
        pub associated_types: Vec<AssociatedType>,
        pub associated_consts: Vec<AssociatedConst>,
    }
);

item!(FnDecl {
    /// The whole signature as written, up to but not including the body.
    pub signature: String,
    pub params: Vec<Param>,
    /// Null for `-> ()`, which is what the source says when it says nothing.
    pub return_type: Option<TypeRef>,
    /// `self` in some form; null for a free function or an associated
    /// function like `Self::new`.
    pub receiver: Option<Receiver>,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_const: bool,
    /// `extern "C"`.
    pub abi: Option<String>,
    /// False for a trait's required method, which has a signature and no
    /// body — "must implement" against "may override".
    pub has_body: bool,
    pub generics: Generics,
    /// The trait or type this belongs to; null for a free function.
    pub owner: Option<String>,
});

item!(TypeAliasDecl {
    pub aliased: TypeRef,
    pub generics: Generics,
});

item!(ConstDecl {
    #[graphql(name = "type")]
    pub ty: TypeRef,
    pub value: String,
    /// True for `static`, a different declaration from `const`: it has an
    /// address and may be `mut`.
    pub is_static: bool,
    pub is_mutable: bool,
});

item!(
    /// A `macro_rules!` definition, by name.
    ///
    /// By name and its arms' patterns, deliberately: a macro's body is a
    /// pattern language over token trees, and a model that claimed to know
    /// what it expands to would be inventing. That one exists, and where, is
    /// what a generator can honestly use.
    MacroDecl {
        /// `#[macro_export]`, which is what puts it at the crate root.
        pub is_exported: bool,
        /// The arms' patterns, as text.
        pub rules: Vec<String>,
    }
);

item!(
    /// One `use`, and whether it landed anywhere.
    UseDecl {
        /// The full path as written, one entry per leaf of a brace group.
        pub path: String,
        pub alias: Option<String>,
        /// The first segment: `crate`, `self`, `super`, or a crate name.
        pub root: String,
        /// The crate this names, once `crate`/`self`/`super` are followed.
        pub crate_name_of_target: Option<String>,
        /// False when the target crate is outside this workspace — the honest
        /// answer for `use serde::Serialize`, which is also recorded in
        /// `unresolved` so nothing is dropped in silence.
        pub is_workspace_local: bool,
        /// A glob: `use foo::*`. Named because it is the one import a lexical
        /// resolver cannot follow.
        pub is_glob: bool,
    }
);

/// Every kind of item, as a GraphQL interface.
///
/// An interface and not one object with a `kind` field, for the reason the
/// whole design exists: a caller writing `... on StructDecl { fields { … } }`
/// is asking a question the type system should answer, and collapsing nine
/// declaration kinds into one bag of nullable fields is the lowering an index
/// format does and a model must not.
#[derive(Debug, Clone, Interface)]
// Nine `field(...)` entries, and clippy sees nine attributes of the same
// shape rather than one list. `async-graphql` has no other spelling for an
// interface's fields, and losing one is what the repetition against
// `item!` exists to catch.
#[allow(clippy::duplicated_attributes)]
#[graphql(
    field(name = "name", ty = "&String"),
    field(name = "module", ty = "&String"),
    field(name = "package", ty = "&String"),
    field(name = "crate_name", ty = "&String"),
    field(name = "visibility", ty = "&Visibility"),
    field(name = "docs", ty = "&String"),
    field(name = "attributes", ty = "&Vec<Attribute>"),
    field(name = "cfg", ty = "&Vec<String>"),
    field(name = "declaration", ty = "&Span")
)]
pub enum Item {
    Struct(StructDecl),
    Enum(EnumDecl),
    Union(UnionDecl),
    Trait(TraitDecl),
    Impl(ImplBlock),
    Fn(FnDecl),
    TypeAlias(TypeAliasDecl),
    Const(ConstDecl),
    Macro(MacroDecl),
    Use(UseDecl),
}

// ---------------------------------------------------------------------------
// Modules and packages.
// ---------------------------------------------------------------------------

/// One module, and the file it was reached through.
#[derive(Debug, Clone, SimpleObject)]
pub struct Module {
    /// `crate::api::routes`.
    pub path: String,
    pub package: String,
    pub crate_name: String,
    /// Relative to the root.
    pub file: String,
    /// `mod foo { … }` rather than `mod foo;` — the two are the same module
    /// and a generator emitting one where the other is needs to know which.
    pub is_inline: bool,
    /// `#[path = "…"]`, when the file was not where the name says.
    pub path_attribute: Option<String>,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    /// Gating `cfg` predicates, including the ones inherited from above.
    pub cfg: Vec<String>,
    pub docs: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum CrateKind {
    Lib,
    Bin,
    ProcMacro,
    Test,
    Bench,
    Example,
    Other,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct CrateRoot {
    /// The crate name as it is spelled in a `use`.
    pub name: String,
    pub kind: CrateKind,
    /// Relative to the root.
    pub file: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Package {
    pub name: String,
    pub version: String,
    /// Relative to the root.
    pub manifest_path: String,
    /// A package is not a crate: one `Cargo.toml` can produce a library and
    /// several binaries, each its own crate root and its own module tree.
    pub crate_roots: Vec<CrateRoot>,
}

// ---------------------------------------------------------------------------
// Asking an item what it is, without unwrapping it.
// ---------------------------------------------------------------------------

/// Which declaration an [`Item`] is, as a filter argument.
///
/// A GraphQL enum for the ARGUMENT, and never a field replacing the
/// interface: `... on StructDecl` stays the way a caller reaches a struct's
/// fields, and this only exists so `items(kind: STRUCT)` does not have to be
/// spelled as a fragment that discards everything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum ItemKind {
    Struct,
    Enum,
    Union,
    Trait,
    Impl,
    Fn,
    TypeAlias,
    Const,
    Macro,
    Use,
}

macro_rules! every_item {
    ($self:expr, $binding:ident => $body:expr) => {
        match $self {
            Item::Struct($binding) => $body,
            Item::Enum($binding) => $body,
            Item::Union($binding) => $body,
            Item::Trait($binding) => $body,
            Item::Impl($binding) => $body,
            Item::Fn($binding) => $body,
            Item::TypeAlias($binding) => $body,
            Item::Const($binding) => $body,
            Item::Macro($binding) => $body,
            Item::Use($binding) => $body,
        }
    };
}

impl Item {
    pub fn kind(&self) -> ItemKind {
        match self {
            Item::Struct(_) => ItemKind::Struct,
            Item::Enum(_) => ItemKind::Enum,
            Item::Union(_) => ItemKind::Union,
            Item::Trait(_) => ItemKind::Trait,
            Item::Impl(_) => ItemKind::Impl,
            Item::Fn(_) => ItemKind::Fn,
            Item::TypeAlias(_) => ItemKind::TypeAlias,
            Item::Const(_) => ItemKind::Const,
            Item::Macro(_) => ItemKind::Macro,
            Item::Use(_) => ItemKind::Use,
        }
    }

    pub fn ident(&self) -> &str {
        every_item!(self, d => &d.name)
    }

    pub fn in_module(&self) -> &str {
        every_item!(self, d => &d.module)
    }

    pub fn in_package(&self) -> &str {
        every_item!(self, d => &d.package)
    }

    pub fn vis(&self) -> &Visibility {
        every_item!(self, d => &d.visibility)
    }

    pub fn attrs(&self) -> &[Attribute] {
        every_item!(self, d => &d.attributes)
    }

    /// Every trait named in a `#[derive(...)]` on this item, by last segment.
    ///
    /// The single most useful question a Rust generator asks, and the reason
    /// `derives` is parsed on the attribute rather than left in `tokens`:
    /// "every type that derives `Describe`" is one filter, and string-matching
    /// an attribute's raw text would also match `#[derive(NotDescribe)]`.
    pub fn derives(&self) -> Vec<&str> {
        self.attrs()
            .iter()
            .flat_map(|a| a.derives.iter().map(String::as_str))
            .collect()
    }
}
