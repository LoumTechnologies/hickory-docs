//! The Rust emitter, in its own file because `emit.rs` is near the
//! thousand-line ratchet and a fifth target should not be what pushes it
//! over.
//!
//! Rust is the first target whose *own* rules reach back into the emitter,
//! which is the same thing that happened when Go was added and then C#:
//!
//! * **A field name may be a keyword.** This schema has a field called
//!   `type`, so `r#type` is not a curiosity here — it is the first field of
//!   the first query anyone writes against `hick-model-rust`.
//! * **Rust reads `snake_case` and GraphQL sends `camelCase`.** Every field
//!   therefore carries a `#[serde(rename)]`, which is not decoration: without
//!   it `startLine` silently arrives as a missing field.
//! * **A variant selection is an internally tagged enum**, which Rust has and
//!   Go does not — so the discriminated union survives here where Go's
//!   emitter has to fall back to a map.

use super::emit::{banner, pascal, screaming, shapes_of, snake};
use super::{Field, Kind, ObjectShape, Operation, Shape};

pub fn rust(operations: &[Operation], language: &str) -> String {
    let mut out = banner("//", language);
    out.push_str(
        "#![allow(dead_code)]\n\n\
         use serde::Deserialize;\n\n\
         use crate::hick_model::{Model, ModelError};\n\n",
    );
    for op in operations {
        for (name, fields) in &shapes_of(op) {
            out.push_str(&struct_for(name, fields));
        }
        for (base, variants) in variant_groups(op) {
            out.push_str(&variant_enum(&base, &variants));
        }
        out.push_str(&struct_for(
            &format!("{}Result", op.name),
            &op.result.fields,
        ));
        out.push_str(&format!(
            "pub const {}_QUERY: &str = r#\"\n{}\n\"#;\n\n",
            screaming(&op.name),
            op.text
        ));
        out.push_str(&function(op));
    }
    out
}

/// `pub fn domain_services(model: &mut Model) -> Result<…, ModelError>`.
fn function(op: &Operation) -> String {
    let params: Vec<String> = op
        .variables
        .iter()
        .map(|v| format!("{}: serde_json::Value", ident(&snake(&v.name))))
        .collect();
    let arglist = if params.is_empty() {
        String::new()
    } else {
        format!(", {}", params.join(", "))
    };
    let variables = if op.variables.is_empty() {
        "None".to_string()
    } else {
        format!(
            "Some(serde_json::json!({{ {} }}))",
            op.variables
                .iter()
                .map(|v| format!("\"{}\": {}", v.name, ident(&snake(&v.name))))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!(
        "/// Ask `{}`, typed.\n\
         pub fn {}(model: &mut Model{arglist}) -> Result<{}Result, ModelError> {{\n    \
         model.query({}_QUERY, {variables})\n}}\n\n",
        op.name,
        ident(&snake(&op.name)),
        op.name,
        screaming(&op.name),
    )
}

fn struct_for(name: &str, fields: &[Field]) -> String {
    let mut out = format!(
        "#[derive(Debug, Clone, Deserialize)]\npub struct {} {{\n",
        pascal(name)
    );
    if fields.is_empty() {
        // A selection with no fields of its own still has to deserialise from
        // `{}`, and a unit struct does not.
        out.push_str("}\n\n");
        return out;
    }
    for field in fields {
        if let Some(doc) = &field.description {
            out.push_str(&format!("    /// {}\n", doc.lines().next().unwrap_or("")));
        }
        out.push_str(&serde_attr(field));
        out.push_str(&format!(
            "    pub {}: {},\n",
            ident(&snake_key(&field.key)),
            rust_type(&field.shape)
        ));
    }
    out.push_str("}\n\n");
    out
}

/// The `#[serde(...)]` line for one field.
///
/// `rename` on every field, always: Rust reads `snake_case` and the response
/// speaks `camelCase`, and a rename that is only written where the two
/// differ is a rule with an exception nobody remembers.
///
/// `default` on `__typename` for a reason worth stating: serde's internally
/// tagged enums consume the tag before handing the rest to the variant, so a
/// variant struct that also *requires* `__typename` fails to deserialise the
/// very response it was generated for.
fn serde_attr(field: &Field) -> String {
    if field.key == "__typename" {
        return format!("    #[serde(rename = \"{}\", default)]\n", field.key);
    }
    format!("    #[serde(rename = \"{}\")]\n", field.key)
}

/// An interface selection, as an internally tagged enum.
///
/// The shape the C# emitter builds with `JsonPolymorphic` and the Go emitter
/// cannot build at all. `#[serde(tag = "__typename")]` is exactly what
/// GraphQL's discriminator is, so nothing is lost and nothing is invented.
fn variant_enum(base: &str, variants: &[ObjectShape]) -> String {
    let mut out = format!(
        "#[derive(Debug, Clone, Deserialize)]\n#[serde(tag = \"__typename\")]\npub enum {} {{\n",
        pascal(base)
    );
    for variant in variants {
        let bare = variant.name.rsplit('_').next().unwrap_or(&variant.name);
        out.push_str(&format!(
            "    #[serde(rename = \"{bare}\")]\n    {}({}),\n",
            pascal(bare),
            pascal(&variant.name)
        ));
    }
    out.push_str("}\n\n");
    out
}

/// Every variant selection in an operation, as (base name, variants).
///
/// The same walk C# needs, for the same reason: the union has to be declared
/// as its own type, and the declaration and every use site must arrive at the
/// same name.
fn variant_groups(op: &Operation) -> Vec<(String, Vec<ObjectShape>)> {
    fn walk(shape: &Shape, into: &mut Vec<(String, Vec<ObjectShape>)>) {
        match &shape.kind {
            Kind::List(inner) => walk(inner, into),
            Kind::Object(object) => {
                for field in &object.fields {
                    walk(&field.shape, into);
                }
            }
            Kind::Variants(variants) => {
                into.push((variant_base(variants), variants.clone()));
                for variant in variants {
                    for field in &variant.fields {
                        walk(&field.shape, into);
                    }
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    for field in &op.result.fields {
        walk(&field.shape, &mut out);
    }
    let mut seen = std::collections::HashSet::new();
    out.retain(|(name, _)| seen.insert(name.clone()));
    out
}

fn variant_base(variants: &[ObjectShape]) -> String {
    let first = variants.first().map(|v| v.name.as_str()).unwrap_or("");
    let stem = first
        .rsplit_once('_')
        .map(|(head, _)| head)
        .unwrap_or(first);
    format!("{stem}Kind")
}

fn rust_type(shape: &Shape) -> String {
    let inner = match &shape.kind {
        Kind::List(item) => format!("Vec<{}>", rust_type(item)),
        Kind::Scalar(name) => match name.as_str() {
            "String" | "ID" => "String".into(),
            "Int" => "i64".into(),
            "Float" => "f64".into(),
            "Boolean" => "bool".into(),
            _ => "serde_json::Value".into(),
        },
        // A GraphQL enum arrives as its name on the wire. A `String` rather
        // than a generated Rust enum, for the reason the C# emitter gives: a
        // value the schema gains and this client has not seen would fail to
        // deserialise, and a generator should be able to read a model newer
        // than itself.
        Kind::Enum { .. } => "String".into(),
        Kind::Object(object) => pascal(&object.name),
        Kind::Variants(variants) => pascal(&variant_base(variants)),
    };
    if shape.nullable {
        format!("Option<{inner}>")
    } else {
        inner
    }
}

/// `__typename` is not an identifier, and `type` is a keyword.
fn snake_key(key: &str) -> String {
    let bare = key.trim_start_matches('_');
    snake(bare)
}

/// A raw identifier where the name is a keyword.
///
/// `type` is not a corner case here: this schema HAS a field called `type`,
/// so the first query anybody writes against `hick-model-rust` needs it.
fn ident(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "box", "break", "const", "continue", "crate", "dyn", "else",
        "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
        "move", "mut", "override", "priv", "pub", "ref", "return", "static", "struct", "super",
        "trait", "true", "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while",
        "yield",
    ];
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_keyword_field_is_a_raw_identifier_and_keeps_its_wire_name() {
        // `type` is a field in the Rust model's own schema, so this is the
        // first query anybody writes and not a corner case.
        let field = Field {
            key: "type".into(),
            shape: Shape {
                nullable: false,
                kind: Kind::Scalar("String".into()),
            },
            description: None,
        };
        let source = struct_for("Field", std::slice::from_ref(&field));
        assert!(source.contains("pub r#type: String"), "{source}");
        assert!(source.contains("#[serde(rename = \"type\")]"), "{source}");
    }

    #[test]
    fn a_camel_case_field_is_renamed_rather_than_silently_missing() {
        let field = Field {
            key: "startLine".into(),
            shape: Shape {
                nullable: true,
                kind: Kind::Scalar("Int".into()),
            },
            description: None,
        };
        let source = struct_for("Span", std::slice::from_ref(&field));
        assert!(
            source.contains("#[serde(rename = \"startLine\")]"),
            "{source}"
        );
        assert!(source.contains("pub start_line: Option<i64>"), "{source}");
    }

    #[test]
    fn a_typename_inside_a_variant_defaults_rather_than_failing() {
        // serde consumes the tag before handing the rest to the variant, so a
        // variant that REQUIRES `__typename` cannot deserialise the response
        // it was generated for. Found by running the emitted client, which is
        // the only way a generated client is ever believed.
        let field = Field {
            key: "__typename".into(),
            shape: Shape {
                nullable: false,
                kind: Kind::Scalar("String".into()),
            },
            description: None,
        };
        let source = struct_for("QTypesStructDecl", std::slice::from_ref(&field));
        assert!(source.contains("default"), "{source}");
        assert!(source.contains("pub typename: String"), "{source}");
    }
}
