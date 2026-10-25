<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/coverage/commands.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `2d4ffc2f0421b7f423bde57bde95d830e470fbcaec2e6b245d07fe1f7582e400`.

<book:entry path="crates/hickory-cli/src/coverage/commands.rs">
use super::*;
#[derive(Debug)]
pub(super) struct Plan {
    pub argv: Vec<String>,
    pub format: String,
    pub report: PathBuf,
    pub result: PathBuf,
    pub finish: Vec<String>,
    pub version: Vec<String>,
}
fn strings(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}
fn js_runner(project: &Path) -> Result<&'static str> {
    let v: Value = serde_json::from_slice(&std::fs::read(project.join("package.json"))?)?;
    for name in ["vitest", "jest"] {
        if v["dependencies"].get(name).is_some() || v["devDependencies"].get(name).is_some() {
            return Ok(name);
        }
    }
    bail!("Install and configure Vitest or Jest in this project")
}
fn python(project: &Path) -> String {
    hick_project_env::python_interpreter(project, project)
        .map(|p| p.to_string_lossy().into_owned())
        .or_else(|| {
            let path = project.join(if cfg!(windows) {
                ".venv/Scripts/python.exe"
            } else {
                ".venv/bin/python"
            });
            path.is_file().then(|| path.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "python3".into())
}
pub(super) fn discovery(project: &Path, language: &str) -> Result<Vec<String>> {
    Ok(match language {
        "python" => vec![
            python(project),
            "-m".into(),
            "pytest".into(),
            "--collect-only".into(),
            "-q".into(),
        ],
        "rust" => strings(&["cargo", "test", "--", "--list"]),
        "csharp" => strings(&["dotnet", "test", "--list-tests"]),
        "typescript" | "javascript" | "tsx" | "jsx" | "typescriptreact" | "javascriptreact" => {
            let runner = js_runner(project)?;
            if runner == "vitest" {
                strings(&["npx", "--no-install", "vitest", "list", "--json"])
            } else {
                strings(&[
                    "npx",
                    "--no-install",
                    "jest",
                    "--runInBand",
                    "--testNamePattern=a^",
                    "--json",
                    "--passWithNoTests",
                ])
            }
        }
        _ => bail!("Coverage supports Python, Rust, C# and TypeScript/JavaScript"),
    })
}
pub(super) fn plan(
    root: &Path,
    project: &Path,
    request: &Request,
    record: &mut Record,
) -> Result<Plan> {
    if let Some(name) = &request.profile {
        ensure!(
            request.action == "manual-start",
            "Application profiles require manual-start"
        );
        let original = record
            .metadata
            .manual
            .get(name)
            .context("Unknown manual profile")?;
        let profile = manual_profile(root, project, &record.id, original)?;
        ensure!(!profile.argv.is_empty(), "Manual profile has no command");
        ensure!(
            ["lcov", "coverage-py", "cobertura", "istanbul"].contains(&profile.format.as_str()),
            "Unsupported manual report format"
        );
        let report = root.join(&profile.report);
        ensure!(
            !Path::new(&profile.report).is_absolute(),
            "Manual report must be workspace-relative"
        );
        let mut ancestor = report.parent().context("Report parent required")?;
        while !ancestor.exists() {
            ancestor = ancestor.parent().context("Invalid report directory")?;
        }
        ensure!(
            ancestor.canonicalize()?.starts_with(root),
            "Report directory escapes workspace through symlink"
        );
        ensure!(
            report.starts_with(root)
                && !Path::new(&profile.report)
                    .components()
                    .any(|p| matches!(p, std::path::Component::ParentDir)),
            "Report must be in workspace"
        );
        std::fs::create_dir_all(report.parent().unwrap())?;
        // Refuse previous evidence: every session gets fresh collector output.
        ensure!(
            !report.exists(),
            "Manual report already exists. Configure a fresh report path or move the old report before collecting"
        );
        record.identity.collector = profile.collector.clone();
        record.identity.version = profile.version.clone();
        record.identity.configuration = serde_json::to_string(original)?;
        record.argv = profile.argv.clone();
        Ok(Plan {
            argv: profile.argv.clone(),
            format: profile.format.clone(),
            report,
            result: PathBuf::new(),
            finish: profile.finish.clone(),
            version: vec![],
        })
    } else {
        let selectors = record
            .group
            .as_ref()
            .and_then(|g| record.metadata.groups.get(g))
            .map(|g| g.selectors.clone())
            .unwrap_or_default();
        automated(root, project, record, &selectors)
    }
}
pub(super) fn automated(
    root: &Path,
    project: &Path,
    record: &mut Record,
    selectors: &[String],
) -> Result<Plan> {
    ensure!(
        selectors
            .iter()
            .all(|s| !s.starts_with('-') && !s.is_empty()),
        "Selectors cannot be runner options"
    );
    let artifacts = store::dir(root)?.join(&record.id);
    std::fs::create_dir(&artifacts)?;
    let report = artifacts.join("coverage");
    let result = artifacts.join("results");
    let report_arg = report.to_string_lossy().into_owned();
    let result_arg = result.to_string_lossy().into_owned();
    let (mut argv, format, collector, version) = match record.language.as_str() {
        "python" => (
            vec![
                python(project),
                "-m".into(),
                "pytest".into(),
                "--cov=./".into(),
                "--cov-branch".into(),
                format!("--cov-report=json:{report_arg}"),
                format!("--junitxml={result_arg}"),
            ],
            "coverage-py",
            "pytest-cov",
            vec![
                python(project),
                "-m".into(),
                "coverage".into(),
                "--version".into(),
            ],
        ),
        "rust" => (
            strings(&["cargo", "llvm-cov", "--lcov", "--output-path", &report_arg]),
            "lcov",
            "cargo-llvm-cov",
            strings(&["cargo", "llvm-cov", "--version"]),
        ),
        "csharp" => (
            strings(&[
                "dotnet",
                "test",
                "--collect:XPlat Code Coverage",
                "--results-directory",
                &artifacts.to_string_lossy(),
                "--logger",
                &format!("trx;LogFileName={result_arg}"),
            ]),
            "cobertura",
            "coverlet",
            strings(&["dotnet", "list", "package", "--include-transitive"]),
        ),
        "typescript" | "javascript" | "tsx" | "jsx" | "typescriptreact" | "javascriptreact" => {
            let runner = js_runner(project)?;
            let mut argv = strings(&["npx", "--no-install", runner]);
            if runner == "vitest" {
                argv.push("run".into());
            }
            argv.extend(strings(&[
                "--coverage",
                "--coverage.reporter=lcov",
                "--coverage.reportsDirectory",
                &artifacts.to_string_lossy(),
                "--reporter=json",
                "--outputFile",
                &result_arg,
            ]));
            if runner == "jest" {
                argv = strings(&[
                    "npx",
                    "--no-install",
                    "jest",
                    "--coverage",
                    "--coverageReporters=lcov",
                    "--coverageDirectory",
                    &artifacts.to_string_lossy(),
                    "--json",
                    "--outputFile",
                    &result_arg,
                ]);
            }
            (
                argv,
                "lcov",
                runner,
                strings(&["npx", "--no-install", runner, "--version"]),
            )
        }
        _ => bail!("Unsupported coverage language"),
    };
    match record.language.as_str() {
        "python" => argv.extend(selectors.iter().cloned()),
        "rust" => {
            if !selectors.is_empty() {
                argv.push("--".into());
                argv.extend(selectors.iter().cloned());
                if record.group.is_none() {
                    argv.push("--exact".into());
                }
            }
        }
        "csharp" => {
            if !selectors.is_empty() {
                argv.extend([
                    "--filter".into(),
                    selectors
                        .iter()
                        .map(|s| format!("FullyQualifiedName={s}"))
                        .collect::<Vec<_>>()
                        .join("|"),
                ]);
            }
        }
        _ => {
            if !selectors.is_empty() {
                argv.extend([
                    "-t".into(),
                    selectors
                        .iter()
                        .map(|s| format!("^(?:{})$", regex::escape(s)))
                        .collect::<Vec<_>>()
                        .join("|"),
                ]);
            }
        }
    }
    record.argv = argv.clone();
    record.identity.collector = collector.into();
    // Exclude selectors and output paths: these change the contributing group, not instrumentation.
    record.identity.configuration = format!("{collector}:default-project-config:branch-requested");
    let report = match collector {
        "vitest" | "jest" => artifacts.join("lcov.info"),
        _ => report,
    };
    Ok(Plan {
        argv,
        format: format.into(),
        report,
        result,
        finish: vec![],
        version,
    })
}

pub(super) fn manual_profile(
    root: &Path,
    project: &Path,
    id: &str,
    original: &ManualProfile,
) -> Result<ManualProfile> {
    let mut profile = original.clone();
    let base = |text: &str| {
        text.replace("{root}", &root.to_string_lossy())
            .replace("{project}", &project.to_string_lossy())
            .replace("{run}", id)
    };
    profile.report = base(&profile.report);
    let report = root.join(&profile.report).to_string_lossy().into_owned();
    for argv in [
        &mut profile.argv,
        &mut profile.finish,
        &mut profile.checkpoint,
    ] {
        for arg in argv {
            *arg = base(arg).replace("{report}", &report);
        }
    }
    Ok(profile)
}
</book:entry>

</book:doc>
