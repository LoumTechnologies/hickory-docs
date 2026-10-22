use super::{Attach, Attached, Endpoint, HEARTBEAT, PROTOCOL, WORKER, directory};
use anyhow::{Context, Result, bail, ensure};
use fs2::FileExt;
use std::{process::Stdio, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Connection {
    endpoint: Arc<RwLock<Endpoint>>,
    ended: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) attach: Attach,
    pub slot: String,
    pub log_offset: u64,
    pub(crate) http: reqwest::Client,
}

impl Connection {
    pub(crate) async fn url(&self, path: &str) -> (String, String) {
        let endpoint = self.endpoint.read().await;
        (format!("{}{}", endpoint.url, path), endpoint.token.clone())
    }
    pub(crate) async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> reqwest::RequestBuilder {
        let (url, token) = self.url(path).await;
        self.http.request(method, url).bearer_auth(token)
    }
    pub(crate) async fn api_url(&self, path: &str) -> (String, String) {
        self.url(&format!("/clients/{}{}", self.attach.client, path))
            .await
    }
    pub async fn reconnect(&self) -> Result<()> {
        ensure!(
            !self.ended.load(std::sync::atomic::Ordering::SeqCst),
            "Private session ended; open a new window for a fresh context"
        );
        let endpoint = discover().await?;
        let _ = attach(&self.http, &endpoint, &self.attach).await?;
        *self.endpoint.write().await = endpoint;
        Ok(())
    }
    pub async fn detach(&self) -> Result<()> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/clients/{}/lease", self.attach.client),
        )
        .await
        .send()
        .await?;
        Ok(())
    }
    /// Human host lifecycle; this is deliberately absent from model catalogues.
    pub(crate) async fn end_private(&self) -> Result<serde_json::Value> {
        let response = self
            .request(reqwest::Method::POST, "/private/end")
            .await
            .json(&serde_json::json!({"client":self.attach.client}))
            .send()
            .await?;
        let success = response.status().is_success();
        let value: serde_json::Value = response.json().await?;
        ensure!(
            success && value["ended"] == true,
            "{}",
            value["error"]
                .as_str()
                .unwrap_or("Private session could not end")
        );
        self.ended.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(value)
    }
    pub fn guard(&self) -> ClientGuard {
        let connection = self.clone();
        let task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(HEARTBEAT).await;
                if connection.ended.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let lease = connection
                    .request(
                        reqwest::Method::POST,
                        &format!("/clients/{}/lease", connection.attach.client),
                    )
                    .await
                    .send()
                    .await;
                if !lease.is_ok_and(|r| r.status().is_success())
                    && let Err(error) = connection.reconnect().await
                {
                    log::warn!("engine reconnect: {error:#}");
                }
            }
        });
        ClientGuard {
            task,
            connection: self.clone(),
        }
    }
}

pub struct ClientGuard {
    task: tokio::task::JoinHandle<()>,
    connection: Connection,
}
impl Drop for ClientGuard {
    fn drop(&mut self) {
        self.task.abort();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let connection = self.connection.clone();
            runtime.spawn(async move {
                let _ = connection.detach().await;
            });
        }
    }
}

pub async fn connect(attach_opts: Attach) -> Result<Connection> {
    let endpoint = discover().await?;
    let http = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(std::time::Duration::from_secs(2))
        .build()?;
    let attached = attach(&http, &endpoint, &attach_opts).await?;
    Ok(Connection {
        endpoint: Arc::new(RwLock::new(endpoint)),
        ended: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        attach: attach_opts,
        slot: attached.slot,
        log_offset: attached.log_offset,
        http,
    })
}

async fn attach(http: &reqwest::Client, endpoint: &Endpoint, opts: &Attach) -> Result<Attached> {
    if opts.prediction.is_some() {
        let response = http
            .get(format!("{}/health", endpoint.url))
            .bearer_auth(&endpoint.token)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await?;
        ensure!(
            response.status().is_success(),
            "Private prediction engine capability unavailable; no attach attempted"
        );
        let health: serde_json::Value = response.json().await?;
        ensure!(
            health["capabilities"]
                .as_array()
                .is_some_and(|capabilities| capabilities
                    .iter()
                    .any(|capability| capability == super::PREDICTION_PROTOCOL)),
            "This engine cannot enforce private prediction startup; close its clients and reopen with a supporting build. No ordinary attach fallback was attempted"
        );
    }
    let response = http
        .post(format!("{}/attach", endpoint.url))
        .bearer_auth(&endpoint.token)
        .json(opts)
        .send()
        .await?;
    if !response.status().is_success() {
        bail!(
            "{}",
            response.json::<serde_json::Value>().await?["error"]
                .as_str()
                .unwrap_or("Could not connect to the local engine")
        );
    }
    let attached: Attached = response.json().await?;
    if let Some(profile) = &opts.prediction {
        ensure!(
            attached.prediction.as_ref() == Some(&profile.fingerprint()?),
            "Engine did not acknowledge the exact private startup binding; no ordinary fallback is permitted"
        );
    }
    Ok(attached)
}

async fn live(http: &reqwest::Client, endpoint: &Endpoint) -> Result<bool> {
    let Ok(response) = http
        .get(format!("{}/health", endpoint.url))
        .bearer_auth(&endpoint.token)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    else {
        return Ok(false);
    };
    if !response.status().is_success() {
        return Ok(false);
    }
    if endpoint.protocol != PROTOCOL {
        bail!(
            "The running Hickory engine uses a different protocol. Finish its active work and close its clients before reopening with this version."
        );
    }
    let health: serde_json::Value = response.json().await?;
    ensure!(
        health["protocol"] == PROTOCOL,
        "The running Hickory engine uses an incompatible protocol. Finish its work and close its clients before reopening with this version."
    );
    Ok(true)
}

async fn discover() -> Result<Endpoint> {
    let dir = directory()?;
    let rendezvous = dir.clone();
    let startup = tokio::task::spawn_blocking(move || -> Result<std::fs::File> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(rendezvous.join("startup.lock"))?;
        file.lock_exclusive()?;
        Ok(file)
    })
    .await??;
    let http = reqwest::Client::builder().no_proxy().build()?;
    if let Ok(bytes) = std::fs::read(dir.join("endpoint.json"))
        && let Ok(endpoint) = serde_json::from_slice::<Endpoint>(&bytes)
        && live(&http, &endpoint).await?
    {
        return Ok(endpoint);
    }
    let owner = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("owner.lock"))?;
    if owner.try_lock_exclusive().is_ok() {
        // Stale discovery is replaceable; the lock file is never removed.
        FileExt::unlock(&owner)?;
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("engine.log"))?;
        let mut executable = std::env::current_exe()?;
        if executable
            .parent()
            .and_then(|p| p.file_name())
            .is_some_and(|n| n == "deps")
        {
            let dir = executable.parent().unwrap().parent().unwrap();
            executable = ["hick", "hickory-desktop"]
                .into_iter()
                .map(|name| dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX)))
                .find(|p| p.is_file())
                .context("build the Hickory executable before running engine integration tests")?;
        }
        let mut command = std::process::Command::new(executable);
        command
            .arg("__engine")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(log));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            // Survives its launching window, including a terminal's Ctrl+C.
            command.process_group(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000 | 0x00000200); // NO_WINDOW | NEW_PROCESS_GROUP
        }
        let child = command
            .spawn()
            .context("starting the local Hickory engine")?;
        std::thread::spawn(move || {
            let mut child = child;
            let _ = child.wait();
        });
    }
    drop(owner);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        if let Ok(bytes) = std::fs::read(dir.join("endpoint.json"))
            && let Ok(endpoint) = serde_json::from_slice::<Endpoint>(&bytes)
            && live(&http, &endpoint).await?
        {
            drop(startup);
            return Ok(endpoint);
        }
        ensure!(
            tokio::time::Instant::now() < deadline,
            "The local Hickory engine did not become ready. Its diagnostic log is {}. Reopen Hickory after resolving the error in that log.",
            dir.join("engine.log").display()
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

/// A context path without private snapshot isolation cannot start or continue
/// while this engine owns a private workspace. This is refusal, not a fallback.
pub(crate) async fn private_prediction_active() -> Result<bool> {
    if super::private_policy::active()? {
        return Ok(true);
    }
    let path = hickory_workspace::data_root()?.join("engine/endpoint.json");
    if !path.exists() {
        return Ok(false);
    }
    let endpoint = discover().await?;
    let http = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(2))
        .build()?;
    let health: serde_json::Value = http
        .get(format!("{}/health", endpoint.url))
        .bearer_auth(&endpoint.token)
        .send()
        .await?
        .json()
        .await?;
    if !health["capabilities"]
        .as_array()
        .is_some_and(|caps| caps.iter().any(|cap| cap == super::PREDICTION_PROTOCOL))
    {
        return Ok(false);
    }
    let response = http
        .get(format!("{}/prediction-policy", endpoint.url))
        .bearer_auth(&endpoint.token)
        .send()
        .await?;
    ensure!(
        response.status().is_success(),
        "Private context policy unavailable; no ordinary agent context fallback"
    );
    let policy: serde_json::Value = response.json().await?;
    ensure!(
        policy["active"].is_boolean(),
        "Invalid private prediction policy response"
    );
    Ok(policy["active"] == true)
}
pub(crate) async fn require_ordinary_agent_context_at(root: &std::path::Path) -> Result<()> {
    ensure!(
        !super::private_policy::overlaps(&root.canonicalize()?)?,
        "Ordinary agent context cannot read a retained private workspace or evidence store; reopen its guarded private practice"
    );
    require_ordinary_agent_context().await
}
pub(crate) async fn require_ordinary_agent_context() -> Result<()> {
    ensure!(
        !private_prediction_active().await?,
        "This agent context path is unavailable while the engine retains a private prediction workspace; use its guarded MCP or captured-source tools"
    );
    Ok(())
}

pub async fn up(config: crate::up::UpConfig) -> Result<()> {
    // Keep the CLI's useful empty-folder diagnosis.
    crate::expand_docs(&config.root)?;
    let mut opts = Attach::new(config.root.clone(), config.executor);
    opts.params = config.params;
    opts.run = config.run;
    let connection = connect(opts).await?;
    let _guard = connection.guard();
    eprintln!(
        "hick up: connected to the local engine for {}",
        config.root.display()
    );
    let mut offset = connection.log_offset;
    loop {
        let path = directory()?.join("engine.log");
        if let Ok(mut log) = std::fs::File::open(path) {
            use std::io::{Read, Seek};
            log.seek(std::io::SeekFrom::Start(offset))?;
            let mut bytes = Vec::new();
            log.read_to_end(&mut bytes)?;
            offset += bytes.len() as u64;
            eprint!("{}", String::from_utf8_lossy(&bytes));
        }
        tokio::select! {
            _ = tokio::signal::ctrl_c() => { connection.detach().await?; return Ok(()); }
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
        }
    }
}

/// All one-shot CLI writers run as an engine-owned child under the same write
/// coordinator. Workers inherit the marker so git hooks never recurse into it.
pub async fn forward_cli(args: &[String]) -> Result<Option<i32>> {
    if std::env::var_os(WORKER).is_some() {
        return Ok(None);
    }
    let Some(verb) = args.first().map(String::as_str) else {
        return Ok(None);
    };
    // Python supervision owns a distinct terminal approval input. A daemon
    // command has piped stdin and cannot collect that independent approval.
    if verb == "agent" {
        return Ok(None);
    }
    if !matches!(
        verb,
        "run"
            | "test"
            | "weave"
            | "ingest"
            | "refresh"
            | "init"
            | "doc"
            | "repair"
            | "merge-driver"
            | "merge-generated"
    ) || args.iter().any(|a| a == "--help" || a == "-h")
    {
        return Ok(None);
    }
    let endpoint = discover().await?;
    let opts = super::daemon::CommandRequest {
        executable: std::env::current_exe()?,
        args: args.to_vec(),
        cwd: std::env::current_dir()?,
        env: std::env::vars().collect(),
        input: {
            use std::io::{IsTerminal, Read};
            let mut input = Vec::new();
            if !std::io::stdin().is_terminal() {
                std::io::stdin().read_to_end(&mut input)?;
            }
            input
        },
    };
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()?
        .post(format!("{}/command", endpoint.url))
        .bearer_auth(endpoint.token)
        .json(&opts)
        .send()
        .await?;
    if !response.status().is_success() {
        bail!("{}", response.text().await?);
    }
    let result: super::daemon::CommandResult = response.json().await?;
    print!("{}", result.stdout);
    eprint!("{}", result.stderr);
    Ok(Some(result.code))
}

#[cfg(test)]
#[path = "client_prediction_tests.rs"]
mod prediction_tests;
