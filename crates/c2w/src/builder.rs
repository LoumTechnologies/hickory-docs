use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use crate::cli::Cli;

pub fn write_dockerfile_to_temp() -> Result<tempfile::NamedTempFile> {
    let mut f = tempfile::NamedTempFile::new().context("failed to create temp Dockerfile")?;
    f.write_all(crate::DOCKERFILE)
        .context("failed to write embedded Dockerfile")?;
    f.flush()?;
    Ok(f)
}

pub fn build(
    builder_path: &str,
    src_img_path: &str,
    dest_dir: &str,
    dest_file: &str,
    cli: &Cli,
) -> Result<()> {
    let mut args: Vec<String> = vec![
        "buildx".into(),
        "build".into(),
        "--progress=plain".into(),
        "--build-arg".into(),
        format!("TARGETARCH={}", cli.target_arch),
        "--build-arg".into(),
        format!("TARGETPLATFORM=linux/{}", cli.target_arch),
        "--platform=linux/amd64".into(),
    ];

    let _temp_dockerfile;
    let dockerfile_path: String;
    if let Some(ref df) = cli.dockerfile {
        dockerfile_path = df.clone();
    } else {
        _temp_dockerfile = write_dockerfile_to_temp()?;
        dockerfile_path = _temp_dockerfile
            .path()
            .to_str()
            .context("temp path not valid UTF-8")?
            .to_string();
    }
    args.extend_from_slice(&["-f".into(), dockerfile_path]);

    if let Some(ref assets) = cli.assets {
        args.extend_from_slice(&["--build-context".into(), format!("assets={}", assets)]);
    }
    if let Some(ref pack) = cli.pack {
        args.extend_from_slice(&[
            "--build-context".into(),
            format!("qemu-aarch64-pack={}", pack),
        ]);
    }

    if cli.to_js {
        args.push("--target=js".into());
        if cli.target_arch == "aarch64" {
            args.extend_from_slice(&["--build-arg".into(), "NO_BINFMT=true".into()]);
        }
    } else if let Some(ref ts) = cli.target_stage {
        args.push(format!("--target={}", ts));
        args.extend_from_slice(&["--build-arg".into(), "OPTIMIZATION_MODE=native".into()]);
    }

    args.extend_from_slice(&["--output".into(), format!("type=local,dest={}", dest_dir)]);

    if !dest_file.is_empty() {
        args.extend_from_slice(&["--build-arg".into(), format!("OUTPUT_NAME={}", dest_file)]);
    }

    let (linux_log_level, init_debug) = if cli.debug_image {
        (7, true)
    } else {
        (0, false)
    };
    args.extend_from_slice(&[
        "--build-arg".into(),
        format!("LINUX_LOGLEVEL={}", linux_log_level),
        "--build-arg".into(),
        format!("INIT_DEBUG={}", init_debug),
    ]);

    if cli.external_bundle {
        args.extend_from_slice(&["--build-arg".into(), "EXTERNAL_BUNDLE=true".into()]);
    }

    // When --wizer is set, tell the Dockerfile to skip its own Wizer step
    // so we can apply it ourselves after the build with the Rust library/CLI.
    if cli.wizer {
        args.extend_from_slice(&["--build-arg".into(), "SKIP_WIZER=true".into()]);
    }

    for a in &cli.build_arg {
        args.extend_from_slice(&["--build-arg".into(), a.clone()]);
    }

    args.extend(cli.extra_flag.iter().cloned());
    args.push(src_img_path.into());

    log::info!("buildx args: {:?}", args);

    let status = Command::new(builder_path)
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to execute buildx build")?;

    if !status.success() {
        anyhow::bail!("buildx build failed");
    }
    Ok(())
}

pub fn build_with_legacy_builder(
    builder_path: &str,
    src_img_path: &str,
    dest_dir: &str,
    dest_file: &str,
    cli: &Cli,
) -> Result<()> {
    let mut args: Vec<String> = vec![
        "build".into(),
        "--progress=plain".into(),
        "--platform=linux/amd64".into(),
        "--build-arg".into(),
        format!("TARGETARCH={}", cli.target_arch),
    ];

    let _temp_dockerfile;
    let dockerfile_path: String;
    if let Some(ref df) = cli.dockerfile {
        dockerfile_path = df.clone();
    } else {
        _temp_dockerfile = write_dockerfile_to_temp()?;
        dockerfile_path = _temp_dockerfile
            .path()
            .to_str()
            .context("temp path not valid UTF-8")?
            .to_string();
    }
    args.extend_from_slice(&["-f".into(), dockerfile_path]);

    if cli.to_js {
        args.push("--target=js".into());
        if cli.target_arch == "aarch64" {
            args.extend_from_slice(&["--build-arg".into(), "NO_BINFMT=true".into()]);
        }
    } else if let Some(ref ts) = cli.target_stage {
        args.push(format!("--target={}", ts));
        args.extend_from_slice(&["--build-arg".into(), "OPTIMIZATION_MODE=native".into()]);
    }

    args.extend_from_slice(&["--output".into(), format!("type=local,dest={}", dest_dir)]);

    if !dest_file.is_empty() {
        args.extend_from_slice(&["--build-arg".into(), format!("OUTPUT_NAME={}", dest_file)]);
    }

    let (linux_log_level, init_debug) = if cli.debug_image {
        (7, true)
    } else {
        (0, false)
    };
    args.extend_from_slice(&[
        "--build-arg".into(),
        format!("LINUX_LOGLEVEL={}", linux_log_level),
        "--build-arg".into(),
        format!("INIT_DEBUG={}", init_debug),
    ]);

    if cli.external_bundle {
        args.extend_from_slice(&["--build-arg".into(), "EXTERNAL_BUNDLE=true".into()]);
    }

    if cli.wizer {
        args.extend_from_slice(&["--build-arg".into(), "SKIP_WIZER=true".into()]);
    }

    for a in &cli.build_arg {
        args.extend_from_slice(&["--build-arg".into(), a.clone()]);
    }

    args.extend(cli.extra_flag.iter().cloned());
    args.push(src_img_path.into());

    log::info!("build args: {:?}", args);

    let status = Command::new(builder_path)
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to execute docker build")?;

    if !status.success() {
        anyhow::bail!("docker build failed");
    }
    Ok(())
}
