//! Typed clients for code model queries, in whatever language the generator
//! is written in.
//!
//! ## Why this is here and not four ecosystem tools
//!
//! Every language has a GraphQL codegen and each is good at what it is for,
//! which is building an application client. Measured before writing this:
//! `graphql-codegen` is 166 npm packages and 72 MB to type one query;
//! `genqlient` needs Go; `ariadne-codegen` needs Python and pydantic; and
//! StrawberryShake is a reactive client framework with stores and dependency
//! injection, which is the wrong shape entirely for "spawn a subprocess and
//! ask three questions".
//!
//! Taking all four would mean four toolchains to install confined, four
//! config formats to explain, and four differently-shaped clients for a
//! generator author to relearn — which is precisely the per-language expense
//! this whole design keeps trying to avoid.
//!
//! ## The shape of the answer
//!
//! The hard part of query codegen is **language-independent**: walk the
//! query against the schema and work out what the response looks like,
//! including nullability, lists, and the variants an interface selection can
//! produce. That is [`Shape`], and it is computed once.
//!
//! Only the last step differs per language, and it is small. An emitter turns
//! a `Shape` into source text; adding a language is one file, not one
//! toolchain.
//!
//! What this deliberately does not implement: fragments defined outside the
//! operation, directives, subscriptions, and custom scalar mapping. A
//! generator's queries are selections over a local schema, and a feature
//! nobody uses is a feature that rots. Anything unsupported is **refused by
//! name** rather than silently dropped.

pub mod emit;

use std::collections::BTreeMap;

use anyhow::{Result, bail};
use graphql_parser::query::{
    Definition, Document, OperationDefinition, Selection, SelectionSet, Type as QueryType,
};
use serde_json::Value;

/// One operation, resolved against a schema.
#[derive(Debug, Clone)]
pub struct Operation {
    /// `DomainServices` from `query DomainServices { … }`.
    pub name: String,
    /// The query text, emitted alongside the types so the two cannot drift.
    pub text: String,
    /// Declared variables, in order.
    pub variables: Vec<Variable>,
    /// The root selection.
    pub result: ObjectShape,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub graphql_type: String,
    pub required: bool,
}

/// A field as it appears in the response.
#[derive(Debug, Clone)]
pub struct Field {
    /// The key in the JSON: the alias when there is one, else the field name.
    pub key: String,
    pub shape: Shape,
    pub description: Option<String>,
}

/// What a value looks like, independent of any target language.
#[derive(Debug, Clone)]
pub struct Shape {
    /// The response may hold null here.
    pub nullable: bool,
    pub kind: Kind,
}

#[derive(Debug, Clone)]
pub enum Kind {
    List(Box<Shape>),
    /// A GraphQL scalar by name: `String`, `Int`, `Boolean`, or a custom one.
    Scalar(String),
    Enum {
        name: String,
        values: Vec<String>,
    },
    Object(ObjectShape),
    /// An interface or union selection: which variant a value is depends on
    /// `__typename`.
    ///
    /// **Every possible type is listed, not only the ones the query wrote a
    /// fragment for.** Selecting `types { name }` on an interface returns
    /// EnumDecls and StructDecls too, whichever fragments were written, so a
    /// union of just the fragment-covered types is a lie the compiler cannot
    /// catch and the runtime will. The first version of this made exactly
    /// that mistake and the generated TypeScript did not compile, which is
    /// the good version of finding out.
    ///
    /// Each variant carries the interface's own selected fields merged with
    /// whatever its fragment added, so a caller switches on `__typename` and
    /// has everything.
    Variants(Vec<ObjectShape>),
}

/// A named object shape. The name is derived from the path, so a nested
/// selection gets a stable, readable type name in every target language.
#[derive(Debug, Clone)]
pub struct ObjectShape {
    pub name: String,
    pub fields: Vec<Field>,
}

/// The introspection response, indexed for lookup.
pub struct Schema {
    types: BTreeMap<String, Value>,
    query_root: String,
}

impl Schema {
    pub fn from_introspection(introspection: &Value) -> Result<Self> {
        let Some(schema) = introspection.pointer("/data/__schema") else {
            bail!("that is not an introspection response");
        };
        let mut types = BTreeMap::new();
        for entry in schema["types"].as_array().into_iter().flatten() {
            if let Some(name) = entry["name"].as_str() {
                types.insert(name.to_string(), entry.clone());
            }
        }
        Ok(Self {
            types,
            query_root: schema
                .pointer("/queryType/name")
                .and_then(Value::as_str)
                .unwrap_or("Query")
                .to_string(),
        })
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.types.get(name)
    }

    /// The declared type of `field` on `owner`, as an introspection type ref.
    fn field_type(&self, owner: &str, field: &str) -> Option<&Value> {
        let entry = self.get(owner)?;
        entry["fields"]
            .as_array()?
            .iter()
            .find(|f| f["name"].as_str() == Some(field))
            .map(|f| &f["type"])
    }

    fn field_description(&self, owner: &str, field: &str) -> Option<String> {
        let entry = self.get(owner)?;
        entry["fields"]
            .as_array()?
            .iter()
            .find(|f| f["name"].as_str() == Some(field))?["description"]
            .as_str()
            .filter(|d| !d.is_empty())
            .map(str::to_string)
    }
}

/// Strip `NON_NULL` and `LIST` wrappers down to a named type.
fn named_of(mut node: &Value) -> Option<&str> {
    loop {
        if let Some(name) = node["name"].as_str() {
            return Some(name);
        }
        node = node.get("ofType")?;
    }
}

/// Resolve every operation in `document` against `schema`.
pub fn resolve(schema: &Schema, document_text: &str) -> Result<Vec<Operation>> {
    let document: Document<'_, String> = graphql_parser::parse_query(document_text)
        .map_err(|e| anyhow::anyhow!("could not parse the query document: {e}"))?;

    let mut out = Vec::new();
    for definition in &document.definitions {
        match definition {
            Definition::Operation(OperationDefinition::Query(query)) => {
                let name = query.name.clone().unwrap_or_else(|| "Anonymous".into());
                // A named operation is required, because the name is what
                // every generated type and function is called. An anonymous
                // query would produce `AnonymousResult`, and two of them
                // would collide silently.
                if query.name.is_none() {
                    bail!(
                        "every operation needs a name — it is what the generated type and \
                         function are called.\n  Write `query SomeName {{ … }}`."
                    );
                }
                let variables = query
                    .variable_definitions
                    .iter()
                    .map(|v| Variable {
                        name: v.name.clone(),
                        graphql_type: v.var_type.to_string(),
                        required: matches!(v.var_type, QueryType::NonNullType(_)),
                    })
                    .collect();
                let result = object_shape(
                    schema,
                    &schema.query_root.clone(),
                    &name,
                    &query.selection_set,
                )?;
                out.push(Operation {
                    name: name.clone(),
                    text: operation_text(document_text, &name)?,
                    variables,
                    result,
                });
            }
            // `{ types { name } }` — GraphQL's shorthand, which parses as a
            // bare selection set rather than a query. It is a perfectly good
            // query and the only thing wrong with it is the missing name, so
            // it gets the missing-name message rather than the one about
            // mutations.
            Definition::Operation(OperationDefinition::SelectionSet(_)) => bail!(
                "this operation needs a name — it is what the generated type and function \
                 are called.\n  Write `query SomeName {{ … }}`."
            ),
            Definition::Operation(_) => {
                bail!(
                    "only queries are supported. A code model is read-only, so a mutation \
                     or subscription has nothing to reach."
                )
            }
            Definition::Fragment(f) => bail!(
                "named fragments are not supported (`fragment {}`). Inline them, or use \
                 `... on TypeName {{ … }}`, which is.",
                f.name
            ),
        }
    }
    if out.is_empty() {
        bail!("no query found in that document");
    }
    Ok(out)
}

/// The source text of one operation, so the emitted client carries the exact
/// query it was typed against.
fn operation_text(document_text: &str, name: &str) -> Result<String> {
    // Re-printed from the parse rather than sliced out of the file: a slice
    // would carry whatever comments and spacing surrounded it, and the point
    // is that the text and the types cannot disagree.
    let document: Document<'_, String> =
        graphql_parser::parse_query(document_text).map_err(|e| anyhow::anyhow!("{e}"))?;
    for definition in &document.definitions {
        if let Definition::Operation(OperationDefinition::Query(q)) = definition
            && q.name.as_deref() == Some(name)
        {
            return Ok(format!("{}", definition));
        }
    }
    bail!("operation {name} vanished between parses")
}

fn object_shape(
    schema: &Schema,
    type_name: &str,
    path: &str,
    selection_set: &SelectionSet<'_, String>,
) -> Result<ObjectShape> {
    let (common, variants) = split_selections(schema, type_name, path, selection_set)?;
    if !variants.is_empty() {
        bail!(
            "`{path}` selects on variants but is used where a single object is expected — \
             this is a bug in the resolver, not in your query"
        );
    }
    Ok(ObjectShape {
        name: path.to_string(),
        fields: common,
    })
}

/// The two halves of a selection set: fields on the type itself, and the
/// fields each `... on Variant` adds.
type Selections = (Vec<Field>, Vec<(String, Vec<Field>)>);

/// Walk one selection set, separating fields common to the type from those
/// under `... on Variant`.
fn split_selections(
    schema: &Schema,
    type_name: &str,
    path: &str,
    selection_set: &SelectionSet<'_, String>,
) -> Result<Selections> {
    let mut common = Vec::new();
    let mut variants: Vec<(String, Vec<Field>)> = Vec::new();

    for selection in &selection_set.items {
        match selection {
            Selection::Field(field) => {
                let key = field.alias.clone().unwrap_or_else(|| field.name.clone());
                // `__typename` is always available and always a string, and
                // is how a caller tells variants apart — so it is answered
                // here rather than looked up and not found.
                if field.name == "__typename" {
                    common.push(Field {
                        key,
                        shape: Shape {
                            nullable: false,
                            kind: Kind::Scalar("String".into()),
                        },
                        description: None,
                    });
                    continue;
                }
                let Some(declared) = schema.field_type(type_name, &field.name) else {
                    bail!(
                        "`{}` has no field `{}`.\n  Run the server with no query to print \
                         what it does have.",
                        type_name,
                        field.name
                    );
                };
                let shape = shape_of(
                    schema,
                    declared,
                    &format!("{path}_{key}"),
                    &field.selection_set,
                )?;
                common.push(Field {
                    key,
                    shape,
                    description: schema.field_description(type_name, &field.name),
                });
            }
            Selection::InlineFragment(fragment) => {
                let Some(on) = fragment.type_condition.as_ref().map(|c| match c {
                    graphql_parser::query::TypeCondition::On(name) => name.clone(),
                }) else {
                    bail!("an inline fragment without `on TypeName` is not supported");
                };
                let (fields, nested) = split_selections(
                    schema,
                    &on,
                    &format!("{path}_{on}"),
                    &fragment.selection_set,
                )?;
                if !nested.is_empty() {
                    bail!("nested variant selections inside `... on {on}` are not supported");
                }
                variants.push((on, fields));
            }
            Selection::FragmentSpread(spread) => bail!(
                "named fragments are not supported (`...{}`). Use `... on TypeName {{ … }}`.",
                spread.fragment_name
            ),
        }
    }
    Ok((common, variants))
}

/// The shape of one field's value.
fn shape_of(
    schema: &Schema,
    declared: &Value,
    path: &str,
    selection_set: &SelectionSet<'_, String>,
) -> Result<Shape> {
    match declared["kind"].as_str().unwrap_or("") {
        "NON_NULL" => {
            let inner = shape_of(schema, &declared["ofType"], path, selection_set)?;
            Ok(Shape {
                nullable: false,
                kind: inner.kind,
            })
        }
        "LIST" => {
            let inner = shape_of(schema, &declared["ofType"], path, selection_set)?;
            Ok(Shape {
                nullable: true,
                kind: Kind::List(Box::new(inner)),
            })
        }
        _ => {
            let Some(name) = named_of(declared) else {
                bail!("a field with no resolvable type at `{path}`");
            };
            let entry = schema.get(name);
            let kind = match entry.map(|e| e["kind"].as_str().unwrap_or("")) {
                Some("ENUM") => Kind::Enum {
                    name: name.to_string(),
                    values: entry
                        .and_then(|e| e["enumValues"].as_array())
                        .map(|vs| {
                            vs.iter()
                                .filter_map(|v| v["name"].as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                },
                Some("OBJECT") | Some("INTERFACE") | Some("UNION") => {
                    let (common, fragments) = split_selections(schema, name, path, selection_set)?;
                    if fragments.is_empty() {
                        Kind::Object(ObjectShape {
                            name: path.to_string(),
                            fields: common,
                        })
                    } else {
                        // Every type the interface can actually produce, not
                        // only those a fragment named.
                        let possible: Vec<String> = entry
                            .and_then(|e| e["possibleTypes"].as_array())
                            .map(|list| {
                                list.iter()
                                    .filter_map(|p| p["name"].as_str().map(str::to_string))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let possible = if possible.is_empty() {
                            fragments.iter().map(|(n, _)| n.clone()).collect()
                        } else {
                            possible
                        };
                        Kind::Variants(
                            possible
                                .into_iter()
                                .map(|type_name| {
                                    let mut fields = common.clone();
                                    if let Some((_, extra)) =
                                        fragments.iter().find(|(n, _)| *n == type_name)
                                    {
                                        fields.extend(extra.clone());
                                    }
                                    ObjectShape {
                                        name: format!("{path}_{type_name}"),
                                        fields,
                                    }
                                })
                                .collect(),
                        )
                    }
                }
                _ => Kind::Scalar(name.to_string()),
            };
            Ok(Shape {
                nullable: true,
                kind,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn schema() -> Schema {
        Schema::from_introspection(&json!({"data": {"__schema": {
            "queryType": {"name": "Query"},
            "types": [
                {"kind": "OBJECT", "name": "Query", "fields": [
                    {"name": "types", "description": "Every type.",
                     "type": {"kind": "NON_NULL", "ofType": {"kind": "LIST", "ofType":
                        {"kind": "NON_NULL", "ofType": {"kind": "INTERFACE", "name": "TypeDecl"}}}}}]},
                {"kind": "INTERFACE", "name": "TypeDecl",
                 "possibleTypes": [{"name": "ClassDecl"}, {"name": "RecordDecl"},
                                   {"name": "EnumDecl"}],
                 "fields": [{"name": "name", "type": {"kind": "NON_NULL",
                    "ofType": {"kind": "SCALAR", "name": "String"}}}]},
                {"kind": "OBJECT", "name": "ClassDecl", "fields": [
                    {"name": "name", "type": {"kind": "NON_NULL", "ofType": {"kind": "SCALAR", "name": "String"}}},
                    {"name": "isSealed", "type": {"kind": "NON_NULL", "ofType": {"kind": "SCALAR", "name": "Boolean"}}}]},
                {"kind": "OBJECT", "name": "RecordDecl", "fields": [
                    {"name": "name", "type": {"kind": "NON_NULL", "ofType": {"kind": "SCALAR", "name": "String"}}}]},
                {"kind": "OBJECT", "name": "EnumDecl", "fields": [
                    {"name": "name", "type": {"kind": "NON_NULL", "ofType": {"kind": "SCALAR", "name": "String"}}}]}
            ]}}}))
        .unwrap()
    }

    /// An interface selection produces EVERY possible type, not only the ones
    /// a fragment named.
    ///
    /// Selecting `types { name }` returns EnumDecls whether or not anyone
    /// wrote `... on EnumDecl`, so a union of the fragment-covered types is a
    /// lie the compiler cannot catch and the runtime will. The first version
    /// of this listed two of three, and the TypeScript it generated did not
    /// compile.
    #[test]
    fn a_variant_selection_lists_every_possible_type() {
        let ops = resolve(
            &schema(),
            "query Q { types { name ... on ClassDecl { isSealed } } }",
        )
        .unwrap();
        let Kind::List(item) = &ops[0].result.fields[0].shape.kind else {
            panic!("types should be a list");
        };
        let Kind::Variants(variants) = &item.kind else {
            panic!(
                "a fragment makes this a variant selection, got {:?}",
                item.kind
            );
        };
        let names: Vec<&str> = variants.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Q_types_ClassDecl",
                "Q_types_RecordDecl",
                "Q_types_EnumDecl"
            ]
        );
        // The interface's own fields are on every variant, and the fragment's
        // are only on its own.
        let sealed = |v: &ObjectShape| v.fields.iter().any(|f| f.key == "isSealed");
        assert!(sealed(&variants[0]), "ClassDecl selected isSealed");
        assert!(!sealed(&variants[1]), "RecordDecl did not");
        assert!(
            variants
                .iter()
                .all(|v| v.fields.iter().any(|f| f.key == "name"))
        );
    }

    #[test]
    fn an_unknown_field_is_refused_by_name() {
        let err = resolve(&schema(), "query Q { types { nombre } }").unwrap_err();
        let text = format!("{err:#}");
        assert!(text.contains("has no field `nombre`"), "{text}");
        assert!(
            text.contains("print"),
            "the message should say how to find out: {text}"
        );
    }

    /// The things a generator's query will not have, refused rather than
    /// silently dropped — because a dropped selection is a field the caller
    /// expects and the response never carries.
    #[test]
    fn unsupported_graphql_is_named_rather_than_ignored() {
        for (source, expected) in [
            (
                "query Q { types { ...Bits } } fragment Bits on TypeDecl { name }",
                "fragment",
            ),
            ("mutation M { types { name } }", "read-only"),
            ("{ types { name } }", "needs a name"),
        ] {
            let err = resolve(&schema(), source).unwrap_err();
            let text = format!("{err:#}");
            assert!(text.contains(expected), "expected {expected:?} in: {text}");
        }
    }
}
