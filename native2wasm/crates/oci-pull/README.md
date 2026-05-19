# oci-pull

Pure-Rust async OCI registry client for pulling and extracting container images
without Docker.

`pull_and_extract(image_ref, arch, rootfs_dir, auth)` calls
`registry::pull_manifest_and_config`, pulls layers in order, and extracts each
via `layer::extract_layer_auto` (auto-detects gzip/zstd tar).

Used by `c2w`'s no-Docker pipeline path to fetch OCI images directly from
registries (Docker Hub, GHCR, etc.) without requiring a Docker daemon.

Wraps the `oci-client` and `oci-spec` crates.
