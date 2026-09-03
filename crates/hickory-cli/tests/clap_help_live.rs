//! The clap help parser, against the real tools on this machine.
//!
//! Protects docs/guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options.md
//!
//! The unit tests in `clap_help.rs` read captured fixtures, which is what
//! makes them stable — and is exactly why they cannot notice that clap
//! changed its layout in a version nobody here has pinned. This runs the
//! parser over whatever `uv` and `cargo` actually print today.
//!
//! Skipped loudly and separately for each tool, because "skipped" and
//! "passed" must never look alike.

use std::process::Command;

use hickory_cli::clap_help::parse;
use hickory_cli::scaffold::OptionKind;

fn help(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[test]
fn uv_init_is_read_as_a_form() {
    let Some(text) = help("uv", &["init", "--help"]) else {
        eprintln!("SKIPPED: no `uv` on this machine — install it to cover this");
        return;
    };
    let parsed = parse(&text);
    assert!(parsed.is_help(), "no Usage: line in uv's own help");
    assert!(
        parsed.usage.starts_with("uv init"),
        "usage was {:?}",
        parsed.usage
    );

    let flags: Vec<&str> = parsed.options.iter().map(|o| o.flag.as_str()).collect();
    // The flags that make `uv init` worth offering at all. If uv renames one
    // this test says so, which is the point of reading help rather than
    // writing the list down.
    for expected in ["--name", "--lib", "--app", "--package", "--vcs", "--python"] {
        assert!(flags.contains(&expected), "no `{expected}` among {flags:?}");
    }
    assert!(!flags.contains(&"--help"), "--help offered as a field");

    let by = |flag: &str| parsed.options.iter().find(|o| o.flag == flag).unwrap();
    assert_eq!(by("--lib").kind, OptionKind::Bool);
    assert_eq!(by("--name").kind, OptionKind::Text);

    // `--vcs` is the live check that choices are read: it wraps across two
    // lines in uv's real output.
    let vcs = by("--vcs");
    assert_eq!(vcs.kind, OptionKind::Choice);
    let values: Vec<&str> = vcs.choices.iter().map(|c| c.value.as_str()).collect();
    assert!(values.contains(&"git"), "vcs choices were {values:?}");

    // Every option must carry a description a person can read, and none may
    // still be holding clap's bracketed notes.
    for option in &parsed.options {
        assert!(
            !option.description.is_empty(),
            "`{}` has no description",
            option.flag
        );
        for leak in ["[possible values", "[env:", "[default"] {
            assert!(
                !option.description.contains(leak),
                "`{}` still shows {leak}: {}",
                option.flag,
                option.description
            );
        }
    }

    // uv's own options are the form; its global machinery is not.
    assert_eq!(by("--name").section, None);
    assert!(
        by("--python").section.is_some(),
        "`--python` is in one of uv's named sections"
    );
}

#[test]
fn cargo_new_is_read_by_the_same_parser() {
    let Some(text) = help("cargo", &["new", "--help"]) else {
        eprintln!("SKIPPED: no `cargo` on this machine");
        return;
    };
    let parsed = parse(&text);
    assert!(parsed.is_help());
    let flags: Vec<&str> = parsed.options.iter().map(|o| o.flag.as_str()).collect();
    for expected in ["--lib", "--edition", "--name", "--vcs"] {
        assert!(flags.contains(&expected), "no `{expected}` among {flags:?}");
    }
    let edition = parsed
        .options
        .iter()
        .find(|o| o.flag == "--edition")
        .unwrap();
    assert_eq!(edition.kind, OptionKind::Choice);
    let years: Vec<&str> = edition.choices.iter().map(|c| c.value.as_str()).collect();
    assert!(years.contains(&"2024"), "editions were {years:?}");
}

/// A subcommand that does not exist must not read as a form.
#[test]
fn an_unknown_subcommand_is_not_a_help_page() {
    let Some(text) = help("uv", &["definitely-not-a-command", "--help"]) else {
        // The usual outcome: clap exits non-zero, which `help` reports as
        // None. That is already the refusal — there is nothing to parse.
        return;
    };
    assert!(
        !parse(&text).is_help(),
        "an unknown subcommand produced something that looked like a form"
    );
}
