//! `hick-model-rust <root>` — the Rust code model, served as GraphQL.
//!
//! One JSON object per line on stdin, one per line on stdout — the framing
//! every model server in `code-models/` uses, and the same one `hick mcp`
//! uses, because a second framing in one product is a second thing to get
//! wrong. Diagnostics go to stderr; stdout carries the protocol and nothing
//! else.
//!
//! ```text
//! → {"query": "{ types(deriving: \"Describe\") { name } }"}
//! ← {"data":{"types":[{"name":"Order"}]}}
//! ```
//!
//! Standard GraphQL introspection is supported and is how the schema
//! documents itself: `hick code-model rust <root>` with no `--query` prints
//! it, and `--sdl` prints it in the form every ecosystem's codegen reads.

mod load;
mod lower;
mod model;
mod schema;
mod syntax;

use std::io::{BufRead, Write};
use std::process::ExitCode;
use std::sync::Arc;

use async_graphql::{EmptyMutation, EmptySubscription, Request, Schema, Variables};
use futures_executor::block_on;

fn main() -> ExitCode {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let path = std::path::Path::new(&root);
    if !path.is_dir() {
        eprintln!("hick-model-rust: no such directory: {root}");
        eprintln!("  Pass the workspace root to model, e.g. `hick-model-rust .`.");
        eprintln!("  It must be, or sit under, a directory holding a Cargo.toml.");
        return ExitCode::from(2);
    }

    let model = match load::load(path) {
        Ok(model) => Arc::new(model),
        Err(message) => {
            eprintln!("hick-model-rust: {message}");
            return ExitCode::from(2);
        }
    };

    eprintln!(
        "hick-model-rust: {} package(s), {} module(s), {} file(s) under {}; ready",
        model.packages.len(),
        model.modules.len(),
        model.file_count,
        model.root.display()
    );
    // The first few, and a count for the rest: a workspace with forty
    // dependencies would otherwise bury the one line that matters, and
    // saying nothing would break the rule this field exists for.
    for gap in model.unresolved.iter().take(5) {
        eprintln!("  unresolved: {}", gap.message);
    }
    if model.unresolved.len() > 5 {
        eprintln!(
            "  … and {} more; ask `{{ unresolved {{ message }} }}` for all of them",
            model.unresolved.len() - 5
        );
    }

    let schema = Schema::build(schema::Query, EmptyMutation, EmptySubscription)
        .data(Arc::clone(&model))
        .finish();

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("hick-model-rust: could not read stdin: {error}");
                return ExitCode::from(2);
            }
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let body = answer(&schema, line);
        // One line, always: the framing has no length prefix, so a newline
        // inside a reply would be read as the end of it.
        let _ = writeln!(stdout, "{}", body.replace('\n', " "));
        let _ = stdout.flush();
    }
    ExitCode::SUCCESS
}

type ModelSchema = Schema<schema::Query, EmptyMutation, EmptySubscription>;

/// One request, answered — including the malformed ones.
///
/// A bad request must not end the session. The caller is a generator being
/// written, and being written means getting it wrong; a server that exited on
/// the first typo would make the interactive case the one it serves worst.
fn answer(schema: &ModelSchema, line: &str) -> String {
    #[derive(serde::Deserialize)]
    struct Incoming {
        query: String,
        #[serde(default)]
        variables: Option<serde_json::Value>,
    }
    let incoming: Incoming = match serde_json::from_str(line) {
        Ok(incoming) => incoming,
        Err(error) => {
            return serde_json::json!({
                "errors": [{
                    "message": format!(
                        "not a request: {error}. Send one JSON object per line, \
                         shaped {{\"query\": \"…\", \"variables\": {{…}}}}."
                    )
                }]
            })
            .to_string();
        }
    };
    let mut request = Request::new(incoming.query);
    if let Some(variables) = incoming.variables {
        request = request.variables(Variables::from_json(variables));
    }
    let response = block_on(schema.execute(request));
    serde_json::to_string(&response).unwrap_or_else(|error| {
        serde_json::json!({ "errors": [{ "message": error.to_string() }] }).to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_over(source: &str) -> (tempdir::Dir, ModelSchema) {
        let dir = tempdir::Dir::new();
        dir.write("Cargo.toml", "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n");
        dir.write("src/lib.rs", source);
        let model = Arc::new(load::load(dir.path()).expect("the probe workspace loads"));
        let schema = Schema::build(schema::Query, EmptyMutation, EmptySubscription)
            .data(model)
            .finish();
        (dir, schema)
    }

    fn ask(schema: &ModelSchema, query: &str) -> serde_json::Value {
        let response = block_on(schema.execute(query));
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        serde_json::to_value(&response).unwrap()["data"].clone()
    }

    #[test]
    fn a_derive_is_findable_and_a_tuple_struct_stays_a_tuple_struct() {
        // The first question every Rust generator asks, and the one lowering
        // a tuple struct into a named one would silently ruin.
        let (_dir, schema) = schema_over(
            "#[derive(Clone, Describe)]\npub struct Order { pub sku: String }\n\
             #[derive(Describe)]\npub struct Sku(pub String);\n\
             pub struct Untouched;\n",
        );
        let data = ask(
            &schema,
            r#"{ types(deriving: "Describe") { name __typename
                 ... on StructDecl { shape fields { name type { text } } } } }"#,
        );
        let types = data["types"].as_array().unwrap();
        assert_eq!(types.len(), 2, "{types:#?}");
        assert_eq!(types[0]["shape"], "NAMED");
        assert_eq!(types[0]["fields"][0]["type"]["text"], "String");
        assert_eq!(types[1]["shape"], "TUPLE");
        // A tuple field's name is its index, which is what Rust calls it.
        assert_eq!(types[1]["fields"][0]["name"], "0");
    }

    #[test]
    fn an_impl_block_says_which_trait_it_is_for() {
        // The fact folding methods onto a type would lose.
        let (_dir, schema) = schema_over(
            "pub struct Order;\nimpl Order { pub fn new() -> Self { Order } }\n\
             impl std::fmt::Display for Order {\n  \
             fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }\n}\n",
        );
        let data = ask(
            &schema,
            r#"{ inherent: impls(inherentOnly: true) { name methods { name } }
                 shown: impls(ofTrait: "Display") { ofTrait selfType { name } } }"#,
        );
        assert_eq!(data["inherent"].as_array().unwrap().len(), 1);
        assert_eq!(data["inherent"][0]["methods"][0]["name"], "new");
        assert_eq!(data["shown"][0]["ofTrait"], "std::fmt::Display");
        assert_eq!(data["shown"][0]["selfType"]["name"], "Order");
    }

    #[test]
    fn a_module_is_reached_through_its_declaration_and_not_the_directory() {
        // `mod` decides what is compiled. An undeclared file is not part of
        // the crate, and a model that read it would answer about code that
        // does not exist.
        let dir = tempdir::Dir::new();
        dir.write("Cargo.toml", "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n");
        dir.write(
            "src/lib.rs",
            "pub mod api;\n#[cfg(test)]\nmod tests { pub struct Fixture; }\n",
        );
        dir.write("src/api.rs", "pub fn route() {}\nfn private() {}\n");
        dir.write("src/orphan.rs", "pub struct NeverCompiled;\n");
        let model = Arc::new(load::load(dir.path()).unwrap());
        let schema = Schema::build(schema::Query, EmptyMutation, EmptySubscription)
            .data(model)
            .finish();
        let data = ask(
            &schema,
            r#"{ public: functions(inModule: "crate::api", publicOnly: true) { name }
                 orphan: types(named: "NeverCompiled") { name }
                 gated: types(named: "Fixture") { cfg } }"#,
        );
        assert_eq!(data["public"].as_array().unwrap().len(), 1);
        assert_eq!(data["public"][0]["name"], "route");
        assert!(
            data["orphan"].as_array().unwrap().is_empty(),
            "a file no `mod` declares is not part of the crate"
        );
        // A `cfg` on the module above is inherited: nothing in the item's own
        // attributes says it is conditional, and a generator emitting it into
        // a release build would be wrong.
        assert_eq!(data["gated"][0]["cfg"][0], "test");
    }

    #[test]
    fn an_external_crate_is_named_rather_than_dropped() {
        // `code-models/README.md`'s rule: a generator that ran against a
        // half-resolved workspace and believed it saw everything emits
        // confidently wrong code.
        let (_dir, schema) = schema_over("use serde::Serialize;\npub struct Order;\n");
        let data = ask(&schema, "{ unresolved { message } }");
        let messages = data["unresolved"].as_array().unwrap();
        assert!(
            messages
                .iter()
                .any(|m| m["message"].as_str().unwrap_or("").contains("`serde`")),
            "{messages:#?}"
        );
    }

    #[test]
    fn a_malformed_request_is_answered_rather_than_fatal() {
        let (_dir, schema) = schema_over("pub struct Order;\n");
        let body = answer(&schema, "{ not json");
        assert!(body.contains("not a request"), "{body}");
        // And the session survives it.
        let body = answer(&schema, r#"{"query":"{ version { language } }"}"#);
        assert!(body.contains("rust"), "{body}");
    }

    /// A throwaway workspace on disk. `cargo metadata` needs real files.
    mod tempdir {
        use std::path::{Path, PathBuf};

        pub struct Dir(PathBuf);

        impl Dir {
            pub fn new() -> Self {
                let base = std::env::temp_dir().join(format!(
                    "hick-model-rust-{}-{}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                ));
                std::fs::create_dir_all(&base).unwrap();
                Self(base)
            }

            pub fn path(&self) -> &Path {
                &self.0
            }

            pub fn write(&self, relative: &str, body: &str) {
                let path = self.0.join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, body).unwrap();
            }
        }

        impl Drop for Dir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }
}
