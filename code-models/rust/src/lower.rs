//! One `syn::Item` at a time, into this schema's items.
//!
//! The only judgement call in this file is what NOT to flatten, and it is
//! made the same way every time: if Rust spells two things differently, they
//! are two things here. A trait's required method and its provided one are
//! one `FnDecl` with `hasBody` telling them apart, because that is what the
//! grammar says; a trait `impl` and an inherent `impl` are one `ImplBlock`
//! with `ofTrait` telling them apart, for the same reason. Neither becomes
//! a method hanging off a type, which is what a flattened model would do and
//! what would lose the only fact worth having.

use crate::model::{
    AssociatedConst, AssociatedType, ConstDecl, EnumDecl, EnumVariant, FnDecl, ImplBlock, Item,
    MacroDecl, Param, Receiver, StructDecl, TraitDecl, TypeAliasDecl, UnionDecl, UseDecl,
    Visibility,
};
use crate::syntax::{Attrs, attrs, fields, generics, span, span_of, tokens, type_ref, visibility};

/// Where the item being read sits.
pub struct Ctx<'a> {
    /// Relative to the root.
    pub file: &'a str,
    /// `crate::api::routes`.
    pub module: &'a str,
    pub package: &'a str,
    pub crate_name: &'a str,
    /// The `cfg` predicates inherited from the `mod` declarations above.
    pub cfg: &'a [String],
}

/// Fill in the nine fields every item shares.
///
/// A macro because the alternative is writing them out ten times, and the
/// tenth one silently omitting `cfg` is the failure this product has already
/// shipped four times in other shapes.
macro_rules! build {
    ($t:ident, $ctx:expr, $name:expr, $vis:expr, $read:expr, $span:expr, { $($rest:tt)* }) => {
        $t {
            name: $name,
            module: $ctx.module.to_string(),
            package: $ctx.package.to_string(),
            crate_name: $ctx.crate_name.to_string(),
            visibility: $vis,
            docs: $read.docs,
            attributes: $read.attributes.into_iter().filter(|a| !a.is_doc).collect(),
            cfg: {
                let mut cfg = $ctx.cfg.to_vec();
                cfg.extend($read.cfg);
                cfg
            },
            declaration: $span,
            $($rest)*
        }
    };
}

/// Everything one `syn::Item` contributes.
///
/// A `Vec` because `use a::{b, c}` is two imports and reporting it as one
/// path with braces in it would make `use`-following a string problem for
/// every caller.
pub fn item(ctx: &Ctx, node: &syn::Item, workspace_crates: &[String]) -> Vec<Item> {
    match node {
        syn::Item::Struct(s) => {
            let read = attrs(ctx.file, &s.attrs);
            let (shape, list) = fields(ctx.file, &s.fields);
            vec![Item::Struct(build!(
                StructDecl,
                ctx,
                s.ident.to_string(),
                visibility(&s.vis),
                read,
                span(ctx.file, s.ident.span()),
                {
                    shape,
                    fields: list,
                    generics: generics(&s.generics),
                }
            ))]
        }
        syn::Item::Enum(e) => {
            let read = attrs(ctx.file, &e.attrs);
            let variants = e
                .variants
                .iter()
                .map(|v| {
                    let read = attrs(ctx.file, &v.attrs);
                    let (shape, list) = fields(ctx.file, &v.fields);
                    EnumVariant {
                        name: v.ident.to_string(),
                        shape,
                        fields: list,
                        discriminant: v.discriminant.as_ref().map(|(_, e)| tokens(e)),
                        attributes: read.attributes,
                        docs: read.docs,
                        declaration: span(ctx.file, v.ident.span()),
                    }
                })
                .collect();
            vec![Item::Enum(build!(
                EnumDecl,
                ctx,
                e.ident.to_string(),
                visibility(&e.vis),
                read,
                span(ctx.file, e.ident.span()),
                {
                    variants,
                    generics: generics(&e.generics),
                }
            ))]
        }
        syn::Item::Union(u) => {
            let read = attrs(ctx.file, &u.attrs);
            let (_, list) = fields(ctx.file, &syn::Fields::Named(u.fields.clone()));
            vec![Item::Union(build!(
                UnionDecl,
                ctx,
                u.ident.to_string(),
                visibility(&u.vis),
                read,
                span(ctx.file, u.ident.span()),
                {
                    fields: list,
                    generics: generics(&u.generics),
                }
            ))]
        }
        syn::Item::Trait(t) => vec![Item::Trait(trait_decl(ctx, t))],
        syn::Item::Impl(i) => vec![Item::Impl(impl_block(ctx, i))],
        syn::Item::Fn(f) => {
            let read = attrs(ctx.file, &f.attrs);
            vec![Item::Fn(function(
                ctx,
                &f.sig,
                visibility(&f.vis),
                read,
                true,
                None,
            ))]
        }
        syn::Item::Type(t) => {
            let read = attrs(ctx.file, &t.attrs);
            vec![Item::TypeAlias(build!(
                TypeAliasDecl,
                ctx,
                t.ident.to_string(),
                visibility(&t.vis),
                read,
                span(ctx.file, t.ident.span()),
                {
                    aliased: type_ref(&t.ty),
                    generics: generics(&t.generics),
                }
            ))]
        }
        syn::Item::Const(c) => {
            let read = attrs(ctx.file, &c.attrs);
            vec![Item::Const(build!(
                ConstDecl,
                ctx,
                c.ident.to_string(),
                visibility(&c.vis),
                read,
                span(ctx.file, c.ident.span()),
                {
                    ty: type_ref(&c.ty),
                    value: tokens(&c.expr),
                    is_static: false,
                    is_mutable: false,
                }
            ))]
        }
        syn::Item::Static(s) => {
            let read = attrs(ctx.file, &s.attrs);
            let is_mutable = matches!(s.mutability, syn::StaticMutability::Mut(_));
            vec![Item::Const(build!(
                ConstDecl,
                ctx,
                s.ident.to_string(),
                visibility(&s.vis),
                read,
                span(ctx.file, s.ident.span()),
                {
                    ty: type_ref(&s.ty),
                    value: tokens(&s.expr),
                    is_static: true,
                    is_mutable,
                }
            ))]
        }
        syn::Item::Macro(m) => {
            let Some(ident) = m.ident.as_ref() else {
                // A macro *invocation* at item position — `lazy_static! { … }`
                // — declares nothing this parser can see. It is reported as
                // unresolved by the caller rather than pretended about here.
                return Vec::new();
            };
            let read = attrs(ctx.file, &m.attrs);
            let is_exported = read.attributes.iter().any(|a| a.name == "macro_export");
            vec![Item::Macro(build!(
                MacroDecl,
                ctx,
                ident.to_string(),
                if is_exported {
                    Visibility {
                        kind: crate::model::VisibilityKind::Pub,
                        path: None,
                        text: "#[macro_export]".into(),
                    }
                } else {
                    Visibility::inherited()
                },
                read,
                span(ctx.file, ident.span()),
                {
                    is_exported,
                    rules: macro_rules_arms(&m.mac.tokens),
                }
            ))]
        }
        syn::Item::Use(u) => use_decls(ctx, u, workspace_crates),
        // `mod` is the caller's business: following it needs the file system,
        // and this file is about one parsed tree.
        _ => Vec::new(),
    }
}

/// The `($pattern) => { … }` heads of a `macro_rules!` body.
///
/// The patterns and not the expansions: an expansion is a token template,
/// and reporting it as if it were code a generator could reason about would
/// be the one thing this model refuses to do — invent.
///
/// Read as token trees rather than by splitting the text on `=>`, because a
/// pattern may CONTAIN `=>` — `from_variant!($src:ty => $dst::$v)` does — and
/// the text split turned that one macro into two arms, neither of them real.
fn macro_rules_arms(body: &proc_macro2::TokenStream) -> Vec<String> {
    let mut arms = Vec::new();
    let mut trees = body.clone().into_iter().peekable();
    while let Some(tree) = trees.next() {
        let proc_macro2::TokenTree::Group(pattern) = tree else {
            continue;
        };
        // `=>` is two joint punctuation tokens; the body group follows.
        let mut arrow = 0;
        while let Some(proc_macro2::TokenTree::Punct(p)) = trees.peek() {
            if p.as_char() == '=' || p.as_char() == '>' {
                arrow += 1;
                trees.next();
            } else {
                break;
            }
        }
        if arrow == 2 {
            arms.push(tokens(&pattern));
            // The expansion, skipped on purpose, and the `;` after it.
            trees.next();
            if let Some(proc_macro2::TokenTree::Punct(p)) = trees.peek()
                && p.as_char() == ';'
            {
                trees.next();
            }
        }
    }
    arms
}

fn trait_decl(ctx: &Ctx, t: &syn::ItemTrait) -> TraitDecl {
    let read = attrs(ctx.file, &t.attrs);
    let mut methods = Vec::new();
    let mut associated_types = Vec::new();
    let mut associated_consts = Vec::new();
    for member in &t.items {
        match member {
            syn::TraitItem::Fn(f) => {
                let read = attrs(ctx.file, &f.attrs);
                methods.push(function(
                    ctx,
                    &f.sig,
                    // A trait method is exactly as visible as its trait: it
                    // has no `pub` of its own, and reporting `inherited`
                    // would make `functions(publicOnly: true)` skip the
                    // whole public surface of every trait.
                    visibility(&t.vis),
                    read,
                    f.default.is_some(),
                    Some(t.ident.to_string()),
                ));
            }
            syn::TraitItem::Type(ty) => {
                let read = attrs(ctx.file, &ty.attrs);
                associated_types.push(AssociatedType {
                    name: ty.ident.to_string(),
                    bounds: (!ty.bounds.is_empty()).then(|| tokens(&ty.bounds)),
                    value: ty.default.as_ref().map(|(_, t)| tokens(t)),
                    docs: read.docs,
                    declaration: span(ctx.file, ty.ident.span()),
                });
            }
            syn::TraitItem::Const(c) => {
                let read = attrs(ctx.file, &c.attrs);
                associated_consts.push(AssociatedConst {
                    name: c.ident.to_string(),
                    ty: type_ref(&c.ty),
                    value: c.default.as_ref().map(|(_, e)| tokens(e)),
                    docs: read.docs,
                    declaration: span(ctx.file, c.ident.span()),
                });
            }
            _ => {}
        }
    }
    build!(
        TraitDecl,
        ctx,
        t.ident.to_string(),
        visibility(&t.vis),
        read,
        span(ctx.file, t.ident.span()),
        {
            generics: generics(&t.generics),
            supertraits: t.supertraits.iter().map(tokens).collect(),
            is_unsafe: t.unsafety.is_some(),
            is_auto: t.auto_token.is_some(),
            methods,
            associated_types,
            associated_consts,
        }
    )
}

fn impl_block(ctx: &Ctx, i: &syn::ItemImpl) -> ImplBlock {
    let read = attrs(ctx.file, &i.attrs);
    let self_type = type_ref(&i.self_ty);
    let of_trait = i.trait_.as_ref().map(|(_, path, _)| tokens(path));
    let trait_name = of_trait
        .as_deref()
        .map(|p| p.split('<').next().unwrap_or(p))
        .and_then(|p| p.rsplit("::").next())
        .map(str::to_string);
    let owner = match (&trait_name, &self_type.text) {
        (Some(t), s) => format!("{t} for {s}"),
        (None, s) => s.clone(),
    };
    let mut methods = Vec::new();
    let mut associated_types = Vec::new();
    let mut associated_consts = Vec::new();
    for member in &i.items {
        match member {
            syn::ImplItem::Fn(f) => {
                let read = attrs(ctx.file, &f.attrs);
                methods.push(function(
                    ctx,
                    &f.sig,
                    visibility(&f.vis),
                    read,
                    true,
                    Some(owner.clone()),
                ));
            }
            syn::ImplItem::Type(ty) => {
                let read = attrs(ctx.file, &ty.attrs);
                associated_types.push(AssociatedType {
                    name: ty.ident.to_string(),
                    bounds: None,
                    value: Some(tokens(&ty.ty)),
                    docs: read.docs,
                    declaration: span(ctx.file, ty.ident.span()),
                });
            }
            syn::ImplItem::Const(c) => {
                let read = attrs(ctx.file, &c.attrs);
                associated_consts.push(AssociatedConst {
                    name: c.ident.to_string(),
                    ty: type_ref(&c.ty),
                    value: Some(tokens(&c.expr)),
                    docs: read.docs,
                    declaration: span(ctx.file, c.ident.span()),
                });
            }
            _ => {}
        }
    }
    // An `impl` has no name. The self type is the name a person would search
    // for, and an empty string would make the block unfindable.
    let name = self_type.text.clone();
    build!(
        ImplBlock,
        ctx,
        name,
        Visibility::inherited(),
        read,
        span_of(ctx.file, &i.self_ty),
        {
            of_trait,
            trait_name,
            self_type,
            generics: generics(&i.generics),
            is_unsafe: i.unsafety.is_some(),
            is_negative: i.trait_.as_ref().is_some_and(|(bang, _, _)| bang.is_some()),
            methods,
            associated_types,
            associated_consts,
        }
    )
}

fn function(
    ctx: &Ctx,
    sig: &syn::Signature,
    vis: Visibility,
    read: Attrs,
    has_body: bool,
    owner: Option<String>,
) -> FnDecl {
    let mut receiver = None;
    let mut params = Vec::new();
    for input in &sig.inputs {
        match input {
            syn::FnArg::Receiver(r) => {
                receiver = Some(Receiver {
                    text: tokens(r),
                    is_reference: r.reference.is_some(),
                    is_mutable: r.mutability.is_some(),
                    lifetime: r
                        .reference
                        .as_ref()
                        .and_then(|(_, l)| l.as_ref())
                        .map(|l| l.to_string()),
                });
            }
            syn::FnArg::Typed(t) => params.push(Param {
                name: match &*t.pat {
                    syn::Pat::Ident(id) => id.ident.to_string(),
                    other => tokens(other),
                },
                pattern: tokens(&t.pat),
                ty: type_ref(&t.ty),
            }),
        }
    }
    let return_type = match &sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(type_ref(ty)),
    };
    build!(
        FnDecl,
        ctx,
        sig.ident.to_string(),
        vis,
        read,
        span(ctx.file, sig.ident.span()),
        {
            signature: tokens(sig),
            params,
            return_type,
            receiver,
            is_async: sig.asyncness.is_some(),
            is_unsafe: sig.unsafety.is_some(),
            is_const: sig.constness.is_some(),
            abi: sig.abi.as_ref().map(tokens),
            has_body,
            generics: generics(&sig.generics),
            owner,
        }
    )
}

// ---------------------------------------------------------------------------
// `use`
// ---------------------------------------------------------------------------

/// One `UseDecl` per leaf, so `use a::{b, c as d}` is two imports.
fn use_decls(ctx: &Ctx, u: &syn::ItemUse, workspace_crates: &[String]) -> Vec<Item> {
    let read = attrs(ctx.file, &u.attrs);
    let vis = visibility(&u.vis);
    let at = span_of(ctx.file, u);
    let mut leaves = Vec::new();
    flatten(&u.tree, &mut Vec::new(), &mut leaves);
    leaves
        .into_iter()
        .map(|leaf| {
            let root = leaf
                .path
                .split("::")
                .next()
                .unwrap_or(&leaf.path)
                .to_string();
            // `crate`, `self` and `super` are always this crate; anything
            // else is a crate name, and whether the workspace has it is the
            // whole question a lexical resolver can answer.
            let (target_crate, local) = match root.as_str() {
                "crate" | "self" | "super" => (Some(ctx.crate_name.to_string()), true),
                other => {
                    let named = other.replace('-', "_");
                    let local = workspace_crates.contains(&named);
                    (Some(named), local)
                }
            };
            let read = Attrs {
                attributes: read.attributes.clone(),
                docs: read.docs.clone(),
                cfg: read.cfg.clone(),
            };
            Item::Use(build!(
                UseDecl,
                ctx,
                leaf.alias.clone().unwrap_or_else(|| leaf
                    .path
                    .rsplit("::")
                    .next()
                    .unwrap_or(&leaf.path)
                    .to_string()),
                vis.clone(),
                read,
                at.clone(),
                {
                    path: leaf.path,
                    alias: leaf.alias,
                    root,
                    crate_name_of_target: target_crate,
                    is_workspace_local: local,
                    is_glob: leaf.is_glob,
                }
            ))
        })
        .collect()
}

struct Leaf {
    path: String,
    alias: Option<String>,
    is_glob: bool,
}

fn flatten(tree: &syn::UseTree, prefix: &mut Vec<String>, out: &mut Vec<Leaf>) {
    match tree {
        syn::UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            flatten(&p.tree, prefix, out);
            prefix.pop();
        }
        syn::UseTree::Name(n) => {
            let mut path = prefix.clone();
            path.push(n.ident.to_string());
            out.push(Leaf {
                path: path.join("::"),
                alias: None,
                is_glob: false,
            });
        }
        syn::UseTree::Rename(r) => {
            let mut path = prefix.clone();
            path.push(r.ident.to_string());
            out.push(Leaf {
                path: path.join("::"),
                alias: Some(r.rename.to_string()),
                is_glob: false,
            });
        }
        syn::UseTree::Glob(_) => out.push(Leaf {
            path: format!("{}::*", prefix.join("::")),
            alias: None,
            is_glob: true,
        }),
        syn::UseTree::Group(g) => {
            for item in &g.items {
                flatten(item, prefix, out);
            }
        }
    }
}
