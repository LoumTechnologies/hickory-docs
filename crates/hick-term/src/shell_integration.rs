//! Teaching a shell to say where it is.
//!
//! Protects docs/guarantees/terminal/a-session-appears-where-it-is-working.md
//!
//! Sessions report their working directory through **OSC 7**, and on macOS no
//! shell emits it: Apple's hook lives in `/etc/zshrc_Apple_Terminal`, reached
//! only when `TERM_PROGRAM` says `Apple_Terminal`. Measured on macOS 15.7.7,
//! `zsh -i` emits zero sequences and `bash -l -i` zero, so every session's row
//! sat at the directory it started in for the life of the session.
//!
//! Claiming to be Terminal.app would fix it and is not done here, for two
//! reasons. It is a lie to every program that asks what terminal it is talking
//! to. And that file is mostly *not* OSC 7 — it is Terminal.app's session
//! save/restore machinery, which splits the user's shell history into
//! per-session files keyed on `$TERM_SESSION_ID`. A terminal launched from
//! another terminal inherits that id, so every session in this app would share
//! one history file; launched from the Dock it would not, and the behaviour
//! would differ by how the app was started.
//!
//! So the shell is *told*, the way editors that need this have settled on: a
//! generated startup file that **sources the user's own configuration first**
//! and then adds one hook. Nothing the user owns is edited, and nothing
//! persists — the scripts live in a temporary directory that goes when the
//! session does.
//!
//! ## The rule this is built around
//!
//! **A shell that fails to integrate must still be a working shell.** Someone
//! opened a terminal; a clever feature is not worth a broken prompt. Every
//! branch here degrades to "no OSC 7", which is the behaviour that was already
//! shipping, and `HICKORY_SHELL_INTEGRATION=0` turns the whole thing off.

use std::path::Path;

/// What a shell needs added to its startup so it reports its directory.
pub struct Integration {
    env: Vec<(String, String)>,
    args: Vec<String>,
    /// Whether the hook this shell got can report the commands it runs.
    ///
    /// The two halves of the integration are not the same promise, and until
    /// this field existed they were conflated. Reporting the working
    /// directory (OSC 7) is a `precmd`/`PROMPT_COMMAND` hook that every
    /// version of both shells has had for decades. Reporting the *command*
    /// (OSC 633) rides on `PS0` in bash, which arrived in **bash 4.4** — and
    /// the bash Apple ships is 3.2, because 4.0 changed licence to GPLv3.
    ///
    /// So on a stock Mac the rcfile installed cleanly, the prompt worked, the
    /// directory updated, and `PS0` sat there as an ordinary unused variable
    /// that bash 3.2 never expands. The terminal looked anchorable and
    /// recorded nothing — exactly the silent anchor
    /// `a-terminal-that-writes-the-document.md` forbids.
    reports_commands: bool,
    /// The generated scripts. Dropped with the session, which is why the
    /// session holds this rather than the spawn function.
    _dir: tempfile::TempDir,
}

impl Integration {
    /// Environment variables to set on the child.
    pub fn env(&self) -> &[(String, String)] {
        &self.env
    }

    /// Arguments to pass to the shell, before any of its own.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Whether this shell reports the commands it runs, not just its
    /// directory.
    ///
    /// False for a bash too old for `PS0`. The session is fully usable and
    /// still reports its directory; what it cannot do is be anchored to a
    /// document, and the anchor endpoint says so by name.
    pub fn reports_commands(&self) -> bool {
        self.reports_commands
    }

    /// Build the integration for `shell`, or `None` when there is none to
    /// build.
    ///
    /// `None` is a normal outcome, not a failure: an unrecognised shell, a
    /// `sh` we will not guess about, or a temporary directory we could not
    /// create all mean the session runs exactly as it did before.
    pub fn install(shell: &str) -> Option<Self> {
        match family(shell)? {
            Family::Zsh => zsh(),
            Family::Bash => bash(shell),
        }
    }
}

/// Which shell this is, by the name it was invoked as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    Zsh,
    Bash,
}

/// Recognised from the basename, and deliberately narrow.
///
/// `sh` is not on the list even though it is usually bash or dash underneath.
/// A shell invoked as `sh` is in POSIX mode and reads a different file, and
/// guessing which one is how a startup file ends up sourced twice — so `sh`
/// keeps the old behaviour and reports its start directory.
fn family(shell: &str) -> Option<Family> {
    let name = Path::new(shell).file_name()?.to_string_lossy().to_string();
    let name = name.trim_end_matches(".exe");
    match name {
        "zsh" | "-zsh" => Some(Family::Zsh),
        "bash" | "-bash" => Some(Family::Bash),
        _ => None,
    }
}

/// The hook, in zsh.
///
/// Percent-encoding is done byte by byte under `LC_ALL=C`, the way Apple's own
/// script does it, so a directory with a space or an accent in it survives the
/// trip. `screen.rs`'s `percent_decode` is the other half.
///
/// `precmd_functions` is appended to directly rather than through
/// `add-zsh-hook`, which is one fewer autoload that can be missing.
const ZSH_HOOK: &str = r#"
# Hickory Docs shell integration: report the working directory (OSC 7).
__hickory_report_cwd() {
  emulate -L zsh
  local url='' i ch hex
  local LC_ALL=C
  for (( i = 1; i <= ${#PWD}; ++i )); do
    ch="${PWD[i]}"
    if [[ "$ch" == [/._~A-Za-z0-9-] ]]; then
      url+="$ch"
    else
      printf -v hex '%02X' "'$ch"
      url+="%$hex"
    fi
  done
  printf '\033]7;file://%s%s\a' "${HOST:-}" "$url"
}
if [[ -z "${precmd_functions[(r)__hickory_report_cwd]}" ]]; then
  precmd_functions+=(__hickory_report_cwd)
fi

# Hickory Docs shell integration: report the command being run (OSC 633).
# `preexec` receives the typed line as $1 — already assembled by the shell, so
# none of readline's editing has to be reconstructed from keystrokes.
__hickory_report_cmd() {
  emulate -L zsh
  local s="$1" url='' i ch hex
  local LC_ALL=C
  for (( i = 1; i <= ${#s}; ++i )); do
    ch="${s[i]}"
    if [[ "$ch" == [/._~A-Za-z0-9-] ]]; then
      url+="$ch"
    else
      printf -v hex '%02X' "'$ch"
      url+="%$hex"
    fi
  done
  # $HISTCMD is the shell's own counter, and it is here for the same reason
  # bash sends a history number: a counter that did NOT move means the shell
  # kept this line out of its history, which is a person saying "not this
  # one" and is recorded as a suspension rather than as a command.
  printf '\033]633;hickory-cmd;%s;%s\a' "${HISTCMD:-0}" "$url"
}
if [[ -z "${preexec_functions[(r)__hickory_report_cmd]}" ]]; then
  preexec_functions+=(__hickory_report_cmd)
fi
"#;

/// The hook, in bash.
///
/// `PROMPT_COMMAND` is *prepended* to whatever the user already had, and their
/// value is kept: a prompt that draws a git branch is theirs, and replacing it
/// to add a feature they did not ask for would be worse than not having the
/// feature.
const BASH_HOOK: &str = r#"
# Hickory Docs shell integration: report the working directory (OSC 7).
__hickory_report_cwd() {
  local url='' i ch hex code
  local LC_ALL=C
  for (( i = 0; i < ${#PWD}; ++i )); do
    ch="${PWD:i:1}"
    case "$ch" in
      [/._~A-Za-z0-9-]) url="$url$ch" ;;
      *)
        # Masked to a byte on purpose. `printf '%d' "'é"` in bash reports -61,
        # not 195 — it sign-extends — and the unmasked form emitted
        # `%FFFFFFFFFFFFFFC3`, so a directory with an accent in it arrived as
        # nonsense. zsh's printf does not do this, which is why only one of
        # these two hooks needs the arithmetic.
        printf -v code '%d' "'$ch"
        printf -v hex '%02X' "$(( code & 0xFF ))"
        url="$url%$hex"
        ;;
    esac
  done
  printf '\033]7;file://%s%s\a' "${HOSTNAME:-}" "$url"
}
case "${PROMPT_COMMAND:-}" in
  *__hickory_report_cwd*) ;;
  *) PROMPT_COMMAND="__hickory_report_cwd${PROMPT_COMMAND:+; $PROMPT_COMMAND}" ;;
esac

# Hickory Docs shell integration: report the command being run (OSC 633).
#
# PS0 and not a DEBUG trap, and not PROMPT_COMMAND. Measured on bash 5.3.9
# against a real PTY, because two obvious answers are wrong:
#
#   * a DEBUG trap fires per EXECUTED command, so `ls | head` arrives as two
#     and a `for` loop as one per iteration — a cell of those does not
#     reproduce, which is the whole invariant;
#   * PROMPT_COMMAND with `history 1` fires at the FIRST prompt too, and
#     reports a line out of the user's ~/.bash_history that they never typed
#     in this session.
#
# PS0 is expanded exactly once per submitted line, after the line is read and
# before it runs. It is expanded in a SUBSHELL, so this cannot keep state
# between firings — which is why the history number is sent and the comparing
# is done in Rust.
__hickory_report_cmd() {
  local s="$1" url='' i ch hex code
  local LC_ALL=C
  for (( i = 0; i < ${#s}; ++i )); do
    ch="${s:i:1}"
    case "$ch" in
      [/._~A-Za-z0-9-]) url="$url$ch" ;;
      *)
        printf -v code '%d' "'$ch"
        printf -v hex '%02X' "$(( code & 0xFF ))"
        url="$url%$hex"
        ;;
    esac
  done
  printf '\033]633;hickory-cmd;%s;%s\a' "$2" "$url"
}
__hickory_ps0() {
  local raw n body
  raw="$(HISTTIMEFORMAT= history 1)"
  # `history 1` prints "<padding><number><SEP><command>", where SEP is
  # EXACTLY two characters and the command follows verbatim. Measured on
  # bash 5.3.9 by printing three entries and looking at the bytes:
  #
  #     "    1  echo plain"      -> "echo plain"
  #     "    2   echo hidden"    -> " echo hidden"
  #     "    3    echo two"      -> "  echo two"
  #
  # so the separator must be cut by LENGTH, not by "strip whitespace". This
  # used to strip all leading whitespace, which got the plain case right and
  # silently ate the user's own leading space in the other two. That space is
  # the "do not record this" gesture every shell with a history honours, and
  # hick honours it in Rust for both shells (`anchor::Suspension::
  # HiddenByLeadingSpace`) — which it cannot do if the space never arrives.
  # Where HISTCONTROL is `ignorespace` the line is never reported at all and
  # the bug is invisible; that is a common default, which is how it survived.
  n="${raw#"${raw%%[![:space:]]*}"}"
  n="${n%%[[:space:]]*}"
  body="${raw#*"$n"}"
  body="${body:2}"
  # HISTCONTROL (ignorespace, ignoredups — both common defaults) can keep a
  # line out of history, and then `history 1` still shows the PREVIOUS one.
  # Sending it would record a command that did not run. The number is what
  # tells the two apart, and an unchanged number is a suspension, decided in
  # Rust because this runs in a subshell and cannot remember anything.
  [[ -n "$n" ]] && __hickory_report_cmd "$body" "$n"
}
PS0='$(__hickory_ps0)'"${PS0:-}"
"#;

/// zsh, through `ZDOTDIR`.
///
/// zsh reads its startup files from `$ZDOTDIR`, so pointing it at a generated
/// directory is the only way in that does not touch a file the user owns. Each
/// generated file sources the user's equivalent first, which is what keeps
/// their prompt, aliases and completions exactly as they were.
///
/// All four files are written, not just `.zshrc`. Setting `ZDOTDIR` moves
/// **every** startup file, so a `.zshenv` that is not forwarded is a `.zshenv`
/// that silently stops running — and `.zshenv` is where people put `PATH`.
fn zsh() -> Option<Integration> {
    let dir = tempfile::Builder::new()
        .prefix("hickory-zdotdir-")
        .tempdir()
        .ok()?;

    // Where the user's own files are. `ZDOTDIR` if they set one, else `$HOME`,
    // which is zsh's own rule.
    let user_zdotdir = std::env::var("ZDOTDIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("HOME").ok())
        .unwrap_or_default();

    // `.zshenv` runs first and is the one that can move the goalposts: a user's
    // own `.zshenv` may set `ZDOTDIR`, and zsh would then read `.zshrc` from
    // there instead of from here. So it is sourced with `ZDOTDIR` pointed at
    // them, and where they changed it, that becomes the directory the rest of
    // this forwards to.
    let zshenv = r#"
# Hickory Docs shell integration. Generated per session; not yours to keep.
__hickory_ours="$ZDOTDIR"
if [[ -n "$HICKORY_USER_ZDOTDIR" ]]; then
  ZDOTDIR="$HICKORY_USER_ZDOTDIR"
  [[ -f "$HICKORY_USER_ZDOTDIR/.zshenv" ]] && . "$HICKORY_USER_ZDOTDIR/.zshenv"
  if [[ "$ZDOTDIR" != "$HICKORY_USER_ZDOTDIR" ]]; then
    export HICKORY_USER_ZDOTDIR="$ZDOTDIR"
  fi
fi
ZDOTDIR="$__hickory_ours"
unset __hickory_ours
"#;

    // The others just forward. `.zshrc` additionally installs the hook, and
    // hands `ZDOTDIR` back afterwards so anything the user runs later sees the
    // value they expect rather than a temporary directory.
    let forward = |file: &str, hook: &str| {
        format!(
            r#"
# Hickory Docs shell integration. Generated per session; not yours to keep.
if [[ -n "$HICKORY_USER_ZDOTDIR" && -f "$HICKORY_USER_ZDOTDIR/{file}" ]]; then
  ZDOTDIR="$HICKORY_USER_ZDOTDIR"
  . "$HICKORY_USER_ZDOTDIR/{file}"
fi
{hook}
if [[ -n "$HICKORY_ZDOTDIR_WAS_SET" ]]; then
  export ZDOTDIR="$HICKORY_USER_ZDOTDIR"
else
  unset ZDOTDIR
fi
"#
        )
    };

    std::fs::write(dir.path().join(".zshenv"), zshenv).ok()?;
    std::fs::write(dir.path().join(".zprofile"), forward(".zprofile", "")).ok()?;
    std::fs::write(dir.path().join(".zshrc"), forward(".zshrc", ZSH_HOOK)).ok()?;
    std::fs::write(dir.path().join(".zlogin"), forward(".zlogin", "")).ok()?;

    let mut env = vec![
        (
            "ZDOTDIR".to_string(),
            dir.path().to_string_lossy().to_string(),
        ),
        ("HICKORY_USER_ZDOTDIR".to_string(), user_zdotdir),
    ];
    // Whether to restore `ZDOTDIR` or unset it afterwards — because "set to
    // $HOME" and "not set" are different states, and turning the second into
    // the first is a change we were not asked to make.
    if std::env::var_os("ZDOTDIR").is_some() {
        env.push(("HICKORY_ZDOTDIR_WAS_SET".to_string(), "1".to_string()));
    }
    Some(Integration {
        env,
        args: Vec::new(),
        // zsh's `preexec` receives the typed line directly and has been in
        // every zsh anyone can install, so there is no version to probe.
        reports_commands: true,
        _dir: dir,
    })
}

/// bash, through `--rcfile`.
///
/// Simpler than zsh: bash reads exactly one file for an interactive
/// non-login shell and `--rcfile` replaces it, so the generated file sources
/// `~/.bashrc` and adds the hook. `--rcfile` is ignored by a login shell, which
/// is a gap this does not close — a session started as `bash -l` reports its
/// start directory, as before.
fn bash(shell: &str) -> Option<Integration> {
    let dir = tempfile::Builder::new()
        .prefix("hickory-bashrc-")
        .tempdir()
        .ok()?;
    let rcfile = dir.path().join("hickory-bashrc");
    let body = format!(
        r#"
# Hickory Docs shell integration. Generated per session; not yours to keep.
if [ -f "$HOME/.bashrc" ]; then
  . "$HOME/.bashrc"
fi
{BASH_HOOK}
"#
    );
    std::fs::write(&rcfile, body).ok()?;
    Some(Integration {
        env: Vec::new(),
        args: vec!["--rcfile".to_string(), rcfile.to_string_lossy().to_string()],
        reports_commands: has_ps0(shell),
        _dir: dir,
    })
}

/// The oldest bash whose `PS0` is expanded before each command.
///
/// bash 4.4 (2016). Apple ships 3.2 and will not move, so this is not an
/// exotic case to guard against — it is what `/bin/bash` is on every Mac.
const BASH_PS0_SINCE: (u32, u32) = (4, 4);

/// Whether this bash expands `PS0`, asked of the binary rather than assumed.
///
/// The shell is run once, non-interactively, to print its own version. That
/// is cheaper and more honest than parsing `bash --version`'s prose, and it
/// asks the executable that will actually be spawned — a user whose
/// `HICKORY_SHELL` points at a Homebrew bash 5 gets command reporting on the
/// same Mac where `/bin/bash` does not.
///
/// A probe that cannot be run at all answers **false**: an integration that
/// might not report is treated as one that does not, because the failure of
/// the other choice is a terminal that says it is recording and is not.
fn has_ps0(shell: &str) -> bool {
    let Ok(out) = std::process::Command::new(shell)
        .arg("-c")
        .arg(r#"printf '%s.%s' "${BASH_VERSINFO[0]}" "${BASH_VERSINFO[1]}""#)
        .output()
    else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut parts = text.trim().split('.');
    let Some(major) = parts.next().and_then(|v| v.parse::<u32>().ok()) else {
        return false;
    };
    let minor = parts
        .next()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    (major, minor) >= BASH_PS0_SINCE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shell_we_do_not_recognise_is_left_alone() {
        assert!(Integration::install("/bin/sh").is_none());
        assert!(Integration::install("/usr/bin/fish").is_none());
        assert!(Integration::install("cmd.exe").is_none());
    }

    #[test]
    fn zsh_and_bash_are_recognised_by_basename_wherever_they_live() {
        assert_eq!(family("/bin/zsh"), Some(Family::Zsh));
        assert_eq!(family("/opt/homebrew/bin/zsh"), Some(Family::Zsh));
        assert_eq!(family("/usr/local/bin/bash"), Some(Family::Bash));
        // A login shell is spelled with a leading dash in `argv[0]`.
        assert_eq!(family("-zsh"), Some(Family::Zsh));
        assert_eq!(family("/bin/sh"), None);
    }

    /// Setting `ZDOTDIR` moves every startup file, so every one of them has to
    /// be forwarded. A missing `.zshenv` is a silently missing `PATH`.
    #[test]
    fn every_zsh_startup_file_is_forwarded_not_just_zshrc() {
        let integration = zsh().expect("a temp dir");
        let dir = integration
            .env()
            .iter()
            .find(|(key, _)| key == "ZDOTDIR")
            .map(|(_, value)| std::path::PathBuf::from(value))
            .expect("ZDOTDIR is set");
        for file in [".zshenv", ".zshrc", ".zprofile", ".zlogin"] {
            let text = std::fs::read_to_string(dir.join(file))
                .unwrap_or_else(|_| panic!("{file} was not written"));
            assert!(
                text.contains(&format!("$HICKORY_USER_ZDOTDIR/{file}")),
                "{file} does not source the user's own:\n{text}"
            );
        }
    }

    #[test]
    fn only_zshrc_installs_the_hook() {
        let integration = zsh().expect("a temp dir");
        let dir = integration
            .env()
            .iter()
            .find(|(key, _)| key == "ZDOTDIR")
            .map(|(_, value)| std::path::PathBuf::from(value))
            .expect("ZDOTDIR is set");
        let zshrc = std::fs::read_to_string(dir.join(".zshrc")).unwrap();
        assert!(zshrc.contains("__hickory_report_cwd"), "{zshrc}");
        for quiet in [".zshenv", ".zprofile", ".zlogin"] {
            let text = std::fs::read_to_string(dir.join(quiet)).unwrap();
            assert!(
                !text.contains("precmd_functions"),
                "{quiet} installs the hook twice:\n{text}"
            );
        }
    }

    #[test]
    fn bash_keeps_whatever_prompt_command_the_user_had() {
        let integration = bash("bash").expect("a temp dir");
        let rcfile = integration
            .args()
            .iter()
            .find(|arg| arg.contains("hickory-bashrc"))
            .expect("an rcfile argument");
        let text = std::fs::read_to_string(rcfile).unwrap();
        assert!(text.contains(". \"$HOME/.bashrc\""), "{text}");
        assert!(
            text.contains("${PROMPT_COMMAND:+; $PROMPT_COMMAND}"),
            "the user's PROMPT_COMMAND is discarded:\n{text}"
        );
    }

    /// The version gate, asked of a program that is definitely not bash.
    ///
    /// `has_ps0` runs the binary and reads what it prints; `true(1)` prints
    /// nothing, so this covers the "probe produced no version" branch, which
    /// must answer false rather than assume the good case. A shell that is
    /// not there at all is the same branch one level up (`Command::output`
    /// fails), and both mean the same thing: do not claim to record.
    #[test]
    fn a_shell_that_cannot_say_its_version_does_not_claim_to_report_commands() {
        assert!(!has_ps0("/usr/bin/true"));
        assert!(!has_ps0("/nonexistent/bash"));
    }

    /// The whole point of the split: a bash too old for `PS0` still gets an
    /// integration — its prompt and its directory work exactly as before —
    /// and simply does not claim to report commands.
    #[test]
    fn an_old_bash_still_integrates_but_reports_no_commands() {
        let integration = bash("/usr/bin/true").expect("a temp dir");
        assert!(!integration.reports_commands());
        assert!(
            integration
                .args()
                .iter()
                .any(|arg| arg.contains("hickory-bashrc")),
            "the rcfile is still installed, so OSC 7 still works"
        );
    }
}
