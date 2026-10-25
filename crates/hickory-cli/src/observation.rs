//! Explicit user execution through the existing cell sandbox, never an agent capability.
use anyhow::{Context, Result, ensure};
use hick_observe::{
    Clock, Event, Handling, LocalStore, Location, Observation, Precision, Run, Source, Status,
};
use hickory_executor_sandbox::policy::{Confinement, Profile, Sandbox, wrap};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(clap::Args)]
pub struct ObserveArgs {
    /// Python file or literate document. This executes its application code confined.
    pub document: PathBuf,
    /// Generated Python entry point for a literate document.
    #[arg(long)]
    pub program: Option<String>,
    /// Maximum execution duration (1–120 seconds).
    #[arg(long, default_value_t = 10)]
    pub seconds: u64,
    /// Capture stdlib cProfile function aggregates instead of trace hooks.
    #[arg(long)]
    pub profile: bool,
}
pub async fn command(args: ObserveArgs) -> Result<std::process::ExitCode> {
    let document = args.document.canonicalize()?;
    let run = capture_mode(
        &document,
        args.program.as_deref(),
        args.seconds,
        args.profile,
    )
    .await?;
    let store = LocalStore::open(document.parent().context("No document directory")?)?;
    store.save(&run)?;
    println!(
        "{} · {:?} · {} events · {}",
        run.id,
        run.status,
        run.events.len(),
        run.detail.as_deref().unwrap_or("")
    );
    println!(
        "Retained locally; open Runtime runs in the native editor. Descriptor/native writes and catching sites are unsupported."
    );
    Ok(if run.status == Status::Completed {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    })
}
fn location(value: &Value, mapping: &hick_dap::Mapping, source: &Source) -> Option<Location> {
    let line = mapping.to_document(
        Path::new(value.get("file")?.as_str()?),
        value.get("line")?.as_u64()? as u32,
    )? as usize;
    let start = source
        .bytes
        .split_inclusive(|b| *b == b'\n')
        .take(line)
        .map(|line| line.len())
        .sum::<usize>();
    let end = start
        + source
            .bytes
            .get(start..)?
            .iter()
            .position(|b| *b == b'\n')
            .unwrap_or(source.bytes.len() - start);
    Some(Location {
        source: source.id.clone(),
        block: None,
        bytes: Some(start..end),
        precision: Precision::AdapterReported,
    })
}
pub async fn capture(document: &Path, program: Option<&str>, seconds: u64) -> Result<Run> {
    capture_mode(document, program, seconds, false).await
}
pub async fn capture_mode(
    document: &Path,
    program: Option<&str>,
    seconds: u64,
    profile: bool,
) -> Result<Run> {
    capture_variant(
        document,
        program,
        seconds,
        if profile { "--profile" } else { "" },
    )
    .await
}
async fn capture_variant(
    document: &Path,
    program: Option<&str>,
    seconds: u64,
    mode: &str,
) -> Result<Run> {
    let profile = mode == "--profile";
    ensure!(
        (1..=120).contains(&seconds),
        "Duration must be 1–120 seconds"
    );
    ensure!(
        Sandbox::detect() != Sandbox::None,
        "No supported cell sandbox; refusing unconfined Python"
    );
    let source = std::fs::read_to_string(document)?;
    ensure!(source.len() <= 1024 * 1024, "Source exceeds 1 MiB");
    let scratch = tempfile::tempdir()?;
    let root = scratch.path().canonicalize()?;
    let (entry, mapping) = if document.extension().and_then(|s| s.to_str()) == Some("py") {
        std::fs::write(root.join("application.py"), &source)?;
        (
            root.join("application.py"),
            hick_dap::Mapping::identity(&root.join("application.py"))
                .with_observation_source(&source),
        )
    } else {
        let state = hick_lsp::document::HickDocumentState::from_source(&source)
            .map_err(anyhow::Error::msg)?;
        for file in &state.virtual_files {
            let path = Path::new(&file.path);
            ensure!(
                !path.is_absolute()
                    && path.components().all(|part| matches!(
                        part,
                        std::path::Component::Normal(_) | std::path::Component::CurDir
                    )),
                "Generated path escapes scratch"
            );
            ensure!(
                path != Path::new("observer.py") && path != Path::new("observations.jsonl"),
                "Reserved provider artifact path"
            );
        }
        let files = hick_dap::program::weave_into(&source, &root)?;
        let entry = match program {
            Some(name) => root.join(name),
            None => hick_dap::program::entry_point(&files)?,
        };
        ensure!(
            entry.canonicalize()?.starts_with(&root),
            "Entry point escapes scratch"
        );
        ensure!(
            entry.extension().and_then(|s| s.to_str()) == Some("py"),
            "Only Python entry points supported"
        );
        (
            entry,
            hick_dap::Mapping::for_document(document, &source, &root)?,
        )
    };
    let captured = Source::capture(
        "document".into(),
        document.to_string_lossy().into_owned(),
        source.into_bytes(),
    );
    let mut run = Run::new(
        vec![captured.clone()],
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
    );
    let kind = if profile {
        hick_plugins::InterfaceKind::Profiling
    } else {
        hick_plugins::InterfaceKind::RuntimeObservation
    };
    let (provider, script) = match hick_plugins::registry().interface("python", kind) {
        Some(
            hick_plugins::Interface::Profiling { provider, script }
            | hick_plugins::Interface::RuntimeObservation { provider, script },
        ) => (*provider, *script),
        _ => anyhow::bail!("No Python observation provider in this build"),
    };
    std::fs::write(root.join("observer.py"), script)?;
    // Shell quoting is applied once; document names never become shell syntax.
    let relative = entry
        .strip_prefix(&root)?
        .to_str()
        .context("Entry point must be UTF-8")?;
    let command = format!(
        "exec python3 -I -B observer.py '{}'",
        relative.replace('\'', "'\\''")
    );
    ensure!(
        ["", "--profile", "--control"].contains(&mode),
        "Unknown observation mode"
    );
    let command = format!("{command} {mode}");
    let (bin, argv) = wrap(
        Sandbox::detect(),
        &Confinement {
            workdir: &root,
            command: &command,
            allow_network: false,
            profile: Profile::Cell,
            tmpdir: None,
            peers: &[],
            tools: &[],
        },
    )
    .context("Confinement unavailable")?;
    let mut diagnostics = tempfile::tempfile()?;
    let mut child = tokio::process::Command::new(bin);
    child
        .args(argv)
        .current_dir(&root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/local/bin:/opt/homebrew/bin")
        .env("HOME", &root)
        .stdout(Stdio::null())
        .stderr(diagnostics.try_clone()?)
        .kill_on_drop(true);
    #[cfg(unix)]
    child.process_group(0);
    let mut child = child.spawn()?;
    #[cfg(unix)]
    let pid = child.id();
    let outcome = tokio::select! {
        result = child.wait() => Some(result?),
        _ = tokio::time::sleep(Duration::from_secs(seconds)) => None,
        _ = tokio::signal::ctrl_c() => None,
    };
    #[cfg(unix)]
    if outcome.is_some() {
        if let Some(pid) = pid {
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
    }
    if outcome.is_none() {
        #[cfg(unix)]
        if let Some(pid) = pid {
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
        let _ = child.kill().await;
        run.status = Status::Cancelled;
    } else {
        run.status = Status::Partial;
    }
    let artifact = root.join("observations.jsonl");
    if artifact.exists() {
        ensure!(
            std::fs::symlink_metadata(&artifact)?.file_type().is_file(),
            "Observation artifact must be a regular file"
        );
        ensure!(
            std::fs::metadata(&artifact)?.len() <= 16 * 1024 * 1024,
            "Observation artifact exceeds 16 MiB"
        );
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        let file = options.open(&artifact)?;
        let mut text = String::new();
        use std::io::Read;
        file.take(16 * 1024 * 1024 + 1).read_to_string(&mut text)?;
        ensure!(
            text.len() <= 16 * 1024 * 1024,
            "Observation artifact exceeds 16 MiB"
        );
        for line in text.lines() {
            let Ok(value) = serde_json::from_str::<Value>(line) else {
                run.truncated = true;
                continue;
            };
            if let Some(terminal) = value.get("terminal").and_then(Value::as_str) {
                if outcome.is_some() {
                    run.status = if terminal == "completed" {
                        Status::Completed
                    } else {
                        Status::Failed
                    };
                }
                run.truncated |= value["truncated"].as_bool().unwrap_or(false);
                run.detail = Some(value.to_string());
                continue;
            }
            if run.events.len() >= 4096 {
                run.truncated = true;
                break;
            }
            let emission = location(&value["location"], &mapping, &captured);
            let payload = &value["payload"];
            let observation = match payload["kind"].as_str() {
                Some("output") => Observation::Output {
                    stream: payload["stream"].as_str().unwrap_or("unknown").into(),
                    category: payload["category"].as_str().unwrap_or("write").into(),
                    bytes: serde_json::from_value(payload["bytes"].clone())?,
                },
                Some("exception") => Observation::Exception {
                    type_name: payload["type_name"].as_str().unwrap_or("unknown").into(),
                    message: payload["message"].as_str().unwrap_or("").into(),
                    handling: if payload["handling"] == "unhandled" {
                        Handling::Unhandled
                    } else {
                        Handling::Throwing
                    },
                    throwing: emission.clone(),
                    catching: None,
                },
                Some("profile") => serde_json::from_value(payload.clone())?,
                _ => continue,
            };
            run.events.push(Event {
                id: format!("{}-E{}", run.id, run.events.len() + 1),
                provider: provider.into(),
                block: None,
                invocation: value["invocation"].as_str().map(str::to_owned),
                parent_invocation: value["parent_invocation"].as_str().map(str::to_owned),
                thread: value["thread"].as_str().map(str::to_owned),
                task: value["task"].as_str().map(str::to_owned),
                time: value["time"].as_f64().map(|value| Clock {
                    domain: "provider-monotonic".into(),
                    unit: "seconds".into(),
                    value,
                }),
                emission,
                callers: value["callers"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| location(v, &mapping, &captured))
                    .collect(),
                observation,
            });
        }
    } else {
        run.missing_artifacts
            .push("Python observation stream unavailable".into());
    }
    use std::io::{Read as _, Seek};
    diagnostics.rewind()?;
    let mut stderr = String::new();
    diagnostics.take(16384).read_to_string(&mut stderr)?;
    if !stderr.is_empty() {
        let mut detail: Value = serde_json::from_str(run.detail.as_deref().unwrap_or("{}"))
            .unwrap_or(serde_json::json!({}));
        detail["unattributed_process_stderr"] = Value::String(stderr);
        run.detail = Some(detail.to_string());
    }
    if outcome.is_some_and(|status| !status.success()) && run.status == Status::Completed {
        run.status = Status::Failed;
        run.missing_artifacts
            .push("Provider process failed after terminal artifact".into());
    }
    run.validate()?;
    Ok(run)
}

#[cfg(test)]
mod tests {
    // Guarantee: docs/guarantees/execution/python-observations-stay-confined.md
    use super::*;
    #[tokio::test]
    async fn real_confined_hooks_preserve_unicode_invocations_and_bypass_limits() {
        let dir = tempfile::tempdir().unwrap();
        let private = tempfile::tempdir_in(std::env::var_os("HOME").unwrap()).unwrap();
        let secret = private.path().join("private");
        std::fs::write(&secret, "secret").unwrap();
        let code = format!(
            "import sys, os, threading\ndef repeat(n):\n print('é🦀', end='')\n sys.stdout.buffer.write(b'bytes')\n sys.stdout.flush()\n if n: repeat(n-1)\nrepeat(2)\nt=threading.Thread(target=lambda: print('thread'))\nt.start(); t.join()\nos.write(1,b'unsupported-descriptor')\ntry:\n open({:?}).read()\nexcept PermissionError:\n print('confined')\ntry:\n raise ValueError('caught')\nexcept ValueError:\n pass\n",
            secret.to_str().unwrap()
        );
        let path = dir.path().join("a.py");
        std::fs::write(&path, code).unwrap();
        let run = capture(&path, None, 10).await.unwrap();
        assert_eq!(run.status, Status::Completed, "{:?}", run.detail);
        let writes: Vec<_> = run
            .events
            .iter()
            .filter_map(|e| match &e.observation {
                Observation::Output { bytes, .. } => Some(bytes.as_slice()),
                _ => None,
            })
            .flatten()
            .copied()
            .collect();
        let output = String::from_utf8(writes).unwrap();
        assert!(output.contains("é🦀bytesé🦀bytesé🦀bytes"));
        assert!(output.contains("confined"));
        assert!(!output.contains("unsupported-descriptor"));
        let calls: std::collections::BTreeSet<_> = run.events.iter().filter(|e| matches!(&e.observation, Observation::Output { category, .. } if category == "print")).filter_map(|e| e.invocation.as_ref()).collect();
        assert!(calls.len() >= 3);
        assert!(run.events.iter().all(|e| e.thread.is_some()));
        assert!(run.events.iter().any(|e| matches!(&e.observation, Observation::Exception { handling: Handling::Throwing, message, .. } if message == "caught")));
        assert!(run.events.iter().filter(|e| matches!(&e.observation, Observation::Output { category, .. } if category == "print")).all(|e| e.emission.is_some()));
        assert!(run.detail.unwrap().contains("descriptor/native writes"));
    }
    #[tokio::test]
    async fn real_literate_failure_and_cancellation_keep_captured_mapping() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        std::fs::write(&path, "# Example\n\n<hick:file path=\"main.py\">\nprint('ok')\nraise RuntimeError('boom')\n</hick:file>\n").unwrap();
        let run = capture(&path, Some("main.py"), 10).await.unwrap();
        assert_eq!(run.status, Status::Failed, "{:?}", run.detail);
        let throwing = run
            .events
            .iter()
            .find(|e| {
                matches!(
                    &e.observation,
                    Observation::Exception {
                        handling: Handling::Unhandled,
                        ..
                    }
                )
            })
            .unwrap();
        let range = throwing.emission.as_ref().unwrap().bytes.clone().unwrap();
        assert!(
            std::str::from_utf8(&run.sources[0].bytes[range])
                .unwrap()
                .contains("raise RuntimeError")
        );
        let path = dir.path().join("loop.py");
        std::fs::write(&path, "print('before', flush=True)\nwhile True: pass\n").unwrap();
        let cancelled = capture(&path, None, 1).await.unwrap();
        assert_eq!(cancelled.status, Status::Cancelled);
        assert!(!cancelled.events.is_empty());
    }
    #[tokio::test]
    async fn hook_overhead_is_measured_without_a_timing_threshold() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("overhead.py");
        std::fs::write(&path, "import sys, time, json\ndef work():\n return sum(range(20))\nhook=sys.gettrace()\nresults=[]\nfor repeat in range(3):\n pair=[]\n for enabled in (False, True):\n  sys.settrace(hook if enabled else None)\n  start=time.perf_counter()\n  value=sum(work() for _ in range(2000))\n  pair.append(time.perf_counter()-start)\n  assert value == 380000\n results.append(pair)\nsys.settrace(hook)\nprint(json.dumps(results))\n").unwrap();
        let run = capture(&path, None, 10).await.unwrap();
        assert_eq!(run.status, Status::Completed);
        let output: Vec<u8> = run
            .events
            .iter()
            .filter_map(|event| match &event.observation {
                Observation::Output { bytes, .. } => Some(bytes.as_slice()),
                _ => None,
            })
            .flatten()
            .copied()
            .collect();
        let pairs: Vec<Vec<f64>> = serde_json::from_slice(&output).unwrap();
        assert_eq!(pairs.len(), 3);
        eprintln!(
            "Observed same-process sandbox hook overhead pairs [untraced,traced] seconds: {:?}; provider {}",
            pairs,
            run.detail.unwrap()
        );
    }
    // Guarantee: docs/guarantees/execution/python-profile-keeps-function-semantics.md
    #[tokio::test]
    async fn real_profile_optimize_rerun_preserves_correctness_and_aggregate_scope() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hotspot.md");
        let before = include_str!("../../../examples/python-profile/hotspot.md");
        let after = before.replace(
            "    total = 0\n    for value in range(n):\n        total += value\n    return total",
            "    return n * (n - 1) // 2",
        );
        let mut timings = vec![];
        for (name, source) in [("before", before.to_owned()), ("after", after)] {
            std::fs::write(&path, source).unwrap();
            let mut pairs = vec![];
            for repetition in 0..3 {
                let control = capture_variant(&path, Some("hotspot.py"), 10, "--control")
                    .await
                    .unwrap();
                let run = capture_mode(&path, Some("hotspot.py"), 10, true)
                    .await
                    .unwrap();
                assert_eq!(control.status, Status::Completed);
                assert_eq!(run.status, Status::Completed);
                let details: Value = serde_json::from_str(run.detail.as_ref().unwrap()).unwrap();
                assert!(details["profiling"].as_bool().unwrap());
                let profiles: Vec<_> = run
                    .events
                    .iter()
                    .filter_map(|event| match &event.observation {
                        Observation::Profile {
                            function: Some(function),
                            metric,
                            value,
                            ..
                        } => Some((event, function, metric, value)),
                        _ => None,
                    })
                    .collect();
                let subtotal = profiles
                    .iter()
                    .find(|(_, f, metric, _)| {
                        f.name == "subtotal" && metric.as_str() == "self_seconds"
                    })
                    .unwrap();
                assert_eq!(subtotal.1.total_calls, 500);
                assert_eq!(subtotal.1.primitive_calls, 500);
                assert!(subtotal.0.emission.is_some());
                let recursive = profiles
                    .iter()
                    .find(|(_, f, _, _)| f.name == "recursive")
                    .unwrap();
                assert_eq!(recursive.1.total_calls, 10);
                assert_eq!(recursive.1.primitive_calls, 1);
                assert!(profiles.iter().any(|(e, _, _, _)| e.emission.is_none()));
                let base: Value = serde_json::from_str(control.detail.as_ref().unwrap()).unwrap();
                pairs.push(serde_json::json!({"control_seconds":base["elapsed"], "profile_seconds":details["elapsed"], "subtotal_self_seconds":subtotal.3, "environment":details["environment"], "python":details["python"]}));
                if repetition == 0 {
                    if let Some(output) = std::env::var_os("HICKORY_PROFILE_ARTIFACTS") {
                        let output = PathBuf::from(output);
                        std::fs::create_dir_all(&output).unwrap();
                        std::fs::write(
                            output.join(format!("{name}.json")),
                            serde_json::to_vec_pretty(&run).unwrap(),
                        )
                        .unwrap();
                    }
                }
            }
            timings.push(serde_json::json!({"variant":name,"pairs":pairs}));
        }
        eprintln!(
            "Observed profile/control measurements: {}",
            serde_json::to_string_pretty(&timings).unwrap()
        );
    }
}
