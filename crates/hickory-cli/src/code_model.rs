//! Talking to a language's code model server — the second half of Gold.
//!
//! A code model server answers questions about source code in that language's
//! own terms: a record is a record, an attribute is an attribute, and a
//! partial class is one type with several declarations. It exists so a script
//! can GENERATE against your code instead of parsing it, which is the
//! difference between reviewing a generator and reviewing its output.
//!
//! ## Why it is a separate process
//!
//! The same reason a language server and an indexer are. Each language's
//! model can only be built on that ecosystem's own compiler front end —
//! Roslyn, the TypeScript compiler, `go/types` — and none of those is a Rust
//! crate. The boundary is also a licence firewall: libclang, Sorbet and the
//! Kotlin Analysis API carry their own terms and none of them ever links into
//! this product.
//!
//! ## Why GraphQL
//!
//! Code is a graph, and a selection set is laziness: binding a symbol is the
//! expensive part, and a generator that wants property names must not pay for
//! method bodies. Interfaces and unions carry a language's own type system
//! without lowering it. And introspection means the schema documents itself —
//! which is what lets a model *discover* what it can ask instead of guessing
//! at an API it half-remembers.
//!
//! Read only, deliberately. Writes need ordering, atomicity and formatting,
//! all of which GraphQL handles badly, and a document already owns its bytes
//! through `hick:file` where lineage and the drift gate work.
//!
//! See `docs/specs/freeform/language-tiers.md`.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

/// Where an installed model server lives, relative to the project root.
pub const MODELS_DIR: &str = ".hick-cache/models";

/// The server binary for `language`, if this machine has one.
///
/// The project's own copy first, then `PATH`: the same order every other
/// spawned tool uses, so a project can pin a version without asking anyone to
/// change their environment.
pub fn discover(language: &str, root: &Path) -> Option<PathBuf> {
    let name = format!("hick-model-{language}");
    let local = root.join(MODELS_DIR).join(&name);
    if local.is_file() {
        return Some(local);
    }
    which(&name)
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// A running model server, one query at a time.
///
/// Held open across a generation pass on purpose: loading a compilation is
/// the whole cost, and a server per query would pay it per query. One at a
/// time because the protocol is line-delimited and interleaving two replies
/// on one pipe has no framing to recover from.
#[derive(Debug)]
pub struct ModelServer {
    child: Child,
    language: String,
}

impl ModelServer {
    /// Spawn the server for `language`, rooted at `source`.
    pub fn start(language: &str, source: &Path, root: &Path) -> Result<Self> {
        let binary = discover(language, root).with_context(|| {
            format!(
                "no code model server for {language}.\n  \
                 A model server answers questions about your source so a script can \
                 generate from it.\n  \
                 `hick lang` shows which languages have one; none ships yet, and \
                 `code-models/csharp` in the hickory-docs checkout builds the first."
            )
        })?;
        let child = Command::new(&binary)
            .arg(source)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // The server's own diagnostics — how many files it read, what it
            // could not bind — go to the terminal rather than into the
            // protocol, exactly as `hick mcp` keeps stdout for the protocol.
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("could not start {}", binary.display()))?;
        Ok(Self {
            child,
            language: language.to_string(),
        })
    }

    /// Ask one question.
    pub fn query(&mut self, query: &str, variables: Option<Value>) -> Result<Value> {
        let request = match variables {
            Some(vars) => json!({ "query": query, "variables": vars }),
            None => json!({ "query": query }),
        };
        {
            let stdin = self
                .child
                .stdin
                .as_mut()
                .context("the model server's stdin closed")?;
            writeln!(stdin, "{request}")?;
            stdin.flush()?;
        }
        let stdout = self
            .child
            .stdout
            .as_mut()
            .context("the model server's stdout closed")?;
        let mut line = String::new();
        BufReader::new(stdout)
            .read_line(&mut line)
            .context("the model server sent no reply")?;
        if line.trim().is_empty() {
            bail!(
                "the {} model server closed without answering — see its output above",
                self.language
            );
        }
        let value: Value = serde_json::from_str(&line)
            .with_context(|| format!("the model server sent something that is not JSON: {line}"))?;

        // GraphQL reports failure inside a 200-shaped body, so an unchecked
        // caller would treat "you asked for a field that does not exist" as
        // an empty result and generate nothing, silently. Errors are raised.
        if let Some(errors) = value.get("errors").and_then(Value::as_array)
            && !errors.is_empty()
        {
            let messages: Vec<String> = errors
                .iter()
                .filter_map(|e| e.get("message").and_then(Value::as_str))
                .map(str::to_string)
                .collect();
            bail!(
                "the {} model server refused the query:\n  {}",
                self.language,
                messages.join("\n  ")
            );
        }
        Ok(value)
    }
}

impl Drop for ModelServer {
    fn drop(&mut self) {
        // Closing stdin is how the server is asked to stop; killing is the
        // fallback for one that does not.
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

/// The introspection query, which is how a schema documents itself.
///
/// Kept here rather than asked of the caller because "what can I ask?" is the
/// question a person or a model has first, and making them write 40 lines of
/// GraphQL to find out defeats the point.
pub const INTROSPECT: &str = r#"
{
  __schema {
    queryType { name }
    types {
      kind
      name
      description
      interfaces { name }
      enumValues { name }
      inputFields { name type { ...ref ofType { ...ref ofType { ...ref } } } }
      fields {
        name
        description
        args { name type { ...ref ofType { ...ref ofType { ...ref } } } }
        type { ...ref ofType { ...ref ofType { ...ref ofType { ...ref } } } }
      }
      possibleTypes { name }
    }
  }
}
fragment ref on __Type { kind name }
"#;

/// Render a GraphQL type reference the way it is written in a query.
///
/// Introspection returns `[Property!]!` as three nested wrappers with no
/// names on the outer two, so a printer that reads only the first level shows
/// `?` for every list — which is exactly the field a generator's author most
/// needs to see.
pub fn type_name(node: &Value) -> String {
    match node.get("kind").and_then(Value::as_str) {
        Some("NON_NULL") => match node.get("ofType") {
            Some(inner) => format!("{}!", type_name(inner)),
            None => "?".to_string(),
        },
        Some("LIST") => match node.get("ofType") {
            Some(inner) => format!("[{}]", type_name(inner)),
            None => "?".to_string(),
        },
        _ => node
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string(),
    }
}

/// Print a schema as SDL, from its introspection response.
///
/// Written once here rather than once per server, because it is the same
/// transformation everywhere and the shared core of this design is the
/// protocol rather than the schema.
///
/// SDL is what makes the untyped client a **choice** instead of a limitation.
/// A generator's author who wants completions and compile-time checking of
/// field names does not need Hickory to grow a typed client per language:
/// every Gold language's ecosystem already has GraphQL codegen — `genqlient`,
/// `graphql-codegen`, StrawberryShake — and all of them take SDL. Emitting it
/// is the whole of Hickory's job here, and the reason not to build the rest.
pub fn to_sdl(introspection: &Value) -> String {
    let Some(schema) = introspection.pointer("/data/__schema") else {
        return String::new();
    };
    let empty = Vec::new();
    let types = schema["types"].as_array().unwrap_or(&empty);
    let query_type = schema
        .pointer("/queryType/name")
        .and_then(Value::as_str)
        .unwrap_or("Query");

    let mut out = format!("schema {{\n  query: {query_type}\n}}\n");
    for entry in types {
        let name = entry["name"].as_str().unwrap_or("");
        // GraphQL's own machinery, and the built-in scalars: a codegen tool
        // supplies both and a duplicate definition is an error.
        if name.starts_with("__") || matches!(name, "String" | "Int" | "Float" | "Boolean" | "ID") {
            continue;
        }
        out.push('\n');
        if let Some(doc) = entry["description"].as_str().filter(|d| !d.is_empty()) {
            out.push_str(&describe(doc, ""));
        }
        match entry["kind"].as_str().unwrap_or("") {
            "OBJECT" | "INTERFACE" => {
                let keyword = if entry["kind"] == "INTERFACE" {
                    "interface"
                } else {
                    "type"
                };
                let implements: Vec<&str> = entry["interfaces"]
                    .as_array()
                    .map(|list| list.iter().filter_map(|i| i["name"].as_str()).collect())
                    .unwrap_or_default();
                out.push_str(&format!("{keyword} {name}"));
                if !implements.is_empty() {
                    out.push_str(&format!(" implements {}", implements.join(" & ")));
                }
                out.push_str(" {\n");
                for field in entry["fields"].as_array().unwrap_or(&empty) {
                    if let Some(doc) = field["description"].as_str().filter(|d| !d.is_empty()) {
                        out.push_str(&describe(doc, "  "));
                    }
                    let args: Vec<String> = field["args"]
                        .as_array()
                        .map(|list| {
                            list.iter()
                                .map(|a| {
                                    format!(
                                        "{}: {}",
                                        a["name"].as_str().unwrap_or(""),
                                        type_name(&a["type"])
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let arglist = if args.is_empty() {
                        String::new()
                    } else {
                        format!("({})", args.join(", "))
                    };
                    out.push_str(&format!(
                        "  {}{arglist}: {}\n",
                        field["name"].as_str().unwrap_or(""),
                        type_name(&field["type"])
                    ));
                }
                out.push_str("}\n");
            }
            "ENUM" => {
                out.push_str(&format!("enum {name} {{\n"));
                for value in entry["enumValues"].as_array().unwrap_or(&empty) {
                    out.push_str(&format!("  {}\n", value["name"].as_str().unwrap_or("")));
                }
                out.push_str("}\n");
            }
            "INPUT_OBJECT" => {
                out.push_str(&format!("input {name} {{\n"));
                for field in entry["inputFields"].as_array().unwrap_or(&empty) {
                    out.push_str(&format!(
                        "  {}: {}\n",
                        field["name"].as_str().unwrap_or(""),
                        type_name(&field["type"])
                    ));
                }
                out.push_str("}\n");
            }
            "UNION" => {
                let members: Vec<&str> = entry["possibleTypes"]
                    .as_array()
                    .map(|l| l.iter().filter_map(|p| p["name"].as_str()).collect())
                    .unwrap_or_default();
                out.push_str(&format!("union {name} = {}\n", members.join(" | ")));
            }
            "SCALAR" => out.push_str(&format!("scalar {name}\n")),
            _ => {}
        }
    }
    out
}

/// A description as an SDL block string, indented.
///
/// Block-quoted rather than escaped: these carry the reasoning a schema is
/// worth reading for, and they contain quotes and newlines.
fn describe(doc: &str, indent: &str) -> String {
    let body = doc.replace('"', "'");
    let mut out = format!("{indent}\"\"\"\n");
    for line in body.lines() {
        out.push_str(&format!("{indent}{line}\n"));
    }
    out.push_str(&format!("{indent}\"\"\"\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdl_carries_arguments_implements_and_descriptions() {
        // The three things a codegen tool cannot work without, and the three
        // the first version of this printer silently dropped: a field's
        // arguments, what an object implements, and the prose that makes the
        // schema worth reading.
        let introspection = json!({"data": {"__schema": {
        "queryType": {"name": "Query"},
        "types": [
            {"kind": "OBJECT", "name": "Query", "interfaces": [], "fields": [
                {"name": "types", "description": "Every type.",
                 "args": [{"name": "nameEndsWith",
                           "type": {"kind": "SCALAR", "name": "String"}}],
                 "type": {"kind": "NON_NULL", "ofType": {"kind": "LIST",
                          "ofType": {"kind": "NON_NULL",
                                     "ofType": {"kind": "INTERFACE", "name": "TypeDecl"}}}}}
            ]},
            {"kind": "OBJECT", "name": "ClassDecl",
             "interfaces": [{"name": "TypeDecl"}], "fields": [
                {"name": "name", "type": {"kind": "NON_NULL",
                    "ofType": {"kind": "SCALAR", "name": "String"}}}]},
            {"kind": "ENUM", "name": "Accessibility",
             "enumValues": [{"name": "PUBLIC"}, {"name": "PRIVATE"}]},
            // GraphQL's own machinery and the built-in scalars must not be
            // re-declared: a codegen tool supplies both and a duplicate
            // definition is an error rather than a warning.
            {"kind": "OBJECT", "name": "__Type", "fields": []},
            {"kind": "SCALAR", "name": "String"}
        ]}}});

        let sdl = to_sdl(&introspection);
        assert!(
            sdl.contains("types(nameEndsWith: String): [TypeDecl!]!"),
            "{sdl}"
        );
        assert!(
            sdl.contains("type ClassDecl implements TypeDecl {"),
            "{sdl}"
        );
        assert!(sdl.contains("Every type."), "{sdl}");
        assert!(sdl.contains("enum Accessibility {"), "{sdl}");
        assert!(
            !sdl.contains("__Type"),
            "GraphQL's own types must not be emitted: {sdl}"
        );
        assert!(
            !sdl.contains("scalar String"),
            "built-in scalars must not be re-declared: {sdl}"
        );
    }

    #[test]
    fn a_list_type_reads_as_a_list() {
        // `[Property!]!` arrives as three nested wrappers, the outer two
        // unnamed. Printing the first level shows `?`, which hides exactly
        // the fields a generator needs most.
        let node = json!({
            "kind": "NON_NULL",
            "name": null,
            "ofType": {
                "kind": "LIST",
                "name": null,
                "ofType": { "kind": "NON_NULL", "name": null,
                            "ofType": { "kind": "OBJECT", "name": "Property" } }
            }
        });
        assert_eq!(type_name(&node), "[Property!]!");
        assert_eq!(
            type_name(&json!({ "kind": "OBJECT", "name": "TypeRef" })),
            "TypeRef"
        );
    }

    #[test]
    fn a_missing_server_says_what_a_model_server_is_for() {
        let dir = tempfile::tempdir().unwrap();
        // A language nobody will ever have a server for, so this cannot pass
        // by accident on a machine that happens to have one installed.
        let err = ModelServer::start("nonesuch", dir.path(), dir.path()).unwrap_err();
        let text = format!("{err:#}");
        assert!(text.contains("no code model server for nonesuch"), "{text}");
        assert!(text.contains("hick lang"), "{text}");
    }

    #[test]
    fn a_projects_own_server_wins_over_one_on_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join(MODELS_DIR);
        std::fs::create_dir_all(&local).unwrap();
        let binary = local.join("hick-model-fictional");
        std::fs::write(&binary, "#!/bin/sh\n").unwrap();
        assert_eq!(
            discover("fictional", dir.path()).as_deref(),
            Some(binary.as_path()),
            "a pinned server must beat whatever is on PATH"
        );
    }
}
