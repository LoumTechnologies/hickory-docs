use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncReadExt;

pub fn config_path(root: &Path) -> Result<PathBuf, String> {
    use sha2::{Digest, Sha256};
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let key = hex::encode(Sha256::digest(root.as_os_str().as_encoded_bytes()));
    Ok(hickory_workspace::data_root()
        .map_err(|e| e.to_string())?
        .join("checks")
        .join(format!("{key}.json")))
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub executable: PathBuf,
    pub command: Vec<String>,
    #[serde(default)]
    pub workflow: bool,
}
impl Config {
    pub fn load(root: &Path) -> Result<Option<Self>, String> {
        let path = config_path(root)?;
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(format!(
                    "Cannot read {}: {e}; check permissions",
                    path.display()
                ));
            }
        };
        Self::parse(&path, &bytes).map(Some)
    }
    fn parse(path: &Path, bytes: &[u8]) -> Result<Self, String> {
        let config: Self = serde_json::from_slice(bytes).map_err(|e| {
            format!(
                "Invalid {}: {e}; rerun just checks-configure",
                path.display()
            )
        })?;
        if !config.executable.is_absolute()
            || config.command.is_empty()
            || config.command[0].is_empty()
        {
            return Err("Checks need an absolute executable and nonempty command; rerun just checks-configure".into());
        }
        Ok(config)
    }
}
pub async fn invoke(config: &Config, root: &Path, args: &[String]) -> Result<Value, String> {
    let mut child = tokio::process::Command::new(&config.executable)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            format!(
                "Cannot launch {}: {e}; rebuild local-ci or rerun just checks-configure",
                config.executable.display()
            )
        })?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let work = async {
        let read = async |stream: tokio::process::ChildStdout| {
            let mut bytes = Vec::new();
            stream.take(2_097_153).read_to_end(&mut bytes).await?;
            Ok::<_, std::io::Error>(bytes)
        };
        let err = async {
            let mut bytes = Vec::new();
            stderr.take(65_537).read_to_end(&mut bytes).await?;
            Ok::<_, std::io::Error>(bytes)
        };
        let (out, err) = tokio::try_join!(read(stdout), err).map_err(|e| e.to_string())?;
        if out.len() > 2_097_152 || err.len() > 65_536 {
            return Err(
                "local-ci response exceeded the display limit; use local-ci runs directly".into(),
            );
        }
        let status = child.wait().await.map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!(
                "local-ci is unavailable: {}; run local-ci service start and retry",
                String::from_utf8_lossy(&err).trim()
            ));
        }
        if args.first().is_some_and(|a| a == "run") {
            let id = String::from_utf8(out).map_err(|e| e.to_string())?;
            if id.trim().is_empty() || id.trim().contains(char::is_whitespace) {
                return Err(
                    "local-ci returned no single run ID; inspect local-ci runs before retrying"
                        .into(),
                );
            }
            Ok(serde_json::json!({ "id": id.trim() }))
        } else {
            serde_json::from_slice(&out).map_err(|e| {
                format!("Unsupported local-ci response: {e}; check the installed local-ci version")
            })
        }
    };
    if args.first().is_some_and(|a| a == "run") {
        work.await
    } else {
        tokio::time::timeout(std::time::Duration::from_secs(15), work)
            .await
            .map_err(|_| {
                "local-ci did not respond within 15 seconds; check its service and retry"
                    .to_string()
            })?
    }
}
#[derive(Debug, Serialize)]
pub struct Run {
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub status: String,
    pub revision: Option<String>,
    pub created_ms: u64,
    pub code: Option<i64>,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub availability: &'static str,
    pub message: Option<String>,
    pub runs: Vec<Run>,
}
pub fn normalize(value: Value, root: &Path) -> Result<Vec<Run>, String> {
    let rows = value["runs"]
        .as_array()
        .ok_or("local-ci returned no runs array; check its version")?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut runs = Vec::new();
    for row in rows {
        let request = &row["request"];
        let project = request["project"]
            .as_str()
            .or_else(|| request["cwd"].as_str());
        if !project.is_some_and(|p| Path::new(p).canonicalize().is_ok_and(|p| p == root)) {
            continue;
        }
        let id = row["id"]
            .as_str()
            .ok_or("local-ci run has no ID; check its version")?;
        let raw = row["status"].as_str().unwrap_or("unknown");
        let status = match raw {
            "queued"
            | "running"
            | "finishing"
            | "observing"
            | "passed"
            | "failed"
            | "blocked"
            | "cancelled"
            | "timed_out"
            | "infrastructure_failed"
            | "incomplete" => raw,
            _ => "unknown",
        };
        let args = request["args"].as_array();
        let name = request["check"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| {
                let args: Vec<_> = args
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect();
                let start = args.iter().position(|v| *v == "--").map_or(0, |i| i + 1);
                args[start..].join(" ")
            });
        runs.push(Run {
            id: id.into(),
            parent: request["parent"].as_str().map(str::to_owned),
            name,
            status: status.into(),
            revision: request["source"]["revision"].as_str().map(str::to_owned),
            created_ms: row["created_ms"].as_u64().unwrap_or(0),
            code: row["code"].as_i64(),
        });
    }
    runs.sort_by_key(|r| std::cmp::Reverse(r.created_ms));
    Ok(runs)
}
pub async fn snapshot(root: &Path) -> Snapshot {
    let result = async {
        let Some(config) = Config::load(root)? else {
            return Ok(None);
        };
        let value = invoke(&config, root, &["runs".into()]).await?;
        normalize(value, root).map(Some)
    }
    .await;
    match result {
        Ok(Some(runs)) => Snapshot { availability: "ready", message: None, runs },
        Ok(None) => Snapshot { availability: "disabled", message: Some("Configure local-ci with just checks-configure EXECUTABLE [COMMAND…] in the opened folder.".into()), runs: vec![] },
        Err(message) => Snapshot { availability: "unavailable", message: Some(message), runs: vec![] },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    // Contract: .hickory-source/mechanisms/optional-checks.md
    #[test]
    fn isolates_projects_and_preserves_unknown_and_revision() {
        let root = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let rows = json!({"runs":[
            {"id":"one","status":"new-provider-state","request":{"project":root.path(),"source":{"revision":"abc"},"args":["run","--","just","check"]}},
            {"id":"two","status":"passed","request":{"project":other.path()}}
        ]});
        let runs = normalize(rows, root.path()).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "unknown");
        assert_eq!(runs[0].revision.as_deref(), Some("abc"));
        assert_eq!(runs[0].name, "just check");
        assert!(normalize(json!({}), root.path()).is_err());
    }
    #[tokio::test]
    async fn absent_is_disabled_and_repository_cannot_configure_execution() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(snapshot(root.path()).await.availability, "disabled");
        assert!(Config::parse(root.path(), b"{}").is_err());
        // A file supplied by a checkout cannot authorize process execution.
        std::fs::write(root.path().join(".hickory-checks.local.json"), "{}").unwrap();
        assert_eq!(snapshot(root.path()).await.availability, "disabled");
    }
}
