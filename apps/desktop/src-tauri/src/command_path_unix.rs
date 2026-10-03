use super::*;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

const START: &str = "# >>> Hickory Docs command PATH >>>";
const END: &str = "# <<< Hickory Docs command PATH <<<";

fn command(installer: &Installer) -> PathBuf {
    installer.home.join(".local/bin/hick")
}
fn quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}
fn wrapper(installer: &Installer) -> Option<String> {
    installer.appimage.as_ref().map(|image| format!("#!/bin/sh\n# Hickory Docs command launcher\nexport HICKORY_DESKTOP={}\nexec {} --hick-cli \"$@\"\n", quote(image), quote(image)))
}
fn owned(installer: &Installer) -> bool {
    let target = command(installer);
    if let Some(text) = wrapper(installer) {
        !target.is_symlink() && fs::read_to_string(target).ok().as_deref() == Some(&text)
    } else {
        fs::read_link(target).ok().as_ref() == Some(&installer.source)
    }
}

fn profiles(installer: &Installer) -> Result<Vec<(PathBuf, String)>> {
    let shell = Path::new(&installer.shell)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("zsh");
    let bin = quote(command(installer).parent().unwrap());
    // Bash reads only the first existing login profile. Creating .bash_profile
    // over an existing .profile would silently stop loading the person's setup.
    let login = [".bash_profile", ".bash_login", ".profile"]
        .into_iter()
        .find(|name| installer.home.join(name).exists())
        .unwrap_or(".profile");
    let (names, line) = match shell {
        "fish" => (
            vec![".config/fish/conf.d/hickory-docs.fish"],
            format!("fish_add_path --path {bin}"),
        ),
        "bash" => (
            vec![".bashrc", login],
            format!("export PATH={bin}:\"$PATH\""),
        ),
        "zsh" | "" => (vec![".zshrc"], format!("export PATH={bin}:\"$PATH\"")),
        "sh" | "dash" | "ksh" => (vec![".profile"], format!("export PATH={bin}:\"$PATH\"")),
        _ => bail!(
            "Automatic PATH setup is unavailable for {shell}. Add {} to your shell's PATH, then retry.",
            command(installer).parent().unwrap().display()
        ),
    };
    Ok(names
        .into_iter()
        .map(|name| {
            (
                installer.home.join(name),
                format!("{START}\n{line}\n{END}\n"),
            )
        })
        .collect())
}

pub fn status(installer: &Installer) -> Result<Status> {
    let target = command(installer);
    let own = owned(installer);
    let available =
        installer.source.is_file() && installer.appimage.as_ref().is_none_or(|p| p.is_file());
    let conflict = if fs::symlink_metadata(&target).is_ok() && !own {
        Some(target.clone())
    } else {
        installer.conflict(&target)
    };
    let on_path = installer.source.exists()
        && std::env::split_paths(&installer.search_path)
            .any(|p| p.join("hick").canonicalize().ok() == installer.source.canonicalize().ok());
    let profiles_ready = profiles(installer).is_ok_and(|profiles| {
        profiles
            .iter()
            .all(|(path, block)| read_profile(path).is_ok_and(|text| text.contains(block)))
    });
    Ok(Status {
        available,
        installed: (own && profiles_ready) || on_path,
        can_remove: own,
        command: target,
        source: installer.source.clone(),
        conflict,
        message: if (own && profiles_ready) || on_path {
            "Installed for your user. Open a new terminal to use hick."
        } else if !available {
            "The command is available in packaged desktop releases."
        } else {
            "Install hick for your user to open documents and use the CLI from a terminal."
        }
        .into(),
    })
}

pub fn install(installer: &Installer) -> Result<()> {
    let status = status(installer)?;
    require_available(&status)?;
    let profiles = profiles(installer)?;
    // Validate every profile before creating anything. An edited block is the
    // person's text and must not be replaced or deleted by this installer.
    for (path, block) in &profiles {
        let text = read_profile(path)?;
        if text.contains(START) && !text.contains(block) {
            bail!(
                "{} contains an edited Hickory Docs PATH block. Remove that block before retrying.",
                path.display()
            );
        }
    }
    fs::create_dir_all(status.command.parent().unwrap())?;
    if !owned(installer) {
        if let Some(text) = wrapper(installer) {
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&status.command)?;
            file.write_all(text.as_bytes())?;
            file.set_permissions(fs::Permissions::from_mode(0o755))?;
        } else {
            symlink(&installer.source, &status.command)?;
        }
    }
    for (path, block) in profiles {
        let mut text = read_profile(&path)?;
        if !text.contains(&block) {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&block);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, text)?;
        }
    }
    Ok(())
}

fn read_profile(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error)
            .with_context(|| format!("Cannot read {}; check its permissions", path.display())),
    }
}

pub fn remove(installer: &Installer) -> Result<()> {
    if !owned(installer) {
        bail!(
            "{} is not this app's command. Remove it through the installer that created it.",
            command(installer).display()
        );
    }
    for (path, block) in profiles(installer)? {
        let text = read_profile(&path)?;
        if text.contains(&block) {
            fs::write(path, text.replace(&block, ""))?;
        }
    }
    fs::remove_file(command(installer))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn installer(home: &Path) -> Installer {
        let source = home.join("Hickory Docs.app/Contents/MacOS/hick");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "cli").unwrap();
        Installer {
            source,
            home: home.into(),
            shell: "/bin/zsh".into(),
            appimage: None,
            search_path: Default::default(),
        }
    }
    // Guarantee: docs/guarantees/release/the-app-installs-its-command-for-the-user.md
    #[test]
    fn install_is_idempotent_and_removal_preserves_the_profile() {
        let home = tempfile::tempdir().unwrap();
        let setup = installer(home.path());
        let profile = home.path().join(".zshrc");
        fs::write(&profile, "# my settings\n").unwrap();
        install(&setup).unwrap();
        install(&setup).unwrap();
        assert!(owned(&setup));
        assert_eq!(
            fs::read_to_string(&profile).unwrap().matches(START).count(),
            1
        );
        remove(&setup).unwrap();
        assert_eq!(fs::read_to_string(profile).unwrap(), "# my settings\n");
        assert!(!command(&setup).exists());
    }
    #[test]
    fn existing_command_and_edited_blocks_are_preserved() {
        let home = tempfile::tempdir().unwrap();
        let setup = installer(home.path());
        fs::create_dir_all(command(&setup).parent().unwrap()).unwrap();
        fs::write(command(&setup), "someone else's command").unwrap();
        assert!(install(&setup).is_err());
        assert!(remove(&setup).is_err());
        assert_eq!(
            fs::read_to_string(command(&setup)).unwrap(),
            "someone else's command"
        );
        fs::remove_file(command(&setup)).unwrap();
        fs::write(
            home.path().join(".zshrc"),
            format!("{START}\ncustom\n{END}\n"),
        )
        .unwrap();
        assert!(install(&setup).is_err());
        assert!(!command(&setup).exists());
    }
    #[test]
    fn portable_launcher_uses_the_saved_appimage() {
        let home = tempfile::tempdir().unwrap();
        let mut setup = installer(home.path());
        let image = home.path().join("Hickory's Docs.AppImage");
        fs::write(&image, "appimage").unwrap();
        setup.appimage = Some(image);
        install(&setup).unwrap();
        let text = fs::read_to_string(command(&setup)).unwrap();
        assert!(text.contains("--hick-cli"));
        assert!(text.contains("'\\''"));
        assert!(!text.contains("Contents/MacOS"));
        remove(&setup).unwrap();
    }
}

#[cfg(test)]
mod bash_tests {
    use super::*;
    // Guarantee: docs/guarantees/release/the-app-installs-its-command-for-the-user.md
    #[test]
    fn bash_keeps_the_existing_login_profile_in_use() {
        let home = tempfile::tempdir().unwrap();
        let source = home.path().join("hick");
        fs::write(&source, "cli").unwrap();
        fs::write(home.path().join(".profile"), "# login setup\n").unwrap();
        let installer = Installer {
            source,
            home: home.path().into(),
            shell: "/bin/bash".into(),
            appimage: None,
            search_path: Default::default(),
        };
        install(&installer).unwrap();
        assert!(!home.path().join(".bash_profile").exists());
        assert!(
            fs::read_to_string(home.path().join(".profile"))
                .unwrap()
                .starts_with("# login setup\n")
        );
        remove(&installer).unwrap();
        assert_eq!(
            fs::read_to_string(home.path().join(".profile")).unwrap(),
            "# login setup\n"
        );
    }
}
