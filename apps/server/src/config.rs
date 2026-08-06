//! Strongly-typed server configuration, validated at boot.
//!
//! Convention (graceful degradation): in dev (`APP_ENV` unset or `dev`),
//! missing optional integrations (Stripe, PostHog, canopy) degrade to
//! feature no-ops with a boot log line. In strict mode
//! (`APP_ENV=staging|production`), invalid or unsafe configuration fails
//! fast at boot. Canonical env var names are documented in
//! `docs/operators/ENVIRONMENTS.md` — one name set, no per-env twins.

use std::path::PathBuf;

use anyhow::{Context as _, Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Dev,
    Staging,
    Production,
}

impl AppEnv {
    pub fn strict(self) -> bool {
        !matches!(self, AppEnv::Dev)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorKind {
    Local,
    Canopy,
}

impl ExecutorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ExecutorKind::Local => "local",
            ExecutorKind::Canopy => "canopy",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PosthogConfig {
    pub api_key: String,
    pub host: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: AppEnv,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub git_data_dir: PathBuf,
    pub executor: ExecutorKind,
    pub app_base_url: String,
    /// `None` → billing endpoints answer 503 "billing not configured".
    pub stripe: Option<StripeConfig>,
    /// `None` → analytics capture is a no-op.
    pub posthog: Option<PosthogConfig>,
    /// Serve this directory as the web app (SPA fallback) when present.
    pub web_dist_dir: Option<PathBuf>,
    /// Explicit plan-set override (else PostHog flag, else "default").
    pub plan_set: Option<String>,
    /// `None` → the agent endpoint answers 503 "agent not configured".
    pub anthropic_api_key: Option<String>,
}

fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

impl Config {
    /// Read + validate configuration from the environment.
    pub fn from_env() -> Result<Config> {
        let app_env = match env_opt("APP_ENV").as_deref() {
            None | Some("dev") | Some("development") => AppEnv::Dev,
            Some("staging") => AppEnv::Staging,
            Some("production") => AppEnv::Production,
            Some(other) => bail!("invalid APP_ENV '{other}' (dev|staging|production)"),
        };
        let strict = app_env.strict();

        let port: u16 = match env_opt("PORT") {
            Some(p) => p.parse().context("invalid PORT")?,
            None => 8080,
        };

        let database_url = match env_opt("DATABASE_URL") {
            Some(u) => u,
            None if strict => bail!("DATABASE_URL is required in staging/production"),
            None => {
                let fallback =
                    "postgres://hickory:hickory@localhost:5433/hickory".to_string();
                log::warn!("DATABASE_URL not set; using dev default {fallback}");
                fallback
            }
        };

        let jwt_secret = match env_opt("JWT_SECRET") {
            Some(s) => {
                if strict && s.len() < 32 {
                    bail!("JWT_SECRET must be at least 32 bytes in staging/production");
                }
                s
            }
            None if strict => bail!("JWT_SECRET is required in staging/production"),
            None => {
                log::warn!("JWT_SECRET not set; using an insecure dev-only default");
                "hickory-dev-secret-do-not-use-in-production".to_string()
            }
        };

        let git_data_dir = PathBuf::from(
            env_opt("GIT_DATA_DIR").unwrap_or_else(|| "./data/git".to_string()),
        );

        // Canopy configuration itself is validated in
        // apps/server/src/executor.rs (the only module that may know
        // canopy's API), called from build_state.
        let executor = match env_opt("HICKORY_EXECUTOR").as_deref() {
            None | Some("local") => ExecutorKind::Local,
            Some("canopy") => ExecutorKind::Canopy,
            Some(other) => bail!("invalid HICKORY_EXECUTOR '{other}' (local|canopy)"),
        };

        let anthropic_api_key = env_opt("ANTHROPIC_API_KEY");
        if anthropic_api_key.is_none() {
            log::info!(
                "Anthropic not configured (ANTHROPIC_API_KEY unset); the agent endpoint answers 503"
            );
        }

        let stripe = match env_opt("STRIPE_SECRET_KEY") {
            Some(secret_key) => {
                if app_env == AppEnv::Production && !secret_key.starts_with("sk_live_") {
                    bail!("STRIPE_SECRET_KEY must be a live-mode key in production");
                }
                if app_env == AppEnv::Staging && secret_key.starts_with("sk_live_") {
                    bail!("STRIPE_SECRET_KEY must be a sandbox/test key in staging");
                }
                let webhook_secret = env_opt("STRIPE_WEBHOOK_SECRET");
                if webhook_secret.is_none() {
                    if strict {
                        bail!("STRIPE_WEBHOOK_SECRET is required when Stripe is configured in staging/production");
                    }
                    log::warn!(
                        "STRIPE_WEBHOOK_SECRET not set; webhook endpoint disabled"
                    );
                }
                Some(StripeConfig { secret_key, webhook_secret })
            }
            None => {
                log::info!("Stripe not configured (STRIPE_SECRET_KEY unset); billing endpoints answer 503");
                None
            }
        };

        let posthog = match env_opt("POSTHOG_API_KEY") {
            Some(api_key) => Some(PosthogConfig {
                api_key,
                host: env_opt("POSTHOG_HOST")
                    .unwrap_or_else(|| "https://us.i.posthog.com".to_string()),
            }),
            None => {
                log::info!("PostHog not configured (POSTHOG_API_KEY unset); analytics capture is a no-op");
                None
            }
        };

        let web_dist_dir = env_opt("WEB_DIST_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                let default = PathBuf::from("./apps/web/dist");
                default.is_dir().then_some(default)
            });

        let app_base_url = env_opt("APP_BASE_URL")
            .unwrap_or_else(|| format!("http://localhost:{port}"));

        Ok(Config {
            app_env,
            port,
            database_url,
            jwt_secret,
            git_data_dir,
            executor,
            app_base_url,
            stripe,
            posthog,
            web_dist_dir,
            plan_set: env_opt("PLAN_SET"),
            anthropic_api_key,
        })
    }
}
