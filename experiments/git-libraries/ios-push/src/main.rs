//! What the iOS sandbox does to a `git2` push over HTTPS with a token.
//!
//! Protects the "Git on a phone" section of
//! `docs/specs/freeform/shipping-mobile-and-desktop.md`, whose push claim was
//! established by reading `Remote::push`'s signature rather than by pushing.
//!
//! Runs as an installed `.app` on a booted simulator. Configuration arrives in
//! the environment (`SIMCTL_CHILD_*` at launch) so the token is never written to
//! a file and never appears in output: every line below prints its length, not
//! its value.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::PathBuf;

/// Everything the probe learned, in the order it learned it.
struct Report(String);

impl Report {
    fn new() -> Self {
        Self(String::new())
    }

    fn say(&mut self, line: impl AsRef<str>) {
        let line = line.as_ref();
        println!("{line}");
        let _ = std::io::stdout().flush();
        let _ = writeln!(self.0, "{line}");
    }
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

/// The app's own container, which is the only place iOS lets it write.
fn sandbox_root() -> PathBuf {
    env("HOME").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
}

/// Can a sandboxed iOS app spawn a process? The spec asserts it cannot, which
/// is the load-bearing reason `hick-store` is not portable.
///
/// **A simulator answers this wrongly and it is worth knowing why.** A
/// simulator app is a macOS process wearing an iOS runtime, so `posix_spawn`
/// works here and does not on a device. Measured anyway, and reported as what it
/// is: a measurement of the simulator, not of iOS. Absolute path and no reliance
/// on `PATH`, which is empty inside an app container.
fn measure_spawning(report: &mut Report) {
    let attempt = std::process::Command::new("/bin/echo").arg("spawned").output();
    match attempt {
        Ok(out) => report.say(format!(
            "SPAWN: /bin/echo ran, status {:?}, stdout {:?} — fork/exec PERMITTED \
             (this is the simulator being macOS underneath; a device would refuse)",
            out.status,
            String::from_utf8_lossy(&out.stdout).trim()
        )),
        Err(error) => report.say(format!(
            "SPAWN: refused — {} (kind {:?})",
            error,
            error.kind()
        )),
    }
}

fn write_file(path: &std::path::Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)
}

/// One commit in a fresh repository inside the container.
fn build_commit(work: &std::path::Path, branch: &str) -> Result<git2::Oid, git2::Error> {
    let repo = git2::Repository::init(work)?;
    let note = work.join("note.hick");
    write_file(&note, "a note written inside an iOS app container\n")
        .map_err(|e| git2::Error::from_str(&format!("writing note.hick: {e}")))?;

    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("note.hick"))?;
    index.write()?;
    let tree = repo.find_tree(index.write_tree()?)?;

    // A signature with no git config to read it from: on a phone there is no
    // ~/.gitconfig, so anything that relies on one fails here rather than on the
    // desktop where it happens to work.
    let who = git2::Signature::now("hickory ios probe", "probe@example.invalid")?;
    let oid = repo.commit(None, &who, &who, "push probe", &tree, &[])?;
    repo.reference(
        &format!("refs/heads/{branch}"),
        oid,
        true,
        "probe branch",
    )?;
    Ok(oid)
}

fn callbacks(user: &str, token: &str) -> git2::RemoteCallbacks<'static> {
    let user = user.to_string();
    let token = token.to_string();
    let mut callbacks = git2::RemoteCallbacks::new();
    // HTTPS with a token, never SSH and never a credential helper — the
    // constraint the spec derives from all three libraries spawning for both.
    callbacks.credentials(move |url, username, allowed| {
        eprintln!(
            "CRED: url={url} username_from_url={username:?} allowed={allowed:?}"
        );
        git2::Cred::userpass_plaintext(&user, &token)
    });
    callbacks
}

fn main() {
    let mut report = Report::new();
    report.say("=== hickory iOS push probe ===");
    let v = git2::Version::get();
    let (major, minor, patch) = v.libgit2_version();
    report.say(format!(
        "libgit2 {major}.{minor}.{patch} — https={} ssh={} threads={}",
        v.https(),
        v.ssh(),
        v.threads()
    ));
    report.say(format!("HOME (container): {}", sandbox_root().display()));

    measure_spawning(&mut report);

    let Some(remote_url) = env("PROBE_REMOTE") else {
        report.say("SKIP: PROBE_REMOTE unset — nothing to push to");
        finish(&report);
        return;
    };
    let Some(token) = env("PROBE_TOKEN") else {
        report.say("SKIP: PROBE_TOKEN unset");
        finish(&report);
        return;
    };
    let user = env("PROBE_USER").unwrap_or_else(|| "x-access-token".to_string());
    let branch = env("PROBE_BRANCH").unwrap_or_else(|| "ios-probe".to_string());
    report.say(format!(
        "remote: {remote_url} (token {} chars, user {user}, branch {branch})",
        token.len()
    ));

    let work = sandbox_root().join("Documents").join("probe-repo");
    let _ = std::fs::remove_dir_all(&work);
    let oid = match build_commit(&work, &branch) {
        Ok(oid) => {
            report.say(format!("COMMIT: {oid} in {}", work.display()));
            oid
        }
        Err(error) => {
            report.say(format!("COMMIT FAILED: {error}"));
            finish(&report);
            return;
        }
    };

    let repo = match git2::Repository::open(&work) {
        Ok(repo) => repo,
        Err(error) => {
            report.say(format!("REOPEN FAILED: {error}"));
            finish(&report);
            return;
        }
    };

    let mut remote = match repo.remote("origin", &remote_url) {
        Ok(remote) => remote,
        Err(error) => {
            report.say(format!("REMOTE FAILED: {error}"));
            finish(&report);
            return;
        }
    };

    // Does the sandbox let libgit2 open a TLS connection at all? `ls` needs the
    // reference-advertisement half of smart HTTP and nothing else, so it
    // separates "the network is refused" from "the push is refused".
    let mut connect_callbacks = callbacks(&user, &token);
    connect_callbacks.sideband_progress(|data| {
        eprintln!("SIDEBAND: {}", String::from_utf8_lossy(data).trim_end());
        true
    });
    match remote.connect_auth(git2::Direction::Push, Some(connect_callbacks), None) {
        Ok(connection) => {
            let advertised: Vec<String> = connection
                .list()
                .map(|heads| {
                    heads
                        .iter()
                        .map(|h| format!("{} {}", h.oid(), h.name()))
                        .collect()
                })
                .unwrap_or_default();
            report.say(format!(
                "CONNECT: ok, remote advertised {} ref(s)",
                advertised.len()
            ));
            for line in advertised.iter().take(10) {
                report.say(format!("  advertised: {line}"));
            }
        }
        Err(error) => {
            report.say(format!(
                "CONNECT FAILED: class={:?} code={:?} {}",
                error.class(),
                error.code(),
                error.message()
            ));
            finish(&report);
            return;
        }
    }
    remote.disconnect().ok();

    let mut push_options = git2::PushOptions::new();
    let mut push_callbacks = callbacks(&user, &token);
    push_callbacks.push_update_reference(|refname, status| {
        match status {
            None => eprintln!("PUSH-STATUS: {refname} accepted"),
            Some(reason) => eprintln!("PUSH-STATUS: {refname} REJECTED: {reason}"),
        }
        Ok(())
    });
    push_callbacks.push_transfer_progress(|current, total, bytes| {
        if total > 0 && current == total {
            eprintln!("PUSH-TRANSFER: {current}/{total} objects, {bytes} bytes");
        }
    });
    push_options.remote_callbacks(push_callbacks);

    let refspec = format!("+refs/heads/{branch}:refs/heads/{branch}");
    match remote.push(&[refspec.as_str()], Some(&mut push_options)) {
        Ok(()) => report.say(format!("PUSH: ok, {refspec}")),
        Err(error) => {
            report.say(format!(
                "PUSH FAILED: class={:?} code={:?} {}",
                error.class(),
                error.code(),
                error.message()
            ));
            finish(&report);
            return;
        }
    }

    // What does the remote say the ref is now? Read it back over a second
    // connection rather than trusting the push's own report.
    let mut verify = match repo.find_remote("origin") {
        Ok(remote) => remote,
        Err(error) => {
            report.say(format!("VERIFY FAILED to find remote: {error}"));
            finish(&report);
            return;
        }
    };
    let verify_callbacks = callbacks(&user, &token);
    match verify.connect_auth(git2::Direction::Fetch, Some(verify_callbacks), None) {
        Ok(connection) => match connection.list() {
            Ok(heads) => {
                let wanted = format!("refs/heads/{branch}");
                let found = heads.iter().find(|h| h.name() == wanted);
                match found {
                    Some(head) => report.say(format!(
                        "VERIFY: remote {} = {} (local commit was {oid}) — {}",
                        head.name(),
                        head.oid(),
                        if head.oid() == oid { "MATCH" } else { "MISMATCH" }
                    )),
                    None => report.say(format!("VERIFY: remote does not advertise {wanted}")),
                }
            }
            Err(error) => report.say(format!("VERIFY list failed: {error}")),
        },
        Err(error) => report.say(format!("VERIFY connect failed: {error}")),
    }

    finish(&report);
}

/// Leave the report where the host can read it with `simctl get_app_container`,
/// because a launched app's stdout is not always attachable.
fn finish(report: &Report) {
    let out = sandbox_root().join("Documents").join("probe-report.txt");
    match write_file(&out, &report.0) {
        Ok(()) => println!("=== report written to {} ===", out.display()),
        Err(error) => println!("=== could not write report: {error} ==="),
    }
    let _ = std::io::stdout().flush();
}
