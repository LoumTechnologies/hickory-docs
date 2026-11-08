use anyhow::{Context, Result, bail, ensure};
use hickory_cli::{ExecutorChoice, engine};
use serde_json::Value;
use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    net::TcpListener,
    process::{Child, Command},
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Some(result) = engine::run_argv(&args) {
        return result;
    }
    let platform = args.first().map(String::as_str).unwrap_or("browser");
    ensure!(
        args.len() <= 1 && ["browser", "apple", "ios", "android", "all"].contains(&platform),
        "Usage: just simple-dev browser|apple|ios|android|all"
    );
    if ["apple", "ios", "all"].contains(&platform) {
        configure_xcode()?;
    }
    tokio::runtime::Runtime::new()?.block_on(async {
        let mut session = Session::default();
        let result = tokio::select! {
            result = launch(platform, &mut session) => result,
            result = interrupted() => result,
        };
        session.cleanup().await;
        result
    })
}

fn configure_xcode() -> Result<()> {
    ensure!(cfg!(target_os = "macos"), "Apple frontends require macOS");
    if env::var_os("DEVELOPER_DIR").is_some() {
        return Ok(());
    }
    let selected = std::process::Command::new("xcode-select")
        .arg("-p")
        .output()?;
    let selected = PathBuf::from(String::from_utf8_lossy(&selected.stdout).trim());
    if selected.join("usr/bin/xcodebuild").is_file() {
        return Ok(());
    }
    let mut apps: Vec<PathBuf> = std::fs::read_dir("/Applications")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("Xcode"))
        })
        .map(|path| path.join("Contents/Developer"))
        .filter(|path| path.join("usr/bin/xcodebuild").is_file())
        .collect();
    apps.sort();
    let developer = apps
        .pop()
        .context("Install full Xcode and select it with DEVELOPER_DIR")?;
    println!("Xcode: {}", developer.display());
    // This runs on the initial thread, before constructing the async runtime.
    unsafe {
        env::set_var("DEVELOPER_DIR", developer);
    }
    Ok(())
}

async fn interrupted() -> Result<()> {
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! { result = tokio::signal::ctrl_c() => result?, _ = term.recv() => {} }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c().await?;
    Ok(())
}

#[derive(Default)]
struct Session {
    children: Vec<Child>,
    undo: Vec<Command>,
    guard: Option<engine::ClientGuard>,
}
impl Session {
    fn spawn(&mut self, command: &mut Command) -> Result<usize> {
        command.kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let child = command
            .spawn()
            .with_context(|| format!("Starting {command:?}"))?;
        self.children.push(child);
        Ok(self.children.len() - 1)
    }
    async fn run(&mut self, command: &mut Command) -> Result<()> {
        let index = self.spawn(command)?;
        let status = self.children[index].wait().await?;
        ensure!(status.success(), "Command failed ({status}): {command:?}");
        Ok(())
    }
    async fn output(&mut self, command: &mut Command) -> Result<String> {
        use tokio::io::AsyncReadExt;
        command.stdout(Stdio::piped());
        let index = self.spawn(command)?;
        let mut stdout = self.children[index].stdout.take().unwrap();
        let mut bytes = Vec::new();
        let (read, status) =
            tokio::join!(stdout.read_to_end(&mut bytes), self.children[index].wait());
        read?;
        ensure!(status?.success(), "Command failed: {command:?}");
        Ok(String::from_utf8(bytes)?)
    }
    async fn cleanup(&mut self) {
        for command in self.undo.iter_mut().rev() {
            match tokio::time::timeout(Duration::from_secs(10), command.kill_on_drop(true).status())
                .await
            {
                Ok(Ok(status)) if status.success() => {}
                result => eprintln!("Cleanup {command:?}: {result:?}"),
            }
        }
        // Child launchers have their own service groups. Give their signal
        // handlers time to clean those up before escalating to a hard kill.
        #[cfg(unix)]
        for child in self.children.iter_mut().rev() {
            if let Some(pid) = child.id() {
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGTERM);
                }
            }
        }
        for child in self.children.iter_mut().rev() {
            if tokio::time::timeout(Duration::from_secs(5), child.wait())
                .await
                .is_err()
            {
                stop_group(child);
                let _ = child.wait().await;
            }
        }
        if let Some(guard) = self.guard.take() {
            match tokio::time::timeout(Duration::from_secs(10), guard.detach()).await {
                Ok(Ok(())) => {}
                result => eprintln!("Backend detach: {result:?}"),
            }
        }
    }
}
fn stop_group(child: &mut Child) {
    // A completed child has no live PID; never signal a recycled process ID.
    if let Some(pid) = child.id() {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/T", "/F", "/PID", &pid.to_string()])
                .status();
        }
        let _ = child.start_kill();
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        for child in self.children.iter_mut().rev() {
            stop_group(child);
        }
    }
}
fn cmd(program: impl AsRef<std::ffi::OsStr>, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    command
}

async fn launch(platform: &str, session: &mut Session) -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    env::set_current_dir(&root)?;
    let scratch = root.join(".dev/simple");
    let project = if let Some(path) = env::var_os("HICKORY_PROJECT_DIR") {
        PathBuf::from(path)
            .canonicalize()
            .context("HICKORY_PROJECT_DIR must be an existing folder")?
    } else {
        session
            .run(
                cmd("bash", &["scripts/dev-seed.sh"])
                    .env("HICKORY_SEED_ROOT", &scratch)
                    .env("HICKORY_SEED_WEAVE", "0"),
            )
            .await?;
        scratch.join("project")
    };
    ensure!(
        project.is_dir(),
        "Workspace must be a directory: {}",
        project.display()
    );
    if platform == "all" {
        println!(
            "Launching browser, macOS, iOS simulator and Android against {}",
            project.display()
        );
        session
            .run(&mut cmd(
                "swiftc",
                &[
                    "scripts/simple-window-layout.swift",
                    "-o",
                    "target/simple-window-layout",
                ],
            ))
            .await?;
        let first_platform = session.children.len();
        let mut platforms = Vec::new();
        for platform in ["browser", "apple", "ios", "android"] {
            session.spawn(
                cmd(env::current_exe()?, &[platform]).env("HICKORY_PROJECT_DIR", &project),
            )?;
            platforms.push(platform);
        }
        session.spawn(&mut cmd(
            root.join("target/simple-window-layout"),
            &[&std::process::id().to_string()],
        ))?;
        loop {
            for (child, platform) in session
                .children
                .iter_mut()
                .skip(first_platform)
                .take(4)
                .zip(&platforms)
            {
                if let Some(status) = child.try_wait()? {
                    bail!(
                        "{platform} comparison session stopped ({status}); stopping the other sessions. Fix its reported error and rerun just simple-dev all."
                    );
                }
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
    // Reserve an OS-assigned browser port before configuring the backend origin.
    // Vite's strictPort fails safely if another process wins the handoff race.
    let browser_port = if platform == "browser" {
        Some(TcpListener::bind("127.0.0.1:0").await?.local_addr()?.port())
    } else {
        None
    };
    let ui_origin = browser_port.map(|port| format!("http://127.0.0.1:{port}"));
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let api_origin = format!("http://{}", listener.local_addr()?);
    let mut attach = engine::Attach::new(project.clone(), ExecutorChoice::Local);
    attach.callback = Some(api_origin.clone());
    attach.callback_token = engine::window_token();
    attach.key_store_path = hickory_agent::KeyStore::desktop_path();
    let token = attach.callback_token.clone();
    let connection = engine::connect(attach).await?;
    session.guard = Some(connection.guard());
    let router = engine::client_router(connection, None, token, ui_origin.clone());
    let server =
        tokio::spawn(async move { axum::serve(listener, router.into_make_service()).await });
    println!(
        "Workspace: {}\nBackend: {api_origin}\nStop this launch with Ctrl-C.",
        project.display()
    );
    tokio::select! {
        result = frontend(platform, &root, session, &api_origin, browser_port) => result,
        result = server => { result??; bail!("Workspace server stopped unexpectedly") },
    }
}

async fn frontend(
    platform: &str,
    root: &Path,
    session: &mut Session,
    api: &str,
    port: Option<u16>,
) -> Result<()> {
    session
        .run(&mut cmd("python3", &["scripts/prepare-simple-resources.py"]))
        .await?;
    match platform {
        "browser" => {
            let web = root.join("apps/web");
            let installed = web.join("node_modules");
            if !installed.exists()
                || std::fs::metadata(web.join("package-lock.json"))?.modified()?
                    > std::fs::metadata(&installed)?.modified()?
            {
                session
                    .run(
                        cmd(if cfg!(windows) { "npm.cmd" } else { "npm" }, &["ci"])
                            .current_dir(&web),
                    )
                    .await?;
                std::fs::File::create(installed.join(".simple-dev-installed"))?;
            }
            let port = port.unwrap().to_string();
            println!("Browser: http://127.0.0.1:{port}/simple.html");
            session
                .run(
                    cmd(
                        "node",
                        &[
                            "node_modules/vite/bin/vite.js",
                            "--config",
                            "vite.simple.config.ts",
                            "--host",
                            "127.0.0.1",
                            "--port",
                            &port,
                            "--strictPort",
                            "--open",
                            "/simple.html",
                        ],
                    )
                    .current_dir(web)
                    .env("HICKORY_API_ORIGIN", api),
                )
                .await
        }
        "apple" => apple(root, session, api).await,
        "android" => android(root, session, api).await,
        "ios" => ios(session, api).await,
        _ => unreachable!(),
    }
}

async fn apple(root: &Path, session: &mut Session, api: &str) -> Result<()> {
    ensure!(
        cfg!(target_os = "macos"),
        "The Apple frontend requires macOS and Xcode"
    );
    let arguments = [
        "build",
        "--package-path",
        "apps/apple-simple",
        "--scratch-path",
        "target/apple-simple-spm",
    ];
    session.run(&mut cmd("swift", &arguments)).await?;
    let binary_dir = session
        .output(cmd("swift", &arguments).arg("--show-bin-path"))
        .await?;
    let port = api.rsplit(':').next().context("Backend port missing")?;
    let bundle = root
        .join("target/simple-apple-launch")
        .join(port)
        .join("Hickory Simple.app");
    let contents = bundle.join("Contents");
    std::fs::create_dir_all(contents.join("MacOS"))?;
    let executable = contents.join("MacOS/HickorySimple");
    std::fs::copy(
        Path::new(binary_dir.trim()).join("HickorySimple"),
        &executable,
    )?;
    std::fs::write(
        contents.join("Info.plist"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>HickorySimple</string>
<key>CFBundleIdentifier</key><string>com.hickorydocs.simple.mac</string>
<key>CFBundleName</key><string>Hickory Simple</string>
<key>CFBundleDisplayName</key><string>Hickory Simple</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>LSMinimumSystemVersion</key><string>14.0</string>
<key>LSUIElement</key><false/>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
<key>NSAppTransportSecurity</key><dict><key>NSAllowsLocalNetworking</key><true/></dict>
</dict></plist>
"#,
    )?;
    session
        .run(&mut cmd("codesign", &["--force", "--sign", "-"]).arg(&bundle))
        .await?;
    println!("Opening {}", bundle.display());
    let backend_env = format!("HICKORY_API_ORIGIN={api}");
    let host_env = format!("HICKORY_SIMPLE_HOST_PID={}", std::process::id());
    session
        .run(
            cmd(
                "open",
                &["-n", "-W", "--env", &backend_env, "--env", &host_env],
            )
            .arg(&bundle),
        )
        .await
}
fn sdk() -> Result<PathBuf> {
    let mut candidates: Vec<PathBuf> = ["ANDROID_HOME", "ANDROID_SDK_ROOT"]
        .iter()
        .filter_map(env::var_os)
        .map(PathBuf::from)
        .collect();
    if let Ok(properties) = std::fs::read_to_string("apps/android-simple/local.properties")
        && let Some(path) = properties
            .lines()
            .find_map(|line| line.strip_prefix("sdk.dir="))
    {
        candidates.push(PathBuf::from(
            path.replace("\\:", ":")
                .replace("\\\\", "\\")
                .replace("\\ ", " "),
        ));
    }
    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        candidates.push(home.join("Library/Android/sdk"));
        candidates.push(home.join("Android/Sdk"));
    }
    if let Some(local) = env::var_os("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local).join("Android/Sdk"));
    }
    candidates.into_iter().find(|path| path.join("platform-tools").is_dir()).context("Android SDK not found. Set ANDROID_HOME or open apps/android-simple in Android Studio first.")
}
fn tool(sdk: &Path, folder: &str, name: &str) -> PathBuf {
    sdk.join(folder).join(if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    })
}
fn adb(adb: &Path, serial: &str, args: &[&str]) -> Command {
    let mut command = cmd(adb, &["-s", serial]);
    command.args(args);
    command
}
fn device_serial(output: &str, requested: Option<&str>) -> Result<Option<String>> {
    let devices: Vec<_> = output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let serial = fields.next()?;
            (fields.next()? == "device").then_some(serial)
        })
        .collect();
    if let Some(serial) = requested {
        ensure!(
            devices.contains(&serial),
            "ANDROID_SERIAL={serial} is not connected and authorized"
        );
        return Ok(Some(serial.to_string()));
    }
    ensure!(
        devices.len() <= 1,
        "Several Android devices are connected. Set ANDROID_SERIAL to select one: {}",
        devices.join(", ")
    );
    Ok(devices.first().map(|value| value.to_string()))
}
async fn android(root: &Path, session: &mut Session, api: &str) -> Result<()> {
    let sdk = sdk()?;
    let adb_path = tool(&sdk, "platform-tools", "adb");
    let listing = session.output(&mut cmd(&adb_path, &["devices"])).await?;
    let mut serial = device_serial(&listing, env::var("ANDROID_SERIAL").ok().as_deref())?;
    if serial.is_none() {
        ensure!(
            listing.lines().skip(1).all(|line| line.trim().is_empty()),
            "An Android device is offline or unauthorized. Unlock/authorize it or restart its emulator."
        );
        let emulator = tool(&sdk, "emulator", "emulator");
        let avds = session.output(&mut cmd(&emulator, &["-list-avds"])).await?;
        let avds: Vec<_> = avds
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        let chosen = match env::var("HICKORY_ANDROID_AVD") {
            Ok(name) => {
                ensure!(avds.contains(&name.as_str()), "Unknown AVD {name}");
                name
            }
            Err(_) => {
                ensure!(
                    avds.len() == 1,
                    "Create an emulator in Android Studio Device Manager, or choose one with HICKORY_ANDROID_AVD. Available: {}",
                    avds.join(", ")
                );
                avds[0].to_string()
            }
        };
        session.spawn(&mut cmd(&emulator, &["-avd", &chosen]))?;
        for _ in 0..120 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let listing = session.output(&mut cmd(&adb_path, &["devices"])).await?;
            serial = device_serial(&listing, None)?;
            if serial.is_some() {
                break;
            }
        }
    }
    let serial = serial.context("Emulator did not become available within two minutes")?;
    let mut booted = false;
    for _ in 0..120 {
        if session
            .output(&mut adb(
                &adb_path,
                &serial,
                &["shell", "getprop", "sys.boot_completed"],
            ))
            .await?
            .trim()
            == "1"
        {
            booted = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    ensure!(
        booted,
        "Android device did not finish booting within two minutes"
    );
    session
        .run(&mut cmd(
            "rustc",
            &[
                "--edition=2021",
                "scripts/simple-android-gradle.rs",
                "-o",
                "target/simple-android-gradle",
            ],
        ))
        .await?;
    session
        .run(&mut cmd(
            root.join(if cfg!(windows) {
                "target/simple-android-gradle.exe"
            } else {
                "target/simple-android-gradle"
            }),
            &[":app:assembleDebug"],
        ))
        .await?;
    session
        .run(&mut adb(
            &adb_path,
            &serial,
            &[
                "install",
                "-r",
                "apps/android-simple/app/build/outputs/apk/debug/app-debug.apk",
            ],
        ))
        .await?;
    let port = api.rsplit(':').next().context("Backend port missing")?;
    let mapping = format!("tcp:{port}");
    session
        .run(&mut adb(
            &adb_path,
            &serial,
            &["reverse", "--no-rebind", &mapping, &mapping],
        ))
        .await?;
    session
        .undo
        .push(adb(&adb_path, &serial, &["reverse", "--remove", &mapping]));
    session.undo.push(adb(
        &adb_path,
        &serial,
        &["shell", "am", "force-stop", "com.hickorydocs.simple"],
    ));
    session
        .run(&mut adb(
            &adb_path,
            &serial,
            &[
                "shell",
                "am",
                "start",
                "-W",
                "-n",
                "com.hickorydocs.simple/.MainActivity",
                "--es",
                "hickory_api_origin",
                api,
            ],
        ))
        .await?;
    println!("Android running on {serial}. Ctrl-C stops this test app and its backend.");
    std::future::pending().await
}
fn simulator(value: &Value, requested: Option<&str>) -> Result<(String, bool)> {
    let devices: Vec<_> = value["devices"]
        .as_object()
        .context("Missing simulator devices")?
        .values()
        .filter_map(Value::as_array)
        .flatten()
        .filter(|d| {
            d["isAvailable"] == true
                && (d["name"].as_str().unwrap_or("").starts_with("iPhone")
                    || d["name"].as_str().unwrap_or("").starts_with("iPad"))
        })
        .collect();
    let selected = if let Some(id) = requested {
        devices
            .iter()
            .find(|d| d["udid"] == id || d["name"] == id)
            .copied()
            .context(
                "HICKORY_IOS_DEVICE must name an available iPhone/iPad simulator or its UDID",
            )?
    } else {
        let booted: Vec<_> = devices
            .iter()
            .filter(|d| d["state"] == "Booted")
            .copied()
            .collect();
        ensure!(
            booted.len() <= 1,
            "Several iOS simulators are booted. Set HICKORY_IOS_DEVICE to a name or UDID."
        );
        booted
            .first()
            .copied()
            .or_else(|| devices.first().copied())
            .context("Install an iOS simulator runtime in Xcode first")?
    };
    Ok((
        selected["udid"]
            .as_str()
            .context("Missing simulator UDID")?
            .into(),
        selected["state"] == "Booted",
    ))
}
async fn ios(session: &mut Session, api: &str) -> Result<()> {
    ensure!(
        cfg!(target_os = "macos"),
        "iOS simulator launch requires macOS and Xcode"
    );
    let list = session
        .output(&mut cmd(
            "xcrun",
            &["simctl", "list", "devices", "available", "--json"],
        ))
        .await?;
    let (device, booted) = simulator(
        &serde_json::from_str(&list)?,
        env::var("HICKORY_IOS_DEVICE").ok().as_deref(),
    )?;
    if !booted {
        session
            .run(&mut cmd("xcrun", &["simctl", "boot", &device]))
            .await?;
        session
            .undo
            .push(cmd("xcrun", &["simctl", "shutdown", &device]));
    }
    session
        .run(&mut cmd("xcrun", &["simctl", "bootstatus", &device, "-b"]))
        .await?;
    session
        .run(&mut cmd(
            "open",
            &["-a", "Simulator", "--args", "-CurrentDeviceUDID", &device],
        ))
        .await?;
    session
        .run(&mut cmd("just", &["simple-build", "ios"]))
        .await?;
    session.run(&mut cmd("xcrun", &["simctl", "install", &device, "target/apple-simple-derived/Build/Products/Debug-iphonesimulator/HickorySimpleMobile.app"])).await?;
    session.undo.push(cmd(
        "xcrun",
        &[
            "simctl",
            "terminate",
            &device,
            "com.hickorydocs.simple.mobile",
        ],
    ));
    session
        .run(
            cmd(
                "xcrun",
                &[
                    "simctl",
                    "launch",
                    "--terminate-running-process",
                    &device,
                    "com.hickorydocs.simple.mobile",
                ],
            )
            .env("SIMCTL_CHILD_HICKORY_API_ORIGIN", api),
        )
        .await?;
    println!("iOS running on {device}. Ctrl-C stops this test app and its backend.");
    std::future::pending().await
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_ambiguous_or_unauthorized_android_targets() {
        let devices = "List of devices attached\na\tdevice\nb\tdevice\nc\tunauthorized\n";
        assert!(device_serial(devices, None).is_err());
        assert_eq!(
            device_serial(devices, Some("b")).unwrap().as_deref(),
            Some("b")
        );
        assert!(device_serial(devices, Some("c")).is_err());
        assert_eq!(
            device_serial("List of devices attached\n", None).unwrap(),
            None
        );
    }
    #[test]
    fn ios_prefers_booted_target_and_honors_explicit_ipad() {
        let list = serde_json::json!({"devices":{"runtime":[
            {"name":"iPad Test","udid":"pad","state":"Shutdown","isAvailable":true},
            {"name":"iPhone Test","udid":"phone","state":"Booted","isAvailable":true},
            {"name":"iPhone Missing","udid":"gone","state":"Shutdown","isAvailable":false}
        ]}});
        assert_eq!(simulator(&list, None).unwrap(), ("phone".into(), true));
        assert_eq!(
            simulator(&list, Some("iPad Test")).unwrap(),
            ("pad".into(), false)
        );
        assert!(simulator(&list, Some("gone")).is_err());
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_reaps_owned_process_and_its_descendant() {
        let temp = tempfile::tempdir().unwrap();
        let pidfile = temp.path().join("descendant");
        let mut session = Session::default();
        session
            .spawn(cmd("sh", &["-c", "sleep 300 & echo $! > \"$1\"; wait", "test"]).arg(&pidfile))
            .unwrap();
        for _ in 0..100 {
            if pidfile.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let pid: i32 = std::fs::read_to_string(pidfile)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        session.cleanup().await;
        for _ in 0..100 {
            if unsafe { libc::kill(pid, 0) } != 0 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("Owned descendant {pid} survived cleanup");
    }
}
