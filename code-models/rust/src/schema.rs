//! The queries, and the four a generator actually asks.
//!
//! Every argument here exists because a generator would otherwise fetch the
//! whole workspace and filter it in its own language — which works, and
//! throws away the reason the protocol is GraphQL. A selection set is
//! laziness; an argument is the other half of it.
//!
//! The four, worked in `code-models/rust/README.md`:
//!
//! 1. every type deriving a trait — `types(deriving: "Describe")`
//! 2. a type's fields, with their types and attributes —
//!    `types(named: "Order") { ... on StructDecl { fields { … } } }`
//! 3. every impl of a trait — `impls(ofTrait: "Display")`
//! 4. every public fn in a module —
//!    `functions(inModule: "crate::api", publicOnly: true)`

use std::sync::Arc;

use async_graphql::{Context, Object};

use crate::load::Model;
use crate::model::{
    FnDecl, ImplBlock, Item, ItemKind, Module, Package, ServerInfo, UnresolvedReference,
};

pub struct Query;

fn model<'a>(ctx: &'a Context<'_>) -> &'a Arc<Model> {
    ctx.data_unchecked::<Arc<Model>>()
}

/// Whether a name passes the name filters a caller gave.
fn name_matches(name: &str, is: &Option<String>, ends_with: &Option<String>) -> bool {
    if let Some(exact) = is
        && name != exact
    {
        return false;
    }
    if let Some(suffix) = ends_with
        && !name.ends_with(suffix.as_str())
    {
        return false;
    }
    true
}

/// Whether a module path passes `inModule`.
///
/// A prefix match on module-path boundaries, so `crate::api` selects
/// `crate::api::routes` and not `crate::apiary` — the bug a plain
/// `starts_with` would ship and nothing would catch until a generator emitted
/// handlers for the wrong module.
fn module_matches(module: &str, wanted: &Option<String>) -> bool {
    match wanted {
        None => true,
        Some(prefix) => {
            module == prefix
                || module
                    .strip_prefix(prefix.as_str())
                    .is_some_and(|rest| rest.starts_with("::"))
        }
    }
}

// A GraphQL field's arguments are its arguments; there is no struct to hide
// them in that would not also hide them from introspection, which is the
// documentation.
#[allow(clippy::too_many_arguments)]
#[Object]
impl Query {
    /// What this server is, and how much of the workspace it read.
    async fn version(&self, ctx: &Context<'_>) -> ServerInfo {
        model(ctx).info()
    }

    /// Everything the model could not bind.
    ///
    /// Never empty for a real workspace, because every crate outside it is
    /// named here — which is the point. A generator that treats an empty
    /// `unresolved` as "I saw everything" is right only when it is true.
    async fn unresolved(&self, ctx: &Context<'_>) -> Vec<UnresolvedReference> {
        model(ctx).unresolved.clone()
    }

    /// The workspace's packages and their crate roots.
    async fn packages(&self, ctx: &Context<'_>) -> Vec<Package> {
        model(ctx).packages.clone()
    }

    /// Modules, in the order they were reached from each crate root.
    async fn modules(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "`crate::api` also selects `crate::api::routes`.")] in_module: Option<
            String,
        >,
        package_is: Option<String>,
        crate_is: Option<String>,
    ) -> Vec<Module> {
        model(ctx)
            .modules
            .iter()
            .filter(|m| module_matches(&m.path, &in_module))
            .filter(|m| package_is.as_ref().is_none_or(|p| &m.package == p))
            .filter(|m| crate_is.as_ref().is_none_or(|c| &m.crate_name == c))
            .cloned()
            .collect()
    }

    /// Every declaration, filtered.
    async fn items(
        &self,
        ctx: &Context<'_>,
        kind: Option<ItemKind>,
        named: Option<String>,
        name_ends_with: Option<String>,
        in_module: Option<String>,
        package_is: Option<String>,
        #[graphql(desc = "Only `pub` items — the crate's surface rather than its insides.")]
        public_only: Option<bool>,
        #[graphql(
            desc = "Only items carrying this attribute, by its last path segment: `serde`, `test`, `tokio`."
        )]
        with_attribute: Option<String>,
        #[graphql(desc = "Only items whose `#[derive(...)]` names this trait.")] deriving: Option<
            String,
        >,
    ) -> Vec<Item> {
        model(ctx)
            .items()
            .filter(|item| kind.is_none_or(|k| item.kind() == k))
            .filter(|item| name_matches(item.ident(), &named, &name_ends_with))
            .filter(|item| module_matches(item.in_module(), &in_module))
            .filter(|item| package_is.as_ref().is_none_or(|p| item.in_package() == p))
            .filter(|item| !public_only.unwrap_or(false) || item.vis().is_public())
            .filter(|item| {
                with_attribute
                    .as_ref()
                    .is_none_or(|want| item.attrs().iter().any(|a| &a.name == want))
            })
            .filter(|item| {
                deriving
                    .as_ref()
                    .is_none_or(|want| item.derives().iter().any(|d| d == want))
            })
            .cloned()
            .collect()
    }

    /// The type declarations only: `struct`, `enum`, `union`.
    ///
    /// Its own field rather than `items(kind: …)` because "every type that
    /// derives Describe" is the first question every Rust generator asks, and
    /// a `struct` and an `enum` both answer it. One argument that spans three
    /// kinds beats three queries stitched together by the caller.
    async fn types(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The trait named in a `#[derive(...)]`, by last segment.")]
        deriving: Option<String>,
        named: Option<String>,
        name_ends_with: Option<String>,
        in_module: Option<String>,
        public_only: Option<bool>,
    ) -> Vec<Item> {
        model(ctx)
            .items()
            .filter(|item| {
                matches!(
                    item.kind(),
                    ItemKind::Struct | ItemKind::Enum | ItemKind::Union
                )
            })
            .filter(|item| name_matches(item.ident(), &named, &name_ends_with))
            .filter(|item| module_matches(item.in_module(), &in_module))
            .filter(|item| !public_only.unwrap_or(false) || item.vis().is_public())
            .filter(|item| {
                deriving
                    .as_ref()
                    .is_none_or(|want| item.derives().iter().any(|d| d == want))
            })
            .cloned()
            .collect()
    }

    /// Every `impl` block, filtered.
    ///
    /// `ofTrait: null` is not the same argument as omitting it: omitted means
    /// every block, and `inherentOnly` means the ones belonging to no trait.
    /// Two different questions, and conflating them would make one of them
    /// unaskable.
    async fn impls(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "By last segment, so `Display` matches `std::fmt::Display`.")]
        of_trait: Option<String>,
        #[graphql(desc = "By the self type's last segment: `Order` matches `Order<'a, T>`.")]
        for_type: Option<String>,
        in_module: Option<String>,
        #[graphql(desc = "Only blocks implementing no trait.")] inherent_only: Option<bool>,
    ) -> Vec<ImplBlock> {
        model(ctx)
            .items()
            .filter_map(|item| match item {
                Item::Impl(block) => Some(block),
                _ => None,
            })
            .filter(|block| {
                of_trait
                    .as_ref()
                    .is_none_or(|want| block.trait_name.as_deref() == Some(want.as_str()))
            })
            .filter(|block| {
                for_type
                    .as_ref()
                    .is_none_or(|want| block.self_type.name.as_deref() == Some(want.as_str()))
            })
            .filter(|block| module_matches(&block.module, &in_module))
            .filter(|block| !inherent_only.unwrap_or(false) || block.of_trait.is_none())
            .cloned()
            .collect()
    }

    /// Free functions and, unless asked otherwise, the methods of every
    /// `impl` and `trait` in scope.
    ///
    /// Methods are included by default because "every public fn in this
    /// module" means what a reader of the module would say it means, and a
    /// module whose functions are all inherent methods would otherwise answer
    /// with nothing.
    async fn functions(
        &self,
        ctx: &Context<'_>,
        in_module: Option<String>,
        named: Option<String>,
        name_ends_with: Option<String>,
        public_only: Option<bool>,
        #[graphql(desc = "Off for free functions alone.")] include_methods: Option<bool>,
        #[graphql(desc = "Only `async fn`.")] async_only: Option<bool>,
    ) -> Vec<FnDecl> {
        let include_methods = include_methods.unwrap_or(true);
        let mut out = Vec::new();
        for item in model(ctx).items() {
            match item {
                Item::Fn(f) => out.push(f.clone()),
                Item::Impl(block) if include_methods => out.extend(block.methods.iter().cloned()),
                Item::Trait(t) if include_methods => out.extend(t.methods.iter().cloned()),
                _ => {}
            }
        }
        out.retain(|f| {
            name_matches(&f.name, &named, &name_ends_with)
                && module_matches(&f.module, &in_module)
                && (!public_only.unwrap_or(false) || f.visibility.is_public())
                && (!async_only.unwrap_or(false) || f.is_async)
        });
        out
    }
}
