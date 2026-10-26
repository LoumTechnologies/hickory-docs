<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# New Project Offers The Scaffolder's Own Options, Not A List This Product Maintains

For: Authors editing executable documents and the files those documents produce.

Given a machine with a .NET SDK, when File → New Project is opened, then the
templates offered are exactly what `dotnet new list` reports on that machine,
and the fields shown for a chosen template are exactly the options `dotnet new
<template> --help` names — including templates from packages this product has
never heard of.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-41bf7e2555df"></a>
## `crates/hickory-cli/tests/clap_help_live.rs`

Related contracts: [New Project Offers The Scaffolder's Own Options, Not A List This Product Maintains](contract.md), [A New Project Uses The Toolchain The Language Uses](../a-new-project-uses-the-toolchain-the-language-uses/contract.md).

<a id="f-41bf7e2555df-help"></a>
### `help`

The clap help parser, against the real tools on this machine.
Protects docs/guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options.md
The unit tests in `clap_help.rs` read captured fixtures, which is what
makes them stable — and is exactly why they cannot notice that clap
changed its layout in a version nobody here has pinned. This runs the
parser over whatever `uv` and `cargo` actually print today.
Skipped loudly and separately for each tool, because "skipped" and
"passed" must never look alike.

<storycode:copy id="f-41bf7e2555df-help">
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

</storycode:copy>
<a id="f-41bf7e2555df-uv_init_is_read_as_a_form"></a>
### `uv_init_is_read_as_a_form`

<storycode:copy id="f-41bf7e2555df-uv_init_is_read_as_a_form">
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

</storycode:copy>
<a id="f-41bf7e2555df-cargo_new_is_read_by_the_same_parser"></a>
### `cargo_new_is_read_by_the_same_parser`

<storycode:copy id="f-41bf7e2555df-cargo_new_is_read_by_the_same_parser">
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

</storycode:copy>
<a id="f-41bf7e2555df-an_unknown_subcommand_is_not_a_help_page"></a>
### `an_unknown_subcommand_is_not_a_help_page`

A subcommand that does not exist must not read as a form.

<storycode:copy id="f-41bf7e2555df-an_unknown_subcommand_is_not_a_help_page">
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
</storycode:copy>

### Assemble `crates/hickory-cli/tests/clap_help_live.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/clap_help_live.rs"><storycode:paste select="#f-41bf7e2555df-help"/><storycode:paste select="#f-41bf7e2555df-uv_init_is_read_as_a_form"/><storycode:paste select="#f-41bf7e2555df-cargo_new_is_read_by_the_same_parser"/><storycode:paste select="#f-41bf7e2555df-an_unknown_subcommand_is_not_a_help_page"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/NewProjectDialog.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-65e48889b856)
- [crates/hick-plugins/src/scaffold.rs](../../../mechanisms/crates/hick-plugins/src/scaffold.rs.md#output-6b3ddf926d91)
- [crates/hickory-cli/src/scaffold.rs](../../../mechanisms/crates/hickory-cli/src/scaffold.rs.md#output-a3d176caef4d)
- [crates/hickory-cli/src/serve/scaffold.rs](../../../mechanisms/crates/hickory-cli/src/serve/scaffold.rs.md#output-3fd865a33211)

### Tests and fixtures

- [apps/web/src/components/NewProjectDialog.test.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-065ce188ea68)
- [crates/hickory-cli/tests/clap_help_live.rs](contract.md#output-41bf7e2555df)
- [crates/hickory-cli/tests/scaffold_templates.rs](../../../mechanisms/crates/hickory-cli/tests/scaffold_templates.rs.md#output-65595a9b8013)
- [crates/hickory-cli/tests/serve_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/serve_scaffold.rs.md#output-5a76df8fd4d1)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
