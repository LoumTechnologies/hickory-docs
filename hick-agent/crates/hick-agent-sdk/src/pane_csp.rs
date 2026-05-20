//! Content Security Policy generation for HTML panes.
//!
//! [`generate_pane_csp`] builds a CSP string from the agent's runtime network allowlist
//! and optional extra origins declared in `pane.yaml`.

use crate::security_config::HostEntry;

/// Build a `Content-Security-Policy` header value for an HTML pane.
///
/// `allowlist` is the agent's runtime network allow-list (from `AgentSecurityConfig`).
/// `extra` is the `extra_connect_src` list from `pane.yaml` (additive with allowlist).
///
/// The resulting policy uses:
/// - `default-src 'self'` — no implicit inheritance from parent
/// - `connect-src 'none'` when both inputs are empty; otherwise the union of origins
/// - `sandbox allow-scripts allow-forms` is NOT emitted here (it goes on the `<iframe>` element)
pub fn generate_pane_csp(allowlist: &[HostEntry], extra: &[String]) -> String {
    let connect_src = build_connect_src(allowlist, extra);
    format!(
        "default-src 'self'; \
         script-src 'self'; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data:; \
         connect-src {connect_src}; \
         frame-ancestors 'self'; \
         form-action 'none'"
    )
}

fn build_connect_src(allowlist: &[HostEntry], extra: &[String]) -> String {
    let mut origins: Vec<String> = Vec::new();

    for entry in allowlist {
        let host = &entry.host;
        if host.starts_with("*.") {
            origins.push(format!("https://{host}"));
        } else {
            // Add both http and https so localhost dev servers work too.
            origins.push(format!("https://{host}"));
        }
    }

    for src in extra {
        let s = src.trim().to_string();
        if !s.is_empty() && !origins.contains(&s) {
            origins.push(s);
        }
    }

    if origins.is_empty() {
        "'none'".to_string()
    } else {
        origins.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(h: &str) -> HostEntry {
        HostEntry { host: h.to_string(), port: "443".to_string() }
    }

    #[test]
    fn empty_allowlist() {
        let csp = generate_pane_csp(&[], &[]);
        assert!(csp.contains("connect-src 'none'"), "got: {csp}");
    }

    #[test]
    fn single_host() {
        let csp = generate_pane_csp(&[host("api.example.com")], &[]);
        assert!(csp.contains("connect-src https://api.example.com"), "got: {csp}");
    }

    #[test]
    fn multiple_hosts() {
        let csp = generate_pane_csp(&[host("a.com"), host("b.com")], &[]);
        assert!(csp.contains("https://a.com"), "got: {csp}");
        assert!(csp.contains("https://b.com"), "got: {csp}");
    }

    #[test]
    fn wildcard_host() {
        let csp = generate_pane_csp(&[host("*.example.com")], &[]);
        assert!(csp.contains("https://*.example.com"), "got: {csp}");
    }

    #[test]
    fn extra_connect_src() {
        let csp = generate_pane_csp(&[], &["https://fonts.googleapis.com".to_string()]);
        assert!(csp.contains("connect-src https://fonts.googleapis.com"), "got: {csp}");
    }
}
