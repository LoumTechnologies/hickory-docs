//! Ephemeral CA for TLS MITM of auth-rule-matched connections.
//!
//! Generates a self-signed CA at startup, then issues per-host certificates
//! on demand with caching. CA lifetime is one session — it's never persisted.

use std::sync::Arc;

use dashmap::DashMap;
use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair, KeyUsagePurpose};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

/// A certified key pair: certificate + private key, ready for rustls.
pub struct CertifiedKey {
    pub cert_der: CertificateDer<'static>,
    pub key_der: PrivateKeyDer<'static>,
}

/// Ephemeral certificate authority for TLS interception.
pub struct HickCa {
    /// CA certificate in PEM format (for injecting into container trust stores).
    pub cert_pem: String,
    ca_cert_der: CertificateDer<'static>,
    ca_key_pair: KeyPair,
    ca_params: CertificateParams,
    host_cert_cache: DashMap<String, Arc<CertifiedKey>>,
}

impl HickCa {
    /// Generate a new ephemeral CA.
    pub fn generate() -> anyhow::Result<Self> {
        let mut params = CertificateParams::default();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "Hick Auth Injection CA");
        params
            .distinguished_name
            .push(rcgen::DnType::OrganizationName, "Hick");

        let key_pair = KeyPair::generate()?;
        let ca_cert = params.self_signed(&key_pair)?;

        let cert_pem = ca_cert.pem();
        let cert_der = CertificateDer::from(ca_cert.der().to_vec());

        Ok(Self {
            cert_pem,
            ca_cert_der: cert_der,
            ca_key_pair: key_pair,
            ca_params: params,
            host_cert_cache: DashMap::new(),
        })
    }

    /// Generate (or return cached) a TLS certificate for a specific hostname.
    pub fn cert_for_host(&self, hostname: &str) -> anyhow::Result<Arc<CertifiedKey>> {
        if let Some(cached) = self.host_cert_cache.get(hostname) {
            return Ok(cached.clone());
        }

        let mut params = CertificateParams::new(vec![hostname.to_string()])?;
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, hostname);

        let host_key_pair = KeyPair::generate()?;
        let issuer = rcgen::Issuer::from_params(&self.ca_params, &self.ca_key_pair);
        let host_cert = params.signed_by(&host_key_pair, &issuer)?;

        let cert_der = CertificateDer::from(host_cert.der().to_vec());
        let key_der = PrivateKeyDer::try_from(host_key_pair.serialize_der())
            .map_err(|e| anyhow::anyhow!("failed to serialize host key: {e}"))?;

        let certified = Arc::new(CertifiedKey { cert_der, key_der });
        self.host_cert_cache
            .insert(hostname.to_string(), certified.clone());
        Ok(certified)
    }

    /// Return the CA certificate in DER format.
    pub fn ca_cert_der(&self) -> &CertificateDer<'static> {
        &self.ca_cert_der
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ca() {
        let ca = HickCa::generate().unwrap();
        assert!(ca.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(!ca.ca_cert_der.is_empty());
    }

    #[test]
    fn cert_for_host_generates_cert() {
        let ca = HickCa::generate().unwrap();
        let cert = ca.cert_for_host("api.openai.com").unwrap();
        assert!(!cert.cert_der.is_empty());
    }

    #[test]
    fn cert_for_host_caches() {
        let ca = HickCa::generate().unwrap();
        let cert1 = ca.cert_for_host("api.openai.com").unwrap();
        let cert2 = ca.cert_for_host("api.openai.com").unwrap();
        // Same Arc — pointer equality
        assert!(Arc::ptr_eq(&cert1, &cert2));
    }

    #[test]
    fn different_hosts_get_different_certs() {
        let ca = HickCa::generate().unwrap();
        let cert1 = ca.cert_for_host("api.openai.com").unwrap();
        let cert2 = ca.cert_for_host("api.anthropic.com").unwrap();
        assert!(!Arc::ptr_eq(&cert1, &cert2));
    }
}
