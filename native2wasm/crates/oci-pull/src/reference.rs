//! Parse OCI image references like `alpine:3.23.2` or `docker.io/library/alpine:latest`.

use anyhow::{bail, Context, Result};
use oci_client::Reference as OciReference;

/// Parsed OCI image reference with registry, repository, and tag/digest.
#[derive(Debug, Clone)]
pub struct ImageReference {
    /// The oci-client Reference for use with the registry client.
    pub reference: OciReference,
}

impl ImageReference {
    /// Parse an image reference string.
    ///
    /// Accepts formats like:
    /// - `alpine` → `docker.io/library/alpine:latest`
    /// - `alpine:3.23.2` → `docker.io/library/alpine:3.23.2`
    /// - `ghcr.io/user/repo:tag`
    /// - `registry.example.com/repo@sha256:abc...`
    pub fn parse(image_ref: &str) -> Result<Self> {
        if image_ref.is_empty() {
            bail!("empty image reference");
        }

        let reference: OciReference = image_ref
            .parse()
            .with_context(|| format!("failed to parse image reference {:?}", image_ref))?;

        Ok(Self { reference })
    }

    pub fn registry(&self) -> &str {
        self.reference.registry()
    }

    pub fn repository(&self) -> &str {
        self.reference.repository()
    }

    pub fn tag(&self) -> Option<&str> {
        self.reference.tag()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_name() {
        let r = ImageReference::parse("alpine").unwrap();
        assert_eq!(r.repository(), "library/alpine");
        assert!(r.tag().is_some());
    }

    #[test]
    fn parse_name_with_tag() {
        let r = ImageReference::parse("alpine:3.23.2").unwrap();
        assert_eq!(r.repository(), "library/alpine");
        assert_eq!(r.tag(), Some("3.23.2"));
    }

    #[test]
    fn parse_full_reference() {
        let r = ImageReference::parse("ghcr.io/user/repo:v1.0").unwrap();
        assert_eq!(r.registry(), "ghcr.io");
        assert_eq!(r.repository(), "user/repo");
        assert_eq!(r.tag(), Some("v1.0"));
    }

    #[test]
    fn parse_empty_fails() {
        assert!(ImageReference::parse("").is_err());
    }
}
