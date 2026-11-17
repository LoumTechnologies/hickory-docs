<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/coverage/commands.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-024ef726fd84"></a>
## `crates/hickory-cli/src/coverage/commands.rs`

<a id="f-024ef726fd84-Plan"></a>
### `Plan`

Failing test bodies still produce native reports: Vitest explicitly reports on failure, and cargo-llvm-cov finishes its report despite failed bodies. Its successful export exit code does not turn failed structured test results into a successful test run.

Automated report paths live in a fresh private temporary directory outside the source tree. A renamed source root therefore cannot redirect report exports. The original source directory still supplies process working text; this does not claim frozen inputs or sandbox project-owned commands.

<storycode:copy id="f-024ef726fd84-Plan">
use super::*;
/// Private output is independent of the workspace path. Cleanup refuses a
/// replacement directory, so finishing a run never deletes foreign artifacts.
#[derive(Debug)]
pub(super) struct PrivateArtifacts { directory:Option<tempfile::TempDir>, identity:same_file::Handle, children:Vec<(PathBuf,same_file::Handle)> }
impl PrivateArtifacts {
 pub(super) fn new()->Result<Self>{let directory=tempfile::Builder::new().prefix("hickory-collector-").tempdir()?;let identity=same_file::Handle::from_path(directory.path())?;Ok(Self{directory:Some(directory),identity,children:vec![]})}
 pub fn path(&self)->&Path{self.directory.as_ref().unwrap().path()}
 pub fn child(&mut self,name:&str)->Result<PathBuf>{ensure!(Path::new(name).components().count()==1&&matches!(Path::new(name).components().next(),Some(std::path::Component::Normal(_))),"Invalid private collector directory");let path=self.path().join(name);std::fs::create_dir(&path)?;self.children.push((path.clone(),same_file::Handle::from_path(&path)?));Ok(path)}
 pub fn check(&self)->Result<()>{for(path,expected)in std::iter::once((self.path(),&self.identity)).chain(self.children.iter().map(|(path,identity)|(path.as_path(),identity))){let metadata=std::fs::symlink_metadata(path)?;ensure!(metadata.is_dir()&&!metadata.file_type().is_symlink()&&same_file::Handle::from_path(path)?==*expected,"Private collector artifact directory changed; report authority is unavailable");}Ok(())}
 pub fn read(&self,path:&Path,limit:u64)->Result<String>{
  use std::io::Read;self.check()?;let canonical=self.path().canonicalize()?;let relative=path.strip_prefix(self.path()).or_else(|_|path.strip_prefix(&canonical))?;let parts=relative.components().map(|part|match part{std::path::Component::Normal(value)=>Ok(value),_=>bail!("Unsafe private collector report path")}).collect::<Result<Vec<_>>>()?;ensure!(!parts.is_empty(),"Collector report file required");
  #[cfg(unix)] let file={use std::{ffi::CString,os::{fd::{AsRawFd,FromRawFd},unix::ffi::OsStrExt}};let mut parent=self.identity.as_file().try_clone()?;let mut walked=PathBuf::new();for(index,part)in parts.iter().enumerate(){let name=CString::new(part.as_bytes())?;let leaf=index+1==parts.len();let flags=libc::O_RDONLY|libc::O_NOFOLLOW|libc::O_CLOEXEC|if leaf{0}else{libc::O_DIRECTORY};let descriptor=unsafe{libc::openat(parent.as_raw_fd(),name.as_ptr(),flags)};ensure!(descriptor>=0,"Cannot read original private collector artifact: {}",std::io::Error::last_os_error());let opened=unsafe{std::fs::File::from_raw_fd(descriptor)};walked.push(part);if !leaf{if let Some((_,expected))=self.children.iter().find(|(path,_)|path==&self.path().join(&walked)){ensure!(same_file::Handle::from_file(opened.try_clone()?)?==*expected,"Private collector report parent was substituted");}}parent=opened;}parent};
  #[cfg(not(unix))] let file={bail!("Held-directory collector report import is unavailable on this host");std::fs::File::open(path)?};
  ensure!(file.metadata()?.is_file()&&file.metadata()?.len()<=limit,"Collector artifact exceeds its bounded regular-file limit");let mut bytes=Vec::new();file.take(limit+1).read_to_end(&mut bytes)?;ensure!(bytes.len()as u64<=limit,"Collector artifact grew beyond its read limit");self.check()?;Ok(String::from_utf8(bytes)?)
 }
}
impl Drop for PrivateArtifacts {fn drop(&mut self){if self.check().is_err(){if let Some(directory)=self.directory.take(){let _=directory.keep();}}}}
#[derive(Debug)]
pub(super) struct Plan {
    pub argv: Vec<String>,
    pub exact_results: bool,
    pub artifacts: Option<PrivateArtifacts>,
    pub format: String,
    pub report: PathBuf,
    pub result: PathBuf,
    pub finish: Vec<String>,
    pub version: Vec<String>,
}
</storycode:copy>
<a id="f-024ef726fd84-strings"></a>
### `strings`

<storycode:copy id="f-024ef726fd84-strings">
fn strings(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}
pub(super) fn csharp_filter(selectors:&[String])->String{selectors.iter().map(|name|format!("FullyQualifiedName={name}")).collect::<Vec<_>>().join("|")}
pub(super) fn native_vitest_discovery(project:&Path)->Result<Vec<String>>{
 let project=project.canonicalize()?;
 #[cfg(unix)]let(dev,ino)={use std::os::unix::fs::MetadataExt;let metadata=std::fs::metadata(&project)?;(metadata.dev().to_string(),metadata.ino().to_string())};
 #[cfg(not(unix))]let(dev,ino):(String,String)={bail!("Native discovery physical project binding is unavailable on this host")};
 let spec=json!({"mode":"discover","provider":"vitest","project":project,"project_dev":dev,"project_ino":ino});Ok(vec!["node".into(),"--input-type=module".into(),"--eval".into(),hick_plugins::testing::JAVASCRIPT_TEST_DRIVER.into(),spec.to_string()])
}
</storycode:copy>
<a id="f-024ef726fd84-js_runner"></a>
### `js_runner`

<storycode:copy id="f-024ef726fd84-js_runner">
pub(super) fn js_runner(project: &Path) -> Result<&'static str> {
    let v: Value = serde_json::from_slice(&std::fs::read(project.join("package.json"))?)?;
    let providers=["vitest","jest"].into_iter().filter(|name|v["dependencies"].get(*name).is_some()||v["devDependencies"].get(*name).is_some()).collect::<Vec<_>>();
    ensure!(providers.len()==1,"Declare one unambiguous Vitest or Jest package runner; no collector was started");
    Ok(providers[0])
}
</storycode:copy>
<a id="f-024ef726fd84-python"></a>
### `python`

<storycode:copy id="f-024ef726fd84-python">
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
</storycode:copy>
<a id="f-024ef726fd84-discovery"></a>
### `discovery`

<storycode:copy id="f-024ef726fd84-discovery">
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
                native_vitest_discovery(project)?
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
</storycode:copy>
<a id="f-024ef726fd84-plan"></a>
### `plan`

<storycode:copy id="f-024ef726fd84-plan">
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
            exact_results:false,
            argv: profile.argv.clone(),
            artifacts: None,
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
</storycode:copy>
<a id="f-024ef726fd84-automated"></a>
### `automated`

<storycode:copy id="f-024ef726fd84-automated">
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
    let mut private = PrivateArtifacts::new()?;
    let artifacts = private.path().to_path_buf();
    // Providers may clean their report directory; the held private parent is never theirs to replace.
    let reports=private.child("reports")?;
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
                "-o".into(),format!("cache_dir={}",artifacts.join("pytest-cache").display()),
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
            strings(&["cargo", "llvm-cov", "--ignore-run-fail", "--lcov", "--output-path", &report_arg]),
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
                "--coverage.clean=false",
                "--coverage.reportOnFailure=true",
                "--coverage.reporter=lcov",
                "--coverage.reportsDirectory",
                &reports.to_string_lossy(),
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
                    &reports.to_string_lossy(),
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
                    csharp_filter(selectors),
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
        "vitest" | "jest" => reports.join("lcov.info"),
        _ => report,
    };
    Ok(Plan {
        exact_results:false,
        argv,
        artifacts: Some(private),
        format: format.into(),
        report,
        result,
        finish: vec![],
        version,
    })
}

</storycode:copy>
<a id="f-024ef726fd84-manual_profile"></a>
### `manual_profile`

<storycode:copy id="f-024ef726fd84-manual_profile">
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/coverage/commands.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/coverage/commands.rs"><storycode:paste select="#f-024ef726fd84-Plan"/><storycode:paste select="#f-024ef726fd84-strings"/><storycode:paste select="#f-024ef726fd84-js_runner"/><storycode:paste select="#f-024ef726fd84-python"/><storycode:paste select="#f-024ef726fd84-discovery"/><storycode:paste select="#f-024ef726fd84-plan"/><storycode:paste select="#f-024ef726fd84-automated"/><storycode:paste select="#f-024ef726fd84-manual_profile"/></storycode:file>

</storycode:doc>
