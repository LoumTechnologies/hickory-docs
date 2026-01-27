mod builder;
mod cli;
mod image;

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Parser;

use cli::Cli;

const DEFAULT_OUTPUT_FILE: &str = "out.wasm";
const DOCKERFILE: &[u8] = include_bytes!("../../Dockerfile");

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{:+}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    if cli.show_dockerfile {
        print!("{}", String::from_utf8_lossy(DOCKERFILE));
        println!();
        return Ok(());
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

    let src_img_name = arg1;
    let tmpdir = tempfile::tempdir().context("failed to create temp dir")?;
    let src_img_path = tmpdir.path().join("img");
    fs::create_dir(&src_img_path).context("failed to create img dir in temp")?;

    if needs_img {
        image::prepare_source_img(
            builder_path,
            &src_img_name,
            &src_img_path,
            &cli.target_arch,
        )
        .context("failed to prepare image")?;
    }

    let src_img_path_str = src_img_path
        .to_str()
        .context("temp path not valid UTF-8")?;

    if legacy {
        builder::build_with_legacy_builder(
            builder_path,
            src_img_path_str,
            &dest_dir,
            &dest_file,
            &cli,
        )
    } else {
        builder::build(builder_path, src_img_path_str, &dest_dir, &dest_file, &cli)
    }
}
