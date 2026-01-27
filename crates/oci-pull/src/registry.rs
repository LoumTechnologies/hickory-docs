//! OCI registry client wrapper for pulling image manifests and layers.

use anyhow::{Context, Result};
use oci_client::client::{ClientConfig, ClientProtocol};
use oci_client::manifest::OciImageManifest;
use oci_client::secrets::RegistryAuth;
use oci_client::Client;
use oci_spec::image::ImageConfiguration;

use crate::reference::ImageReference;

/// Pull the image manifest and config for a given reference and platform.
///
/// Returns (manifest, image_configuration, manifest_digest).
pub async fn pull_manifest_and_config(
    image_ref: &ImageReference,
    _target_arch: &str,
    auth: &RegistryAuth,
) -> Result<(OciImageManifest, ImageConfiguration, String)> {
    let client = build_client()?;

    // pull_manifest_and_config returns (manifest, digest, config_json_string)
    let (manifest, digest, config_json) = client
        .pull_manifest_and_config(&image_ref.reference, auth)
        .await
        .context("failed to pull manifest and config")?;

    let image_config: ImageConfiguration =
        serde_json::from_str(&config_json).context("failed to parse image configuration JSON")?;

    Ok((manifest, image_config, digest))
}

/// Pull a single layer blob by descriptor into memory.
pub async fn pull_layer(
    image_ref: &ImageReference,
    auth: &RegistryAuth,
    descriptor: &oci_client::manifest::OciDescriptor,
) -> Result<Vec<u8>> {
    let client = build_client()?;

    // Auth is required before pulling blobs (pull_manifest_and_config
    // caches the token, but we create a fresh client here).
    client
        .auth(
            &image_ref.reference,
            auth,
            oci_client::RegistryOperation::Pull,
        )
        .await
        .context("failed to authenticate with registry")?;

    // pull_blob writes to an AsyncWrite sink
    let mut buf: Vec<u8> = Vec::with_capacity(descriptor.size as usize);
    client
        .pull_blob(&image_ref.reference, descriptor, &mut buf)
        .await
        .with_context(|| format!("failed to pull layer blob {}", descriptor.digest))?;

    Ok(buf)
}

fn build_client() -> Result<Client> {
    let config = ClientConfig {
        protocol: ClientProtocol::Https,
        ..Default::default()
    };

    Ok(Client::new(config))
}
