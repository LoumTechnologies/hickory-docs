<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Freeze declared source and export read-only language artifacts

Export the exact declared source closure and constructed model inputs from one Git revision. Native evaluation already knows which copied bytes came from which document; retain those origins instead of searching generated text. Include every registered syntax language, even when its semantic provider is absent. Serialize UTF-8 text rather than per-byte JSON numbers so large projects do not multiply their input memory. The worker receives no dirty checkout files and never executes a document cell.

<static:copy id="static-crates-hickory-cli-src-project_export-rs-0">
use anyhow::{Context, Result, bail};
use clap::Args;
use hick_lang::{EntryKind, ProjectTree, evaluate_project, load_project_sources};
use serde_json::json;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Args)]
pub struct ExportArgs {
    /// Entry document, relative to the project root.
    pub entry: String,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long, default_value = "HEAD")]
    pub revision: String,
    /// Directory containing index.json and immutable read-only artifacts.
    #[arg(long)]
    pub output: PathBuf,
    /// Installed compiler-backed provider, e.g. typescript=hick-model-typescript.
    #[arg(long = "model", value_name = "LANGUAGE=EXECUTABLE")]
    pub models: Vec<String>,
    /// Optional fallback for pages not covered by a model.
    #[arg(long)]
    pub lsp: Option<PathBuf>,
    /// Install frozen npm lockfile dependencies in the isolated analysis workspace (scripts disabled).
    #[arg(long)]
    pub install_dependencies: bool,
    #[arg(long, hide = true)]
    pub snapshot_only: Option<PathBuf>,
    /// Public source-link prefix; {revision} expands to the frozen commit.
    #[arg(long)]
    pub source_url: Option<String>,
    /// Write plain JSON payloads instead of deterministic gzip artifacts.
    #[arg(long)]
    pub uncompressed: bool,
}

fn capture_snapshot(args: &ExportArgs, snapshot_path: &std::path::Path) -> Result<()> {
    let root = args.root.canonicalize()?;
    hick_lang::validate_project_path(&args.entry).map_err(anyhow::Error::msg)?;
    let revision = Command::new("git")
        .current_dir(&root)
        .args([
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{}^{{commit}}", args.revision),
        ])
        .output()?;
    if !revision.status.success() {
        bail!(
            "Export requires an exact Git commit: {}",
            String::from_utf8_lossy(&revision.stderr)
        );
    }
    let revision = String::from_utf8(revision.stdout)?.trim().to_owned();
    let tree = crate::project_revision::git(&root, &revision)?;
    let sources = load_project_sources(&args.entry, &|name| {
        let entry = tree
            .get(name)
            .ok_or_else(|| format!("Revision lacks declared source {name}"))?;
        if entry.kind != EntryKind::File {
            return Err(format!("Source must be a regular file: {name}"));
        }
        String::from_utf8(entry.bytes.to_vec()).map_err(|error| error.to_string())
    })
    .map_err(anyhow::Error::msg)?;
    let project = evaluate_project(&args.entry, &sources, &ProjectTree::default())
        .map_err(anyhow::Error::msg)?;
    let files = project
        .new
        .entries()
        .iter()
        .filter(|(name, entry)| entry.kind == EntryKind::File && analysis_input(name))
        .map(|(name, entry)| {
            let source = std::str::from_utf8(entry.bytes.as_ref())
                .with_context(|| format!("Analysis input must be UTF-8: {name}"))?;
            Ok(json!({"path": name, "source": source, "origins": entry.origins}))
        })
        .collect::<Result<Vec<_>>>()?;
    let snapshot = json!({"version":1, "entry":args.entry, "revision":revision, "source_url":args.source_url.as_ref().map(|url| url.replace("{revision}", &revision)), "sources":sources, "files":files});
    std::fs::write(snapshot_path, serde_json::to_vec(&snapshot)?)?;
    Ok(())
}

</static:copy>

Evaluate and serialize provenance in a short-lived child before starting analysis. Exiting that child releases allocator arenas as well as live project data, keeping the native evaluator and model from occupying memory together. The CLI carries the protocol workers. The grammar bundle is a release asset shared with the editor; source builds may use the checkout compiler. Node and Python run only at export time, and the resulting files need neither runtime nor a running server.

<static:copy id="static-crates-hickory-cli-src-project_export-rs-1">
pub fn run(args: ExportArgs) -> Result<std::process::ExitCode> {
    if let Some(path) = &args.snapshot_only {
        capture_snapshot(&args, path)?;
        return Ok(std::process::ExitCode::SUCCESS);
    }
    let temporary = tempfile::tempdir()?;
    let snapshot_path = temporary.path().join("snapshot.json");
    let mut capture = Command::new(std::env::current_exe()?);
    capture
        .args(["project", "export"])
        .arg(&args.entry)
        .arg("--root")
        .arg(&args.root)
        .arg("--revision")
        .arg(&args.revision)
        .arg("--output")
        .arg(&args.output)
        .arg("--snapshot-only")
        .arg(&snapshot_path);
    if let Some(url) = &args.source_url {
        capture.arg("--source-url").arg(url);
    }
    let status = capture.status()?;
    if !status.success() {
        return Ok(std::process::ExitCode::from(1));
    }
    for (name, source) in [
        (
            "export-source-site.py",
            include_str!("../../../scripts/export-source-site.py"),
        ),
        (
            "capture-source-models.py",
            include_str!("../../../scripts/capture-source-models.py"),
        ),
        (
            "capture-source-language.py",
            include_str!("../../../scripts/capture-source-language.py"),
        ),
        (
            "map-source-index.py",
            include_str!("../../../scripts/map-source-index.py"),
        ),
    ] {
        std::fs::write(temporary.path().join(name), source)?;
    }
    let executable = std::env::current_exe()?;
    let installed = executable
        .parent()
        .context("CLI executable has no directory")?
        .join("hick-export/static-highlights.mjs");
    let checkout = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (compiler, compiler_root) = if installed.is_file() {
        (installed, temporary.path().to_owned())
    } else {
        let compiler = checkout.join("scripts/compile-source-highlights.mjs");
        if !compiler.is_file() {
            bail!(
                "Static grammar asset is missing; install a release containing hick-export/static-highlights.mjs"
            );
        }
        (compiler, checkout)
    };
    let mut command = Command::new("python3");
    command
        .arg(temporary.path().join("export-source-site.py"))
        .arg("--snapshot")
        .arg(snapshot_path)
        .arg("--output")
        .arg(&args.output)
        .arg("--compiler")
        .arg(compiler)
        .arg("--compiler-root")
        .arg(compiler_root)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit());
    if args.install_dependencies {
        command.arg("--install-dependencies");
    }
    if !args.uncompressed {
        command.arg("--compress");
    }
    for model in args.models {
        command.arg("--model").arg(model);
    }
    if let Some(lsp) = args.lsp {
        command.arg("--lsp").arg(lsp.canonicalize()?);
    }
    let status = command
        .status()
        .context("Static export needs Python 3, Node and the selected model providers installed")?;
    Ok(if status.success() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::from(1)
    })
}
fn analysis_input(name: &str) -> bool {
    hick_plugins::routing::language_id(name).is_some()
        || matches!(
            std::path::Path::new(name)
                .extension()
                .and_then(|value| value.to_str()),
            Some("csproj" | "sln" | "mts" | "cts")
        )
        || matches!(
            std::path::Path::new(name)
                .file_name()
                .and_then(|value| value.to_str()),
            Some("go.mod" | "go.sum")
        )
}
</static:copy>

<static:file path="crates/hickory-cli/src/project_export.rs"><static:paste select="#static-crates-hickory-cli-src-project_export-rs-0"/><static:paste select="#static-crates-hickory-cli-src-project_export-rs-1"/></static:file>
</static:doc>
