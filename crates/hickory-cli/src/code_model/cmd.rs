//! `hick code-model` — the command, next to the thing it drives.
//!
//! Lifted out of `main.rs` when the model catalogue arrived, for the reason
//! `scripts/check-file-length.sh` exists: a 4.4k-line `main.rs` is the
//! architecture by default, and every command that stays in it makes the
//! next one harder to find. The argument struct comes with it — a command's
//! flags and its behaviour are one thing to read.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};

#[derive(clap::Args)]
pub struct CodeModelArgs {
    /// The language, e.g. `csharp`.
    pub language: String,
    /// The source root to model.
    #[arg(default_value = ".")]
    pub source: PathBuf,
    /// The GraphQL query. Omit to print the schema instead, which is how a
    /// person — or a model writing a generator — finds out what to ask.
    #[arg(long)]
    pub query: Option<String>,
    /// Generate a typed client for the queries in this file.
    ///
    /// The types and the query text are emitted together, so they cannot
    /// drift the way a hand-written struct beside a hand-written query does.
    #[arg(long)]
    pub client: Option<PathBuf>,
    /// The language to generate the client IN — which is the language the
    /// GENERATOR is written in, not the one being modelled.
    #[arg(long, default_value = "python")]
    pub target: String,
    /// Where to write it (default: stdout).
    #[arg(long = "out", short)]
    pub out: Option<PathBuf>,
    /// Write the client RUNTIME for `--target` instead of a client: the
    /// plumbing that talks to a model server, which every generator would
    /// otherwise vendor a copy of.
    #[arg(long)]
    pub runtime: bool,
    /// Build this language's model server and install it into the project's
    /// `.hick-cache/models/`, so `hick code-model` and `hick lang` find it.
    ///
    /// It is BUILT rather than fetched: a model server is not published
    /// anywhere, because there is nowhere to publish four binaries in four
    /// ecosystems to. So this needs a hickory-docs checkout — `--from` says
    /// which one, and the default is the working directory.
    #[arg(long)]
    pub install: bool,
    /// The hickory-docs checkout holding `code-models/`, for `--install`.
    #[arg(long)]
    pub from: Option<PathBuf>,
    /// Print the schema as SDL rather than as a summary.
    ///
    /// This is the form every ecosystem's GraphQL codegen reads — `genqlient`,
    /// `graphql-codegen`, StrawberryShake — so a generator's author who wants
    /// a typed client generates one with the tool their language already has,
    /// instead of waiting for Hickory to grow one per language.
    #[arg(long)]
    pub sdl: bool,
}

/// `hick code-model <language> [source] --query …`
pub fn run(args: CodeModelArgs) -> Result<ExitCode> {
    use super::{INTROSPECT, ModelServer};
    use crate::typed_client::emit;

    let parse_target = |name: &str| -> Result<emit::Target> {
        emit::Target::parse(name).ok_or_else(|| {
            anyhow::anyhow!(
                "`{name}` is not a language this can generate a client in.\n  Available: {}",
                emit::Target::names()
            )
        })
    };

    // The runtime needs no model server: it is the same bytes whatever
    // language is being modelled, which is the point of it.
    if args.runtime {
        let target = parse_target(&args.target)?;
        let Some((name, source)) = target.runtime() else {
            anyhow::bail!(
                "there is no client runtime for {} yet.\n  \
                 `--client` still emits types for it; sending them is your \
                 ecosystem's GraphQL client's job.\n  \
                 Runtimes ship for: python, csharp, rust",
                args.target
            );
        };
        match args.out.as_deref() {
            // A directory is the common case in a document — the cell has an
            // output volume and wants the library's own name inside it, which
            // is what the emitted client imports.
            Some(dir) if dir.is_dir() => {
                let path = dir.join(name);
                std::fs::write(&path, source)
                    .with_context(|| format!("could not write {}", path.display()))?;
                // Stdout, not stderr: writing to a file leaves stdout free,
                // and a document that pins what its generator produced needs
                // this line in the transcript.
                println!(
                    "wrote {} — the {} client runtime",
                    path.display(),
                    args.target
                );
            }
            Some(path) => {
                std::fs::write(path, source)
                    .with_context(|| format!("could not write {}", path.display()))?;
                // Stdout, not stderr: writing to a file leaves stdout free,
                // and a document that pins what its generator produced needs
                // this line in the transcript.
                println!(
                    "wrote {} — the {} client runtime",
                    path.display(),
                    args.target
                );
            }
            None => print!("{source}"),
        }
        return Ok(ExitCode::SUCCESS);
    }

    let root = std::env::current_dir()?;

    if args.install {
        let checkout = args.from.clone().unwrap_or_else(|| root.clone());
        let installed = super::install::install(&root, &checkout, &args.language)?;
        println!("installed {}", installed.display());
        println!(
            "  `hick lang` reports {} as Gold once this and a typed-client emitter both exist.",
            args.language
        );
        return Ok(ExitCode::SUCCESS);
    }

    let mut server = ModelServer::start(&args.language, &args.source, &root)?;

    if let Some(queries) = args.client.as_deref() {
        use crate::typed_client::{Schema, resolve};

        let target = parse_target(&args.target)?;
        let document = std::fs::read_to_string(queries)
            .with_context(|| format!("could not read {}", queries.display()))?;
        let schema = Schema::from_introspection(&server.query(INTROSPECT, None)?)?;
        let operations = resolve(&schema, &document)?;
        let source = emit::emit(target, &operations, &args.language);
        match args.out.as_deref() {
            Some(path) => {
                std::fs::write(path, &source)
                    .with_context(|| format!("could not write {}", path.display()))?;
                println!(
                    "wrote {} — {} operation(s), typed against the {} model",
                    path.display(),
                    operations.len(),
                    args.language
                );
            }
            None => print!("{source}"),
        }
        return Ok(ExitCode::SUCCESS);
    }

    let Some(query) = args.query.as_deref() else {
        // No query: print what CAN be asked. Introspection is the whole
        // discoverability story — a generator's author should never have to
        // guess at fields.
        let schema = server.query(INTROSPECT, None)?;
        if args.sdl {
            print!("{}", super::to_sdl(&schema));
        } else {
            print_schema(&schema);
        }
        return Ok(ExitCode::SUCCESS);
    };

    let answer = server.query(query, None)?;
    println!("{}", serde_json::to_string_pretty(&answer)?);
    Ok(ExitCode::SUCCESS)
}

/// The schema, as something a person reads rather than a JSON dump.
fn print_schema(schema: &serde_json::Value) {
    let Some(types) = schema
        .pointer("/data/__schema/types")
        .and_then(|t| t.as_array())
    else {
        println!("{schema}");
        return;
    };
    for entry in types {
        let name = entry["name"].as_str().unwrap_or("");
        // Introspection returns GraphQL's own machinery too; a person asking
        // what they can query does not want `__Directive`.
        if name.starts_with("__") || entry["kind"] == "SCALAR" {
            continue;
        }
        let kind = entry["kind"].as_str().unwrap_or("");
        println!("{kind} {name}");
        if let Some(possible) = entry["possibleTypes"].as_array()
            && !possible.is_empty()
        {
            let names: Vec<&str> = possible.iter().filter_map(|p| p["name"].as_str()).collect();
            println!("    one of: {}", names.join(" | "));
        }
        if let Some(fields) = entry["fields"].as_array() {
            for field in fields {
                let ty = super::type_name(&field["type"]);
                println!("    {:<24} {ty}", field["name"].as_str().unwrap_or(""));
            }
        }
        println!();
    }
}
