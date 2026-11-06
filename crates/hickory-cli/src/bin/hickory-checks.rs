use hickory_cli::checks::{Config, config_path, invoke, snapshot};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("checks: {error}");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let root = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let path = config_path(&root)?;
    match args.next().as_deref() {
        Some("configure") => {
            let executable = args
                .next()
                .ok_or("Supply the local-ci executable path; use --off to disable")?;
            if executable == "--off" {
                match std::fs::remove_file(&path) {
                    Ok(()) => (),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                    Err(e) => {
                        return Err(format!(
                            "Cannot remove {}: {e}; check its permissions",
                            path.display()
                        ));
                    }
                }
                return Ok(());
            }
            let executable = std::path::PathBuf::from(executable)
                .canonicalize()
                .map_err(|e| format!("Cannot find local-ci: {e}; build ../local-ci first"))?;
            let mut command: Vec<_> = args.collect();
            if command.is_empty() {
                command = vec!["just".into(), "check".into()];
            }
            let config = Config {
                executable,
                command,
                workflow: false,
            };
            if !config.executable.is_file() || config.command[0].is_empty() {
                return Err("Choose a local-ci executable file and a nonempty command; rerun just checks-configure".into());
            }
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
                format!(
                    "Cannot create checks configuration directory {}: {e}; check permissions",
                    path.parent().unwrap().display()
                )
            })?;
            std::fs::write(&path, serde_json::to_vec_pretty(&config).unwrap())
                .map_err(|e| format!("Cannot write {}: {e}; check permissions", path.display()))?;
            println!(
                "Configured {}; use just checks-hook local-ci to enable commit runs",
                path.display()
            );
        }
        Some("status") => {
            let result = snapshot(&root).await;
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            if result.availability == "unavailable" {
                return Err("Provider unavailable; see the diagnostic above".into());
            }
        }
        Some("run") => {
            let config = Config::load(&root)?
                .ok_or("Run just checks-configure EXECUTABLE [COMMAND…] first")?;
            let mut command = vec!["run".into(), "--detach".into()];
            if config.workflow {
                command.push("--workflow".into());
            }
            if let Some(revision) = args.next() {
                if ![40, 64].contains(&revision.len())
                    || !revision.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(
                        "Commit runs need a full Git object ID; use git rev-parse HEAD".into(),
                    );
                }
                command.extend([
                    "--source".into(),
                    "git".into(),
                    "--repo".into(),
                    root.to_string_lossy().into_owned(),
                    "--revision".into(),
                    revision,
                ]);
            } else {
                command.extend(["--workspace".into(), root.to_string_lossy().into_owned()]);
            }
            command.push("--".into());
            command.extend(config.command);
            // Submission can prepare a snapshot; it does not wait for checks.
            let value = invoke(
                &Config {
                    command: vec![],
                    ..config
                },
                &root,
                &command,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&value).unwrap());
        }
        _ => return Err("Use configure EXECUTABLE [COMMAND…], status, or run [COMMIT]".into()),
    }
    Ok(())
}
