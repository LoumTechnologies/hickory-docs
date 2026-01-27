//! Pure Rust OCI registry client for pulling and extracting container images.
//!
//! This crate provides the ability to pull OCI container images directly from
//! registries over HTTP without requiring Docker or any other container runtime.

pub mod auth;
pub mod layer;
pub mod reference;
pub mod registry;

use std::path::Path;

use anyhow::{Context, Result};
use oci_client::secrets::RegistryAuth;
use oci_spec::image::ImageConfiguration;

use reference::ImageReference;

/// Pull an OCI image from a registry and extract its filesystem layers
/// into the specified rootfs directory.
///
/// Returns the image configuration which contains metadata like entrypoint,
/// cmd, env, working directory, etc.
///
/// # Arguments
///
/// * `image_ref` - Image reference (e.g. `"alpine:3.23.2"`, `"docker.io/library/alpine:latest"`)
/// * `target_arch` - Target CPU architecture (e.g. `"amd64"`, `"arm64"`)
/// * `rootfs_dir` - Directory to extract the image filesystem layers into
/// * `auth` - Registry authentication credentials
pub async fn pull_and_extract(
    image_ref: &str,
    target_arch: &str,
    rootfs_dir: &Path,
    auth: &RegistryAuth,
) -> Result<ImageConfiguration> {
    let parsed = ImageReference::parse(image_ref)?;

    log::info!(
        "pulling image {} from {}",
        parsed.repository(),
        parsed.registry()
    );

    // Pull manifest and image config
    let (manifest, image_config, _digest) =
        registry::pull_manifest_and_config(&parsed, target_arch, auth)
            .await
            .context("failed to pull image manifest")?;

    // Verify architecture matches
    let arch = image_config.architecture();
    let arch_str = arch.to_string();
    if !arch_str.is_empty() && arch_str != target_arch {
        log::warn!(
            "image architecture {:?} does not match target {:?}",
            arch_str,
            target_arch
        );
    }

    // Create rootfs directory
    std::fs::create_dir_all(rootfs_dir)
        .with_context(|| format!("failed to create rootfs dir {:?}", rootfs_dir))?;

    // Pull and extract layers in order (bottom to top)
    let layers = &manifest.layers;
    log::info!("extracting {} layers", layers.len());

    for (i, layer_desc) in layers.iter().enumerate() {
        log::info!(
            "pulling layer {}/{}: {} ({} bytes)",
            i + 1,
            layers.len(),
            layer_desc.digest,
            layer_desc.size
        );

        let layer_data = registry::pull_layer(&parsed, auth, layer_desc)
            .await
            .with_context(|| format!("failed to pull layer {}", layer_desc.digest))?;

        layer::extract_layer_auto(&layer_data, rootfs_dir)
            .with_context(|| format!("failed to extract layer {}", layer_desc.digest))?;

        log::info!("extracted layer {}/{}", i + 1, layers.len());
    }

    log::info!("image extraction complete");
    Ok(image_config)
}
