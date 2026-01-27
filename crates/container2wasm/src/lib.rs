pub mod assembler;
pub mod builder;
pub mod cli;
pub mod image;
pub mod iso;
pub mod rootfs;
pub mod spec;
pub mod template;
pub mod wizer;

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use cli::Cli;

pub const DEFAULT_OUTPUT_FILE: &str = "out.wasm";
pub const DOCKERFILE: &[u8] = include_bytes!("../../../Dockerfile");

pub fn run(cli: Cli) -> Result<()> {
    if cli.show_dockerfile {
        print!("{}", String::from_utf8_lossy(DOCKERFILE));
        println!();
        return Ok(());
    }

    // --wizer-only mode: skip Docker build entirely, just pre-initialize
    // an existing .wasm file.
    if cli.wizer_only {
        return run_wizer_only(&cli);
    }

    let arg1 = match &cli.image_name {
        Some(name) => name.clone(),
        None => {
            if cli.external_bundle {
                bail!("specify output image name");
            }
            bail!("specify image name");
        }
    };

    // Determine whether to use the no-Docker pipeline
    let use_no_docker =
        cli.no_docker || (which::which(&cli.builder).is_err() && !cli.show_dockerfile);

    if use_no_docker {
        return run_no_docker_pipeline(&cli, &arg1);
    }

    // --- Existing Docker-based pipeline ---
    let output_path: String;
    let needs_img: bool;
    if cli.external_bundle || cli.pack.is_some() {
        output_path = arg1.clone();
        needs_img = false;
        if cli.output.is_some() {
            bail!("command receives only 1 arg (output image path)");
        }
    } else {
        output_path = cli.output.clone().unwrap_or_default();
        needs_img = true;
    }

    let builder_path = which::which(&cli.builder)
        .with_context(|| format!("builder {:?} not found in PATH", cli.builder))?;
    let builder_path = builder_path
        .to_str()
        .context("builder path not valid UTF-8")?;

    let mut legacy = false;
    match Command::new(builder_path).arg("buildx").output() {
        Ok(output) if output.status.success() => {}
        _ => {
            log::info!("buildx unavailable. falling back to the normal builder.");
            legacy = true;
        }
    }
    if cli.legacy {
        legacy = true;
    }

    let mut dest_dir = ".".to_string();
    let mut dest_file = DEFAULT_OUTPUT_FILE.to_string();

    if cli.to_js {
        dest_file.clear();
    }
    if cli.target_stage.is_some() {
        dest_file.clear();
    }

    if !output_path.is_empty() {
        let path = Path::new(&output_path);
        let parent = path.parent().unwrap_or(Path::new("."));
        dest_dir = fs::canonicalize(parent)
            .unwrap_or_else(|_| parent.to_path_buf())
            .to_str()
            .context("output dir not valid UTF-8")?
            .to_string();

        if let Some(file_name) = path.file_name() {
            let f = file_name.to_str().unwrap_or("");
            if !f.is_empty() {
                dest_file = f.to_string();
            }
        }
    }

    if cli.to_js && !dest_file.is_empty() {
        bail!("output destination must be a slash-terminated directory path when using \"to-js\" option");
    }
    if cli.target_stage.is_some() && !dest_file.is_empty() {
        bail!("output destination must be a slash-terminated directory path when using \"target-stage\" option");
    }
    if cli.assets.is_some() && legacy {
        bail!("\"assets\" unsupported on docker build as of now; install docker buildx instead");
    }
    if cli.pack.is_some() && legacy {
        bail!("\"pack\" unsupported on docker build as of now; install docker buildx instead");
    }

    if cli.wizer && cli.to_js {
        bail!("--wizer cannot be used with --to-js (Wizer only works on WASI targets)");
    }

    let src_img_name = arg1;
    let tmpdir = tempfile::tempdir().context("failed to create temp dir")?;
    let src_img_path = tmpdir.path().join("img");
    fs::create_dir(&src_img_path).context("failed to create img dir in temp")?;

    if needs_img {
        image::prepare_source_img(builder_path, &src_img_name, &src_img_path, &cli.target_arch)
            .context("failed to prepare image")?;
    }

    let src_img_path_str = src_img_path.to_str().context("temp path not valid UTF-8")?;

    let build_result = if legacy {
        builder::build_with_legacy_builder(
            builder_path,
            src_img_path_str,
            &dest_dir,
            &dest_file,
            &cli,
        )
    } else {
        builder::build(builder_path, src_img_path_str, &dest_dir, &dest_file, &cli)
    };

    build_result?;

    // Post-build Wizer pre-initialization
    if cli.wizer && !dest_file.is_empty() {
        let wasm_path = Path::new(&dest_dir).join(&dest_file);
        if wasm_path.exists() {
            let opts = wizer::WizerOpts::from_cli(&cli);

            // Pre-initialize to a temp file, then replace the original
            let tmp_output = tempfile::NamedTempFile::new_in(&dest_dir)
                .context("failed to create temp file for Wizer output")?;

            wizer::pre_initialize(&wasm_path, tmp_output.path(), &opts)
                .context("Wizer pre-initialization failed")?;

            fs::rename(tmp_output.path(), &wasm_path)
                .or_else(|_| {
                    // rename can fail across filesystems; fall back to copy
                    fs::copy(tmp_output.path(), &wasm_path)?;
                    Ok::<_, std::io::Error>(())
                })
                .with_context(|| {
                    format!(
                        "failed to replace {:?} with pre-initialized output",
                        wasm_path
                    )
                })?;

            log::info!("wizer pre-initialization applied to {:?}", wasm_path);
        } else {
            log::warn!(
                "--wizer was set but output file {:?} not found; skipping pre-initialization",
                wasm_path
            );
        }
    }

    Ok(())
}

/// Run the pure Rust no-Docker pipeline.
pub fn run_no_docker_pipeline(cli: &Cli, image_ref: &str) -> Result<()> {
    let output_path = cli.output.as_deref().unwrap_or(DEFAULT_OUTPUT_FILE);
    let output = Path::new(output_path);

    // Build a tokio runtime for async operations (OCI pull)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to create tokio runtime")?;

    rt.block_on(assembler::build_without_docker(image_ref, output, cli))
}

/// Run Wizer pre-initialization on an existing .wasm file without performing
/// any Docker build. Useful for re-processing or separately optimizing a wasm
/// module that was previously built without Wizer.
pub fn run_wizer_only(cli: &Cli) -> Result<()> {
    let input_path = cli
        .image_name
        .as_deref()
        .context("specify the input .wasm file path")?;

    let input = Path::new(input_path);
    if !input.exists() {
        bail!("input file {:?} does not exist", input);
    }

    let output = match &cli.output {
        Some(out) => Path::new(out).to_path_buf(),
        None => {
            // Default: write to <name>.pre-init.wasm alongside the input
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let parent = input.parent().unwrap_or(Path::new("."));
            parent.join(format!("{}.pre-init.wasm", stem))
        }
    };

    let opts = wizer::WizerOpts::from_cli(cli);
    wizer::pre_initialize(input, &output, &opts)?;

    log::info!("wrote pre-initialized module to {:?}", output);
    Ok(())
}
