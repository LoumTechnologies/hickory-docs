//! Self-preparing acceptance adapter; the Python runner owns native fixtures.
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Output},
    time::{SystemTime, UNIX_EPOCH},
};
const DEBUGPY: &str = "1.8.22";
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Host {
    Apple,
    Android,
}
impl Host {
    fn parse(value: &OsStr) -> Result<Self> {
        match value.to_str() {
            Some("apple") => Ok(Self::Apple),
            Some("android") => Ok(Self::Android),
            _ => Err("Choose apple or android: just native-debug-smoke <platform>".into()),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Apple => "apple",
            Self::Android => "android",
        }
    }
}
#[derive(Clone)]
struct Machine {
    python: PathBuf,
    variables: Vec<(OsString, OsString)>,
}
impl Default for Machine {
    fn default() -> Self {
        Self {
            python: PathBuf::from("python3"),
            variables: Vec::new(),
        }
    }
}
fn installed_python(root: &Path) -> Result<PathBuf> {
    let valid = |path: &Path| {
        capture(
            path,
            &words(&[
                "-I",
                "-c",
                "import sys,venv; assert sys.version_info >= (3,10)",
            ]),
            root,
            &Machine::default(),
        )
        .is_ok()
    };
    if let Some(path) = env::var_os("HICKORY_NATIVE_DEBUG_PYTHON") {
        let path = PathBuf::from(path);
        if valid(&path) {
            return Ok(path);
        }
        return Err(
            "HICKORY_NATIVE_DEBUG_PYTHON must name an installed Python 3.10+ with venv support"
                .into(),
        );
    }
    let mut candidates = vec![root.join("target/thread-debug-env/bin/python")];
    for name in [
        "python3",
        "python3.14",
        "python3.13",
        "python3.12",
        "python3.11",
        "python3.10",
    ] {
        if let Some(path) = find_tool(name) {
            candidates.push(path)
        }
    }
    for path in candidates {
        if valid(&path) {
            return Ok(path);
        }
    }
    if let Some(uv) = find_tool("uv") {
        if let Ok(out) = capture(
            uv,
            &words(&["python", "find", "--no-python-downloads", ">=3.10"]),
            root,
            &Machine::default(),
        ) {
            let path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
            if valid(&path) {
                return Ok(path);
            }
        }
    }
    Err("Machine prerequisite missing: installed Python 3.10+ with venv support (debugpy1.8.22 requires it). Install Python or set HICKORY_NATIVE_DEBUG_PYTHON; this smoke never installs global interpreters.".into())
}
fn command(
    program: impl AsRef<OsStr>,
    args: &[OsString],
    root: &Path,
    machine: &Machine,
) -> Command {
    let mut c = Command::new(program);
    c.args(args).current_dir(root);
    for (k, v) in &machine.variables {
        c.env(k, v);
    }
    c
}
fn run(
    label: &str,
    program: impl AsRef<OsStr>,
    args: &[OsString],
    root: &Path,
    machine: &Machine,
) -> Result<()> {
    eprintln!("Native debug smoke: {label}");
    let status = command(program, args, root, machine).status()?;
    if !status.success() {
        return Err(format!("{label} failed ({status}); the smoke did not pass.").into());
    }
    Ok(())
}
fn capture(
    program: impl AsRef<OsStr>,
    args: &[OsString],
    root: &Path,
    machine: &Machine,
) -> Result<Output> {
    let out = command(program, args, root, machine).output()?;
    if !out.status.success() {
        return Err(format!(
            "Tool probe failed ({}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(out)
}
fn words(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
fn find_tool(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|value| {
        env::split_paths(&value)
            .map(|dir| dir.join(name))
            .find(|path| path.is_file())
    })
}
fn require_tool(name: &str) -> Result<PathBuf> {
    find_tool(name).ok_or_else(|| {
        format!("Machine prerequisite missing: {name}. Install it and retry native-debug-smoke.")
            .into()
    })
}
fn repository(executable: &Path) -> Result<PathBuf> {
    executable.ancestors().find(|path|path.join("Cargo.lock").is_file()&&path.join("scripts/test-native-debug.py").is_file()).map(Path::to_path_buf).ok_or_else(||"Run the repository's native-debug-smoke recipe; its helper must remain under this checkout's target directory.".into())
}
fn xcode_installations(directory: &Path) -> Vec<PathBuf> {
    let mut result = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("Xcode"))
                && path.extension() == Some(OsStr::new("app"))
        })
        .map(|path| path.join("Contents/Developer"))
        .filter(|path| {
            path.join("Platforms/MacOSX.platform/Developer/SDKs")
                .is_dir()
        })
        .collect::<Vec<_>>();
    result.sort();
    result.reverse();
    result
}
fn full_xcode(root: &Path) -> Result<PathBuf> {
    if let Some(value) = env::var_os("DEVELOPER_DIR") {
        let path = PathBuf::from(value);
        if path
            .join("Platforms/MacOSX.platform/Developer/SDKs")
            .is_dir()
        {
            return Ok(path);
        }
        return Err("DEVELOPER_DIR must name a full Xcode Contents/Developer directory with the macOS SDK; no global Xcode selection was changed.".into());
    }
    let selected = capture("xcode-select", &words(&["-p"]), root, &Machine::default())
        .ok()
        .map(|out| PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()));
    let mut candidates = selected.into_iter().collect::<Vec<_>>();
    candidates.push(PathBuf::from("/Applications/Xcode.app/Contents/Developer"));
    if let Some(home) = env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join("Applications/Xcode.app/Contents/Developer"));
    }
    candidates.extend(xcode_installations(Path::new("/Applications")));
    if let Some(home) = env::var_os("HOME") {
        candidates.extend(xcode_installations(
            &PathBuf::from(home).join("Applications"),
        ));
    }
    candidates.into_iter().find(|path|path.join("Platforms/MacOSX.platform/Developer/SDKs").is_dir()).ok_or_else(||"Apple smoke requires full Xcode and its macOS SDK. Install Xcode, then select it for this invocation with DEVELOPER_DIR=/path/to/Xcode.app/Contents/Developer. This command never runs xcode-select --switch.".into())
}
// AGP's major/minor release pin may install as android-37.0. A major-only
// spelling is equivalent only for minor zero; never accept another minor.
fn android_platform_available(sdk:&Path,major:&str,minor:&str)->bool {
    let exact=sdk.join(format!("platforms/android-{major}.{minor}/android.jar"));
    exact.is_file() || minor=="0"&&sdk.join(format!("platforms/android-{major}/android.jar")).is_file()
}
fn android_sdk(root: &Path) -> Result<PathBuf> {
    let mut candidates = ["ANDROID_SDK_ROOT", "ANDROID_HOME"]
        .into_iter()
        .filter_map(env::var_os)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.extend([home.join("Library/Android/sdk"), home.join("Android/Sdk")]);
    }
    let build = fs::read_to_string(root.join("apps/android-simple/app/build.gradle.kts"))?;
    let version = build
        .split("compileSdk")
        .nth(1)
        .and_then(|value| value.split("release(").nth(1))
        .map(|value| {
            value
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .filter(|value| !value.is_empty())
        .ok_or("Cannot read the project's compileSdk pin")?;
    let minor=build.split("minorApiLevel").nth(1).and_then(|value|value.split('=').nth(1)).map(str::trim_start).map(|value|value.chars().take_while(char::is_ascii_digit).collect::<String>()).filter(|value|!value.is_empty()).ok_or("Cannot read the project's compileSdk minor pin")?;
    candidates.into_iter().find(|sdk|android_platform_available(sdk,&version,&minor)&&sdk.join("build-tools").is_dir()).ok_or_else(||format!("Android smoke requires installed Android SDK platform {version}.{minor} and build-tools. Install them with Android Studio/SDK Manager, and set ANDROID_SDK_ROOT. SDK packages/licenses are never installed or accepted by this smoke.").into())
}
fn java_home() -> Result<PathBuf> {
    if let Some(value) = env::var_os("JAVA_HOME") {
        let path = PathBuf::from(value);
        if path.join("bin/java").is_file() && path.join("bin/javac").is_file() {
            return Ok(path);
        }
        return Err("JAVA_HOME must name a JDK (including bin/java and bin/javac).".into());
    }
    let mut studios = env::var_os("ANDROID_STUDIO_HOME")
        .map(PathBuf::from)
        .into_iter()
        .collect::<Vec<_>>();
    studios.push(PathBuf::from("/Applications/Android Studio.app"));
    if let Some(home) = env::var_os("HOME") {
        studios.push(PathBuf::from(home).join("Applications/Android Studio.app"));
    }
    for studio in studios {
        for path in [
            studio.join("Contents/jbr/Contents/Home"),
            studio.join("jbr"),
        ] {
            if path.join("bin/javac").is_file() {
                return Ok(path);
            }
        }
    }
    let javac = require_tool("javac")?.canonicalize()?;
    javac
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "Set JAVA_HOME to an installed JDK 17 or later.".into())
}
fn supported_host(host: Host, os: &str) -> Result<()> {
    if !matches!(os, "macos" | "linux") {
        return Err("Native debugger smoke supports macOS (apple/android) and Linux (android); this host is unsupported.".into());
    }
    if host == Host::Apple && os != "macos" {
        return Err("The apple debugger smoke requires macOS and full Xcode; use android on a configured Linux host.".into());
    }
    Ok(())
}
fn machine(root: &Path, host: Host) -> Result<Machine> {
    supported_host(host, env::consts::OS)?;
    for tool in ["rustc", "cargo", "cc"] {
        let executable = require_tool(tool)?;
        capture(
            executable,
            &words(&["--version"]),
            root,
            &Machine::default(),
        )?;
    }
    let mut result = Machine {
        python: installed_python(root)?,
        ..Machine::default()
    };
    if host == Host::Apple {
        result
            .variables
            .push(("DEVELOPER_DIR".into(), full_xcode(root)?.into_os_string()));
        require_tool("swift")?;
        capture(
            "xcrun",
            &words(&["--sdk", "macosx", "--show-sdk-path"]),
            root,
            &result,
        )?;
    } else {
        let sdk = android_sdk(root)?;
        let java = java_home()?;
        result.variables.extend([
            ("ANDROID_SDK_ROOT".into(), sdk.clone().into_os_string()),
            ("ANDROID_HOME".into(), sdk.into_os_string()),
            ("JAVA_HOME".into(), java.clone().into_os_string()),
        ]);
        let output = capture(java.join("bin/javac"), &words(&["-version"]), root, &result)?;
        let version = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let major = version
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .split('.')
            .next()
            .and_then(|part| part.parse::<u32>().ok())
            .unwrap_or(0);
        if major < 17 {
            return Err(
                format!("Android smoke requires JDK 17 or later; reported {version}").into(),
            );
        }
    }
    Ok(result)
}
fn python_version(python: &Path, root: &Path, machine: &Machine) -> Result<String> {
    let out = capture(
        python,
        &words(&[
            "-I",
            "-c",
            "import importlib.metadata as m\ntry: print(m.version('debugpy'))\nexcept m.PackageNotFoundError: print('')",
        ]),
        root,
        machine,
    )?;
    Ok(String::from_utf8(out.stdout)?.trim().into())
}
fn verify_python(python: &Path, root: &Path, machine: &Machine) -> Result<()> {
    let script =
        format!("import debugpy; assert debugpy.__version__ == {DEBUGPY:?}, debugpy.__version__");
    capture(
        python,
        &["-I".into(), "-c".into(), script.into()],
        root,
        machine,
    )?;
    Ok(())
}
fn prepare_python(root: &Path, machine: &Machine) -> Result<PathBuf> {
    let owned = root.join(format!("target/native-debug-smoke/debugpy-{DEBUGPY}"));
    let legacy = root.join("target/thread-debug-env");
    for candidate in [&owned, &legacy] {
        let python = candidate.join("bin/python");
        if python.is_file()
            && python_version(&python, root, machine).ok().as_deref() == Some(DEBUGPY)
            && verify_python(&python, root, machine).is_ok()
        {
            eprintln!(
                "Native debug smoke: reuse pinned debugpy {DEBUGPY} at {}",
                candidate.display()
            );
            return Ok(candidate.to_path_buf());
        }
    }
    if owned.exists() && !owned.join("pyvenv.cfg").is_file() {
        return Err(format!("Preserved unrecognized preparation directory {}. Move it aside explicitly before retrying.",owned.display()).into());
    }
    if !owned.join("bin/python").is_file() {
        fs::create_dir_all(owned.parent().unwrap())?;
        run(
            "prepare target-local Python environment",
            &machine.python,
            &["-m".into(), "venv".into(), owned.as_os_str().into()],
            root,
            machine,
        )?;
    }
    let python = owned.join("bin/python");
    let existing = python_version(&python, root, machine)?;
    if !existing.is_empty() {
        return Err(format!("Preserved existing debugpy {existing} at {}; required pin is {DEBUGPY}. Repair/move this target-local environment explicitly; no dependencies were removed.",owned.display()).into());
    }
    run(
        "install pinned debugpy into target-local environment",
        &python,
        &[
            "-I".into(),
            "-m".into(),
            "pip".into(),
            "--isolated".into(),
            "install".into(),
            "--require-virtualenv".into(),
            "--disable-pip-version-check".into(),
            "--no-deps".into(),
            format!("debugpy=={DEBUGPY}").into(),
        ],
        root,
        machine,
    )?;
    verify_python(&python, root, machine)?;
    Ok(owned)
}
#[derive(Debug)]
struct GradlePin {
    version: String,
    url: String,
    sha: String,
}
fn gradle_pin(root: &Path) -> Result<GradlePin> {
    let text = fs::read_to_string(
        root.join("apps/android-simple/gradle/wrapper/gradle-wrapper.properties"),
    )?;
    let get = |key: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(key))
            .map(str::trim)
            .map(str::to_string)
    };
    let url = get("distributionUrl=")
        .ok_or("Missing pinned Gradle URL")?
        .replace("\\:", ":");
    let sha = get("distributionSha256Sum=").ok_or("Missing pinned Gradle checksum")?;
    if sha.len() != 64 || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Invalid Gradle SHA-256 pin".into());
    }
    if !url.starts_with("https://") && !url.starts_with("file://") {
        return Err(
            "Pinned Gradle URL must use https:// (or a local file:// verification fixture)".into(),
        );
    }
    let version = url
        .rsplit("gradle-")
        .next()
        .and_then(|value| value.strip_suffix("-bin.zip"))
        .ok_or("Expected pinned gradle-VERSION-bin.zip URL")?
        .to_string();
    if version.is_empty()
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".-".contains(&byte))
    {
        return Err("Invalid pinned Gradle version spelling".into());
    }
    Ok(GradlePin { version, url, sha })
}
fn gradle_matches(home: &Path, pin: &GradlePin, root: &Path, machine: &Machine) -> Result<bool> {
    let executable = home.join("bin/gradle");
    if !executable.is_file() {
        return Ok(false);
    }
    let out = capture(executable, &words(&["--version"]), root, machine)?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|line| line.trim() == format!("Gradle {}", pin.version)))
}
fn checksum(path: &Path, sha: &str, root: &Path, machine: &Machine) -> Result<()> {
    capture(&machine.python,&["-I".into(),"-c".into(),"import hashlib,pathlib,sys; actual=hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest(); assert actual==sys.argv[2].lower(), 'Gradle archive checksum mismatch: '+actual".into(),path.as_os_str().into(),sha.into()],root,machine)?;
    Ok(())
}
fn prepare_gradle(root: &Path, machine: &Machine) -> Result<PathBuf> {
    let pin = gradle_pin(root)?;
    let storage = root.join("target/native-debug-smoke");
    let owned = storage.join(format!("gradle-{}", pin.version));
    let mut candidates = vec![owned.clone()];
    if let Some(home) = env::var_os("GRADLE_HOME") {
        candidates.push(home.into())
    }
    if let Some(path) = find_tool("gradle").and_then(|path| path.canonicalize().ok()) {
        if let Some(home) = path.parent().and_then(Path::parent) {
            candidates.push(home.into())
        }
    }
    let cache = env::var_os("GRADLE_USER_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".gradle")));
    if let Some(cache) = cache {
        for kind in ["bin", "all"] {
            let folder = cache
                .join("wrapper/dists")
                .join(format!("gradle-{}-{kind}", pin.version));
            if let Ok(entries) = fs::read_dir(folder) {
                for entry in entries.flatten() {
                    candidates.push(entry.path().join(format!("gradle-{}", pin.version)));
                }
            }
        }
    }
    for home in candidates {
        if gradle_matches(&home, &pin, root, machine)? {
            eprintln!(
                "Native debug smoke: reuse pinned Gradle {} at {}",
                pin.version,
                home.display()
            );
            return Ok(home);
        }
    }
    if owned.exists() {
        return Err(format!(
            "Preserved invalid Gradle preparation at {}; move it aside explicitly before retrying.",
            owned.display()
        )
        .into());
    }
    fs::create_dir_all(&storage)?;
    let archive = storage.join(format!("gradle-{}-bin.zip", pin.version));
    if !archive.exists() {
        require_tool("curl")?;
        let temporary = storage.join(format!(
            "gradle-{}.download-{}",
            pin.version,
            std::process::id()
        ));
        run(
            "download checksum-pinned Gradle distribution",
            "curl",
            &[
                "--fail".into(),
                "--location".into(),
                "--proto".into(),
                "=https,file".into(),
                "--output".into(),
                temporary.as_os_str().into(),
                pin.url.clone().into(),
            ],
            root,
            machine,
        )?;
        checksum(&temporary, &pin.sha, root, machine)?;
        fs::rename(&temporary, &archive)?;
    } else {
        checksum(&archive, &pin.sha, root, machine)?;
    }
    let stage = storage.join(format!(
        "gradle-stage-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&stage)?;
    let extract = "import pathlib,sys,zipfile\narchive,destination,prefix=sys.argv[1:]\nwith zipfile.ZipFile(archive) as z:\n for member in z.infolist():\n  parts=pathlib.PurePosixPath(member.filename).parts\n  assert parts and parts[0]==prefix and not member.filename.startswith('/') and '..' not in parts, 'Unsafe Gradle archive path'\n z.extractall(destination)";
    run(
        "extract verified Gradle distribution",
        &machine.python,
        &[
            "-I".into(),
            "-c".into(),
            extract.into(),
            archive.as_os_str().into(),
            stage.as_os_str().into(),
            format!("gradle-{}", pin.version).into(),
        ],
        root,
        machine,
    )?;
    let extracted = stage.join(format!("gradle-{}", pin.version));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            extracted.join("bin/gradle"),
            fs::Permissions::from_mode(0o755),
        )?;
    }
    if !gradle_matches(&extracted, &pin, root, machine)? {
        return Err("Verified Gradle archive did not contain its pinned working launcher".into());
    }
    fs::rename(extracted, &owned)?;
    fs::remove_dir(&stage)?;
    Ok(owned)
}
fn prepare(root: &Path, host: Host, machine: &mut Machine) -> Result<(PathBuf, PathBuf)> {
    fs::create_dir_all(root.join("target/native-debug-smoke"))?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("target/native-debug-smoke/preparation.lock"))?;
    eprintln!("Native debug smoke: waiting for this checkout's preparation lock");
    lock.lock()?;
    if host == Host::Android {
        let gradle = prepare_gradle(root, machine)?;
        machine
            .variables
            .push(("GRADLE_HOME".into(), gradle.into_os_string()));
    }
    let venv = prepare_python(root, machine)?;
    let target = root.join("target/source-edit-focus");
    run(
        "prepare locked engine build",
        "cargo",
        &[
            "build".into(),
            "--locked".into(),
            "--package".into(),
            "hickory-cli".into(),
            "--example".into(),
            "engine_client".into(),
            "--target-dir".into(),
            target.as_os_str().into(),
        ],
        root,
        machine,
    )?;
    let engine = target.join("debug/examples/engine_client");
    if !engine.is_file() {
        return Err("Engine build succeeded without producing engine_client".into());
    }
    drop(lock);
    Ok((engine, venv))
}
fn smoke(root: &Path, host: Host, machine: &mut Machine) -> Result<()> {
    fs::create_dir_all(root.join("target/native-debug-smoke"))?;
    let lease = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join(format!(
            "target/native-debug-smoke/{}.run.lock",
            host.name()
        )))?;
    eprintln!(
        "Native debug smoke: waiting for this checkout's {} smoke lock",
        host.name()
    );
    lease.lock()?;
    let (engine, venv) = prepare(root, host, machine)?;
    run(
        "execute exactly four real native debugger flows",
        &machine.python,
        &[
            root.join("scripts/test-native-debug.py").into_os_string(),
            host.name().into(),
            "--engine".into(),
            engine.into_os_string(),
            "--venv".into(),
            venv.into_os_string(),
        ],
        root,
        machine,
    )
}
fn entry() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        return Err("Usage: just native-debug-smoke <apple|android>".into());
    }
    let host = Host::parse(&args[0])?;
    let root = repository(&env::current_exe()?.canonicalize()?)?;
    let mut machine = machine(&root, host)?;
    smoke(&root, host, &mut machine)
}
fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Native debug smoke: {error}");
            ExitCode::FAILURE
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    static FIXTURE_SEQUENCE:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = env::temp_dir().join(format!(
                "native smoke fixture # {}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                FIXTURE_SEQUENCE.fetch_add(1,std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn fixture_pin(root: &Path, sha: &str, url: &str) {
        let folder = root.join("apps/android-simple/gradle/wrapper");
        fs::create_dir_all(&folder).unwrap();
        fs::write(
            folder.join("gradle-wrapper.properties"),
            format!("distributionUrl={url}\ndistributionSha256Sum={sha}\n"),
        )
        .unwrap();
    }
    #[test]
    fn scoped_platform_and_root_resolution_do_not_interpret_shell_text() {
        assert_eq!(Host::parse(OsStr::new("apple")).unwrap(), Host::Apple);
        assert_eq!(Host::parse(OsStr::new("android")).unwrap(), Host::Android);
        assert!(Host::parse(OsStr::new("android; touch unexpected")).is_err());
        assert!(Host::parse(OsStr::new("browser")).is_err());
        assert!(supported_host(Host::Apple, "macos").is_ok());
        assert!(supported_host(Host::Android, "linux").is_ok());
        assert!(supported_host(Host::Apple, "linux").is_err());
        assert!(supported_host(Host::Android, "windows").is_err());
        assert!(supported_host(Host::Android, "freebsd").is_err());
        let fixture = Fixture::new();
        fs::write(fixture.0.join("Cargo.lock"), "").unwrap();
        fs::create_dir(fixture.0.join("scripts")).unwrap();
        fs::write(fixture.0.join("scripts/test-native-debug.py"), "").unwrap();
        let nested = fixture.0.join("target/nested/helper");
        assert_eq!(repository(&nested).unwrap(), fixture.0);
    }
    #[test]
    fn versioned_xcode_discovery_ignores_manager_and_incomplete_apps() {
        let fixture = Fixture::new();
        let expected = fixture.0.join("Xcode-27.0.0.app/Contents/Developer");
        fs::create_dir_all(expected.join("Platforms/MacOSX.platform/Developer/SDKs")).unwrap();
        fs::create_dir(fixture.0.join("Xcodes.app")).unwrap();
        fs::create_dir(fixture.0.join("Xcode-incomplete.app")).unwrap();
        assert_eq!(xcode_installations(&fixture.0), vec![expected]);
    }
    #[test]
    fn sdk_platform_matches_the_declared_major_and_minor() {
        let fixture=Fixture::new();let sdk=&fixture.0;
        fs::create_dir_all(sdk.join("platforms/android-37.0")).unwrap();
        fs::write(sdk.join("platforms/android-37.0/android.jar"),"fixture").unwrap();
        assert!(android_platform_available(sdk,"37","0"));
        assert!(!android_platform_available(sdk,"37","1"));
        assert!(!android_platform_available(sdk,"36","0"));
        fs::remove_file(sdk.join("platforms/android-37.0/android.jar")).unwrap();
        fs::create_dir_all(sdk.join("platforms/android-37")).unwrap();
        fs::write(sdk.join("platforms/android-37/android.jar"),"fixture").unwrap();
        assert!(android_platform_available(sdk,"37","0"));
        assert!(!android_platform_available(sdk,"37","1"));
    }
    #[test]
    fn wrapper_requires_a_pinned_archive_and_safe_version() {
        let fixture = Fixture::new();
        fixture_pin(
            &fixture.0,
            &"a".repeat(64),
            "https\\://services.gradle.org/distributions/gradle-9.3.1-bin.zip",
        );
        let pin = gradle_pin(&fixture.0).unwrap();
        assert_eq!(pin.version, "9.3.1");
        assert!(pin.url.starts_with("https://"));
        fixture_pin(
            &fixture.0,
            "not-a-sha",
            "https://example.invalid/gradle-9.3.1-bin.zip",
        );
        assert!(gradle_pin(&fixture.0).is_err());
        fixture_pin(
            &fixture.0,
            &"a".repeat(64),
            "http://example.invalid/gradle-9.3.1-bin.zip",
        );
        assert!(gradle_pin(&fixture.0).is_err());
        fixture_pin(
            &fixture.0,
            &"a".repeat(64),
            "https://example.invalid/gradle-../../bad-bin.zip",
        );
        assert!(gradle_pin(&fixture.0).is_err());
    }
    #[test]
    fn invalid_cached_archive_is_preserved_and_rejected() {
        let fixture = Fixture::new();
        let archive = fixture.0.join("archive with spaces.zip");
        fs::write(&archive, b"invalid cached bytes").unwrap();
        assert!(checksum(&archive, &"0".repeat(64), &fixture.0, &Machine::default()).is_err());
        assert_eq!(fs::read(&archive).unwrap(), b"invalid cached bytes");
    }
    #[cfg(unix)]
    #[test]
    fn existing_gradle_probe_failure_is_not_replaced_by_a_download() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Fixture::new();
        fixture_pin(
            &fixture.0,
            &"a".repeat(64),
            "https://example.invalid/gradle-9.3.1-bin.zip",
        );
        let launcher = fixture
            .0
            .join("target/native-debug-smoke/gradle-9.3.1/bin/gradle");
        fs::create_dir_all(launcher.parent().unwrap()).unwrap();
        let text = "#!/bin/sh\necho deliberate-sdk-probe-failure >&2\nexit 17\n";
        fs::write(&launcher, text).unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
        let error = prepare_gradle(&fixture.0, &Machine::default()).unwrap_err();
        assert!(error.to_string().contains("deliberate-sdk-probe-failure"));
        assert_eq!(fs::read_to_string(&launcher).unwrap(), text);
        assert!(
            !fixture
                .0
                .join("target/native-debug-smoke/gradle-9.3.1-bin.zip")
                .exists()
        );
    }
    #[test]
    #[ignore = "prepares a private actual Cargo example, pinned debugpy venv and local checksum-pinned Gradle fixture"]
    fn cold_preparation_then_reuse_and_failure_preserve_existing_dependencies() {
        let fixture = Fixture::new();
        let root = &fixture.0;
        let machine = Machine {
            python: installed_python(root).unwrap(),
            ..Machine::default()
        };
        fs::create_dir(root.join("scripts")).unwrap();
        fs::write(root.join("Cargo.toml"),"[package]\nname='hickory-cli'\nversion='0.0.0'\nedition='2024'\n[[example]]\nname='engine_client'\npath='engine_client.rs'\n[workspace]\n").unwrap();
        fs::write(root.join("engine_client.rs"), "fn main() {}\n").unwrap();
        run(
            "create isolated fixture lockfile",
            "cargo",
            &words(&["generate-lockfile", "--offline"]),
            root,
            &machine,
        )
        .unwrap();
        let lock = fs::read(root.join("Cargo.lock")).unwrap();
        let fake_runner = "import argparse,pathlib\np=argparse.ArgumentParser();p.add_argument('host');p.add_argument('--engine',type=pathlib.Path);p.add_argument('--venv',type=pathlib.Path);a=p.parse_args();assert a.host=='android' and a.engine.is_file() and (a.venv/'bin/python').is_file()\nwith pathlib.Path('smoke-calls').open('a') as output: output.write(a.host+'\\n')\n";
        fs::write(root.join("scripts/test-native-debug.py"), fake_runner).unwrap();
        let archive = root.join("gradle-0.0.0-fixture-bin.zip");
        let out=capture(&machine.python,&["-I".into(),"-c".into(),"import hashlib,pathlib,sys,zipfile\np=pathlib.Path(sys.argv[1])\nwith zipfile.ZipFile(p,'w') as z:z.writestr('gradle-0.0.0-fixture/bin/gradle', '#!/bin/sh\\nprintf \\\'Gradle 0.0.0-fixture\\\\n\\\'\\n')\nprint(p.as_uri());print(hashlib.sha256(p.read_bytes()).hexdigest())".into(),archive.as_os_str().into()],root,&machine).unwrap();
        let text = String::from_utf8(out.stdout).unwrap();
        let rows = text.lines().collect::<Vec<_>>();
        fixture_pin(root, rows[1], rows[0]);
        let mut first = machine.clone();
        smoke(root, Host::Android, &mut first).unwrap();
        let engine = root.join("target/source-edit-focus/debug/examples/engine_client");
        let venv = root.join(format!("target/native-debug-smoke/debugpy-{DEBUGPY}"));
        let gradle = root.join("target/native-debug-smoke/gradle-0.0.0-fixture/bin/gradle");
        assert!(engine.is_file());
        assert_eq!(
            python_version(&venv.join("bin/python"), root, &machine).unwrap(),
            DEBUGPY
        );
        let engine_time = fs::metadata(&engine).unwrap().modified().unwrap();
        let python_time = fs::metadata(venv.join("pyvenv.cfg"))
            .unwrap()
            .modified()
            .unwrap();
        let gradle_time = fs::metadata(&gradle).unwrap().modified().unwrap();
        let mut again = machine.clone();
        smoke(root, Host::Android, &mut again).unwrap();
        assert_eq!(
            fs::metadata(&engine).unwrap().modified().unwrap(),
            engine_time
        );
        assert_eq!(
            fs::metadata(venv.join("pyvenv.cfg"))
                .unwrap()
                .modified()
                .unwrap(),
            python_time
        );
        assert_eq!(
            fs::metadata(&gradle).unwrap().modified().unwrap(),
            gradle_time
        );
        assert_eq!(
            fs::read_to_string(root.join("smoke-calls")).unwrap(),
            "android\nandroid\n"
        );
        assert_eq!(fs::read(root.join("Cargo.lock")).unwrap(), lock);
        fs::write(
            root.join("engine_client.rs"),
            "compile_error!(\"deliberate preparation refusal\"); fn main(){}\n",
        )
        .unwrap();
        let mut failed = machine.clone();
        assert!(smoke(root, Host::Android, &mut failed).is_err());
        assert_eq!(
            fs::read_to_string(root.join("smoke-calls")).unwrap(),
            "android\nandroid\n"
        );
        assert!(venv.join("bin/python").exists());
        assert!(gradle.exists());
        assert_eq!(
            fs::metadata(venv.join("pyvenv.cfg"))
                .unwrap()
                .modified()
                .unwrap(),
            python_time
        );
    }
}
