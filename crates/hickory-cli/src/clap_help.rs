//! `--help` from a clap-built CLI, parsed into the same form fields
//! `dotnet new --help` produces.
//!
//! # Why parse help at all
//!
//! For the same reason `scaffold.rs` does, and the reason is worth repeating
//! because it looks like the wrong choice every time: **the form fields are
//! the scaffolder's own**. A hand-written list of `uv init`'s flags is a list
//! beside the thing it describes, which is the bug this repository has now
//! shipped six times. Reading `uv init --help` means a flag added by a newer
//! uv appears in the dialog without anyone here noticing, and a flag removed
//! stops being offered.
//!
//! # Why one parser for several tools
//!
//! `uv`, `cargo`, and most modern Rust and Python CLIs are clap-built, and
//! clap's help has one shape. So this is not "a uv parser" — it is a clap
//! parser, and adding `cargo new` after `uv init` costs a table row rather
//! than a module.
//!
//! # The two layouts
//!
//! clap prints an option one of two ways depending on how wide the flags are,
//! and a single `uv init --help` contains both:
//!
//! ```text
//!       --name <NAME>          The name of the project        <- aligned
//!   -q, --quiet...
//!           Use quiet output                                  <- wrapped
//! ```
//!
//! Reading only the aligned form loses every option in uv's own "Global
//! options" section; reading only the wrapped form loses all of `cargo new`.
//! The test is simply whether anything follows the flags on their own line.

use crate::scaffold::{Choice, OptionKind, TemplateOption};

/// One `--help` page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClapHelp {
    /// The sentence above `Usage:` — clap prints the command's `about`.
    pub summary: String,
    /// The `Usage:` line, verbatim, without its label.
    pub usage: String,
    pub options: Vec<TemplateOption>,
}

impl ClapHelp {
    /// Whether this looks like a real help page rather than an error.
    ///
    /// The same trap `dotnet new <unknown> --help` set: a tool asked about
    /// something it does not have may still exit zero. A `Usage:` line is
    /// what says the command exists.
    pub fn is_help(&self) -> bool {
        !self.usage.is_empty()
    }
}

/// A section heading, or `None` for clap's own unnamed `Options:`.
///
/// Carried rather than filtered because the judgment is the dialog's, not
/// the parser's: `uv init`'s own options are the form, and its "Cache
/// options" and "Global options" are real but belong behind a disclosure. A
/// parser that dropped them would make that decision permanently, and for
/// every tool at once.
fn section_name(heading: &str) -> Option<String> {
    let name = heading.trim_end_matches(':').trim();
    if name.eq_ignore_ascii_case("options") {
        None
    } else {
        Some(name.to_string())
    }
}

/// Whether a line is a section heading: unindented, ending in a colon.
fn heading_of(line: &str) -> Option<&str> {
    if line.starts_with(char::is_whitespace) || line.trim().is_empty() {
        return None;
    }
    line.trim_end().strip_suffix(':')
}

/// Whether a line begins an option: indented, and starting with a dash.
fn starts_option(line: &str) -> bool {
    let indent = line.len() - line.trim_start().len();
    indent > 0 && line.trim_start().starts_with('-')
}

/// Split a flags column into its names and its value placeholder.
///
/// `-p, --python <PYTHON>` → `(["-p", "--python"], Some("PYTHON"))`.
/// `-q, --quiet...` → `(["-q", "--quiet"], None)` — the `...` is clap saying
/// the flag repeats, not part of its name.
fn split_flags(text: &str) -> (Vec<String>, Option<String>) {
    let mut names = Vec::new();
    let mut placeholder = None;
    for token in text.split([',', ' ']).filter(|token| !token.is_empty()) {
        if let Some(inner) = token
            .strip_prefix('<')
            .and_then(|rest| rest.strip_suffix('>'))
            .or_else(|| {
                token
                    .strip_prefix('[')
                    .and_then(|rest| rest.strip_suffix(']'))
            })
        {
            placeholder = Some(inner.to_string());
        } else if token.starts_with('-') {
            names.push(token.trim_end_matches('.').to_string());
        }
    }
    (names, placeholder)
}

/// Pull a bracketed clause out of a description, returning what it held.
///
/// clap appends these to the description text, so they have to come off
/// before the description is shown: `[possible values: git, none]`,
/// `[default: auto]`, `[env: UV_PYTHON=]`.
fn take_clause(text: &mut String, label: &str) -> Option<String> {
    let needle = format!("[{label}:");
    let start = text.find(&needle)?;
    let end = text[start..].find(']')? + start;
    let value = text[start + needle.len()..end].trim().to_string();
    text.replace_range(start..=end, "");
    Some(value)
}

/// One option under construction, before its description has finished
/// arriving — a clap description can span any number of continuation lines.
struct RawOption {
    names: Vec<String>,
    /// `<NAME>` from the flags column; `None` for a switch.
    placeholder: Option<String>,
    /// The description, still being joined, still holding clap's brackets.
    raw: String,
    section: Option<String>,
}

/// Everything after the flags, folded into one line and cleaned of clap's
/// bracketed notes.
struct Description {
    text: String,
    choices: Vec<Choice>,
    default: Option<String>,
}

fn describe(raw: &str) -> Description {
    let mut text = raw.trim().to_string();
    let possible = take_clause(&mut text, "possible values");
    let mut default = take_clause(&mut text, "default");
    // clap's environment note is real and is not a form field: an env var
    // set on the machine running the scaffolder is not something the dialog
    // can offer, and showing `[env: UV_PYTHON=]` beside a text box explains
    // nothing to the person reading it.
    take_clause(&mut text, "env");
    // `[default]` with no value — cargo marks `--bin` this way — means "this
    // is what you get if you say nothing", which for a switch is `true`.
    if text.contains("[default]") {
        text = text.replace("[default]", "");
        default.get_or_insert_with(|| "true".to_string());
    }
    let choices = possible
        .map(|values| {
            values
                .split(',')
                .map(|value| Choice {
                    value: value.trim().to_string(),
                    description: String::new(),
                })
                .filter(|choice| !choice.value.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Description {
        text: collapse(&text),
        choices,
        default,
    }
}

/// Squeeze the runs of spaces a re-joined wrap leaves behind.
///
/// Applied to prose only. clap wraps at word boundaries, so a single space
/// is always the right join — unlike `dotnet`, whose table can break
/// mid-token and where the bytes decide.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parse a clap `--help` page.
pub fn parse(text: &str) -> ClapHelp {
    let mut help = ClapHelp::default();
    let mut section: Option<String> = None;
    let mut in_options = false;
    // The option being built, and the indent its flags started at, so a
    // continuation can be told from the next option.
    let mut pending: Option<RawOption> = None;

    let lines: Vec<&str> = text.lines().collect();
    let mut summary: Vec<&str> = Vec::new();

    let flush = |pending: &mut Option<RawOption>, options: &mut Vec<TemplateOption>| {
        let Some(RawOption {
            names,
            placeholder,
            raw,
            section,
        }) = pending.take()
        else {
            return;
        };
        if names.is_empty() {
            return;
        }
        // Help itself is not a form field.
        if names.iter().any(|name| name == "--help" || name == "-h") {
            return;
        }
        let described = describe(&raw);
        let kind = if placeholder.is_none() {
            OptionKind::Bool
        } else if !described.choices.is_empty() {
            OptionKind::Choice
        } else {
            OptionKind::Text
        };
        // The longest spelling, for the same reason `dotnet`'s parser picks
        // it: a command read a year later should not say `-p`.
        let flag = names
            .iter()
            .max_by_key(|name| name.len())
            .cloned()
            .unwrap_or_default();
        options.push(TemplateOption {
            names,
            flag,
            kind,
            choices: described.choices,
            default: described.default,
            description: described.text,
            enabled_if: None,
            section,
        });
    };

    for line in lines {
        if let Some(rest) = line.trim_start().strip_prefix("Usage:") {
            help.usage = rest.trim().to_string();
            in_options = false;
            flush(&mut pending, &mut help.options);
            continue;
        }
        if let Some(heading) = heading_of(line) {
            flush(&mut pending, &mut help.options);
            // `Arguments:` is positional and has no flag to pass, so it is
            // not a form field; everything else is a group of options.
            let name = heading.trim();
            in_options = !name.eq_ignore_ascii_case("arguments");
            section = section_name(heading);
            continue;
        }
        if !in_options {
            // Above `Usage:`, clap prints the command's own description.
            if help.usage.is_empty() && !line.trim().is_empty() {
                summary.push(line.trim());
            }
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if starts_option(line) {
            flush(&mut pending, &mut help.options);
            let trimmed = line.trim_start();
            // The description begins after two or more spaces, when it is on
            // this line at all — the aligned layout. Otherwise the flags
            // stand alone and the description is on the lines below.
            let (flags, rest) = match trimmed.find("  ") {
                Some(gap) => (&trimmed[..gap], trimmed[gap..].trim()),
                None => (trimmed, ""),
            };
            let (names, placeholder) = split_flags(flags);
            pending = Some(RawOption {
                names,
                placeholder,
                raw: rest.to_string(),
                section: section.clone(),
            });
        } else if let Some(option) = pending.as_mut() {
            // A continuation, in either layout.
            if !option.raw.is_empty() {
                option.raw.push(' ');
            }
            option.raw.push_str(line.trim());
        }
    }
    flush(&mut pending, &mut help.options);
    help.summary = collapse(&summary.join(" "));
    help
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `uv init --help`, uv 0.11.7, abridged to one option of each shape.
    const UV: &str = r#"Create a new project

Usage: uv init [OPTIONS] [PATH]

Arguments:
  [PATH]  The path to use for the project/script

Options:
      --name <NAME>                    The name of the project
      --bare                           Only create a `pyproject.toml`
      --app                            Create a project for an application
      --vcs <VCS>                      Initialize a version control system for the project [possible
                                       values: git, none]
      --author-from <AUTHOR_FROM>      Fill in the `authors` field in the `pyproject.toml` [possible
                                       values: auto, git, none]

Python options:
  -p, --python <PYTHON>      The Python interpreter to use to determine the minimum supported Python
                             version. [env: UV_PYTHON=]

Global options:
  -q, --quiet...
          Use quiet output
      --color <COLOR_CHOICE>
          Control the use of color in output [possible values: auto, always, never]
  -h, --help
          Display the concise help for this command
"#;

    /// `cargo new --help`, which is aligned throughout and marks a default
    /// with a bare `[default]`.
    const CARGO: &str = r#"Create a new cargo package at <path>

Usage: cargo new [OPTIONS] <PATH>

Arguments:
  <PATH>  

Options:
      --vcs <VCS>                Initialize a new repository for the given version control system,
                                 overriding a global configuration. [possible values: git, hg,
                                 pijul, fossil, none]
      --bin                      Use a binary (application) template [default]
      --lib                      Use a library template
      --edition <YEAR>           Edition to set for the crate generated [possible values: 2015,
                                 2018, 2021, 2024]
      --name <NAME>              Set the resulting package name, defaults to the directory name
  -h, --help                     Print help

Manifest Options:
      --locked   Assert that `Cargo.lock` will remain unchanged
"#;

    fn option<'a>(help: &'a ClapHelp, flag: &str) -> &'a TemplateOption {
        help.options
            .iter()
            .find(|option| option.flag == flag)
            .unwrap_or_else(|| panic!("no `{flag}` among {:?}", flags(help)))
    }

    fn flags(help: &ClapHelp) -> Vec<&str> {
        help.options.iter().map(|o| o.flag.as_str()).collect()
    }

    #[test]
    fn reads_the_summary_and_usage() {
        let help = parse(UV);
        assert!(help.is_help());
        assert_eq!(help.summary, "Create a new project");
        assert_eq!(help.usage, "uv init [OPTIONS] [PATH]");
    }

    #[test]
    fn a_flag_with_no_value_is_a_switch() {
        let help = parse(UV);
        let bare = option(&help, "--bare");
        assert_eq!(bare.kind, OptionKind::Bool);
        assert_eq!(bare.description, "Only create a `pyproject.toml`");
        assert!(bare.choices.is_empty());
    }

    #[test]
    fn a_flag_with_a_placeholder_takes_text() {
        let help = parse(UV);
        assert_eq!(option(&help, "--name").kind, OptionKind::Text);
    }

    /// The clause wraps across two lines in the real output, which is the
    /// whole reason continuations are joined before it is read.
    #[test]
    fn possible_values_become_choices_even_when_they_wrap() {
        let help = parse(UV);
        let vcs = option(&help, "--vcs");
        assert_eq!(vcs.kind, OptionKind::Choice);
        assert_eq!(
            vcs.choices
                .iter()
                .map(|c| c.value.as_str())
                .collect::<Vec<_>>(),
            ["git", "none"]
        );
        // And the clause is taken OFF the description rather than shown.
        assert_eq!(
            vcs.description,
            "Initialize a version control system for the project"
        );
    }

    #[test]
    fn the_wrapped_layout_is_read_too() {
        // uv prints its global options with the description on the next
        // line. Reading only the aligned form loses every one of them.
        let help = parse(UV);
        let color = option(&help, "--color");
        assert_eq!(color.kind, OptionKind::Choice);
        assert_eq!(color.description, "Control the use of color in output");
        assert_eq!(option(&help, "--quiet").kind, OptionKind::Bool);
        // `...` is clap saying the flag repeats, not part of its name.
        assert_eq!(option(&help, "--quiet").names, ["-q", "--quiet"]);
    }

    #[test]
    fn every_option_says_which_section_it_came_from() {
        // The dialog decides what to show; the parser only reports.
        let help = parse(UV);
        assert_eq!(option(&help, "--name").section, None);
        assert_eq!(
            option(&help, "--python").section.as_deref(),
            Some("Python options")
        );
        assert_eq!(
            option(&help, "--color").section.as_deref(),
            Some("Global options")
        );
    }

    #[test]
    fn help_itself_is_not_a_form_field() {
        for text in [UV, CARGO] {
            let help = parse(text);
            assert!(
                !flags(&help).contains(&"--help"),
                "--help offered as a field"
            );
        }
    }

    #[test]
    fn a_positional_argument_is_not_a_field() {
        // `Arguments:` has nothing to pass a flag for. Reading it as options
        // put `[PATH]` in the form with no way to spell it.
        let help = parse(UV);
        assert!(!flags(&help).iter().any(|flag| flag.contains("PATH")));
    }

    #[test]
    fn an_environment_note_is_not_shown_beside_a_text_box() {
        let help = parse(UV);
        let python = option(&help, "--python");
        assert!(
            !python.description.contains("env:"),
            "{}",
            python.description
        );
        assert!(python.description.starts_with("The Python interpreter"));
    }

    #[test]
    fn cargo_is_the_same_parser() {
        let help = parse(CARGO);
        assert_eq!(help.usage, "cargo new [OPTIONS] <PATH>");
        assert_eq!(option(&help, "--lib").kind, OptionKind::Bool);
        let edition = option(&help, "--edition");
        assert_eq!(
            edition
                .choices
                .iter()
                .map(|c| c.value.as_str())
                .collect::<Vec<_>>(),
            ["2015", "2018", "2021", "2024"]
        );
        // A bare `[default]` means "this is what you get if you say nothing".
        assert_eq!(option(&help, "--bin").default.as_deref(), Some("true"));
        assert_eq!(option(&help, "--lib").default, None);
        assert_eq!(
            option(&help, "--locked").section.as_deref(),
            Some("Manifest Options")
        );
    }

    #[test]
    fn the_longest_spelling_is_the_one_a_command_uses() {
        let help = parse(UV);
        assert_eq!(option(&help, "--python").names, ["-p", "--python"]);
        assert_eq!(option(&help, "--python").flag, "--python");
    }

    /// A tool asked about a subcommand it does not have may still exit zero,
    /// exactly as `dotnet new <unknown> --help` does. The `Usage:` line is
    /// what says the command exists.
    #[test]
    fn something_that_is_not_a_help_page_is_recognised() {
        assert!(!parse("error: unrecognized subcommand 'nope'").is_help());
        assert!(!parse("").is_help());
    }
}
