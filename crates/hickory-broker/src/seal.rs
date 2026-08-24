//! The seal: what a machine asserts about itself, and how it is checked.
//!
//! Sealing is a property a machine asserts about itself, set by the person
//! standing at it, recorded in that machine's own config. It is deliberately
//! **not** a grant another machine can hand out or take away: a fleet peer
//! that could unseal a box would mean a compromised laptop unseals the box,
//! which inverts the whole point.
//!
//! `hick sealed --check` is the whole of it as one command. It **verifies
//! rather than performs**: the default route being denied at the OS is
//! firewall configuration on the engineer's own machine, and we do not
//! reconfigure somebody's networking for them.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The provider variables a sealed machine must not have. Their names are the
/// ecosystem's, not ours, so a user's existing environment works without
/// translation — which is exactly why they are the ones to look for.
pub const PROVIDER_VARS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "DEEPSEEK_API_KEY",
    "XAI_API_KEY",
    "OPENROUTER_API_KEY",
    "GAB_API_KEY",
    "GOOGLE_API_KEY",
    "GEMINI_API_KEY",
];

/// The base-URL variables that must point at the broker.
pub const BASE_URL_VARS: &[&str] = &[
    "ANTHROPIC_BASE_URL",
    "OPENAI_BASE_URL",
    "DEEPSEEK_BASE_URL",
    "XAI_BASE_URL",
    "OPENROUTER_BASE_URL",
    "GAB_BASE_URL",
];

/// One thing the check found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealFinding {
    /// Short name of what was checked.
    pub check: String,
    pub ok: bool,
    /// What is true, and — when it is not ok — what to do about it.
    pub detail: String,
}

/// The whole verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealCheck {
    /// Whether this machine claims to be sealed at all.
    pub sealed: bool,
    /// The broker it says it reaches the model through.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broker: Option<String>,
    pub findings: Vec<SealFinding>,
}

impl SealCheck {
    pub fn ok(&self) -> bool {
        self.findings.iter().all(|f| f.ok)
    }
}

/// What a machine's `sealed.toml`-equivalent holds. JSON rather than TOML
/// because everything else this product writes is JSON, and one format is one
/// thing to get wrong.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SealConfig {
    #[serde(default)]
    pub sealed: bool,
    /// `http://broker.lan:7749` — where this machine reaches the model.
    #[serde(default)]
    pub broker: Option<String>,
    /// The stub credential. Worthless everywhere else: minted by the broker,
    /// bound to this machine's key, and rejected by the vendor because it was
    /// never a vendor credential. An agent that leaks it leaks nothing.
    #[serde(default)]
    pub stub: Option<String>,
}

impl SealConfig {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// Check the seal, given the config and this machine's environment.
///
/// Pure in its inputs so the awkward cases are arguable in a test rather than
/// against a real machine. `key_file_present` is whether a provider key store
/// exists on disk; `route_denied` is whether the default route is denied at
/// the OS, which the caller determines and this only reports.
pub fn check_seal(
    config: &SealConfig,
    env: &BTreeMap<String, String>,
    key_file_present: bool,
    route_denied: Option<bool>,
) -> SealCheck {
    let mut findings = Vec::new();

    if !config.sealed {
        findings.push(SealFinding {
            check: "sealed".into(),
            ok: false,
            detail: "this machine is not sealed. Sealing is something the person \
                     standing at a machine asserts about it — deliberately not a \
                     grant another machine can hand out, because a fleet peer \
                     that could unseal this box would mean a compromised laptop \
                     unseals it.\n  Next step: `hick sealed --set --broker \
                     http://broker.lan:7749`."
                .into(),
        });
        return SealCheck {
            sealed: false,
            broker: config.broker.clone(),
            findings,
        };
    }

    // A provider variable existing at all is a boot failure, not a fallback:
    // silently ignoring it would leave a real key sitting in a process that is
    // not supposed to hold one.
    let present: Vec<&str> = PROVIDER_VARS
        .iter()
        .copied()
        .filter(|v| env.get(*v).is_some_and(|value| !value.trim().is_empty()))
        .collect();
    findings.push(if present.is_empty() {
        SealFinding {
            check: "no provider key in the environment".into(),
            ok: true,
            detail: "no provider variable is set, so there is no real credential \
                     in this process."
                .into(),
        }
    } else {
        SealFinding {
            check: "no provider key in the environment".into(),
            ok: false,
            detail: format!(
                "{} is set on a sealed machine. Unset it; this machine reaches \
                 the model through the broker at {}, using the stub credential \
                 in this machine's seal config.\n  \
                 Leaving it set is not a harmless fallback — it is a real key \
                 sitting in a process that is not supposed to hold one.",
                present.join(", "),
                config.broker.as_deref().unwrap_or("(no broker configured)"),
            ),
        }
    });

    findings.push(SealFinding {
        check: "no key file on disk".into(),
        ok: !key_file_present,
        detail: if key_file_present {
            "a provider key store exists on this machine. On a sealed machine \
             `KeyStore::set` refuses and keys live on the broker.\n  \
             Next step: delete it, and check the broker holds the real key."
                .into()
        } else {
            "no provider key store is on this machine.".into()
        },
    });

    let broker = config.broker.clone();
    findings.push(match &broker {
        Some(url) if !url.trim().is_empty() => SealFinding {
            check: "a broker is configured".into(),
            ok: true,
            detail: format!("this machine reaches the model through {url}."),
        },
        _ => SealFinding {
            check: "a broker is configured".into(),
            ok: false,
            detail: "this machine is sealed but names no broker, so it cannot \
                     use a model at all. That is a coherent state — it is what \
                     the seal alone buys — and it is worth knowing you are in \
                     it.\n  Next step: `hick sealed --set --broker \
                     http://broker.lan:7749`."
                .into(),
        },
    });

    // Every provider base URL must resolve to the broker, and a client
    // configured otherwise refuses to start rather than reaching the vendor
    // directly.
    if let Some(url) = broker.as_deref().filter(|u| !u.trim().is_empty()) {
        let wrong: Vec<String> = BASE_URL_VARS
            .iter()
            .filter_map(|var| {
                let value = env.get(*var)?;
                (!value.starts_with(url)).then(|| format!("{var}={value}"))
            })
            .collect();
        findings.push(if wrong.is_empty() {
            SealFinding {
                check: "base URLs point at the broker".into(),
                ok: true,
                detail: "every provider base URL that is set points at the broker.".into(),
            }
        } else {
            SealFinding {
                check: "base URLs point at the broker".into(),
                ok: false,
                detail: format!(
                    "{} does not point at {url}, so that client would reach the \
                     vendor directly and go around the toll booth entirely.\n  \
                     Next step: set it to {url}, or unset it.",
                    wrong.join(", ")
                ),
            }
        });
    }

    findings.push(SealFinding {
        check: "the default route is denied".into(),
        ok: route_denied.unwrap_or(false),
        detail: match route_denied {
            Some(true) => "the default route is denied, so a process that ignores \
                           the proxy settings gets nothing rather than getting out."
                .into(),
            Some(false) => "the default route is NOT denied, so a process that \
                            ignores the proxy settings reaches the internet \
                            directly — the road out is not the only road.\n  \
                            This is firewall configuration on your own machine, \
                            and this command verifies it rather than performing \
                            it: we do not reconfigure somebody's networking for \
                            them."
                .into(),
            None => "could not determine whether the default route is denied. \
                     That is firewall configuration on your own machine, and \
                     this command verifies rather than performs — so an answer \
                     it cannot get is reported, never assumed."
                .into(),
        },
    });

    SealCheck {
        sealed: true,
        broker,
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn sealed() -> SealConfig {
        SealConfig {
            sealed: true,
            broker: Some("http://broker.lan:7749".into()),
            stub: Some("stub-not-a-vendor-credential".into()),
        }
    }

    #[test]
    fn an_unsealed_machine_says_so_and_says_who_may_seal_it() {
        let check = check_seal(&SealConfig::default(), &env(&[]), false, Some(true));
        assert!(!check.sealed);
        assert!(!check.ok());
        let detail = &check.findings[0].detail;
        // Not a grant another machine can hand out: a fleet peer that could
        // unseal this box would mean a compromised laptop unseals it.
        assert!(detail.contains("standing at a machine"), "{detail}");
    }

    #[test]
    fn a_provider_variable_is_a_failure_and_not_a_fallback() {
        let check = check_seal(
            &sealed(),
            &env(&[("ANTHROPIC_API_KEY", "sk-real")]),
            false,
            Some(true),
        );
        assert!(!check.ok());
        let finding = check
            .findings
            .iter()
            .find(|f| f.check.contains("environment"))
            .unwrap();
        assert!(!finding.ok);
        assert!(finding.detail.contains("ANTHROPIC_API_KEY"), "{finding:?}");
        assert!(finding.detail.contains("Unset it"), "{finding:?}");
        // The message must not leak the value it found.
        assert!(!finding.detail.contains("sk-real"), "{finding:?}");
    }

    #[test]
    fn an_empty_provider_variable_is_not_a_key() {
        let check = check_seal(
            &sealed(),
            &env(&[("OPENAI_API_KEY", "")]),
            false,
            Some(true),
        );
        assert!(check.ok(), "{check:?}");
    }

    #[test]
    fn a_base_url_pointing_past_the_broker_is_caught() {
        // Otherwise that client goes around the toll booth entirely.
        let check = check_seal(
            &sealed(),
            &env(&[("ANTHROPIC_BASE_URL", "https://api.anthropic.com")]),
            false,
            Some(true),
        );
        assert!(!check.ok());
        let finding = check
            .findings
            .iter()
            .find(|f| f.check.contains("base URLs"))
            .unwrap();
        assert!(
            finding.detail.contains("around the toll booth"),
            "{finding:?}"
        );
    }

    #[test]
    fn a_sealed_machine_with_no_broker_is_a_coherent_state_that_is_named() {
        // On its own the seal is a machine that cannot use a model at all,
        // which is exactly what step 1 buys.
        let config = SealConfig {
            sealed: true,
            broker: None,
            stub: None,
        };
        let check = check_seal(&config, &env(&[]), false, Some(true));
        let finding = check
            .findings
            .iter()
            .find(|f| f.check.contains("broker"))
            .unwrap();
        assert!(!finding.ok);
        assert!(finding.detail.contains("coherent state"), "{finding:?}");
    }

    #[test]
    fn an_undetermined_route_is_reported_and_never_assumed() {
        let check = check_seal(&sealed(), &env(&[]), false, None);
        let finding = check
            .findings
            .iter()
            .find(|f| f.check.contains("default route"))
            .unwrap();
        assert!(!finding.ok);
        assert!(
            finding.detail.contains("verifies rather than performs"),
            "{finding:?}"
        );
    }

    #[test]
    fn a_fully_sealed_machine_passes() {
        let check = check_seal(
            &sealed(),
            &env(&[("ANTHROPIC_BASE_URL", "http://broker.lan:7749/v1")]),
            false,
            Some(true),
        );
        assert!(check.ok(), "{check:?}");
    }
}
