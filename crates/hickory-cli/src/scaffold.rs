//! `dotnet new` as a New Project dialog: what templates this machine has,
//! what each one asks for, and the document that runs the chosen one.
//!
//! ## Why this reads `--help` and not something structured
//!
//! `dotnet new` has no machine-readable listing. `--output json` is accepted
//! and ignored (measured on SDK 10.0.111, 2026-09-01: it prints the same
//! table). The template engine *does* keep a JSON cache at
//! `~/.templateengine/dotnetcli/<sdk>/templatecache.json` with every symbol,
//! its data type, its choices and its default — but the names in it are the
//! template's **symbol** names, not the CLI's option names, and the mapping
//! between the two lives in a `dotnetcli.host.json` inside a `.nupkg`. Passing
//! a symbol name straight through is rejected:
//!
//! ```text
//! $ dotnet new webapi --ExcludeLaunchSettings true
//! '--ExcludeLaunchSettings' is not a valid option
//! ```
//!
//! So the help text is not the convenient source, it is the only one that
//! names options a person can actually pass. It is also the contract `dotnet`
//! shows its own users, which makes it the right thing to be bound to: a
//! spelling that changes there has changed for everybody.
//!
//! ## The one trick the parser needs
//!
//! Help output is two columns, hard-wrapped to a width no environment
//! variable moves (`COLUMNS` is ignored). The second column carries prose,
//! `Type:`, `Default:`, and — indented one step further — the rows of a
//! choice. Rejoining a wrapped line looks ambiguous until you look at the
//! bytes: a line broken **at a space** keeps its trailing space, and a line
//! broken **mid-token** does not. So the join rule is the whole of it —
//! concatenate, and the source already says whether a space belongs there.
//! That one observation is what brings a wrapped default like
//! `https://qualified.domain.name.b2clogin.` + `com/` back exact instead of
//! gaining a space in the middle of a URL.
//!
//! ## What is deliberately not done
//!
//! `Enabled if: UseMSTestSdk && (TestRunner == …)` is carried through as the
//! sentence it is and shown beside the field. Evaluating it would mean
//! reimplementing the template engine's expression language inside a dialog,
//! and a form that greys out the wrong field is worse than one that says what
//! the condition is.

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context as _, Result, bail};

use serde::{Deserialize, Serialize};

/// `dotnet` is not on this machine's PATH.
///
/// A **type**, not a sentence, for the same reason `hick_dap::MissingAdapter`
/// is one: the New Project dialog answers this case with its own screen, and
/// a reworded message must not be able to take that screen away.
#[derive(Debug, Clone, Copy)]
pub struct NoDotnetSdk;

impl std::fmt::Display for NoDotnetSdk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "no `dotnet` on this machine's PATH, so there are no templates to offer.\n  \
             Next step: install the .NET SDK from https://dotnet.microsoft.com/download, \
             reopen this window so the app sees the new PATH, and try again.\n  \
             Common cause: an SDK installed after this app started — a process keeps the \
             PATH it was launched with.",
        )
    }
}

impl std::error::Error for NoDotnetSdk {}

// ---------------------------------------------------------------------------
// The catalogue
// ---------------------------------------------------------------------------

/// One row of `dotnet new list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Template {
    /// Every short name the row lists; the first is the one commands use.
    pub short_names: Vec<String>,
    /// The human name ("ASP.NET Core Web API").
    pub name: String,
    /// Every language the template can be instantiated in.
    pub languages: Vec<String>,
    /// The language `dotnet` picks when none is given — the one the listing
    /// brackets. `None` for a template that takes no `--language` at all
    /// (`gitignore`, `editorconfig`).
    pub default_language: Option<String>,
    /// The row's `Tags` column, split on `/`.
    pub tags: Vec<String>,
}

impl Template {
    /// The name a command uses.
    pub fn short_name(&self) -> &str {
        self.short_names.first().map(String::as_str).unwrap_or("")
    }

    /// The heading this template sits under in the dialog: its first tag,
    /// which is how `dotnet new list` itself groups them ("Web", "Common",
    /// "Test"). Templates with no tags land under "Other".
    pub fn group(&self) -> &str {
        self.tags.first().map(String::as_str).unwrap_or("Other")
    }
}

/// What this machine can scaffold.
#[derive(Debug, Clone, Serialize)]
pub struct Catalog {
    /// `dotnet --version`, so the dialog can say which SDK it is offering and
    /// the document can name a matching image.
    pub sdk_version: String,
    pub templates: Vec<Template>,
}

// ---------------------------------------------------------------------------
// One template's options
// ---------------------------------------------------------------------------

/// What kind of value an option takes, as `dotnet` reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OptionKind {
    Bool,
    Choice,
    Integer,
    Float,
    /// `string`, `text`, and anything a future SDK invents. The dialog draws
    /// a text field, which is right for all of them.
    Text,
}

/// One value of a `Type: choice` option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Choice {
    pub value: String,
    /// May be empty: not every choice is described.
    pub description: String,
}

/// One template option, as a form field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemplateOption {
    /// Every spelling, in the order help prints them (`["-au", "--auth"]`).
    pub names: Vec<String>,
    /// The spelling a generated command uses: the longest, because a command
    /// somebody reads a year later should not say `-uld`.
    pub flag: String,
    pub kind: OptionKind,
    pub choices: Vec<Choice>,
    /// What `dotnet` uses when the flag is absent. Absent itself for an
    /// option with no default at all (`--langVersion` on `console`).
    pub default: Option<String>,
    pub description: String,
    /// The template engine's own condition for this option mattering,
    /// verbatim. Shown, never evaluated.
    pub enabled_if: Option<String>,
}

/// `dotnet new <template> --help`, parsed.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TemplateDetail {
    /// The first line: `ASP.NET Core Web API (C#)`.
    pub title: String,
    pub author: String,
    pub description: String,
    pub options: Vec<TemplateOption>,
    /// The languages the footer says to ask for separately.
    pub other_languages: Vec<String>,
}

// ---------------------------------------------------------------------------
// Running dotnet
// ---------------------------------------------------------------------------

/// A `dotnet` invocation with the environment pinned.
///
/// `DOTNET_CLI_UI_LANGUAGE=en` is the load-bearing one: this parser reads a
/// table and the words `Type:` and `Default:`, and a machine set to another
/// locale prints them translated. The rest keep a template listing from
/// printing a first-run banner, phoning home, or checking for updates —
/// nothing in this product talks to anyone, and that includes the tools it
/// asks questions of.
fn dotnet(args: &[&str]) -> Command {
    let mut cmd = Command::new("dotnet");
    cmd.args(args)
        .env("DOTNET_CLI_UI_LANGUAGE", "en")
        .env("DOTNET_NOLOGO", "1")
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .env("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1");
    cmd
}

fn run(args: &[&str]) -> Result<String> {
    let output = match dotnet(args).output() {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => bail!(NoDotnetSdk),
        Err(e) => {
            return Err(e).with_context(|| format!("could not run `dotnet {}`", args.join(" ")));
        }
    };
    // `dotnet new <template> --help` exits non-zero for an unknown template
    // and prints why; the message is the useful part, so it is carried up
    // rather than replaced by an exit code.
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let said = if stderr.trim().is_empty() {
            stdout
        } else {
            stderr
        };
        bail!(
            "`dotnet {}` failed: {}",
            args.join(" "),
            said.trim().lines().take(6).collect::<Vec<_>>().join(" ")
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Every template this machine's SDK offers.
pub fn catalog() -> Result<Catalog> {
    let version = run(&["--version"])?.trim().to_string();
    let listing = run(&["new", "list"])?;
    Ok(Catalog {
        sdk_version: version,
        templates: parse_template_list(&listing),
    })
}

/// One template's options, for the language the dialog is asking about.
///
/// A template nobody has is the case worth naming: `dotnet new nope --help`
/// exits **zero** and prints "No templates or subcommands found matching" on
/// stdout, so the exit code cannot be trusted here. Without this check that
/// sentence came back as a template's *title*, and the dialog would have
/// drawn an empty form for a template that does not exist. A real help page
/// always has a `Usage:` line — including the templates that take no options
/// at all, which is why the presence of options cannot be the test.
pub fn template_detail(short_name: &str, language: Option<&str>) -> Result<TemplateDetail> {
    let mut args = vec!["new", short_name, "--help"];
    if let Some(language) = language {
        args.push("--language");
        args.push(language);
    }
    let text = run(&args)?;
    if !text.lines().any(|l| l.trim_start().starts_with("Usage:")) {
        bail!(
            "`dotnet` has no template `{short_name}`{}: {}\n  \
             Next step: pick one from the list — `dotnet new list` names every \
             template this machine's SDK has. A template from a package needs \
             `dotnet new install <package>` first.",
            language.map(|l| format!(" in {l}")).unwrap_or_default(),
            text.lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("no output")
                .trim()
        );
    }
    Ok(parse_template_help(&text))
}

// ---------------------------------------------------------------------------
// Parsing `dotnet new list`
// ---------------------------------------------------------------------------

/// Column spans, read off the rule line rather than off the header.
///
/// The header words are translated when the CLI is; the rule under them is
/// dashes and two-space gaps in every locale, and it is what actually marks
/// where each column starts. Reading the rule keeps the parser tied to the
/// layout instead of to English.
fn columns(rule: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    let chars: Vec<char> = rule.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        match (ch, start) {
            ('-', None) => start = Some(i),
            ('-', Some(_)) => {}
            (_, Some(s)) => {
                spans.push((s, i));
                start = None;
            }
            (_, None) => {}
        }
    }
    if let Some(s) = start {
        spans.push((s, chars.len()));
    }
    spans
}

/// Slice a row by a column span, in characters rather than bytes — a template
/// name is free to hold anything, and `dotnet` pads by display width.
fn cell(row: &[char], span: (usize, usize), last: bool) -> String {
    if span.0 >= row.len() {
        return String::new();
    }
    let end = if last {
        row.len()
    } else {
        span.1.min(row.len())
    };
    row[span.0..end.max(span.0)]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

/// The table `dotnet new list` prints.
///
/// Four columns in a fixed order — name, short name(s), language(s), tags —
/// and the last one runs to the end of the line, because a tag list is
/// allowed to be wider than the rule under it.
pub fn parse_template_list(text: &str) -> Vec<Template> {
    let lines: Vec<&str> = text.lines().collect();
    let Some(rule_at) = lines.iter().position(|l| {
        let t = l.trim();
        t.len() > 8 && t.chars().all(|c| c == '-' || c == ' ')
    }) else {
        return Vec::new();
    };
    let spans = columns(lines[rule_at]);
    if spans.len() < 4 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for line in &lines[rule_at + 1..] {
        if line.trim().is_empty() {
            continue;
        }
        let row: Vec<char> = line.chars().collect();
        let name = cell(&row, spans[0], false);
        let short = cell(&row, spans[1], false);
        let language = cell(&row, spans[2], false);
        let tags = cell(&row, spans[3], true);
        if name.is_empty() || short.is_empty() {
            continue;
        }

        // `[C#],F#,VB` — the brackets mark the one you get without asking.
        let mut languages = Vec::new();
        let mut default_language = None;
        for part in language.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let bare = part.trim_start_matches('[').trim_end_matches(']');
            if part.starts_with('[') {
                default_language = Some(bare.to_string());
            }
            languages.push(bare.to_string());
        }

        out.push(Template {
            short_names: short
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            name,
            languages,
            default_language,
            tags: tags
                .split('/')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Parsing `dotnet new <template> --help`
// ---------------------------------------------------------------------------

/// Where the description column begins, in characters: the first non-space
/// that follows a gutter of two or more spaces.
fn description_column(line: &str) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();
    let first = chars.iter().position(|c| !c.is_whitespace())?;
    let mut run = 0usize;
    for (i, c) in chars.iter().enumerate().skip(first) {
        if *c == ' ' {
            run += 1;
        } else {
            if run >= 2 {
                return Some(i);
            }
            run = 0;
        }
    }
    None
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}

/// One option under construction, before its fields are decided.
#[derive(Default)]
struct RawOption {
    names: String,
    description: String,
    type_name: String,
    default: String,
    enabled_if: String,
    /// `(value, description)`, descriptions still accumulating.
    choices: Vec<(String, String)>,
}

/// Which of the second column's fields the last line was feeding, so a
/// wrapped continuation goes back where it came from.
enum Sink {
    Description,
    Default,
    EnabledIf,
    Choice,
    Type,
}

pub fn parse_template_help(text: &str) -> TemplateDetail {
    let mut detail = TemplateDetail::default();
    let lines: Vec<&str> = text.lines().collect();

    for line in &lines {
        let t = line.trim();
        if t.starts_with("Usage:") {
            break;
        }
        if let Some(rest) = t.strip_prefix("Author:") {
            detail.author = rest.trim().to_string();
        } else if let Some(rest) = t.strip_prefix("Description:") {
            detail.description = rest.trim().to_string();
        } else if detail.title.is_empty() && !t.is_empty() {
            detail.title = t.to_string();
        }
    }

    let Some(start) = lines
        .iter()
        .position(|l| l.trim_end() == "Template options:")
    else {
        return detail;
    };

    let mut raw: Vec<RawOption> = Vec::new();
    let mut desc_col: Option<usize> = None;
    let mut sink = Sink::Description;

    for line in &lines[start + 1..] {
        if line.trim().is_empty() {
            continue;
        }
        let indent = indent_of(line);
        // Back to the left margin: the "To see help for other template
        // languages (F#, VB)" footer, which ENDS the section — and stopping
        // is the point, not tidiness. Its second line is an indented example
        // command (`   dotnet new winformslib -h --language VB`), which sits
        // left of the description column and would otherwise be read as more
        // names for the last option: `--nullable` came back spelled
        // `--language`, which is a wrong flag in a generated command rather
        // than a cosmetic slip.
        if indent == 0 {
            detail.other_languages.extend(other_languages(line));
            break;
        }

        let col = desc_col.unwrap_or(usize::MAX);
        let content = line.trim();

        if indent < col {
            // The names column. A line starting with `-` opens an option;
            // anything else is the rest of a name that did not fit
            // (`-ssp, --susi-policy-id` / `<susi-policy-id>`).
            if content.starts_with('-') {
                raw.push(RawOption::default());
                if desc_col.is_none() {
                    desc_col = description_column(line);
                }
            }
            let Some(option) = raw.last_mut() else {
                continue;
            };
            let here = description_column(line).unwrap_or(line.chars().count());
            let names: String = line.chars().take(here).collect();
            if !option.names.is_empty() {
                option.names.push(' ');
            }
            option.names.push_str(names.trim());
            // The same line usually carries the start of the description.
            let rest: String = line.chars().skip(here).collect();
            if !rest.trim().is_empty() {
                option.description.push_str(&rest);
                sink = Sink::Description;
            }
            continue;
        }

        let Some(option) = raw.last_mut() else {
            continue;
        };

        if indent > col {
            // Deeper than the description column: a choice row, `value  what
            // it means`, either of which may then wrap back to `col`.
            //
            // Sliced from the indent rather than trimmed: a trailing space is
            // the wrap marker, and trimming it here would glue the next line
            // onto this one — "Use VSTest" + "platform".
            let row: String = line.chars().skip(indent).collect();
            let (value, description) = match row.split_once("  ") {
                Some((v, d)) => (v.trim().to_string(), d.trim_start().to_string()),
                None => (row.trim().to_string(), String::new()),
            };
            option.choices.push((value, description));
            sink = Sink::Choice;
            continue;
        }

        // Exactly at the description column: a key, or a continuation.
        let text_here: String = line.chars().skip(col).collect();
        if let Some(rest) = text_here.strip_prefix("Type:") {
            option.type_name = rest.trim().to_string();
            sink = Sink::Type;
        } else if let Some(rest) = text_here.strip_prefix("Default:") {
            // `Default:` with nothing after it is a value that did not fit on
            // the line; the next line is it.
            option.default.clear();
            option.default.push_str(rest.trim_start());
            sink = Sink::Default;
        } else if let Some(rest) = text_here.strip_prefix("Enabled if:") {
            option.enabled_if.push_str(rest.trim_start());
            sink = Sink::EnabledIf;
        } else {
            match sink {
                Sink::Description => option.description.push_str(&text_here),
                Sink::Default => option.default.push_str(&text_here),
                Sink::EnabledIf => option.enabled_if.push_str(&text_here),
                Sink::Choice => {
                    if let Some(last) = option.choices.last_mut() {
                        last.1.push_str(&text_here);
                    }
                }
                // `Type:` never wraps — the word is one token. A line after
                // it that is not a key belongs to the description before it.
                Sink::Type => option.description.push_str(&text_here),
            }
        }
    }

    detail.options = raw.into_iter().filter_map(finish_option).collect();
    detail
}

/// `To see help for other template languages (F#, VB), use --language option:`
fn other_languages(line: &str) -> Vec<String> {
    let Some(open) = line.find("template languages (") else {
        return Vec::new();
    };
    let rest = &line[open + "template languages (".len()..];
    let Some(close) = rest.find(')') else {
        return Vec::new();
    };
    rest[..close]
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn finish_option(raw: RawOption) -> Option<TemplateOption> {
    // `-au, --auth <choice>` — the value placeholder is not a name.
    let names: Vec<String> = raw
        .names
        .split(',')
        .flat_map(str::split_whitespace)
        .map(str::trim)
        .filter(|s| s.starts_with('-'))
        .map(str::to_string)
        .collect();
    let flag = names.iter().max_by_key(|n| n.len())?.clone();

    let kind = match raw.type_name.trim().to_ascii_lowercase().as_str() {
        "bool" => OptionKind::Bool,
        "choice" => OptionKind::Choice,
        "integer" | "int" => OptionKind::Integer,
        "float" => OptionKind::Float,
        _ => OptionKind::Text,
    };

    Some(TemplateOption {
        names,
        flag,
        kind,
        choices: raw
            .choices
            .into_iter()
            .map(|(value, description)| Choice {
                value,
                description: collapse(&description),
            })
            .collect(),
        default: match raw.default.trim() {
            "" => None,
            value => Some(value.to_string()),
        },
        description: collapse(&raw.description),
        enabled_if: match collapse(&raw.enabled_if) {
            s if s.is_empty() => None,
            s => Some(s),
        },
    })
}

/// Squeeze the runs of spaces a re-joined wrap leaves behind. Applied to
/// prose only — never to a default, whose bytes are the answer.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// The document
// ---------------------------------------------------------------------------

/// One `--flag value` the dialog decided on.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChosenOption {
    pub flag: String,
    /// `None` for a bare switch (`--no-restore`).
    #[serde(default)]
    pub value: Option<String>,
}

/// What the New Project dialog asked for.
#[derive(Debug, Clone, Deserialize)]
pub struct ScaffoldSpec {
    /// The template's short name (`webapi`).
    pub template: String,
    /// Its human name, for the document's prose. Optional: a spec built by
    /// hand need not carry it.
    #[serde(default)]
    pub title: String,
    /// `--language`, when the template takes one.
    #[serde(default)]
    pub language: Option<String>,
    /// `-n`: the project's name, which is also its root namespace.
    pub name: String,
    /// The directory the scaffold lands in, relative to the repository.
    pub output: String,
    /// The SDK image a containerised replay would pull, recorded in the
    /// recipe. The scaffold itself runs on this machine's own `dotnet`.
    pub image: String,
    #[serde(default)]
    pub options: Vec<ChosenOption>,
}

/// Quote an argument for the POSIX shell a cell's command runs under.
///
/// A project name is a .NET root namespace and will almost never need this.
/// It is here because "almost never" is the case that reaches a user as a
/// document whose cell does not run.
fn shell_quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=+@,".contains(c));
    if plain {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

/// The `dotnet new` line the recipe records — run from the repository's
/// root, writing into `-o <output>`.
///
/// This is the `Hick-Recipe` trailer, spelled the way a person would type
/// it at the root of the checkout, so a replay is the same line run in the
/// same place. Only what was changed from the template's defaults is on it.
pub fn dotnet_new_command(spec: &ScaffoldSpec) -> String {
    let output = spec.output.trim().trim_end_matches('/');
    let mut parts = vec![
        "dotnet".to_string(),
        "new".to_string(),
        shell_quote(&spec.template),
        "-o".to_string(),
        shell_quote(if output.is_empty() { "." } else { output }),
        "-n".to_string(),
        shell_quote(&spec.name),
    ];
    if let Some(language) = spec.language.as_deref().filter(|l| !l.is_empty()) {
        parts.push("--language".to_string());
        parts.push(shell_quote(language));
    }
    for option in &spec.options {
        parts.push(shell_quote(&option.flag));
        if let Some(value) = &option.value {
            parts.push(shell_quote(value));
        }
    }
    parts.join(" ")
}

/// The `dotnet new` arguments, unquoted, for running rather than recording.
fn dotnet_new_args<'a>(spec: &'a ScaffoldSpec, into: &'a str) -> Vec<&'a str> {
    let mut args = vec![
        "new",
        spec.template.as_str(),
        "-o",
        into,
        "-n",
        spec.name.as_str(),
    ];
    if let Some(language) = spec.language.as_deref().filter(|l| !l.is_empty()) {
        args.push("--language");
        args.push(language);
    }
    for option in &spec.options {
        args.push(option.flag.as_str());
        if let Some(value) = &option.value {
            args.push(value.as_str());
        }
    }
    args
}

/// The scaffolder's whole command line, `dotnet` included, for a terminal
/// session to run in a scratch directory.
///
/// The scaffold is not run by a subprocess whose output nobody sees: it runs
/// in a terminal, where a person reads `dotnet`'s own words. So what this
/// answers is an argv, not a result. `into` is a *name* rather than a path,
/// because the session's working directory is the scratch directory — which
/// also makes the line on screen the same line the recipe records.
/// See `crates/hickory-cli/src/serve/scaffold.rs`.
pub fn dotnet_new_argv(spec: &ScaffoldSpec, into: &str) -> Vec<String> {
    let mut argv = vec!["dotnet".to_string()];
    argv.extend(dotnet_new_args(spec, into).into_iter().map(str::to_string));
    argv
}

/// Whether this machine has a `dotnet` at all, and which one.
///
/// Asked before a terminal is opened, so "no SDK" stays the typed refusal it
/// has always been rather than arriving as a shell's "command not found" in
/// a terminal nobody asked for.
pub fn require_sdk() -> Result<String> {
    Ok(run(&["--version"])?.trim().to_string())
}

/// The commit message a scaffold is recorded under: prose a person reads,
/// then the trailers a replay reads.
///
/// `output_tree` is the git tree hash of `output/` as the scaffolder wrote
/// it — `Hick-Output` — which is what lets the history lens tell a commit
/// that is exactly the scaffold from one that was edited before it was
/// committed, with one `git rev-parse` and no replay.
pub fn commit_message(spec: &ScaffoldSpec, output_tree: &str, output: &str) -> String {
    let what = if spec.title.is_empty() {
        format!("`dotnet new {}`", spec.template)
    } else {
        format!("{} (`dotnet new {}`)", spec.title, spec.template)
    };
    format!(
        "Scaffold {name} with `dotnet new {template}`\n\n\
         {what}, scaffolded into `{output}/`. Every byte in this commit is the \
         scaffolder's; nothing was edited before it was committed. To take a newer \
         SDK's scaffold, replay this commit and move the commits after it onto the \
         result.\n\n\
         Hick-Recipe: {command}\n\
         Hick-Image: {image}\n\
         Hick-Output: {output_tree} {output}\n",
        name = spec.name,
        template = spec.template,
        what = what,
        output = output,
        command = dotnet_new_command(spec),
        image = spec.image,
        output_tree = output_tree,
    )
}

/// The SDK image that matches an installed `dotnet --version`.
///
/// `10.0.111` is SDK 10.0, whose image tag is `10.0`. A version this cannot
/// read falls back to `latest`, which is honest — the tag is a hint for a
/// container executor, and guessing a major version that does not exist would
/// be worse than saying "the newest one".
pub fn sdk_image(version: &str) -> String {
    let mut parts = version.split('.');
    match (parts.next(), parts.next()) {
        (Some(major), Some(minor))
            if !major.is_empty()
                && major.chars().all(|c| c.is_ascii_digit())
                && !minor.is_empty()
                && minor.chars().all(|c| c.is_ascii_digit()) =>
        {
            format!("mcr.microsoft.com/dotnet/sdk:{major}.{minor}")
        }
        _ => "mcr.microsoft.com/dotnet/sdk:latest".to_string(),
    }
}

/// A default output directory for a project name.
pub fn suggested_output(name: &str) -> String {
    slug(name)
}

/// Lowercase, and any run of non-word characters becomes one hyphen.
/// `Company.WebApplication1` -> `company-webapplication1`.
fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut hyphen = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.extend(ch.to_lowercase());
            hyphen = false;
        } else if !out.is_empty() && !hyphen {
            out.push('-');
            hyphen = true;
        }
    }
    let trimmed = out.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        "project".to_string()
    } else {
        trimmed
    }
}

/// Group templates for the dialog's list. Within a group the catalogue's own
/// order is kept: `dotnet new list` is alphabetical by name, which is as good
/// an order as any and the one a person who has used the CLI already knows.
pub fn grouped(templates: &[Template]) -> BTreeMap<String, Vec<&Template>> {
    let mut out: BTreeMap<String, Vec<&Template>> = BTreeMap::new();
    for template in templates {
        out.entry(template.group().to_string())
            .or_default()
            .push(template);
    }
    out
}
