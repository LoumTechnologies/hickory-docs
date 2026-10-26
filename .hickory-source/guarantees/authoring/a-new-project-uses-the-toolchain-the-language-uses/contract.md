<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A New Project Uses The Toolchain The Language Uses

For: Authors editing executable documents and the files those documents produce.

Given a machine with more than one scaffolder installed, when File → New
Project is opened, then it offers each one — `.NET`, `Python (uv)`,
`Rust (cargo)` — and every entry in that picker is a tool this machine
actually has, measured by running it, never a list written down. A machine
with one scaffolder draws no picker and looks exactly as it did.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-new-project-uses-the-toolchain-the-language-uses.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-3e906396f1f4"></a>
## `crates/hickory-cli/src/clap_help.rs`

Related contracts: [A New Project Uses The Toolchain The Language Uses](contract.md).

<a id="f-3e906396f1f4-ClapHelp"></a>
### `ClapHelp`

`--help` from a clap-built CLI, parsed into the same form fields
`dotnet new --help` produces.
# Why parse help at all
For the same reason `scaffold.rs` does, and the reason is worth repeating
because it looks like the wrong choice every time: **the form fields are
the scaffolder's own**. A hand-written list of `uv init`'s flags is a list
beside the thing it describes, which is the bug this repository has now
shipped six times. Reading `uv init --help` means a flag added by a newer
uv appears in the dialog without anyone here noticing, and a flag removed
stops being offered.
# Why one parser for several tools
`uv`, `cargo`, and most modern Rust and Python CLIs are clap-built, and
clap's help has one shape. So this is not "a uv parser" — it is a clap
parser, and adding `cargo new` after `uv init` costs a table row rather
than a module.
# The two layouts
clap prints an option one of two ways depending on how wide the flags are,
and a single `uv init --help` contains both:
\```text
--name <NAME>          The name of the project        <- aligned
-q, --quiet...
Use quiet output                                  <- wrapped
\```
Reading only the aligned form loses every option in uv's own "Global
options" section; reading only the wrapped form loses all of `cargo new`.
The test is simply whether anything follows the flags on their own line.

<storycode:copy id="f-3e906396f1f4-ClapHelp">
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

</storycode:copy>
<a id="f-3e906396f1f4-impl-ClapHelp"></a>
### `impl-ClapHelp`

<storycode:copy id="f-3e906396f1f4-impl-ClapHelp">
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

</storycode:copy>
<a id="f-3e906396f1f4-section_name"></a>
### `section_name`

A section heading, or `None` for clap's own unnamed `Options:`.
Carried rather than filtered because the judgment is the dialog's, not
the parser's: `uv init`'s own options are the form, and its "Cache
options" and "Global options" are real but belong behind a disclosure. A
parser that dropped them would make that decision permanently, and for
every tool at once.

<storycode:copy id="f-3e906396f1f4-section_name">
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

</storycode:copy>
<a id="f-3e906396f1f4-heading_of"></a>
### `heading_of`

Whether a line is a section heading: unindented, ending in a colon.

<storycode:copy id="f-3e906396f1f4-heading_of">
/// Whether a line is a section heading: unindented, ending in a colon.
fn heading_of(line: &str) -> Option<&str> {
    if line.starts_with(char::is_whitespace) || line.trim().is_empty() {
        return None;
    }
    line.trim_end().strip_suffix(':')
}

</storycode:copy>
<a id="f-3e906396f1f4-starts_option"></a>
### `starts_option`

Whether a line begins an option: indented, and starting with a dash.

<storycode:copy id="f-3e906396f1f4-starts_option">
/// Whether a line begins an option: indented, and starting with a dash.
fn starts_option(line: &str) -> bool {
    let indent = line.len() - line.trim_start().len();
    indent > 0 && line.trim_start().starts_with('-')
}

</storycode:copy>
<a id="f-3e906396f1f4-split_flags"></a>
### `split_flags`

Split a flags column into its names and its value placeholder.
`-p, --python <PYTHON>` → `(["-p", "--python"], Some("PYTHON"))`.
`-q, --quiet...` → `(["-q", "--quiet"], None)` — the `...` is clap saying
the flag repeats, not part of its name.

<storycode:copy id="f-3e906396f1f4-split_flags">
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

</storycode:copy>
<a id="f-3e906396f1f4-take_clause"></a>
### `take_clause`

Pull a bracketed clause out of a description, returning what it held.
clap appends these to the description text, so they have to come off
before the description is shown: `[possible values: git, none]`,
`[default: auto]`, `[env: UV_PYTHON=]`.

<storycode:copy id="f-3e906396f1f4-take_clause">
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

</storycode:copy>
<a id="f-3e906396f1f4-RawOption"></a>
### `RawOption`

One option under construction, before its description has finished
arriving — a clap description can span any number of continuation lines.

<storycode:copy id="f-3e906396f1f4-RawOption">
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

</storycode:copy>
<a id="f-3e906396f1f4-Description"></a>
### `Description`

Everything after the flags, folded into one line and cleaned of clap's
bracketed notes.

<storycode:copy id="f-3e906396f1f4-Description">
/// Everything after the flags, folded into one line and cleaned of clap's
/// bracketed notes.
struct Description {
    text: String,
    choices: Vec<Choice>,
    default: Option<String>,
}

</storycode:copy>
<a id="f-3e906396f1f4-describe"></a>
### `describe`

<storycode:copy id="f-3e906396f1f4-describe">
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

</storycode:copy>
<a id="f-3e906396f1f4-collapse"></a>
### `collapse`

Squeeze the runs of spaces a re-joined wrap leaves behind.
Applied to prose only. clap wraps at word boundaries, so a single space
is always the right join — unlike `dotnet`, whose table can break
mid-token and where the bytes decide.

<storycode:copy id="f-3e906396f1f4-collapse">
/// Squeeze the runs of spaces a re-joined wrap leaves behind.
///
/// Applied to prose only. clap wraps at word boundaries, so a single space
/// is always the right join — unlike `dotnet`, whose table can break
/// mid-token and where the bytes decide.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

</storycode:copy>
<a id="f-3e906396f1f4-parse"></a>
### `parse`

<storycode:copy id="f-3e906396f1f4-parse">
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

</storycode:copy>
<a id="f-3e906396f1f4-tests"></a>
### `tests`

<storycode:copy id="f-3e906396f1f4-tests">
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/clap_help.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/clap_help.rs"><storycode:paste select="#f-3e906396f1f4-ClapHelp"/><storycode:paste select="#f-3e906396f1f4-impl-ClapHelp"/><storycode:paste select="#f-3e906396f1f4-section_name"/><storycode:paste select="#f-3e906396f1f4-heading_of"/><storycode:paste select="#f-3e906396f1f4-starts_option"/><storycode:paste select="#f-3e906396f1f4-split_flags"/><storycode:paste select="#f-3e906396f1f4-take_clause"/><storycode:paste select="#f-3e906396f1f4-RawOption"/><storycode:paste select="#f-3e906396f1f4-Description"/><storycode:paste select="#f-3e906396f1f4-describe"/><storycode:paste select="#f-3e906396f1f4-collapse"/><storycode:paste select="#f-3e906396f1f4-parse"/><storycode:paste select="#f-3e906396f1f4-tests"/></storycode:file>

<a id="output-fdab1d9c56f7"></a>
## `crates/hickory-cli/src/toolchain.rs`

Related contracts: [A New Project Uses The Toolchain The Language Uses](contract.md).

<a id="f-fdab1d9c56f7-OutputArg"></a>
### `OutputArg`

Which scaffolders this machine can start a project with.
File → New Project was `dotnet new` and nothing else. The rule it was
built on generalises perfectly — **the templates and the form fields are
the scaffolder's own**, read from the tool rather than written down beside
it — but the reading was `dotnet`'s table, so no other tool could be
offered at all. A Python project today starts with `uv`, and a Rust one
with `cargo`, and neither was reachable.
# What differs between scaffolders, and what does not
Almost everything downstream is already general: `scaffold_commit` takes a
command string and an image string and knows nothing about .NET, and the
recipe trailers, the temporary index, and the history lens's
`output_matches` check are the same for every tool. What differs is small
and is exactly this table:
* **How many templates there are.** `dotnet new` has a catalogue and a
template argument. `uv init` and `cargo new` are one command each, whose
shape is chosen by flags — `--lib`, `--app`, `--package`. So a
single-shape tool is offered as a catalogue of one, rather than as a
second kind of thing for the dialog to understand.
* **Where the output path goes.** `dotnet new -o <path>`; `uv init <path>`
and `cargo new <path>` take it positionally.
* **How the name is spelled.** `-n` against `--name`.
* **Which help parser reads it.** `dotnet`'s own table, or clap's.
# On images
`Hick-Image` records what a containerised replay would pull. It is a
**record of what this scaffold was made with**, never something this
product goes and runs — the scaffold itself always runs on the machine's
own tool. So the version is read from the tool that actually ran.

<storycode:copy id="f-fdab1d9c56f7-OutputArg">
//! Which scaffolders this machine can start a project with.
//!
//! File → New Project was `dotnet new` and nothing else. The rule it was
//! built on generalises perfectly — **the templates and the form fields are
//! the scaffolder's own**, read from the tool rather than written down beside
//! it — but the reading was `dotnet`'s table, so no other tool could be
//! offered at all. A Python project today starts with `uv`, and a Rust one
//! with `cargo`, and neither was reachable.
//!
//! # What differs between scaffolders, and what does not
//!
//! Almost everything downstream is already general: `scaffold_commit` takes a
//! command string and an image string and knows nothing about .NET, and the
//! recipe trailers, the temporary index, and the history lens's
//! `output_matches` check are the same for every tool. What differs is small
//! and is exactly this table:
//!
//! * **How many templates there are.** `dotnet new` has a catalogue and a
//!   template argument. `uv init` and `cargo new` are one command each, whose
//!   shape is chosen by flags — `--lib`, `--app`, `--package`. So a
//!   single-shape tool is offered as a catalogue of one, rather than as a
//!   second kind of thing for the dialog to understand.
//! * **Where the output path goes.** `dotnet new -o <path>`; `uv init <path>`
//!   and `cargo new <path>` take it positionally.
//! * **How the name is spelled.** `-n` against `--name`.
//! * **Which help parser reads it.** `dotnet`'s own table, or clap's.
//!
//! # On images
//!
//! `Hick-Image` records what a containerised replay would pull. It is a
//! **record of what this scaffold was made with**, never something this
//! product goes and runs — the scaffold itself always runs on the machine's
//! own tool. So the version is read from the tool that actually ran.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::clap_help;
use crate::scaffold::{self, Catalog, ScaffoldSpec, Template, TemplateDetail, shell_quote};

/// Where a scaffolder wants the directory it is writing into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputArg {
    /// `-o <path>`.
    Flag(&'static str),
    /// The first positional argument.
    Positional,
}

</storycode:copy>
<a id="f-fdab1d9c56f7-Toolchain"></a>
### `Toolchain`

<storycode:copy id="f-fdab1d9c56f7-Toolchain">
/// One scaffolder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Toolchain {
    Dotnet,
    Uv,
    Cargo,
}

</storycode:copy>
<a id="f-fdab1d9c56f7-Shape"></a>
### `Shape`

Everything that differs between one scaffolder and the next.

<storycode:copy id="f-fdab1d9c56f7-Shape">
/// Everything that differs between one scaffolder and the next.
pub struct Shape {
    pub id: &'static str,
    /// What the dialog calls it.
    pub label: &'static str,
    /// The language a project made with it is written in, for the listing.
    pub language: &'static str,
    pub program: &'static str,
    /// `new` for dotnet and cargo, `init` for uv.
    pub subcommand: &'static str,
    /// Whether the tool takes a template name after its subcommand.
    pub templated: bool,
    pub output: OutputArg,
    pub name_flag: &'static str,
    /// The image a replay would pull, with `{version}` substituted.
    pub image: &'static str,
    /// Arguments this scaffolder is always given, and the options it is
    /// therefore never offered.
    ///
    /// `uv init` and `cargo new` both make a git repository **inside** the
    /// project unless told not to. That is right for their usual use and
    /// wrong for every use here: a scaffold made through this dialog is
    /// committed to whichever repository contains its location, as one act,
    /// through a temporary index — so a `.git` of the scaffolder's own would
    /// nest a repository inside that one. The dialog already asks about
    /// version control at the level where the question belongs, with a
    /// checkbox that means *see to it that there is a repository*, and two
    /// answers to one question is one too many.
    ///
    /// It goes on the recorded recipe as well as the run, because a replay
    /// that omitted it would produce a different tree and the history lens
    /// would report drift that is not there.
    pub forced: &'static [(&'static str, &'static str)],
}

</storycode:copy>
<a id="f-fdab1d9c56f7-impl-Toolchain"></a>
### `impl-Toolchain`

<storycode:copy id="f-fdab1d9c56f7-impl-Toolchain">
impl Toolchain {
    pub const ALL: [Toolchain; 3] = [Toolchain::Dotnet, Toolchain::Uv, Toolchain::Cargo];

    pub fn shape(self) -> Shape {
        match self {
            Toolchain::Dotnet => Shape {
                id: "dotnet",
                label: ".NET",
                language: "C#",
                program: "dotnet",
                subcommand: "new",
                templated: true,
                output: OutputArg::Flag("-o"),
                name_flag: "-n",
                image: "mcr.microsoft.com/dotnet/sdk:{version}",
                // `dotnet new` has never made a repository of its own.
                forced: &[],
            },
            Toolchain::Uv => Shape {
                id: "uv",
                label: "Python (uv)",
                language: "Python",
                program: "uv",
                subcommand: "init",
                templated: false,
                output: OutputArg::Positional,
                name_flag: "--name",
                image: "ghcr.io/astral-sh/uv:{version}",
                forced: &[("--vcs", "none")],
            },
            Toolchain::Cargo => Shape {
                id: "cargo",
                label: "Rust (cargo)",
                language: "Rust",
                program: "cargo",
                subcommand: "new",
                templated: false,
                output: OutputArg::Positional,
                name_flag: "--name",
                image: "rust:{version}",
                forced: &[("--vcs", "none")],
            },
        }
    }

    pub fn id(self) -> &'static str {
        self.shape().id
    }

    /// Parse the id the dialog sends back.
    pub fn parse(id: &str) -> Option<Toolchain> {
        Toolchain::ALL
            .into_iter()
            .find(|toolchain| toolchain.id() == id)
    }

    /// The tool's version, and therefore whether it is here at all.
    ///
    /// Asked before a terminal is opened, so "not installed" stays a typed
    /// refusal rather than arriving as a shell's "command not found" in a
    /// terminal nobody asked for.
    pub fn version(self) -> Result<String> {
        if self == Toolchain::Dotnet {
            // `dotnet --version` is already pinned to a quiet, English,
            // offline environment by `scaffold::require_sdk`, and that
            // environment is load-bearing for its table parser.
            return scaffold::require_sdk();
        }
        let shape = self.shape();
        let output = std::process::Command::new(shape.program)
            .arg("--version")
            .output()
            .with_context(|| format!("`{}` is not on this machine's PATH", shape.program))?;
        anyhow::ensure!(
            output.status.success(),
            "`{} --version` failed",
            shape.program
        );
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// The version number alone, out of whatever the tool prints around it.
    ///
    /// `uv 0.11.7 (x86_64-unknown-linux-gnu)` and `cargo 1.96.1 (…)` both
    /// carry more than the number, and an image tag must be the number.
    pub fn version_number(version: &str) -> Option<String> {
        version
            .split_whitespace()
            .find(|word| {
                word.split('.').count() >= 2
                    && word
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
                    && word.starts_with(|c: char| c.is_ascii_digit())
            })
            .map(str::to_string)
    }

    /// The image a replay of this scaffold would pull.
    pub fn image(self, version: &str) -> String {
        if self == Toolchain::Dotnet {
            return scaffold::sdk_image(version);
        }
        let shape = self.shape();
        match Toolchain::version_number(version) {
            Some(number) => shape.image.replace("{version}", &number),
            None => shape.image.replace("{version}", "latest"),
        }
    }

    /// What this scaffolder offers.
    ///
    /// A tool with no template catalogue is offered as a catalogue of one:
    /// its shape is chosen by flags, and those flags are its form.
    pub fn catalog(self) -> Result<Catalog> {
        if self == Toolchain::Dotnet {
            return scaffold::catalog();
        }
        let shape = self.shape();
        let version = self.version()?;
        Ok(Catalog {
            sdk_version: version,
            templates: vec![Template {
                short_names: vec![shape.subcommand.to_string()],
                name: format!("{} project", shape.language),
                languages: vec![shape.language.to_string()],
                default_language: Some(shape.language.to_string()),
                tags: vec![shape.language.to_string()],
            }],
        })
    }

    /// One template's options, as form fields.
    pub fn detail(self, template: &str, language: Option<&str>) -> Result<TemplateDetail> {
        if self == Toolchain::Dotnet {
            return scaffold::template_detail(template, language);
        }
        let shape = self.shape();
        let output = std::process::Command::new(shape.program)
            .args([shape.subcommand, "--help"])
            .output()
            .with_context(|| format!("`{}` is not on this machine's PATH", shape.program))?;
        let text = String::from_utf8_lossy(&output.stdout);
        let help = clap_help::parse(&text);
        anyhow::ensure!(
            help.is_help(),
            "`{} {} --help` did not print a usage line, so its options could not be read",
            shape.program,
            shape.subcommand
        );
        Ok(TemplateDetail {
            title: format!("{} ({} {})", shape.label, shape.program, shape.subcommand),
            author: String::new(),
            description: help.summary,
            // An option this scaffolder is always given is not a question.
            // Offering `--vcs` beside a checkbox that already decides
            // version control would let a person answer it twice, and the
            // two answers would reach the tool as two `--vcs` flags.
            options: help
                .options
                .into_iter()
                .filter(|option| !shape.forced.iter().any(|(flag, _)| option.flag == *flag))
                .collect(),
            other_languages: Vec::new(),
        })
    }

    /// The argv a scaffold actually runs, writing into `into`.
    ///
    /// `into` is a leaf name inside the scratch directory the run happens in,
    /// never the person's own folder — the commit is made from what the
    /// scaffolder wrote, as one act.
    pub fn argv(self, spec: &ScaffoldSpec, into: &str) -> Vec<String> {
        if self == Toolchain::Dotnet {
            return scaffold::dotnet_new_argv(spec, into);
        }
        let shape = self.shape();
        let mut argv = vec![shape.program.to_string(), shape.subcommand.to_string()];
        if shape.templated && !spec.template.is_empty() {
            argv.push(spec.template.clone());
        }
        match shape.output {
            OutputArg::Flag(flag) => {
                argv.push(flag.to_string());
                argv.push(into.to_string());
            }
            OutputArg::Positional => argv.push(into.to_string()),
        }
        if !spec.name.trim().is_empty() {
            argv.push(shape.name_flag.to_string());
            argv.push(spec.name.clone());
        }
        for (flag, value) in shape.forced {
            argv.push((*flag).to_string());
            argv.push((*value).to_string());
        }
        for option in &spec.options {
            // A spec built by hand, or by an older client, could still carry
            // one of these. Hick's answer wins: the alternative is a scaffold
            // that makes a repository inside the one it is committed to.
            if shape.forced.iter().any(|(flag, _)| option.flag == *flag) {
                continue;
            }
            argv.push(option.flag.clone());
            if let Some(value) = &option.value {
                argv.push(value.clone());
            }
        }
        argv
    }

    /// The line the recipe records, spelled the way a person would type it at
    /// the root of the checkout — which is where a replay runs it.
    pub fn command(self, spec: &ScaffoldSpec) -> String {
        if self == Toolchain::Dotnet {
            return scaffold::dotnet_new_command(spec);
        }
        let output = spec.output.trim().trim_end_matches('/');
        let output = if output.is_empty() { "." } else { output };
        self.argv(spec, output)
            .iter()
            .map(|part| shell_quote(part))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// How a person says what this scaffolder was asked to make.
    ///
    /// `dotnet new console`; `uv init`; `cargo new`. A single-shape tool has
    /// no template to name, and naming its subcommand twice would read as a
    /// mistake.
    pub fn phrase(self, template: &str) -> String {
        let shape = self.shape();
        if shape.templated && !template.trim().is_empty() {
            format!("{} {} {}", shape.program, shape.subcommand, template)
        } else {
            format!("{} {}", shape.program, shape.subcommand)
        }
    }

    /// The commit message a scaffold is recorded under: prose a person
    /// reads, then the trailers a replay reads.
    ///
    /// `output_tree` is the git tree hash of `output/` as the scaffolder
    /// wrote it — `Hick-Output` — which is what lets the history lens tell a
    /// commit that is exactly the scaffold from one that was edited before it
    /// was committed, with one `git rev-parse` and no replay.
    pub fn commit_message(self, spec: &ScaffoldSpec, output_tree: &str, output: &str) -> String {
        if self == Toolchain::Dotnet {
            // Byte for byte what it has always been. A commit already in
            // somebody's history carries this wording, and the history lens
            // reads the trailers out of it.
            return scaffold::commit_message(spec, output_tree, output);
        }
        let phrase = self.phrase(&spec.template);
        let what = if spec.title.is_empty() {
            format!("`{phrase}`")
        } else {
            format!("{} (`{phrase}`)", spec.title)
        };
        format!(
            "Scaffold {name} with `{phrase}`\n\n\
             {what}, scaffolded into `{output}/`. Every byte in this commit is the \
             scaffolder's; nothing was edited before it was committed. To take a newer \
             toolchain's scaffold, replay this commit and move the commits after it onto \
             the result.\n\n\
             Hick-Recipe: {command}\n\
             Hick-Image: {image}\n\
             Hick-Output: {output_tree} {output}\n",
            name = spec.name,
            phrase = phrase,
            what = what,
            output = output,
            command = self.command(spec),
            image = spec.image,
            output_tree = output_tree,
        )
    }

    /// Every scaffolder this machine actually has, with its version.
    ///
    /// Measured, never declared — the same rule `hick lang` follows. A dialog
    /// offering `uv` on a machine without it is a dialog that fails after the
    /// person has filled in a form.
    pub fn installed() -> Vec<(Toolchain, String)> {
        Toolchain::ALL
            .into_iter()
            .filter_map(|toolchain| Some((toolchain, toolchain.version().ok()?)))
            .collect()
    }
}

</storycode:copy>
<a id="f-fdab1d9c56f7-tests"></a>
### `tests`

<storycode:copy id="f-fdab1d9c56f7-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::ChosenOption;

    fn spec(options: Vec<ChosenOption>) -> ScaffoldSpec {
        ScaffoldSpec {
            template: "init".into(),
            title: String::new(),
            language: None,
            name: "orders".into(),
            output: "services/orders".into(),
            image: String::new(),
            options,
        }
    }

    #[test]
    fn every_id_round_trips() {
        for toolchain in Toolchain::ALL {
            assert_eq!(Toolchain::parse(toolchain.id()), Some(toolchain));
        }
        assert_eq!(Toolchain::parse("nope"), None);
    }

    #[test]
    fn uv_takes_its_path_positionally() {
        // The difference that matters: `uv init -o x` is not a thing.
        let command = Toolchain::Uv.command(&spec(vec![ChosenOption {
            flag: "--lib".into(),
            value: None,
        }]));
        assert_eq!(
            command,
            "uv init services/orders --name orders --vcs none --lib"
        );
    }

    #[test]
    fn cargo_is_the_same_shape_with_a_different_word() {
        let command = Toolchain::Cargo.command(&spec(vec![ChosenOption {
            flag: "--edition".into(),
            value: Some("2024".into()),
        }]));
        assert_eq!(
            command,
            "cargo new services/orders --name orders --vcs none --edition 2024"
        );
    }

    #[test]
    fn dotnet_keeps_the_line_it_always_had() {
        // The existing recipe spelling is a published format: a commit
        // already in somebody's history carries it, and a replay has to run
        // the same line.
        let mut spec = spec(vec![]);
        spec.template = "console".into();
        spec.language = Some("C#".into());
        let command = Toolchain::Dotnet.command(&spec);
        assert!(
            command.starts_with("dotnet new console -o services/orders -n orders"),
            "{command}"
        );
    }

    #[test]
    fn an_output_that_is_the_root_is_spelled_as_one() {
        let mut spec = spec(vec![]);
        spec.output = String::new();
        assert_eq!(
            Toolchain::Uv.command(&spec),
            "uv init . --name orders --vcs none"
        );
    }

    #[test]
    fn a_name_that_needs_quoting_gets_it() {
        let mut spec = spec(vec![]);
        spec.name = "my project".into();
        assert_eq!(
            Toolchain::Uv.command(&spec),
            "uv init services/orders --name 'my project' --vcs none"
        );
    }

    #[test]
    fn a_version_number_is_found_in_what_the_tool_prints_around_it() {
        assert_eq!(
            Toolchain::version_number("uv 0.11.7 (x86_64-unknown-linux-gnu)").as_deref(),
            Some("0.11.7")
        );
        assert_eq!(
            Toolchain::version_number("cargo 1.96.1 (abc 2026-01-01)").as_deref(),
            Some("1.96.1")
        );
        assert_eq!(Toolchain::version_number("nothing numeric here"), None);
    }

    #[test]
    fn an_image_records_the_version_that_ran() {
        assert_eq!(
            Toolchain::Uv.image("uv 0.11.7 (x86_64-unknown-linux-gnu)"),
            "ghcr.io/astral-sh/uv:0.11.7"
        );
        assert_eq!(Toolchain::Cargo.image("cargo 1.96.1 (x)"), "rust:1.96.1");
        // Unreadable rather than wrong: `latest` says "we could not tell",
        // and a made-up tag would say something false.
        assert_eq!(Toolchain::Uv.image("???"), "ghcr.io/astral-sh/uv:latest");
        // dotnet keeps its own two-component rule.
        assert_eq!(
            Toolchain::Dotnet.image("10.0.100"),
            "mcr.microsoft.com/dotnet/sdk:10.0"
        );
    }

    /// The defect this rule exists for: both tools make a repository inside
    /// the project unless told not to, and the scaffold is committed to the
    /// repository that CONTAINS it.
    #[test]
    fn a_scaffolder_never_makes_a_repository_of_its_own() {
        for toolchain in [Toolchain::Uv, Toolchain::Cargo] {
            let command = toolchain.command(&spec(vec![]));
            assert!(
                command.contains("--vcs none"),
                "{} would nest a repository: {command}",
                toolchain.id()
            );
        }
    }

    #[test]
    fn a_chosen_option_cannot_override_that() {
        // Two `--vcs` flags is what a client sending its own would produce.
        let command = Toolchain::Uv.command(&spec(vec![ChosenOption {
            flag: "--vcs".into(),
            value: Some("git".into()),
        }]));
        assert_eq!(command.matches("--vcs").count(), 1, "{command}");
        assert!(command.contains("--vcs none"), "{command}");
    }

    #[test]
    fn an_option_hick_owns_is_not_offered_as_a_question() {
        let Ok(detail) = Toolchain::Uv.detail("init", None) else {
            eprintln!("SKIPPED: no `uv` on this machine");
            return;
        };
        let flags: Vec<&str> = detail.options.iter().map(|o| o.flag.as_str()).collect();
        assert!(!flags.contains(&"--vcs"), "--vcs offered: {flags:?}");
        // And the rest of the form survived the filter.
        assert!(flags.contains(&"--lib"));
    }

    #[test]
    fn a_commit_message_names_the_tool_that_made_it() {
        let mut spec = spec(vec![]);
        spec.image = "ghcr.io/astral-sh/uv:0.11.7".into();
        let message = Toolchain::Uv.commit_message(&spec, "abc123", "services/orders");
        assert!(
            message.starts_with("Scaffold orders with `uv init`"),
            "{message}"
        );
        assert!(message.contains("Hick-Recipe: uv init services/orders --name orders --vcs none"));
        assert!(message.contains("Hick-Image: ghcr.io/astral-sh/uv:0.11.7"));
        assert!(message.contains("Hick-Output: abc123 services/orders"));
        // A single-shape tool has no template to name twice.
        assert!(!message.contains("uv init init"), "{message}");
    }

    #[test]
    fn dotnets_message_is_byte_for_byte_what_it_was() {
        let mut spec = spec(vec![]);
        spec.template = "console".into();
        spec.title = "Console App".into();
        spec.image = "mcr.microsoft.com/dotnet/sdk:10.0".into();
        assert_eq!(
            Toolchain::Dotnet.commit_message(&spec, "abc", "services/orders"),
            scaffold::commit_message(&spec, "abc", "services/orders"),
        );
    }

    #[test]
    fn a_single_shape_tool_is_a_catalogue_of_one() {
        let Ok(catalog) = Toolchain::Uv.catalog() else {
            eprintln!("SKIPPED: no `uv` on this machine");
            return;
        };
        assert_eq!(catalog.templates.len(), 1);
        assert_eq!(catalog.templates[0].short_name(), "init");
        assert_eq!(catalog.templates[0].languages, ["Python"]);
    }

    #[test]
    fn uvs_own_flags_are_the_form() {
        let Ok(detail) = Toolchain::Uv.detail("init", None) else {
            eprintln!("SKIPPED: no `uv` on this machine");
            return;
        };
        let flags: Vec<&str> = detail.options.iter().map(|o| o.flag.as_str()).collect();
        for expected in ["--lib", "--app", "--package", "--name"] {
            assert!(flags.contains(&expected), "no `{expected}` among {flags:?}");
        }
        assert_eq!(detail.description, "Create a new project");
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/toolchain.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/toolchain.rs"><storycode:paste select="#f-fdab1d9c56f7-OutputArg"/><storycode:paste select="#f-fdab1d9c56f7-Toolchain"/><storycode:paste select="#f-fdab1d9c56f7-Shape"/><storycode:paste select="#f-fdab1d9c56f7-impl-Toolchain"/><storycode:paste select="#f-fdab1d9c56f7-tests"/></storycode:file>

## Test the behavior: `has`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_scaffold.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-5a76df8fd4d1-has"></a>
### `has`

A second scaffolder
Protects docs/guarantees/authoring/a-new-project-uses-the-toolchain-the-language-uses.md

<storycode:copy id="f-5a76df8fd4d1-has">
// ---------------------------------------------------------------------------
// A second scaffolder
//
// Protects docs/guarantees/authoring/a-new-project-uses-the-toolchain-the-language-uses.md
// ---------------------------------------------------------------------------

fn has(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/NewProjectDialog.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-65e48889b856)
- [crates/hickory-cli/src/clap_help.rs](contract.md#output-3e906396f1f4)
- [crates/hickory-cli/src/scaffold_commit.rs](../../../mechanisms/crates/hickory-cli/src/scaffold_commit.rs.md#output-b1965dc85e6b)
- [crates/hickory-cli/src/serve/scaffold.rs](../../../mechanisms/crates/hickory-cli/src/serve/scaffold.rs.md#output-3fd865a33211)
- [crates/hickory-cli/src/toolchain.rs](contract.md#output-fdab1d9c56f7)

### Tests and fixtures

- [apps/web/src/components/NewProjectDialog.test.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-065ce188ea68)
- [crates/hickory-cli/tests/clap_help_live.rs](../a-new-project-reads-the-scaffolder-s-own-options/contract.md#output-41bf7e2555df)
- [crates/hickory-cli/tests/serve_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/serve_scaffold.rs.md#output-5a76df8fd4d1)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
