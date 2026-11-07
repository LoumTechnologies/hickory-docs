use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

struct Tools {
    gradle: PathBuf,
    java_home: Option<PathBuf>,
}
fn executable(directory: &Path, name: &str) -> Option<PathBuf> {
    let path = directory.join(if cfg!(windows) {
        format!("{name}.bat")
    } else {
        name.to_string()
    });
    path.is_file().then_some(path)
}
fn tools(root: &Path) -> Result<Tools, Box<dyn std::error::Error>> {
    let home = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
    let wrapper = fs::read_to_string(
        root.join("apps/android-simple/gradle/wrapper/gradle-wrapper.properties"),
    )?;
    let url = wrapper
        .lines()
        .find_map(|line| line.strip_prefix("distributionUrl="))
        .ok_or("Missing Gradle distribution pin")?;
    let version = url
        .rsplit("gradle-")
        .next()
        .and_then(|value| value.strip_suffix("-bin.zip"))
        .ok_or("Expected a pinned binary Gradle distribution")?;
    let installed = env::var_os("GRADLE_HOME")
        .and_then(|path| executable(&PathBuf::from(path).join("bin"), "gradle"))
        .or_else(|| {
            env::var_os("PATH").and_then(|paths| {
                env::split_paths(&paths).find_map(|path| executable(&path, "gradle"))
            })
        });
    let cache = env::var_os("GRADLE_USER_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|path| path.join(".gradle")));
    let cached = cache.and_then(|cache| {
        ["bin", "all"].iter().find_map(|kind| {
            let directory = cache
                .join("wrapper/dists")
                .join(format!("gradle-{version}-{kind}"));
            fs::read_dir(directory)
                .ok()?
                .filter_map(Result::ok)
                .find_map(|entry| {
                    executable(
                        &entry.path().join(format!("gradle-{version}/bin")),
                        "gradle",
                    )
                })
        })
    });
    let gradle = installed.or(cached).ok_or_else(|| format!("Gradle {version} is not available. Open apps/android-simple in Android Studio and let its first sync download Gradle, or install Gradle {version} on PATH. Then retry this command."))?;
    let java_home = env::var_os("JAVA_HOME").map(PathBuf::from).or_else(|| {
        let mut studios: Vec<PathBuf> = env::var_os("ANDROID_STUDIO_HOME")
            .map(PathBuf::from)
            .into_iter()
            .collect();
        if cfg!(target_os = "macos") {
            if let Some(home) = &home {
                studios.push(home.join("Applications/Android Studio.app"));
            }
            studios.push(PathBuf::from("/Applications/Android Studio.app"));
        }
        studios.into_iter().find_map(|studio| {
            [
                studio.join("Contents/jbr/Contents/Home"),
                studio.join("jbr"),
            ]
            .into_iter()
            .find(|path| {
                path.join(if cfg!(windows) {
                    "bin/java.exe"
                } else {
                    "bin/java"
                })
                .is_file()
            })
        })
    });
    Ok(Tools { gradle, java_home })
}
fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let tools = tools(&root)?;
    let arguments: Vec<OsString> = env::args_os().skip(1).collect();
    if arguments.is_empty() {
        return Err("Pass a Gradle task, such as :app:assembleDebug".into());
    }
    let mut command = Command::new(tools.gradle);
    command
        .args(["--no-daemon", "-p"])
        .arg(root.join("apps/android-simple"))
        .args(arguments);
    if let Some(java_home) = tools.java_home {
        command.env("JAVA_HOME", java_home);
    }
    Ok(command.status()?.code().unwrap_or(1))
}
fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            eprintln!("Android build: {error}");
            ExitCode::FAILURE
        }
    }
}
