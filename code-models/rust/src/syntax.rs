//! From `syn`'s tree to this schema's types.
//!
//! Everything here is lexical. There is no `rustc` in this process and no
//! rustdoc JSON behind it — deliberately, because both would pin the server
//! to a nightly toolchain and a build of the whole workspace, and a model
//! server that only answers after a successful `cargo check` is useless
//! exactly when a generator is most wanted. What a parser can see, this sees;
//! what it cannot, [`crate::model::UnresolvedReference`] says so about.

use proc_macro2::Span as PmSpan;
use quote::ToTokens;
use syn::spanned::Spanned;

use crate::model::{
    Attribute, AttributeArg, Field, GenericParam, GenericParamKind, Generics, Span, StructShape,
    TypeKind, TypeRef, Visibility, VisibilityKind,
};

/// A token stream as a person would write it.
///
/// `to_token_stream().to_string()` puts a space between every token, so
/// `Vec<String>` comes back as `Vec < String >`. That text ends up inside
/// generated source — a `where` clause copied into an emitted `impl` — so it
/// has to read like Rust rather than like a token dump. The rule is one
/// pass: drop a space when the character on one side of it says the space
/// was never written.
pub fn tokens<T: ToTokens>(node: &T) -> String {
    let raw = node.to_token_stream().to_string();
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    for (i, &ch) in chars.iter().enumerate() {
        if ch != ' ' {
            out.push(ch);
            continue;
        }
        let prev = out.chars().last();
        let next = chars.get(i + 1).copied();
        if drop_space(prev, next, out.ends_with("::")) {
            continue;
        }
        out.push(' ');
    }
    out
}

fn drop_space(prev: Option<char>, next: Option<char>, out_ends_with_path_separator: bool) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    // `std :: x` closes up on both sides and `f : i32` only on the left: a
    // single colon takes a space after it and `::` never does. The first
    // version dropped both and emitted `f:&mut Formatter` into generated
    // source, which is legal Rust and reads as a typo.
    if prev == Some(':') && out_ends_with_path_separator {
        return true;
    }
    match (prev, next) {
        // `Vec <` · `foo (` · `> (` — a bracket binds to what it follows,
        // and only when something is actually there to bind to. `[` is not in
        // this list: `mut [u8]` is a slice after a keyword, not an index.
        (Some(p), Some('<' | '(')) if ident(p) || p == '>' => true,
        // `Vec < String` · `& 'a` · `#[ ` · `impl ! Send` · `$ name`
        (Some('<' | '&' | '(' | '[' | '#' | '!' | '$'), _) => true,
        // ` >` · ` ,` · ` ;` · ` :` · ` )` — punctuation never takes a space
        // before it, and `->` is one token so it is never seen here.
        (_, Some('>' | ',' | ';' | ':' | ')' | ']')) => true,
        _ => false,
    }
}

/// A span, relative to the root the server was started on.
pub fn span(file: &str, span: PmSpan) -> Span {
    let (start, end) = (span.start(), span.end());
    Span {
        file: file.to_string(),
        start_line: start.line as i32,
        // `proc-macro2` counts columns from zero and every editor counts from
        // one. Converted here, once, rather than in each caller's head.
        start_column: start.column as i32 + 1,
        end_line: end.line as i32,
        end_column: end.column as i32 + 1,
    }
}

pub fn span_of<T: Spanned>(file: &str, node: &T) -> Span {
    span(file, node.span())
}

// ---------------------------------------------------------------------------
// Visibility
// ---------------------------------------------------------------------------

pub fn visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility {
            kind: VisibilityKind::Pub,
            path: None,
            text: "pub".into(),
        },
        syn::Visibility::Restricted(r) => {
            let path = tokens(&r.path);
            let kind = match path.as_str() {
                "crate" => VisibilityKind::Crate,
                "super" => VisibilityKind::Super,
                "self" => VisibilityKind::Inherited,
                _ => VisibilityKind::InPath,
            };
            Visibility {
                kind,
                path: Some(path),
                text: tokens(vis),
            }
        }
        syn::Visibility::Inherited => Visibility::inherited(),
    }
}

// ---------------------------------------------------------------------------
// Attributes
// ---------------------------------------------------------------------------

/// What a run of attributes says: the attributes themselves, the docs they
/// carry, and the `cfg` predicates that gate whatever they are on.
pub struct Attrs {
    pub attributes: Vec<Attribute>,
    pub docs: String,
    pub cfg: Vec<String>,
}

/// Read a declaration's attributes.
///
/// Doc comments are separated out because Rust lowers `/// x` to
/// `#[doc = " x"]`, and a generator reading `attributes` to find `#[serde]`
/// should not have to step over prose. They are still reported, with
/// `isDoc: true`, so nothing here is a hidden filter.
pub fn attrs(file: &str, list: &[syn::Attribute]) -> Attrs {
    let mut attributes = Vec::new();
    let mut docs = Vec::new();
    let mut cfg = Vec::new();
    for attribute in list {
        let path = tokens(attribute.path());
        let name = path.rsplit("::").next().unwrap_or(&path).to_string();
        let is_doc = name == "doc";
        let raw = match &attribute.meta {
            syn::Meta::Path(_) => String::new(),
            syn::Meta::List(list) => tokens(&list.tokens),
            syn::Meta::NameValue(nv) => tokens(&nv.value),
        };
        if is_doc {
            docs.push(strip_doc(&raw));
        }
        if name == "cfg" {
            cfg.push(raw.clone());
        }
        attributes.push(Attribute {
            derives: if name == "derive" {
                split_top_level(&raw)
                    .into_iter()
                    .map(|d| d.rsplit("::").next().unwrap_or(&d).to_string())
                    .collect()
            } else {
                Vec::new()
            },
            args: if matches!(attribute.meta, syn::Meta::List(_)) {
                key_values(&raw)
            } else if let syn::Meta::NameValue(_) = attribute.meta {
                vec![AttributeArg {
                    key: name.clone(),
                    value: Some(unquote(&raw)),
                }]
            } else {
                Vec::new()
            },
            is_inner: matches!(attribute.style, syn::AttrStyle::Inner(_)),
            is_doc,
            declaration: Some(span_of(file, attribute)),
            name,
            path,
            tokens: raw,
        });
    }
    Attrs {
        attributes,
        docs: docs.join("\n"),
        cfg,
    }
}

/// `" A line."` from a doc comment's literal, without the quotes or the one
/// space `///` leaves behind.
fn strip_doc(raw: &str) -> String {
    let text = unquote(raw);
    text.strip_prefix(' ').unwrap_or(&text).to_string()
}

fn unquote(raw: &str) -> String {
    let trimmed = raw.trim();
    match trimmed.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        Some(inner) => inner.replace("\\\"", "\"").replace("\\n", "\n"),
        None => trimmed.to_string(),
    }
}

/// Split on commas that are not inside brackets: `Clone, Serialize` but also
/// `bound(deserialize = "T: A, B")`, which a naive split turns into nonsense.
fn split_top_level(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();
    let mut in_string = false;
    for ch in raw.chars() {
        match ch {
            '"' => {
                in_string = !in_string;
                current.push(ch);
            }
            '(' | '[' | '{' if !in_string => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' if !in_string => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 && !in_string => {
                out.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        out.push(current.trim().to_string());
    }
    out.retain(|s| !s.is_empty());
    out
}

/// `#[serde(rename = "sku", default)]` as pairs.
///
/// A bare word is a key with a null value rather than a key with an empty
/// one: `default` and `default = ""` are different instructions, and a model
/// that could not tell them apart would make the first unaskable.
fn key_values(raw: &str) -> Vec<AttributeArg> {
    split_top_level(raw)
        .into_iter()
        .map(|part| match part.split_once('=') {
            Some((key, value)) => AttributeArg {
                key: key.trim().to_string(),
                value: Some(unquote(value)),
            },
            None => AttributeArg {
                key: part,
                value: None,
            },
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

pub fn type_ref(ty: &syn::Type) -> TypeRef {
    let text = tokens(ty);
    let mut out = TypeRef {
        text,
        kind: TypeKind::Other,
        is_reference: false,
        is_mutable: false,
        lifetime: None,
        path: None,
        name: None,
        arguments: Vec::new(),
    };
    match ty {
        syn::Type::Path(path) => {
            out.kind = TypeKind::Path;
            out.path = Some(tokens(&path.path));
            if let Some(last) = path.path.segments.last() {
                out.name = Some(last.ident.to_string());
                if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner) = arg {
                            out.arguments.push(type_ref(inner));
                        }
                    }
                }
            }
        }
        syn::Type::Reference(r) => {
            out.kind = TypeKind::Reference;
            out.is_reference = true;
            out.is_mutable = r.mutability.is_some();
            out.lifetime = r.lifetime.as_ref().map(|l| l.to_string());
            out.arguments.push(type_ref(&r.elem));
            out.name = out.arguments[0].name.clone();
        }
        syn::Type::Ptr(p) => {
            out.kind = TypeKind::Pointer;
            out.is_mutable = p.mutability.is_some();
            out.arguments.push(type_ref(&p.elem));
        }
        syn::Type::Tuple(t) => {
            out.kind = TypeKind::Tuple;
            out.arguments = t.elems.iter().map(type_ref).collect();
        }
        syn::Type::Slice(s) => {
            out.kind = TypeKind::Slice;
            out.arguments.push(type_ref(&s.elem));
        }
        syn::Type::Array(a) => {
            out.kind = TypeKind::Array;
            out.arguments.push(type_ref(&a.elem));
        }
        syn::Type::ImplTrait(_) => out.kind = TypeKind::ImplTrait,
        syn::Type::TraitObject(_) => out.kind = TypeKind::DynTrait,
        syn::Type::BareFn(_) => out.kind = TypeKind::Fn,
        syn::Type::Never(_) => out.kind = TypeKind::Never,
        syn::Type::Infer(_) => out.kind = TypeKind::Infer,
        syn::Type::Macro(_) => out.kind = TypeKind::Macro,
        syn::Type::Paren(p) => return type_ref(&p.elem),
        syn::Type::Group(g) => return type_ref(&g.elem),
        _ => {}
    }
    out
}

// ---------------------------------------------------------------------------
// Generics
// ---------------------------------------------------------------------------

pub fn generics(generics: &syn::Generics) -> Generics {
    if generics.params.is_empty() && generics.where_clause.is_none() {
        return Generics::none();
    }
    let params = generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(t) => GenericParam {
                name: t.ident.to_string(),
                kind: GenericParamKind::Type,
                bounds: (!t.bounds.is_empty()).then(|| tokens(&t.bounds)),
                default: t.default.as_ref().map(tokens),
            },
            syn::GenericParam::Lifetime(l) => GenericParam {
                name: l.lifetime.to_string(),
                kind: GenericParamKind::Lifetime,
                bounds: (!l.bounds.is_empty()).then(|| tokens(&l.bounds)),
                default: None,
            },
            syn::GenericParam::Const(c) => GenericParam {
                name: c.ident.to_string(),
                kind: GenericParamKind::Const,
                bounds: Some(tokens(&c.ty)),
                default: c.default.as_ref().map(tokens),
            },
        })
        .collect();
    let mut text = String::new();
    if !generics.params.is_empty() {
        text = format!("<{}>", tokens(&generics.params));
    }
    Generics {
        text,
        params,
        where_clause: generics.where_clause.as_ref().map(tokens),
    }
}

// ---------------------------------------------------------------------------
// Fields
// ---------------------------------------------------------------------------

pub fn fields(file: &str, fields: &syn::Fields) -> (StructShape, Vec<Field>) {
    let shape = match fields {
        syn::Fields::Named(_) => StructShape::Named,
        syn::Fields::Unnamed(_) => StructShape::Tuple,
        syn::Fields::Unit => StructShape::Unit,
    };
    let list = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let read = attrs(file, &field.attrs);
            Field {
                // A tuple field's name is its index, which is what Rust
                // itself calls it: `s.0` is a field access, not a hole.
                name: field
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| index.to_string()),
                index: index as i32,
                ty: type_ref(&field.ty),
                visibility: visibility(&field.vis),
                attributes: read.attributes,
                docs: read.docs,
                declaration: span_of(file, field),
            }
        })
        .collect();
    (shape, list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_type_reads_back_as_rust_rather_than_as_tokens() {
        // The text here is copied into generated source, so `Vec < String >`
        // would ship into somebody's file.
        for source in [
            "Vec<String>",
            "&'a mut [u8]",
            "std::collections::HashMap<K, V>",
            "Option<&'static str>",
            "[u8; 32]",
            "(A, B)",
            "fn(i32) -> bool",
        ] {
            let parsed: syn::Type = syn::parse_str(source).unwrap();
            assert_eq!(tokens(&parsed), source);
        }
    }

    #[test]
    fn a_single_colon_keeps_its_space_and_a_path_separator_does_not() {
        // `f:&mut Formatter` is legal Rust and reads as a typo, and this text
        // is copied into generated source.
        let signature: syn::Signature =
            syn::parse_str("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result")
                .unwrap();
        assert_eq!(
            tokens(&signature),
            "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result"
        );
    }

    #[test]
    fn a_derive_list_is_split_and_a_serde_argument_is_not() {
        // `bound(deserialize = "T: A, B")` has a comma inside a string inside
        // parentheses, and a naive split turns one attribute into three.
        let attribute: syn::Attribute =
            syn::parse_quote!(#[serde(rename = "sku", bound(deserialize = "T: A, B"), default)]);
        let read = attrs("x.rs", std::slice::from_ref(&attribute));
        let args = &read.attributes[0].args;
        assert_eq!(args.len(), 3, "{args:?}");
        assert_eq!(args[0].key, "rename");
        assert_eq!(args[0].value.as_deref(), Some("sku"));
        // A bare word is a key with no value, because `default` and
        // `default = ""` are different instructions.
        assert_eq!(args[2].key, "default");
        assert_eq!(args[2].value, None);
    }

    #[test]
    fn a_doc_comment_is_prose_and_not_an_attribute_to_step_over() {
        let attribute: syn::Attribute = syn::parse_quote!(#[doc = " The sku."]);
        let read = attrs("x.rs", std::slice::from_ref(&attribute));
        assert_eq!(read.docs, "The sku.");
        assert!(read.attributes[0].is_doc);
    }
}
