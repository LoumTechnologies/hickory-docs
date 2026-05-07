use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "hick-query-edit",
    about = "Query and edit hick documents and generated output"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse a .hick file and output JSON AST
    Parse {
        /// Path to the .hick file
        file: String,
    },
    /// Query structural elements in generated output
    Query {
        /// Selector (e.g. "fn greet", "struct Config", "class App")
        selector: String,
        /// Path to the generated output file
        #[arg(long)]
        file: String,
        /// Path to the provenance JSON file
        #[arg(long)]
        provenance: String,
    },
    /// Display structural summary of generated output
    Structure {
        /// Path to the generated output file
        file: String,
        /// Path to the provenance JSON file
        #[arg(long)]
        provenance: String,
    },
    /// Get provenance info for a line range in generated output
    Provenance {
        /// Path to the generated output file
        file: String,
        /// Start line (1-based)
        #[arg(long)]
        start: usize,
        /// End line (1-based, inclusive)
        #[arg(long)]
        end: usize,
        /// Path to the provenance JSON file
        #[arg(long)]
        provenance: String,
    },
    /// Compute source edits to replace a selector match with new content
    ComputeEdits {
        /// Path to the generated output file
        file: String,
        /// Selector to match (e.g. "fn greet")
        #[arg(long)]
        selector: String,
        /// New content to replace the matched element
        #[arg(long)]
        new_content: String,
        /// Path to the provenance JSON file
        #[arg(long)]
        provenance: String,
    },
    /// Apply computed edits to .hick source files
    ApplyEdits {
        /// Path to the edits JSON file
        #[arg(long)]
        edits: String,
        /// Root directory containing .hick source files
        #[arg(long)]
        hick_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), hick_query_edit::Error> {
    match cli.command {
        Command::Parse { file } => {
            let output = hick_query_edit::parse_hick(&file)?;
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        Command::Query {
            selector,
            file,
            provenance,
        } => {
            let matches = hick_query_edit::query(&selector, &file, &provenance)?;
            println!("{}", serde_json::to_string_pretty(&matches).unwrap());
        }
        Command::Structure { file, provenance } => {
            let summary = hick_query_edit::structure(&file, &provenance)?;
            println!("{summary}");
        }
        Command::Provenance {
            file,
            start,
            end,
            provenance,
        } => {
            let spans = hick_query_edit::provenance(&file, start, end, &provenance)?;
            println!("{}", serde_json::to_string_pretty(&spans).unwrap());
        }
        Command::ComputeEdits {
            file,
            selector,
            new_content,
            provenance,
        } => {
            let edits =
                hick_query_edit::compute_edits_cmd(&file, &selector, &new_content, &provenance)?;
            println!("{}", serde_json::to_string_pretty(&edits).unwrap());
        }
        Command::ApplyEdits { edits, hick_dir } => {
            hick_query_edit::apply_edits(&edits, &hick_dir)?;
        }
    }
    Ok(())
}
