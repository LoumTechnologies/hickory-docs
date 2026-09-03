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

/// One scaffolder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Toolchain {
    Dotnet,
    Uv,
    Cargo,
}

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
