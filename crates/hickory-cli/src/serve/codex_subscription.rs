//! Subscription inference, not a second agent loop. Codex owns all credentials.
use super::{
    LocalState,
    acp::{config, transport::Rpc},
    api::{ApiError, ApiResult},
};
use anyhow::{Context, Result, bail, ensure};
use async_trait::async_trait;
use axum::{Json, extract::State};
use hickory_agent::{ChatChunk, ChatStream, LlmClient, Message, Usage};
use serde_json::{Value, json};
use std::{path::Path, process::Stdio, sync::Arc, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub const SELECTOR: &str = "codex-subscription";
pub const DEFAULT_MODEL: &str = "gpt-6.1-sol";
const TIMEOUT: Duration = Duration::from_secs(30);
const LIMIT: usize = 800_000;
const DISABLED: &[&str] = &[
    "shell_tool",
    "unified_exec",
    "apps",
    "code_mode",
    "code_mode_host",
    "multi_agent",
    "multi_agent_v2",
    "browser_use",
    "browser_use_external",
    "computer_use",
    "view_image",
    "memories",
    "skill_search",
    "sleep_tool",
    "unbounded_connection_retries",
    "hooks",
    "plugins",
    "remote_plugin",
    "plugin_discovery_prefetch",
    "image_generation",
    "goals",
    "shell_snapshot",
    "tool_suggest",
    "skill_mcp_dependency_install",
];

pub fn select_client(
    state: &LocalState,
    selection: &super::agent::DocSelection,
    limit: Option<u32>,
) -> ApiResult<(Arc<dyn LlmClient>, String, String)> {
    if limit.is_some() {
        return Err(ApiError::unavailable(
            "Subscription Codex is unavailable for private prediction inference; no fallback was attempted.",
        ));
    }
    let mut client = Client::new(state, selection.model.as_deref())
        .map_err(|e| ApiError::unavailable(format!("{e:#}")))?;
    client.effort = selection.effort.clone();
    let model = client.model_name().to_string();
    Ok((Arc::new(client), SELECTOR.into(), model))
}

pub struct Client {
    effort: Option<String>,
    binary: std::path::PathBuf,
    model: String,
}
impl Client {
    pub fn new(state: &LocalState, model: Option<&str>) -> Result<Self> {
        ensure!(
            state.private_prediction.is_none() && !crate::engine::private_policy::active()?,
            "Subscription Codex is unavailable during private prediction; no fallback was attempted."
        );
        Ok(Self {
            effort: None,
            binary: binary(state)?,
            model: model.unwrap_or(DEFAULT_MODEL).into(),
        })
    }
}
fn binary(state: &LocalState) -> Result<std::path::PathBuf> {
    config::executable("codex", &config::directory(state))
        .context("Install Codex CLI 0.166 or later, then retry Codex — ChatGPT subscription.")
}
fn without_api_keys(cmd: &mut tokio::process::Command) {
    for key in ["OPENAI_API_KEY", "CODEX_API_KEY", "OPENAI_BASE_URL"] {
        cmd.env_remove(key);
    }
}
fn inference_command(
    binary: &Path,
    scratch: &Path,
    model: &str,
) -> Result<tokio::process::Command> {
    ensure!(
        cfg!(target_os = "macos") && Path::new("/usr/bin/sandbox-exec").is_file(),
        "Subscription Codex inference requires a macOS workspace server with sandbox-exec. Connect to that server and retry; Hickory will not switch to API billing."
    );
    let home = std::env::var_os("HOME")
        .context("HOME is missing; restart Hickory from your user account.")?;
    let home = Path::new(&home).canonicalize()?;
    ensure!(
        home != Path::new("/"),
        "User home is too broad for Codex confinement"
    );
    let codex_home = std::env::var_os("CODEX_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| home.join(".codex"));
    inference_command_for(binary, scratch, model, &home, &codex_home)
}
fn inference_command_for(
    binary: &Path,
    scratch: &Path,
    model: &str,
    home: &Path,
    codex_home: &Path,
) -> Result<tokio::process::Command> {
    let home = home.canonicalize()?;
    let scratch = scratch.canonicalize()?;
    let codex_home = codex_home
        .canonicalize()
        .context("Sign in to Codex first, then retry.")?;
    ensure!(
        codex_home != home && codex_home.starts_with(&home),
        "Codex credential directory must be inside your home and separate from it."
    );
    let q = |p: &Path| -> Result<String> {
        Ok(serde_json::to_string(
            p.to_str().context("Codex paths must be UTF-8")?,
        )?)
    };
    let denied = ["sessions", "archived_sessions", "memories", "history.jsonl"]
        .map(|name| codex_home.join(name));
    let mut cmd =
        super::subscription_process::command(binary, &scratch, &[codex_home], &[], &denied)?;
    cmd.args([
        "-a",
        "never",
        "exec",
        "--ignore-user-config",
        "--ignore-rules",
        "--ephemeral",
        "--skip-git-repo-check",
        "--sandbox",
        "read-only",
        "--json",
        "--color",
        "never",
        "--cd",
    ])
    .arg(&scratch)
    .args(["--model", model]);
    for setting in [
        "forced_login_method=\"chatgpt\"",
        "model_provider=\"openai\"",
        "web_search=\"disabled\"",
        "agents.enabled=false",
        "skills.include_instructions=false",
        "skills.bundled.enabled=false",
        "project_doc_max_bytes=0",
        "cloud.skills.enabled=false",
        "features.skip_host_skill_discovery=true",
    ] {
        cmd.args(["-c", setting]);
    }
    for (name, path) in [
        ("sqlite_home", scratch.join("state")),
        ("log_dir", scratch.join("logs")),
    ] {
        cmd.args(["-c", &format!("{name}={}", q(&path)?)]);
    }
    for feature in DISABLED {
        cmd.args(["--disable", feature]);
    }
    cmd.arg("-").current_dir(&scratch).kill_on_drop(true);
    without_api_keys(&mut cmd);
    Ok(cmd)
}
fn chatgpt_login(success: bool, stdout: &[u8], stderr: &[u8]) -> bool {
    success
        && (String::from_utf8_lossy(stdout).contains("Logged in using ChatGPT")
            || String::from_utf8_lossy(stderr).contains("Logged in using ChatGPT"))
}
fn startup_warning(message: &str) -> bool {
    message.starts_with("Under-development features enabled: skip_host_skill_discovery. ")
        || message
            == "Code Mode is unavailable because code-mode host is disabled. Code mode will fail closed; enable `features.code_mode_host` and install `codex-code-mode-host`."
}
#[derive(Default)]
struct Response {
    text: String,
    usage: Option<Usage>,
    completed: bool,
}
impl Response {
    fn event(&mut self, event: Value) -> Result<()> {
        match event["type"].as_str().unwrap_or("") {
            "thread.started" | "turn.started" => {}
            "item.started" | "item.updated" | "item.completed" => {
                let item = &event["item"];
                match item["type"].as_str().unwrap_or("") {
                    "error" if startup_warning(item["message"].as_str().unwrap_or("")) => {}
                    "reasoning" => {}
                    "agent_message" => {
                        if event["type"] == "item.completed" {
                            self.text
                                .push_str(item["text"].as_str().context("Codex text is missing")?);
                            ensure!(
                                self.text.len() <= LIMIT,
                                "Codex response exceeded the transport limit"
                            );
                        }
                    }
                    _ => bail!(
                        "Codex returned a native tool or unexpected item. No Hickory action was admitted. Update Codex and retry."
                    ),
                }
            }
            "turn.completed" => {
                ensure!(
                    !self.completed,
                    "Codex returned more than one completed turn"
                );
                self.completed = true;
                let u = &event["usage"];
                self.usage = match (
                    u["input_tokens"].as_u64(),
                    u["output_tokens"].as_u64(),
                    u["cached_input_tokens"].as_u64(),
                ) {
                    (Some(input), Some(output), Some(cache)) if cache <= input => Some(Usage {
                        input_tokens: input - cache,
                        output_tokens: output,
                        cache_read_input_tokens: cache,
                        cache_creation_input_tokens: 0,
                    }),
                    _ => None,
                };
            }
            _ => bail!(
                "Codex could not complete subscription inference. Check your ChatGPT sign-in, connection and usage limits; update Codex if needed. No API billing fallback was attempted."
            ),
        }
        Ok(())
    }
    fn finish(self) -> Result<ChatChunk> {
        ensure!(
            self.completed && !self.text.is_empty(),
            "Codex disconnected before completing a response. Check your network and sign in again; no API billing fallback was attempted."
        );
        Ok(ChatChunk {
            delta: self.text,
            usage: self.usage,
            finish_reason: Some("stop".into()),
            ..Default::default()
        })
    }
}
async fn collect(mut cmd: tokio::process::Command, prompt: String) -> Result<ChatChunk> {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Codex could not start. Install/update Codex CLI and retry.")?;
    // A dropped future (Stop or timeout) drops this kill-on-drop child. No shell descendants are allowed.
    let stdout = child.stdout.take().context("Codex stdout missing")?;
    let mut stdin = child.stdin.take().context("Codex stdin missing")?;
    let writer = async move {
        stdin.write_all(prompt.as_bytes()).await?;
        stdin.shutdown().await?;
        Result::<_>::Ok(())
    };
    let reader = async {
        let mut reader = BufReader::new(stdout);
        let mut response = Response::default();
        let mut bytes = 0;
        loop {
            // Bound each line before allocation can grow without limit.
            let mut line = Vec::new();
            loop {
                let buffer = reader.fill_buf().await?;
                if buffer.is_empty() {
                    break;
                }
                let count = buffer
                    .iter()
                    .position(|b| *b == b'\n')
                    .map_or(buffer.len(), |n| n + 1);
                ensure!(
                    line.len() + count <= LIMIT,
                    "Codex protocol line exceeded the limit"
                );
                line.extend_from_slice(&buffer[..count]);
                reader.consume(count);
                if line.last() == Some(&b'\n') {
                    break;
                }
            }
            if line.is_empty() {
                break;
            }
            bytes += line.len();
            ensure!(
                bytes <= LIMIT * 4,
                "Codex protocol output exceeded the limit"
            );
            response.event(
                serde_json::from_slice(&line)
                    .context("Codex sent invalid protocol JSON; update Codex and retry.")?,
            )?;
        }
        Result::<_>::Ok(response)
    };
    let (_, response) = tokio::try_join!(writer, reader)?;
    ensure!(
        child.wait().await?.success(),
        "Codex subscription request failed. Check sign-in, network and limits, or update Codex CLI. No API billing fallback was attempted."
    );
    response.finish()
}
#[async_trait]
impl LlmClient for Client {
    async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        use futures::StreamExt;
        let mut stream = self.complete_stream(messages).await?;
        Ok(stream
            .next()
            .await
            .context("Codex returned no response")??
            .delta)
    }
    async fn complete_stream(&self, messages: Vec<Message>) -> Result<ChatStream> {
        // Images require a separately audited input path; never silently omit one.
        ensure!(
            messages.iter().all(|m| m.images.is_empty()),
            "Subscription Codex does not yet support captured images. Remove the image context or explicitly select another provider."
        );
        let prompt = format!(
            "Continue this complete conversation with one assistant response. Use no native tools.\n{}",
            serde_json::to_string(&messages)?
        );
        ensure!(
            prompt.len() <= LIMIT,
            "Codex input exceeds the transport limit"
        );
        let scratch = tempfile::tempdir()?;
        let mut cmd = inference_command(&self.binary, scratch.path(), &self.model)?;
        if let Some(effort) = &self.effort {
            // Append an explicit config override; the enum is quoted as TOML.
            cmd.args(["-c", &format!("model_reasoning_effort={effort:?}")]);
        }
        let binary = self.binary.clone();
        Ok(Box::pin(futures::stream::once(async move {
            let _scratch = scratch;
            let mut check = tokio::process::Command::new(binary);
            check.args(["login", "status"]).kill_on_drop(true);
            without_api_keys(&mut check);
            let status = tokio::time::timeout(TIMEOUT, check.output())
                .await
                .context("Codex sign-in check timed out. Check your connection and retry.")??;
            ensure!(
                chatgpt_login(status.status.success(), &status.stdout, &status.stderr),
                "Sign in with ChatGPT in Agent settings before using subscription Codex. API-key authentication is not accepted; no billing fallback was attempted."
            );
            collect(cmd, prompt).await
        })))
    }
    fn provider_name(&self) -> &str {
        SELECTOR
    }
    fn model_name(&self) -> &str {
        &self.model
    }
}
struct Auth {
    rpc: Arc<Rpc>,
    events: tokio::sync::mpsc::UnboundedReceiver<Value>,
    _scratch: tempfile::TempDir,
}
#[derive(Default)]
pub struct Hub {
    auth: tokio::sync::Mutex<Option<Auth>>,
}
impl Auth {
    async fn start(state: &LocalState) -> Result<Self> {
        let scratch = tempfile::tempdir()?;
        let mut cmd = tokio::process::Command::new(binary(state)?);
        cmd.args([
            "app-server",
            "-c",
            "forced_login_method=\"chatgpt\"",
            "-c",
            "model_provider=\"openai\"",
            "-c",
            "mcp_servers={}",
        ])
        .current_dir(scratch.path());
        for feature in DISABLED {
            cmd.args(["--disable", feature]);
        }
        without_api_keys(&mut cmd);
        Self::from_command(cmd, scratch).await
    }
    async fn from_command(
        cmd: tokio::process::Command,
        scratch: tempfile::TempDir,
    ) -> Result<Self> {
        let (rpc, events) = Rpc::spawn(cmd)?;
        rpc.request("initialize", json!({"clientInfo":{"name":"hickory-docs","title":"Hickory Docs","version":env!("CARGO_PKG_VERSION")}}), TIMEOUT).await?;
        rpc.notify("initialized", json!({})).await?;
        Ok(Self {
            rpc,
            events,
            _scratch: scratch,
        })
    }
    async fn status(&mut self) -> Result<Value> {
        let mut error = None;
        while let Ok(event) = self.events.try_recv() {
            if event["method"] == "account/login/completed" && event["params"]["success"] == false {
                error = Some(
                    "ChatGPT sign-in did not complete. Retry sign-in and check your browser and network.",
                );
            }
        }
        let account = self
            .rpc
            .request("account/read", json!({"refreshToken":false}), TIMEOUT)
            .await?;
        Ok(
            json!({"ready":account["account"]["type"] == "chatgpt", "error":error,
            "billing":"chatgpt_subscription", "balance":null,"credit_cost":null}),
        )
    }
}
async fn auth(State(state): State<LocalState>, login: bool) -> ApiResult<Json<Value>> {
    let action = async {
        ensure!(
            state.private_prediction.is_none(),
            "Sign-in is unavailable in private prediction mode"
        );
        crate::engine::require_ordinary_agent_context_at(state.index.root()).await?;
        let mut slot = state.agent.subscription.auth.lock().await;
        if slot.is_none() {
            *slot = Some(Auth::start(&state).await?);
        }
        let auth = slot.as_mut().unwrap();
        let mut status = auth.status().await?;
        if login {
            let result = auth
                .rpc
                .request("account/login/start", json!({"type":"chatgpt"}), TIMEOUT)
                .await?;
            let url = result["authUrl"]
                .as_str()
                .context("Codex did not return a sign-in URL. Update Codex and retry.")?;
            ensure!(
                url.starts_with("https://auth.openai.com/")
                    || url.starts_with("https://chatgpt.com/"),
                "Codex returned an unsupported sign-in URL"
            );
            status["auth_url"] = json!(url);
        }
        Ok::<_, anyhow::Error>(Json(status))
    }
    .await;
    if action.is_err() {
        if let Some(auth) = state.agent.subscription.auth.lock().await.take() {
            auth.rpc.kill().await;
        }
    }
    action.map_err(|e| ApiError::unavailable(format!("Codex sign-in unavailable. Check your connection, install/update Codex CLI and retry: {e:#}")))
}
async fn status(state: State<LocalState>) -> ApiResult<Json<Value>> {
    auth(state, false).await
}
async fn login(state: State<LocalState>) -> ApiResult<Json<Value>> {
    auth(state, true).await
}
async fn usage(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let result = async {
        ensure!(state.private_prediction.is_none(), "Account usage is unavailable in private prediction mode");
        crate::engine::require_ordinary_agent_context_at(state.index.root()).await?;
        let mut slot = state.agent.subscription.auth.lock().await;
        if slot.is_none() { *slot = Some(Auth::start(&state).await?); }
        let auth = slot.as_mut().unwrap();
        let result = auth.rpc.request("account/rateLimits/read", json!({}), TIMEOUT).await?;
        // Forward only quota fields; no credentials or account email enter this response.
        Ok::<_, anyhow::Error>(Json(json!({
            "rateLimits": result["rateLimits"], "rateLimitsByLimitId": result["rateLimitsByLimitId"],
            "rateLimitResetCredits": result["rateLimitResetCredits"].as_object().map(|credits| json!({"availableCount":credits.get("availableCount")}))
        })))
    }.await;
    if result.is_err() {
        if let Some(auth) = state.agent.subscription.auth.lock().await.take() {
            auth.rpc.kill().await;
        }
    }
    result.map_err(|_| ApiError::unavailable("Codex usage is unavailable. Check native Codex sign-in and update Codex CLI, then retry."))
}
pub fn routes() -> axum::Router<LocalState> {
    axum::Router::new()
        .route(
            "/agents/codex-subscription",
            axum::routing::get(status).post(login),
        )
        .route(
            "/agents/codex-subscription/usage",
            axum::routing::get(usage),
        )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[tokio::test]
    async fn selection_keeps_subscription_identity_and_refuses_private_inference_without_fallback()
    {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("note.md"), "Fixture document").unwrap();
        let bin = root.path().join("agents/node_modules/.bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::os::unix::fs::symlink("/usr/bin/true", bin.join("codex")).unwrap();
        std::os::unix::fs::symlink("/usr/bin/true", bin.join("grok")).unwrap();
        let prepared = super::super::prepare(super::super::ServeOptions {
            target: root.path().to_path_buf(),
            port: 0,
            params: vec![],
            executor: crate::ExecutorChoice::Local,
            key_store_path: Some(root.path().join("keys.json")),
            ui_settings_path: Some(root.path().join("ui.json")),
        })
        .await
        .unwrap();
        let state = prepared.state;
        state
            .keys
            .store
            .write()
            .unwrap()
            .set("openai", Some("fixture-key-never-sent".into()))
            .unwrap();
        for (selector, expected_model) in [
            (SELECTOR, DEFAULT_MODEL),
            (
                super::super::grok_subscription::SELECTOR,
                super::super::grok_subscription::DEFAULT_MODEL,
            ),
        ] {
            let choice = super::super::agent::DocSelection {
                effort: None,
                provider: Some(selector.into()),
                model: None,
            };
            let (client, provider, model) =
                super::super::agent::selected_client(&state, &choice).unwrap();
            assert_eq!(provider, selector);
            assert_eq!(client.provider_name(), selector);
            assert_eq!(model, expected_model);
            assert!(
                super::super::agent::selected_client_with_limit(&state, &choice, Some(100), true)
                    .is_err()
            );
        }
    }
    #[test]
    fn subscription_totals_never_apply_an_api_price_even_for_a_priced_model() {
        let turn = super::super::agent::TurnRecord {
            backend: "python".into(),
            id: "fixture".into(),
            parent_id: None,
            prompt: String::new(),
            answer: None,
            status: "ok".into(),
            error: None,
            created_at: String::new(),
            provider: SELECTOR.into(),
            model: "claude-sonnet-5".into(),
            usage: Some(Usage {
                input_tokens: 1_000_000,
                ..Default::default()
            }),
            session: String::new(),
        };
        let (usage, usd) = super::super::agent::totals_of(&[turn]);
        assert_eq!(usage.input_tokens, 1_000_000);
        assert_eq!(usd, None);
    }
    fn message(kind: &str) -> Value {
        json!({"type":"item.completed","item":{"type":kind,"text":"{\"done\":{}}"}})
    }
    fn complete(usage: Value) -> Value {
        json!({"type":"turn.completed","usage":usage})
    }
    // Contract: .hickory-source/guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model/contract.md.
    #[test]
    fn subscription_events_preserve_known_tokens_and_unknown_usage() {
        let mut r = Response::default();
        r.event(message("agent_message")).unwrap();
        r.event(complete(
            json!({"input_tokens":120,"cached_input_tokens":40,"output_tokens":9}),
        ))
        .unwrap();
        let chunk = r.finish().unwrap();
        let usage = chunk.usage.unwrap();
        assert_eq!(
            (
                usage.input_tokens,
                usage.cache_read_input_tokens,
                usage.output_tokens
            ),
            (80, 40, 9)
        );
        let mut missing = Response::default();
        missing.event(message("agent_message")).unwrap();
        missing.event(complete(json!({}))).unwrap();
        assert!(missing.finish().unwrap().usage.is_none());
    }
    #[test]
    fn only_known_startup_warnings_survive_and_tools_fail_closed() {
        let warnings = [
            "Under-development features enabled: skip_host_skill_discovery. Under-development features are incomplete.",
            "Code Mode is unavailable because code-mode host is disabled. Code mode will fail closed; enable `features.code_mode_host` and install `codex-code-mode-host`.",
        ];
        for warning in warnings {
            let mut r = Response::default();
            r.event(json!({"type":"item.completed","item":{"type":"error","message":warning}}))
                .unwrap();
            r.event(message("agent_message")).unwrap();
            r.event(complete(json!({}))).unwrap();
            assert!(r.finish().is_ok());
        }
        for kind in [
            "command_execution",
            "file_change",
            "mcp_tool_call",
            "web_search",
            "error",
            "future_tool",
        ] {
            assert!(Response::default().event(message(kind)).is_err(), "{kind}");
        }
        for kind in ["error", "turn.failed", "future_event"] {
            assert!(Response::default().event(json!({"type":kind})).is_err());
        }
        assert!(Response::default().finish().is_err());
    }
    #[test]
    fn authentication_requires_successful_chatgpt_login_never_api_keys() {
        assert!(chatgpt_login(true, b"", b"Logged in using ChatGPT\n"));
        assert!(!chatgpt_login(true, b"Logged in using an API key", b""));
        assert!(!chatgpt_login(
            false,
            b"Logged in using ChatGPT",
            b"network error"
        ));
        assert!(!chatgpt_login(true, b"", b""));
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn inference_command_forces_subscription_and_disables_native_tools() {
        let scratch = tempfile::tempdir().unwrap();
        // No Codex process starts here. /usr/bin/true stands in for the executable.
        let home = scratch.path().join("home");
        let credentials = home.join(".codex");
        std::fs::create_dir_all(&credentials).unwrap();
        let command = inference_command_for(
            Path::new("/usr/bin/true"),
            scratch.path(),
            "fixture",
            &home,
            &credentials,
        )
        .unwrap();
        let command = command.as_std();
        let args: Vec<_> = command
            .get_args()
            .map(|x| x.to_string_lossy().to_string())
            .collect();
        let joined = args.join(" ");
        for required in [
            "forced_login_method=\"chatgpt\"",
            "model_provider=\"openai\"",
            "--ignore-user-config",
            "--ignore-rules",
            "--ephemeral",
            "read-only",
            "(deny process-exec (require-not",
            "(deny file-read-data (require-all",
        ] {
            assert!(joined.contains(required), "{required}");
        }
        for feature in DISABLED {
            assert!(args.windows(2).any(|a| a == ["--disable", *feature]));
        }
        for key in ["OPENAI_API_KEY", "CODEX_API_KEY", "OPENAI_BASE_URL"] {
            assert!(
                command
                    .get_envs()
                    .any(|(name, value)| name == key && value.is_none())
            );
        }
    }
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn sandbox_denies_workspace_reads_writes_and_native_child_processes() {
        let files = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let home = files.path().join("home");
        let credentials = home.join(".codex");
        std::fs::create_dir_all(&credentials).unwrap();
        let secret = files.path().join("workspace.txt");
        std::fs::write(&secret, "private").unwrap();
        let command = inference_command_for(
            Path::new("/bin/bash"),
            scratch.path(),
            "fixture",
            &home,
            &credentials,
        )
        .unwrap();
        let args: Vec<_> = command.as_std().get_args().collect();
        let profile = args[1];
        let ready = tokio::process::Command::new("/usr/bin/sandbox-exec")
            .arg("-p")
            .arg(profile)
            .args(["/bin/bash", "-c", ":"])
            .current_dir(scratch.path())
            .status()
            .await
            .unwrap();
        assert!(
            ready.success(),
            "sandbox fixture must start before testing denials"
        );
        for script in [
            format!(
                "read value < {}",
                serde_json::to_string(secret.to_str().unwrap()).unwrap()
            ),
            format!(
                "echo changed > {}",
                serde_json::to_string(secret.to_str().unwrap()).unwrap()
            ),
            "/usr/bin/true".into(),
        ] {
            let status = tokio::process::Command::new("/usr/bin/sandbox-exec")
                .args(["-p"])
                .arg(profile)
                .args(["/bin/bash", "-c", &script])
                .stderr(Stdio::null())
                .status()
                .await
                .unwrap();
            assert!(!status.success(), "sandbox unexpectedly admitted {script}");
        }
        assert_eq!(std::fs::read_to_string(secret).unwrap(), "private");
    }
    fn fixture(script: &str) -> tokio::process::Command {
        let mut command = tokio::process::Command::new("python3");
        command.args(["-c", script]).kill_on_drop(true);
        command
    }
    #[tokio::test]
    async fn fixture_transport_rejects_failed_exit_even_after_completion() {
        let script = "import json,sys; sys.stdin.read(); print(json.dumps({'type':'item.completed','item':{'type':'agent_message','text':'answer'}})); print(json.dumps({'type':'turn.completed'})); sys.exit(1)";
        assert!(collect(fixture(script), "fixture".into()).await.is_err());
        let script = script.replace("sys.exit(1)", "sys.exit(0)");
        let chunk = collect(fixture(&script), "fixture".into()).await.unwrap();
        assert_eq!(chunk.delta, "answer");
        assert!(chunk.usage.is_none());
    }
    #[tokio::test]
    async fn account_protocol_reuses_chatgpt_and_refuses_api_key_sessions() {
        for (kind, ready) in [("chatgpt", true), ("apiKey", false), ("none", false)] {
            let script = format!(
                r#"import sys,json
for line in sys.stdin:
    m=json.loads(line)
    if 'id' not in m: continue
    account=None if {kind:?} == 'none' else {{'type':{kind:?}}}
    result={{'account':account}} if m['method']=='account/read' else {{}}
    print(json.dumps({{'id':m['id'],'result':result}}),flush=True)
"#
            );
            let mut auth = Auth::from_command(fixture(&script), tempfile::tempdir().unwrap())
                .await
                .unwrap();
            let status = auth.status().await.unwrap();
            assert_eq!(status["ready"], ready);
            assert!(status["balance"].is_null());
            assert!(status["credit_cost"].is_null());
        }
        let unavailable = Auth::from_command(
            fixture("import sys; sys.exit(1)"),
            tempfile::tempdir().unwrap(),
        )
        .await;
        assert!(unavailable.is_err());
    }
    // Contract: .hickory-source/guarantees/agent/a-running-agent-can-be-stopped/contract.md.
    #[cfg(unix)]
    #[tokio::test]
    async fn dropping_inference_kills_the_transport_process() {
        let dir = tempfile::tempdir().unwrap();
        let pid = dir.path().join("pid");
        let script = format!(
            "import os,time,pathlib; pathlib.Path({}).write_text(str(os.getpid())); time.sleep(60)",
            serde_json::to_string(pid.to_str().unwrap()).unwrap()
        );
        let task = tokio::spawn(collect(fixture(&script), String::new()));
        tokio::time::timeout(Duration::from_secs(5), async {
            while !pid.is_file() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let pid = std::fs::read_to_string(pid).unwrap();
        task.abort();
        let _ = task.await;
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let status = tokio::process::Command::new("/bin/kill")
                    .args(["-0", &pid])
                    .stderr(Stdio::null())
                    .status()
                    .await
                    .unwrap();
                if !status.success() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("cancelled Codex child remained alive");
    }
}
