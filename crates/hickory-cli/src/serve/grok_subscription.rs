//! Native Grok subscription inference; Hickory remains the only agent loop.
use super::{
    LocalState,
    acp::{config, transport::Rpc},
    api::{ApiError, ApiResult},
    subscription_process as process,
};
use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use axum::{Json, extract::State};
use hickory_agent::{ChatChunk, ChatStream, LlmClient, Message, Usage};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
pub const SELECTOR: &str = "grok-subscription";
pub const DEFAULT_MODEL: &str = "grok-build";
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);
const NO_FALLBACK: &str = "No API billing fallback was attempted.";
fn binary(state: &LocalState) -> Result<PathBuf> {
    config::executable("grok", &config::directory(state))
        .context("Install Grok Build CLI 1.0.64 or later, then retry Grok — subscription.")
}
struct Profile {
    scratch: tempfile::TempDir,
    home: PathBuf,
    auth: PathBuf,
}
impl Profile {
    fn new(model: &str) -> Result<Self> {
        ensure!(
            model.starts_with("grok-")
                && model.len() <= 100
                && model
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)),
            "Choose a first-party grok- model id; custom providers are unavailable for subscription inference."
        );
        // Managed requirements outrank our pins. Do not silently bypass them.
        ensure!(
            !Path::new("/etc/grok/requirements.toml").exists()
                && !Path::new("/etc/grok/config.toml").exists(),
            "Grok subscription inference cannot yet verify managed /etc/grok policy. Ask your administrator to use the native Grok agent; no billing fallback was attempted."
        );
        let home = PathBuf::from(
            std::env::var_os("HOME")
                .context("HOME is missing; restart Hickory from your user account")?,
        )
        .canonicalize()?;
        ensure!(
            home != Path::new("/"),
            "User home is too broad for Grok confinement"
        );
        let auth = std::env::var_os("GROK_AUTH_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::var_os("GROK_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home.join(".grok"))
                    .join("auth.json")
            });
        let parent = auth
            .parent()
            .context("Grok credential path has no parent")?;
        // Missing default directory is a normal first sign-in, not an auth-file read.
        if !parent.exists() && parent == home.join(".grok") {
            std::fs::create_dir(parent)?;
        }
        let parent = parent
            .canonicalize()
            .context("Create your Grok credential directory or run grok login, then retry")?;
        ensure!(
            parent != home && parent.starts_with(&home),
            "Grok credentials must be in a separate directory inside your home"
        );
        let auth = parent.join(
            auth.file_name()
                .context("Grok credential file name missing")?,
        );
        ensure!(
            !auth.is_symlink(),
            "Grok credential symlinks are not supported; use the native CLI credential path"
        );
        Self::at(model, home, auth)
    }
    fn at(model: &str, home: PathBuf, auth: PathBuf) -> Result<Self> {
        let scratch = tempfile::tempdir()?;
        let config = format!(
            "[grok_com_config]\ndisable_api_key_auth = true\npreferred_method = \"oidc\"\n[models]\ndefault = {model:?}\n[model.{model:?}]\nmodel = {model:?}\nbase_url = \"https://api.x.ai/v1\"\n[skills]\nignore = [\"/\"]\n"
        );
        for name in ["config.toml", "requirements.toml"] {
            std::fs::write(scratch.path().join(name), &config)?;
        }
        Ok(Self {
            scratch,
            home,
            auth,
        })
    }
    fn environment(&self, cmd: &mut tokio::process::Command) {
        // Allowlist transport/runtime settings; no credential or endpoint env inheritance.
        cmd.env_clear();
        for key in [
            "PATH",
            "LANG",
            "LC_ALL",
            "HTTPS_PROXY",
            "HTTP_PROXY",
            "ALL_PROXY",
            "NO_PROXY",
            "https_proxy",
            "http_proxy",
            "all_proxy",
            "no_proxy",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
        ] {
            if let Some(value) = std::env::var_os(key) {
                cmd.env(key, value);
            }
        }
        cmd.env("HOME", &self.home)
            .env("TMPDIR", self.scratch.path())
            .env("GROK_HOME", self.scratch.path())
            .env("GROK_AUTH_PATH", &self.auth)
            .env("GROK_DISABLE_API_KEY_AUTH", "true");
        for vendor in ["CURSOR", "CLAUDE"] {
            for surface in ["SKILLS", "RULES", "AGENTS", "MCPS", "HOOKS"] {
                cmd.env(format!("GROK_{vendor}_{surface}_ENABLED"), "false");
            }
        }
        cmd.current_dir(self.scratch.path()).kill_on_drop(true);
    }
    fn command(&self, binary: &Path) -> Result<tokio::process::Command> {
        let mut cmd = process::command(
            binary,
            self.scratch.path(),
            &[],
            std::slice::from_ref(&self.auth),
            &[],
        )?;
        self.environment(&mut cmd);
        Ok(cmd)
    }
    async fn inspect(&self, binary: &Path) -> Result<()> {
        let mut cmd = self.command(binary)?;
        cmd.args(["--no-auto-update", "inspect", "--json"]);
        let bytes = tokio::time::timeout(CHECK_TIMEOUT, process::capture(cmd))
            .await
            .context("Grok configuration check timed out; check your network and retry")??;
        inspect(
            &serde_json::from_slice(&bytes)
                .context("Grok inspect returned invalid JSON; update Grok CLI")?,
        )
    }
    async fn authenticated(&self, binary: &Path) -> Result<bool> {
        let mut cmd = self.command(binary)?;
        cmd.args(["--no-auto-update", "agent", "stdio"]);
        let (rpc, _events) = Rpc::spawn(cmd)?;
        let result = cached_auth(&rpc).await;
        rpc.kill().await;
        result
    }
}
fn inspect(report: &Value) -> Result<()> {
    ensure!(
        report["loginPolicy"]["apiKeyAuthDisabled"] == true,
        "Grok could not enforce subscription-only authentication. Update Grok CLI; no API billing fallback was attempted."
    );
    for field in [
        "projectInstructions",
        "hooks",
        "skills",
        "agents",
        "plugins",
        "mcpServers",
        "lspServers",
    ] {
        let entries = report[field]
            .as_array()
            .context("Grok inspect lacks safety fields; update Grok CLI")?;
        ensure!(
            entries.iter().all(|e| e["disabled"] == true
                || (field == "agents" && e["source"]["type"] == "builtin")),
            "Grok loaded native instructions or extensions. Subscription inference requires an isolated tool-free configuration; no request was sent."
        );
    }
    ensure!(
        report
            .get("configWarnings")
            .is_none_or(|v| v.as_array().is_some_and(Vec::is_empty))
            && report
                .get("mcpConfigProblems")
                .is_none_or(|v| v.as_array().is_some_and(Vec::is_empty)),
        "Grok reported configuration warnings; update the CLI and retry. No request was sent."
    );
    Ok(())
}
async fn cached_auth(rpc: &Rpc) -> Result<bool> {
    let init = rpc.request("initialize", json!({"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"hickory-docs","version":env!("CARGO_PKG_VERSION")}}), CHECK_TIMEOUT).await?;
    let methods = init["authMethods"]
        .as_array()
        .context("Grok did not expose supported authentication methods; update Grok CLI")?;
    ensure!(
        !methods.iter().any(|m| m["id"] == "xai.api_key"),
        "Grok still offers API-key authentication; no request was sent. Update Grok CLI."
    );
    if !methods.iter().any(|m| m["id"] == "cached_token") {
        return Ok(false);
    }
    // Failure can mean expired auth or network trouble. Never select another method.
    rpc.request("authenticate", json!({"methodId":"cached_token","_meta":{"headless":true}}), CHECK_TIMEOUT).await
        .context("Grok could not reuse subscription sign-in. Sign in again, check your connection and usage limits; no API billing fallback was attempted")?;
    Ok(true)
}
fn response(bytes: &[u8]) -> Result<ChatChunk> {
    let value: Value = serde_json::from_slice(bytes)
        .context("Grok returned invalid response JSON; update Grok CLI")?;
    ensure!(
        value.get("type").is_none()
            && value["stopReason"] == "end_turn"
            && value.get("num_turns").is_none_or(|n| n == 1),
        "Grok did not complete a single tool-free response. Check sign-in, connection and limits; no API billing fallback was attempted."
    );
    let text = value["text"]
        .as_str()
        .filter(|s| !s.is_empty())
        .context("Grok returned no response text")?;
    let u = &value["usage"];
    let usage = if value["usage_is_incomplete"] == true {
        None
    } else {
        match (
            u["input_tokens"].as_u64(),
            u["output_tokens"].as_u64(),
            u["cache_read_input_tokens"].as_u64(),
            u["cache_creation_input_tokens"].as_u64(),
        ) {
            (
                Some(input_tokens),
                Some(output_tokens),
                Some(cache_read_input_tokens),
                Some(cache_creation_input_tokens),
            ) => Some(Usage {
                input_tokens,
                output_tokens,
                cache_read_input_tokens,
                cache_creation_input_tokens,
            }),
            _ => None,
        }
    };
    Ok(ChatChunk {
        delta: text.into(),
        usage,
        finish_reason: Some("stop".into()),
        ..Default::default()
    })
}
fn inference_command(
    profile: &Profile,
    binary: &Path,
    model: &str,
) -> Result<tokio::process::Command> {
    let mut cmd = profile.command(binary)?;
    cmd.args(["--prompt-file"])
        .arg(profile.scratch.path().join("prompt.txt"))
        .args([
            "--output-format",
            "json",
            "--tools",
            "WebSearch",
            "--disallowed-tools",
            "web_search,web_fetch,search_tool,use_tool",
            "--no-plan",
            "--no-subagents",
            "--no-memory",
            "--disable-web-search",
            "--max-turns",
            "1",
            "--sandbox",
            "read-only",
            "--permission-mode",
            "dontAsk",
            "--no-auto-update",
            "--model",
            model,
        ]);
    Ok(cmd)
}
pub struct Client {
    binary: PathBuf,
    model: String,
}
impl Client {
    pub fn new(state: &LocalState, model: Option<&str>) -> Result<Self> {
        ensure!(
            state.private_prediction.is_none() && !crate::engine::private_policy::active()?,
            "Subscription Grok is unavailable during private prediction; no fallback was attempted."
        );
        let model = model.unwrap_or(DEFAULT_MODEL).to_owned();
        ensure!(
            model.starts_with("grok-")
                && model.len() <= 100
                && model
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)),
            "Choose a first-party grok- model id"
        );
        Ok(Self {
            binary: binary(state)?,
            model,
        })
    }
}
#[async_trait]
impl LlmClient for Client {
    async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        use futures::StreamExt;
        Ok(self
            .complete_stream(messages)
            .await?
            .next()
            .await
            .context("Grok returned no response")??
            .delta)
    }
    async fn complete_stream(&self, messages: Vec<Message>) -> Result<ChatStream> {
        ensure!(
            messages.iter().all(|m| m.images.is_empty()),
            "Subscription Grok does not yet support captured images. Remove image context or explicitly select another provider."
        );
        let prompt = format!(
            "Continue this complete conversation with one assistant response. Use no native tools.\n{}",
            serde_json::to_string(&messages)?
        );
        ensure!(
            prompt.len() <= process::LIMIT,
            "Grok input exceeds the transport limit"
        );
        let profile = Profile::new(&self.model)?;
        std::fs::write(profile.scratch.path().join("prompt.txt"), prompt)?;
        let binary = self.binary.clone();
        let model = self.model.clone();
        Ok(Box::pin(futures::stream::once(async move {
            profile.inspect(&binary).await?;
            ensure!(
                profile.authenticated(&binary).await?,
                "Sign in with Grok in Agent settings, then retry. API-key authentication is not accepted. {NO_FALLBACK}"
            );
            let cmd = inference_command(&profile, &binary, &model)?;
            let bytes = process::capture(cmd).await?;
            response(&bytes)
        })))
    }
    fn provider_name(&self) -> &str {
        SELECTOR
    }
    fn model_name(&self) -> &str {
        &self.model
    }
}
#[derive(Default)]
pub struct Hub {
    login: tokio::sync::Mutex<Option<NativeLogin>>,
}
struct NativeLogin {
    task: tokio::task::JoinHandle<bool>,
}
impl Drop for NativeLogin {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn auth(State(state): State<LocalState>, login: bool) -> ApiResult<Json<Value>> {
    let result = async {
        ensure!(state.private_prediction.is_none(), "Sign-in is unavailable in private prediction mode");
        crate::engine::require_ordinary_agent_context_at(state.index.root()).await?;
        let binary = binary(&state)?;
        let profile = Profile::new(DEFAULT_MODEL)?;
        profile.inspect(&binary).await?;
        let mut slot = state.agent.grok_subscription.login.lock().await;
        let mut error = None;
        if slot.as_ref().is_some_and(|pending| pending.task.is_finished()) {
            let mut pending = slot.take().unwrap();
            if !(&mut pending.task).await.unwrap_or(false) {
                error = Some("Grok sign-in failed or timed out. Run grok login in a terminal, then Check sign-in. Check your browser and network.");
            }
        }
        // Native login owns the browser exchange. No OAuth result enters application code.
        if login && slot.is_none() {
            let mut cmd = tokio::process::Command::new(binary);
            profile.environment(&mut cmd);
            cmd.args(["--no-auto-update", "login", "--oauth"])
                .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
            let mut child = cmd.spawn().context("Grok sign-in could not start; run grok login in a terminal, then Check sign-in")?;
            let task = tokio::spawn(async move {
                let _scratch = profile.scratch;
                tokio::time::timeout(Duration::from_secs(300), child.wait()).await
                    .is_ok_and(|result| result.is_ok_and(|exit| exit.success()))
            });
            *slot = Some(NativeLogin { task });
            return Ok::<_,anyhow::Error>(Json(json!({"ready":false,"login_pending":true,"billing":"grok_subscription","balance":null,"credit_cost":null})));
        }
        let ready = if slot.is_some() { false } else { profile.authenticated(&binary).await? };
        Ok(Json(json!({"ready":ready,"login_pending":slot.is_some(),"error":error,"billing":"grok_subscription","balance":null,"credit_cost":null})))
    }.await;
    result.map_err(|e| ApiError::unavailable(format!("Grok sign-in unavailable. Install/update Grok Build CLI, check your connection, or run grok login then retry: {e:#}")))
}
async fn status(state: State<LocalState>) -> ApiResult<Json<Value>> {
    auth(state, false).await
}
async fn login(state: State<LocalState>) -> ApiResult<Json<Value>> {
    auth(state, true).await
}
pub fn routes() -> axum::Router<LocalState> {
    axum::Router::new().route(
        "/agents/grok-subscription",
        axum::routing::get(status).post(login),
    )
}
#[cfg(test)]
mod tests;
