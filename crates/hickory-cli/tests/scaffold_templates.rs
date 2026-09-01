//! Reading `dotnet new` well enough to draw a form.
//!
//! Protects
//! `docs/guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options.md`
//! and `docs/guarantees/authoring/a-new-project-writes-the-command-it-ran.md`.
//!
//! The fixtures are real `dotnet new … --help` output, captured from SDK
//! 10.0.111 on 2026-09-01 and checked in. A test that shells out to `dotnet`
//! would pass or fail on whether a machine happens to have the SDK, and would
//! silently stop testing the hard part — the hard part is the wrapping, and
//! the wrapping is in the bytes.

use hickory_cli::scaffold::{
    ChosenOption, OptionKind, ScaffoldSpec, dotnet_new_command, parse_template_help,
    parse_template_list, scaffold_document, sdk_image, suggested_output, suggested_path,
};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dotnet")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn the_listing_becomes_templates() {
    let templates = parse_template_list(&fixture("list.txt"));
    assert!(
        templates.len() > 20,
        "the fixture lists dozens of templates, got {}",
        templates.len()
    );

    let webapi = templates
        .iter()
        .find(|t| t.short_name() == "webapi")
        .expect("webapi is in the listing");
    assert_eq!(webapi.name, "ASP.NET Core Web API");
    assert_eq!(webapi.languages, vec!["C#", "F#"]);
    // The brackets in `[C#],F#` are the fact that matters: it says which
    // language you get without asking for one.
    assert_eq!(webapi.default_language.as_deref(), Some("C#"));
    assert_eq!(webapi.group(), "Web");

    // A row with two short names keeps both, and uses the first.
    let razor = templates
        .iter()
        .find(|t| t.short_names.contains(&"razor".to_string()))
        .expect("webapp,razor is in the listing");
    assert_eq!(razor.short_names, vec!["webapp", "razor"]);
    assert_eq!(razor.short_name(), "webapp");

    // A template with no language column at all is not given one.
    let gitignore = templates
        .iter()
        .find(|t| t.short_name() == "gitignore")
        .expect("the gitignore template is in the listing");
    assert!(gitignore.languages.is_empty());
    assert_eq!(gitignore.default_language, None);
}

#[test]
fn a_simple_template_becomes_fields() {
    let detail = parse_template_help(&fixture("help-console.txt"));
    assert_eq!(detail.title, "Console App (C#)");
    assert_eq!(detail.author, "Microsoft");
    assert!(detail.description.starts_with("A project for creating"));
    assert_eq!(detail.other_languages, vec!["F#", "VB"]);

    let framework = detail
        .options
        .iter()
        .find(|o| o.flag == "--framework")
        .expect("every project template has --framework");
    assert_eq!(framework.names, vec!["-f", "--framework"]);
    assert_eq!(framework.kind, OptionKind::Choice);
    assert_eq!(framework.default.as_deref(), Some("net10.0"));
    assert_eq!(framework.choices.len(), 1);
    assert_eq!(framework.choices[0].value, "net10.0");

    let no_restore = detail
        .options
        .iter()
        .find(|o| o.flag == "--no-restore")
        .expect("--no-restore is offered");
    assert_eq!(no_restore.kind, OptionKind::Bool);
    assert_eq!(no_restore.default.as_deref(), Some("false"));
    // The description wrapped across two lines and comes back as one
    // sentence, with exactly one space at the break.
    assert_eq!(
        no_restore.description,
        "If specified, skips the automatic restore of the project on create."
    );

    // An option with no default at all keeps `None` rather than an empty
    // string — "dotnet decides" and "the empty value" are different answers.
    let lang_version = detail
        .options
        .iter()
        .find(|o| o.flag == "--langVersion")
        .expect("--langVersion is offered");
    assert_eq!(lang_version.default, None);
    assert_eq!(lang_version.kind, OptionKind::Text);
}

#[test]
fn a_wrapped_default_comes_back_whole() {
    let detail = parse_template_help(&fixture("help-webapi.txt"));

    // The reason this parser exists. `dotnet` breaks this URL mid-token
    // across two lines; a naive rejoin puts a space inside it and hands the
    // user a value that is not what `dotnet` would use.
    let instance = detail
        .options
        .iter()
        .find(|o| o.flag == "--aad-b2c-instance")
        .expect("--aad-b2c-instance is offered");
    assert_eq!(
        instance.default.as_deref(),
        Some("https://qualified.domain.name.b2clogin.com/")
    );

    let aad = detail
        .options
        .iter()
        .find(|o| o.flag == "--aad-instance")
        .expect("--aad-instance is offered");
    assert_eq!(
        aad.default.as_deref(),
        Some("https://login.microsoftonline.com/")
    );

    // A name too long for its column continues on the next line, and the
    // option is still one option with both spellings.
    let susi = detail
        .options
        .iter()
        .find(|o| o.flag == "--susi-policy-id")
        .expect("--susi-policy-id is offered");
    assert_eq!(susi.names, vec!["-ssp", "--susi-policy-id"]);
    assert_eq!(susi.default.as_deref(), Some("b2c_1_susi"));
}

#[test]
fn choices_keep_their_own_descriptions() {
    let detail = parse_template_help(&fixture("help-webapi.txt"));
    let auth = detail
        .options
        .iter()
        .find(|o| o.flag == "--auth")
        .expect("--auth is offered");
    assert_eq!(auth.kind, OptionKind::Choice);
    let values: Vec<&str> = auth.choices.iter().map(|c| c.value.as_str()).collect();
    assert_eq!(
        values,
        vec!["None", "IndividualB2C", "SingleOrg", "Windows"]
    );
    assert_eq!(auth.default.as_deref(), Some("None"));
    // A choice description that wrapped back to the description column
    // belongs to that choice, not to the next one.
    assert_eq!(
        auth.choices[1].description,
        "Individual authentication with Azure AD B2C"
    );
}

#[test]
fn a_condition_is_carried_not_evaluated() {
    let detail = parse_template_help(&fixture("help-mstest.txt"));
    let profile = detail
        .options
        .iter()
        .find(|o| o.flag == "--extensions-profile")
        .expect("--extensions-profile is offered");
    assert_eq!(
        profile.enabled_if.as_deref(),
        Some("UseMSTestSdk && (TestRunner == Microsoft.Testing.Platform)")
    );

    // Choices whose descriptions wrap across three lines still line up with
    // the right values.
    let runner = detail
        .options
        .iter()
        .find(|o| o.flag == "--test-runner")
        .expect("--test-runner is offered");
    let values: Vec<&str> = runner.choices.iter().map(|c| c.value.as_str()).collect();
    assert_eq!(values, vec!["Microsoft.Testing.Platform", "VSTest"]);
    assert_eq!(runner.choices[1].description, "Use VSTest platform");
    assert_eq!(runner.default.as_deref(), Some("VSTest"));
}

#[test]
fn the_longest_spelling_is_the_one_written() {
    let detail = parse_template_help(&fixture("help-blazor.txt"));
    let interactivity = detail
        .options
        .iter()
        .find(|o| o.names.contains(&"-int".to_string()))
        .expect("-int is offered");
    // A command somebody reads a year later should not say `-int`.
    assert_eq!(interactivity.flag, "--interactivity");
    assert_eq!(interactivity.default.as_deref(), Some("Server"));
}

fn spec() -> ScaffoldSpec {
    ScaffoldSpec {
        template: "webapi".into(),
        title: "ASP.NET Core Web API".into(),
        language: Some("C#".into()),
        name: "Greeter".into(),
        output: "greeter".into(),
        image: "mcr.microsoft.com/dotnet/sdk:10.0".into(),
        options: vec![
            ChosenOption {
                flag: "--no-restore".into(),
                value: None,
            },
            ChosenOption {
                flag: "--auth".into(),
                value: Some("None".into()),
            },
        ],
    }
}

#[test]
fn the_command_is_the_one_a_person_would_type() {
    assert_eq!(
        dotnet_new_command(&spec()),
        "dotnet new webapi -o out -n Greeter --language 'C#' --no-restore --auth None"
    );
}

#[test]
fn a_name_that_needs_quoting_gets_it() {
    let mut spec = spec();
    spec.name = "My Project".into();
    spec.options = vec![];
    spec.language = None;
    assert_eq!(
        dotnet_new_command(&spec),
        "dotnet new webapi -o out -n 'My Project'"
    );

    spec.name = "it's".into();
    assert_eq!(
        dotnet_new_command(&spec),
        r"dotnet new webapi -o out -n 'it'\''s'"
    );
}

#[test]
fn the_document_parses_and_holds_the_command() {
    let source = scaffold_document(&spec());
    let doc = hick_lang::parse(&source).expect("a scaffold document parses");

    // The cell the ingest will aim at, by the id both sides agree on.
    assert!(
        source.contains(r#"<hick:copy id="scaffold">"#),
        "the command lives in an identified copy, not as loose text:\n{source}"
    );
    assert!(source.contains("dotnet new webapi -o out -n Greeter"));
    // `-o out` is the mount point; `output=` is where the tree lands.
    assert!(source.contains(r#"<hick:volume name="project" output="greeter" />"#));
    assert!(source.contains(r#"mount="project:out""#));
    // A bare document: no wrapper, because nothing here rebinds the prefix.
    assert!(!source.contains("<hick:doc"));
    assert_eq!(doc.prefix, "hick");
}

#[test]
fn defaults_are_derived_from_the_project_name() {
    assert_eq!(suggested_path("Greeter"), "greeter.hick");
    assert_eq!(suggested_output("Greeter"), "greeter");
    assert_eq!(
        suggested_path("Company.WebApplication1"),
        "company-webapplication1.hick"
    );
    assert_eq!(suggested_output("My Project"), "my-project");
    // Nothing usable in the name still has to produce a legal path.
    assert_eq!(suggested_path("///"), "project.hick");
}

#[test]
fn the_image_tag_follows_the_installed_sdk() {
    assert_eq!(sdk_image("10.0.111"), "mcr.microsoft.com/dotnet/sdk:10.0");
    assert_eq!(sdk_image("8.0.404"), "mcr.microsoft.com/dotnet/sdk:8.0");
    // A version this cannot read says "the newest one" rather than guessing
    // a major that may not have an image at all.
    assert_eq!(sdk_image("preview"), "mcr.microsoft.com/dotnet/sdk:latest");
}

#[test]
fn the_footer_ends_the_section() {
    // The footer's second line is an indented example command — `   dotnet
    // new winformslib -h --language VB` — sitting left of the description
    // column, where a continued option name lives. Reading it as one turned
    // `--nullable` into `--language`: not a cosmetic slip but a wrong flag in
    // a command the document would then run.
    let detail = parse_template_help(&fixture("help-winformslib.txt"));
    assert_eq!(detail.other_languages, vec!["VB"]);

    let last = detail.options.last().expect("winformslib has options");
    assert_eq!(last.flag, "--nullable");
    assert_eq!(last.names, vec!["--nullable"]);
    assert!(
        !detail.options.iter().any(|o| o.flag == "--language"),
        "--language is the CLI's own option, not a template option"
    );
}
