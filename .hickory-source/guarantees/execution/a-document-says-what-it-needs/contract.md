<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Document Says What It Needs

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a `<hick:container>` carrying `<hick:needs bin="…" />` declarations,
when the document runs, then every declared program is looked for **before
any cell executes**, and a document missing one stops immediately — naming
every missing program at once, with the line each was declared on and the
reason its author gave.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-document-says-what-it-needs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-43ab0b0251a2"></a>
## `crates/hick-literate/src/needs.rs`

Related contracts: [A Document Says What It Needs](contract.md).

<a id="f-43ab0b0251a2-Need"></a>
### `Need`

`<hick:needs>` — what a container's cells expect to find installed.
## The problem
A cell runs against whatever your machine happens to have. `image=` is
recorded and ignored by the local and sandbox executors, so a document
that uses `duckdb` works on the machine it was written on and fails on
everyone else's with `sh: 1: duckdb: not found` — a message from a shell,
about a program, that says nothing about the document that needed it.
## What this is, and what it is deliberately not
It is a **declaration and a check**, not a package manager:
\```xml
<hick:container name="lab" image="python:3.12">
<hick:needs bin="duckdb" />
<hick:needs bin="python3" />
</hick:container>
\```
Nothing is downloaded, nothing is installed, and no network is touched.
Every declared tool is looked for BEFORE any cell runs, and a document
missing one fails immediately, naming every tool it could not find at
once — rather than failing on the first, being fixed, and failing on the
second.
Installing them is the reader's job, on purpose. Resolving tools for
people means either becoming a package manager or depending on one, and
this product would then own the difference between what it installed and
what the reader's own tooling installs. Saying precisely what is missing
is the part that has to exist; the rest is theirs.
## Why the check goes through the executor
Because "installed" means "visible to the cell", and those differ. Under
the sandbox a cell has an empty `$HOME` with only toolchain directories
bound back, so a binary in `~/Desktop/tools` exists on the machine and
not in the cell. Under Docker it is the image's contents that matter, not
the host's at all. Probing through the executor asks the only question
worth asking: can the thing that will run the cell see this?

<storycode:copy id="f-43ab0b0251a2-Need">
//! `<hick:needs>` — what a container's cells expect to find installed.
//!
//! ## The problem
//!
//! A cell runs against whatever your machine happens to have. `image=` is
//! recorded and ignored by the local and sandbox executors, so a document
//! that uses `duckdb` works on the machine it was written on and fails on
//! everyone else's with `sh: 1: duckdb: not found` — a message from a shell,
//! about a program, that says nothing about the document that needed it.
//!
//! ## What this is, and what it is deliberately not
//!
//! It is a **declaration and a check**, not a package manager:
//!
//! ```xml
//! <hick:container name="lab" image="python:3.12">
//!   <hick:needs bin="duckdb" />
//!   <hick:needs bin="python3" />
//! </hick:container>
//! ```
//!
//! Nothing is downloaded, nothing is installed, and no network is touched.
//! Every declared tool is looked for BEFORE any cell runs, and a document
//! missing one fails immediately, naming every tool it could not find at
//! once — rather than failing on the first, being fixed, and failing on the
//! second.
//!
//! Installing them is the reader's job, on purpose. Resolving tools for
//! people means either becoming a package manager or depending on one, and
//! this product would then own the difference between what it installed and
//! what the reader's own tooling installs. Saying precisely what is missing
//! is the part that has to exist; the rest is theirs.
//!
//! ## Why the check goes through the executor
//!
//! Because "installed" means "visible to the cell", and those differ. Under
//! the sandbox a cell has an empty `$HOME` with only toolchain directories
//! bound back, so a binary in `~/Desktop/tools` exists on the machine and
//! not in the cell. Under Docker it is the image's contents that matter, not
//! the host's at all. Probing through the executor asks the only question
//! worth asking: can the thing that will run the cell see this?

use std::collections::BTreeMap;

use anyhow::{Result, bail};
use hick_lang::{HickNode, HickTag};
use hickory_executor::Executor;

use crate::tag_attr;

/// One tool a container's cells expect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    /// The executable name, looked up the way a shell looks it up.
    pub bin: String,
    /// What it is for, shown when it is missing.
    ///
    /// Optional, and worth writing: "duckdb is not installed" tells a reader
    /// what to install, while "duckdb is not installed — the queries in this
    /// document run against it" tells them whether they want to.
    pub reason: Option<String>,
    /// Where the declaration is, so the error can point at it.
    pub source_line: usize,
}

</storycode:copy>
<a id="f-43ab0b0251a2-needs_of"></a>
### `needs_of`

Read every `<hick:needs>` under a `<hick:container>` tag.

<storycode:copy id="f-43ab0b0251a2-needs_of">
/// Read every `<hick:needs>` under a `<hick:container>` tag.
pub fn needs_of(tag: &HickTag) -> Vec<Need> {
    let mut needs = Vec::new();
    for child in &tag.children {
        let HickNode::Tag(child_tag) = child else {
            continue;
        };
        if child_tag.name != "needs" {
            continue;
        }
        // A `needs` with no `bin` declares nothing; it is a typo, and the
        // lint that catches it is better than a silent no-op here.
        if let Some(bin) = tag_attr(child_tag, "bin") {
            needs.push(Need {
                bin,
                reason: tag_attr(child_tag, "for"),
                source_line: child_tag.source_line,
            });
        }
    }
    needs
}

</storycode:copy>
<a id="f-43ab0b0251a2-Missing"></a>
### `Missing`

<storycode:copy id="f-43ab0b0251a2-Missing">
/// What a preflight found.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Missing {
    /// Container name -> the tools it declared and could not find.
    pub by_container: BTreeMap<String, Vec<Need>>,
}

</storycode:copy>
<a id="f-43ab0b0251a2-impl-Missing"></a>
### `impl-Missing`

<storycode:copy id="f-43ab0b0251a2-impl-Missing">
impl Missing {
    pub fn is_empty(&self) -> bool {
        self.by_container.is_empty()
    }

    /// The message a person reads when a document cannot run here.
    ///
    /// Every missing tool, grouped by container, with the line that declared
    /// it — because a document that needs three things should cost one round
    /// trip, not three.
    pub fn report(&self, document: &str) -> String {
        let total: usize = self.by_container.values().map(Vec::len).sum();
        let mut out = format!(
            "{document} needs {total} program{} that {} not installed here:\n",
            if total == 1 { "" } else { "s" },
            if total == 1 { "is" } else { "are" },
        );
        for (container, needs) in &self.by_container {
            out.push_str(&format!("\n  container '{container}':\n"));
            for need in needs {
                out.push_str(&format!(
                    "    {} — declared at line {}",
                    need.bin, need.source_line
                ));
                if let Some(reason) = &need.reason {
                    out.push_str(&format!(" ({reason})"));
                }
                out.push('\n');
            }
        }
        out.push_str(
            "\nInstall them however you normally would; nothing is downloaded for you.\n\
             \n\
             If one IS installed, the cell cannot see it: cells run confined by default, and \n\
             a confined cell sees less of the machine than you do. On Linux and macOS it \n\
             has an empty $HOME apart from the usual toolchain directories; on Windows the \n\
             app container's own permissions decide, and a per-user install is typically \n\
             invisible where a machine-wide one is not. A binary somewhere unusual can be \n\
             reached with HICKORY_EXECUTOR=local, which runs unconfined.\n\
             \n\
             If a declaration is simply wrong, the fix is in the document: <hick:needs bin=\"…\" />",
        );
        out
    }
}

</storycode:copy>
<a id="f-43ab0b0251a2-preflight"></a>
### `preflight`

Check every container's needs before anything runs.
Returns the ones that are missing rather than failing, so the caller
decides whether a missing tool aborts the run (it does) or is merely
reported (`hick doctor`-shaped uses later).

<storycode:copy id="f-43ab0b0251a2-preflight">
/// Check every container's needs before anything runs.
///
/// Returns the ones that are missing rather than failing, so the caller
/// decides whether a missing tool aborts the run (it does) or is merely
/// reported (`hick doctor`-shaped uses later).
pub async fn preflight(
    executor: &dyn Executor,
    needs_by_container: &BTreeMap<String, Vec<Need>>,
    images: &std::collections::HashMap<String, String>,
) -> Result<Missing> {
    let mut missing = Missing::default();
    for (container, needs) in needs_by_container {
        if needs.is_empty() {
            continue;
        }
        // The container has to exist before anything can be asked of it, and
        // starting it is idempotent — the pipeline starts it again later.
        let image = images
            .get(container)
            .map(String::as_str)
            .unwrap_or("alpine");
        executor.ensure_started(container, image).await?;

        for need in needs {
            if !executor.probe_program(container, &need.bin).await? {
                missing
                    .by_container
                    .entry(container.clone())
                    .or_default()
                    .push(need.clone());
            }
        }
    }
    Ok(missing)
}

</storycode:copy>
<a id="f-43ab0b0251a2-require"></a>
### `require`

<storycode:copy id="f-43ab0b0251a2-require">
/// Fail with the report, or return.
pub fn require(missing: &Missing, document: &str) -> Result<()> {
    if missing.is_empty() {
        return Ok(());
    }
    bail!("{}", missing.report(document))
}

</storycode:copy>
<a id="f-43ab0b0251a2-tests"></a>
### `tests`

<storycode:copy id="f-43ab0b0251a2-tests">
#[cfg(test)]
mod tests {
    use super::*;

    /// Parse a real document and hand back its container tag.
    ///
    /// Built by parsing rather than by constructing `HickTag` values: the
    /// thing worth testing is what `needs_of` does with what the PARSER
    /// produces, and a hand-built tag can be shaped in ways the parser never
    /// emits.
    fn container_in(body: &str) -> HickTag {
        let source = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             <hick:container name=\"lab\">\n{body}</hick:container>\n\
             </hick:doc>\n"
        );
        let document = hick_lang::parse(&source).expect("the fixture parses");
        document
            .tags()
            .find(|tag| tag.name == "container")
            .expect("a container")
            .clone()
    }

    #[test]
    fn a_container_declares_what_its_cells_expect() {
        let tag = container_in(
            "  <hick:needs bin=\"duckdb\" for=\"the queries below\" />\n\
             \x20 <hick:needs bin=\"python3\" />\n",
        );
        let needs = needs_of(&tag);
        assert_eq!(needs.len(), 2);
        assert_eq!(needs[0].bin, "duckdb");
        assert_eq!(needs[0].reason.as_deref(), Some("the queries below"));
        assert_eq!(needs[0].source_line, 4);
        assert_eq!(needs[1].reason, None);
    }

    #[test]
    fn other_children_are_not_needs() {
        // `<hick:allow>` lives in the same place and must not be read as one.
        let tag = container_in("  <hick:allow network=\"example.com:443\" />\n");
        assert!(needs_of(&tag).is_empty());
    }

    #[test]
    fn a_needs_with_nothing_to_need_declares_nothing() {
        let tag = container_in("  <hick:needs for=\"something\" />\n");
        assert!(needs_of(&tag).is_empty());
    }

    #[test]
    fn the_report_names_every_missing_tool_at_once() {
        // Three round trips to learn about three missing tools is how a
        // person decides a tool hates them.
        let mut missing = Missing::default();
        missing.by_container.insert(
            "lab".to_string(),
            vec![
                Need {
                    bin: "duckdb".into(),
                    reason: Some("the queries".into()),
                    source_line: 12,
                },
                Need {
                    bin: "gnuplot".into(),
                    reason: None,
                    source_line: 13,
                },
            ],
        );
        let report = missing.report("report.md");
        assert!(report.contains("2 programs"));
        assert!(report.contains("duckdb — declared at line 12 (the queries)"));
        assert!(report.contains("gnuplot — declared at line 13"));
        assert!(report.contains("container 'lab'"));
        // And it says what to do next, including the case where the tool IS
        // installed but the sandbox cannot see it.
        assert!(report.contains("HICKORY_EXECUTOR=local"));
    }

    #[test]
    fn one_missing_tool_reads_as_one() {
        let mut missing = Missing::default();
        missing.by_container.insert(
            "c".to_string(),
            vec![Need {
                bin: "jq".into(),
                reason: None,
                source_line: 3,
            }],
        );
        let report = missing.report("d.md");
        assert!(
            report.contains("1 program that is not installed"),
            "{report}"
        );
    }

    #[test]
    fn nothing_missing_is_not_an_error() {
        assert!(require(&Missing::default(), "d.md").is_ok());
    }
}
</storycode:copy>

### Assemble `crates/hick-literate/src/needs.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/needs.rs"><storycode:paste select="#f-43ab0b0251a2-Need"/><storycode:paste select="#f-43ab0b0251a2-needs_of"/><storycode:paste select="#f-43ab0b0251a2-Missing"/><storycode:paste select="#f-43ab0b0251a2-impl-Missing"/><storycode:paste select="#f-43ab0b0251a2-preflight"/><storycode:paste select="#f-43ab0b0251a2-require"/><storycode:paste select="#f-43ab0b0251a2-tests"/></storycode:file>

<a id="output-8b67c5b91743"></a>
## `crates/hickory-cli/tests/needs_preflight.rs`

Related contracts: [A Document Says What It Needs](contract.md).

<a id="f-8b67c5b91743-common"></a>
### `common`

`<hick:needs>`, through the real binary.
Protects docs/guarantees/execution/a-document-says-what-it-needs.md
Driven as a user drives it — `hick run` on a document — because the claim
is about what happens to a person whose machine is missing something, and
that includes the exit status, the message, and crucially the fact that
nothing ran first.

<storycode:copy id="f-8b67c5b91743-common">
//! `<hick:needs>`, through the real binary.
//!
//! Protects docs/guarantees/execution/a-document-says-what-it-needs.md
//!
//! Driven as a user drives it — `hick run` on a document — because the claim
//! is about what happens to a person whose machine is missing something, and
//! that includes the exit status, the message, and crucially the fact that
//! nothing ran first.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The `hick` this workspace just built.
mod common;
</storycode:copy>
<a id="f-8b67c5b91743-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-8b67c5b91743-use_declaration">
use common::{always_installed_program, create_side_effect_file};

</storycode:copy>
<a id="f-8b67c5b91743-hick"></a>
### `hick`

<storycode:copy id="f-8b67c5b91743-hick">
fn hick() -> Option<PathBuf> {
    let mut path = std::env::current_exe().ok()?;
    path.pop();
    path.pop();
    let candidate = path.join(if cfg!(windows) { "hick.exe" } else { "hick" });
    candidate.is_file().then_some(candidate)
}

</storycode:copy>
<a id="f-8b67c5b91743-document"></a>
### `document`

<storycode:copy id="f-8b67c5b91743-document">
fn document(needs: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
         # Needs\n\n\
         <hick:container name=\"lab\" image=\"alpine\">\n\
         {needs}\
         </hick:container>\n\n\
         <hick:exec container=\"lab\">\n\
         {body}\n\
         </hick:exec>\n\
         </hick:doc>\n"
    )
}

</storycode:copy>
<a id="f-8b67c5b91743-Run"></a>
### `Run`

<storycode:copy id="f-8b67c5b91743-Run">
struct Run {
    ok: bool,
    output: String,
    woven: Option<String>,
    /// Whether the cell's own side effect happened.
    side_effect: bool,
}

</storycode:copy>
<a id="f-8b67c5b91743-run"></a>
### `run`

<storycode:copy id="f-8b67c5b91743-run">
fn run(needs: &str, body: impl Fn(&Path) -> String) -> Option<Run> {
    let hick = hick()?;
    let dir = tempfile::tempdir().unwrap();
    // Evidence a cell ran, as a FILE. The old marker was `echo itran | tr a-z
    // A-Z` — assembled by `tr` so the word could not appear in the command
    // text and be read back as if it were output. The reasoning was right and
    // the tool was not: `tr` does not exist on Windows, so the marker could
    // never appear whether or not the cell ran, and every "nothing ran"
    // assertion below held for a document that had run perfectly well.
    let marker = dir.path().join("ran.txt");
    std::fs::write(dir.path().join("d.md"), document(needs, &body(&marker))).unwrap();
    let out = Command::new(hick)
        .arg("run")
        .arg("d.md")
        .current_dir(dir.path())
        // Unconfined on purpose: a confined cell may write only its own
        // workdir, whose path this test does not know, so it could not see
        // the evidence either way. What is under test is the preflight, not
        // the sandbox.
        .env("HICKORY_EXECUTOR", "local")
        .output()
        .expect("hick runs");
    let woven = std::fs::read_to_string(dir.path().join("out.md")).ok();
    let side_effect = marker.exists();
    Some(Run {
        ok: out.status.success(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        woven,
        side_effect,
    })
}

</storycode:copy>
<a id="f-8b67c5b91743-a_missing_program_stops_the_document_before_anything_runs"></a>
### `a_missing_program_stops_the_document_before_anything_runs`

<storycode:copy id="f-8b67c5b91743-a_missing_program_stops_the_document_before_anything_runs">
#[test]
fn a_missing_program_stops_the_document_before_anything_runs() {
    let Some(result) = run(
        "  <hick:needs bin=\"definitely-not-a-real-program\" for=\"the analysis\" />\n",
        create_side_effect_file,
    ) else {
        eprintln!("SKIPPED: hick has not been built into this target dir");
        return;
    };

    assert!(!result.ok, "a document with a missing program succeeded");
    assert!(
        result.output.contains("definitely-not-a-real-program"),
        "the message does not name what is missing: {}",
        result.output
    );
    // The reason the author wrote, so a reader can decide whether they care.
    assert!(result.output.contains("the analysis"), "{}", result.output);
    // And the line, so they can find the declaration.
    assert!(result.output.contains("line 6"), "{}", result.output);

    // And nothing ran. Half a pipeline's side effects followed by
    // `not found` is the worst ordering available: the document has already
    // changed things and still cannot finish.
    //
    // (`hick run` writes the woven file even when a run fails — that is how
    // it has always behaved, for ordinary cell failures too — so the check
    // is that the cell produced no OUTPUT, not that no file was written.)
    assert!(
        !result.side_effect,
        "a cell ran despite the document missing a program it declared:\n{}",
        result.woven.unwrap_or_default()
    );
    assert!(
        result
            .woven
            .as_deref()
            .is_some_and(|text| text.contains("[never run]")),
        "the woven document does not record the cell as never run:\n{}",
        result.woven.unwrap_or_default()
    );
}

</storycode:copy>
<a id="f-8b67c5b91743-a_satisfied_document_really_does_run_its_cell"></a>
### `a_satisfied_document_really_does_run_its_cell`

The control that makes the test above falsifiable.

<storycode:copy id="f-8b67c5b91743-a_satisfied_document_really_does_run_its_cell">
/// The control that makes the test above falsifiable.
#[test]
fn a_satisfied_document_really_does_run_its_cell() {
    // `a_missing_program_stops_the_document_before_anything_runs` asserts a
    // file does NOT appear. On its own that holds for a run which failed for
    // any reason at all — including a marker that could never be written,
    // which is precisely what happened on Windows while the marker was `tr`.
    // Same document, need satisfied: the file must appear.
    let Some(result) = run(
        &format!("  <hick:needs bin=\"{}\" />\n", always_installed_program()),
        create_side_effect_file,
    ) else {
        return;
    };
    assert!(result.ok, "a satisfied document failed: {}", result.output);
    assert!(
        result.side_effect,
        "the cell did not run, so the absence of this file proves nothing \
         about preflight:\n{}",
        result.woven.unwrap_or_default()
    );
}

</storycode:copy>
<a id="f-8b67c5b91743-every_missing_program_is_named_at_once"></a>
### `every_missing_program_is_named_at_once`

<storycode:copy id="f-8b67c5b91743-every_missing_program_is_named_at_once">
#[test]
fn every_missing_program_is_named_at_once() {
    // Three round trips to learn about three missing programs is how a person
    // decides a tool hates them.
    let Some(result) = run(
        "  <hick:needs bin=\"not-real-one\" />\n  <hick:needs bin=\"not-real-two\" />\n",
        |_| "echo hi".to_string(),
    ) else {
        return;
    };
    assert!(result.output.contains("not-real-one"), "{}", result.output);
    assert!(result.output.contains("not-real-two"), "{}", result.output);
    assert!(result.output.contains("2 programs"), "{}", result.output);
}

</storycode:copy>
<a id="f-8b67c5b91743-a_program_that_is_present_lets_the_document_run"></a>
### `a_program_that_is_present_lets_the_document_run`

<storycode:copy id="f-8b67c5b91743-a_program_that_is_present_lets_the_document_run">
#[test]
fn a_program_that_is_present_lets_the_document_run() {
    // The shell itself is the one program that must exist for any of this to
    // work at all, so it is the only safe thing to assert is installed on
    // somebody else's machine — and it is a different name per platform.
    let Some(result) = run(
        &format!(
            "  <hick:needs bin=\"{}\" for=\"the cells\" />\n",
            always_installed_program()
        ),
        |_| "echo hello".to_string(),
    ) else {
        return;
    };
    assert!(result.ok, "a satisfied document failed: {}", result.output);
    let woven = result.woven.expect("the document wove");
    assert!(woven.contains("hello"), "{woven}");
}

</storycode:copy>
<a id="f-8b67c5b91743-the_check_never_appears_in_the_document"></a>
### `the_check_never_appears_in_the_document`

<storycode:copy id="f-8b67c5b91743-the_check_never_appears_in_the_document">
#[test]
fn the_check_never_appears_in_the_document() {
    // The probe is our bookkeeping. A woven document listing `command -v sh`
    // beside the author's own commands would be a page about our
    // implementation in the middle of somebody else's work.
    let Some(result) = run(
        &format!("  <hick:needs bin=\"{}\" />\n", always_installed_program()),
        |_| "echo hello".to_string(),
    ) else {
        return;
    };
    let woven = result.woven.expect("the document wove");
    // Both spellings, not just this platform's. Asserting only `command -v`
    // would let the cmd probe leak on Windows unnoticed — and under the local
    // executor there is now no shell probe at all, so this also catches a
    // future change that reintroduces one.
    for probe in ["command -v", "where \""] {
        assert!(
            !woven.contains(probe),
            "the probe leaked into the document:\n{woven}"
        );
    }
    assert!(
        !woven.contains("needs"),
        "the declaration was woven as content:\n{woven}"
    );
}

</storycode:copy>
<a id="f-8b67c5b91743-a_document_that_declares_nothing_still_runs"></a>
### `a_document_that_declares_nothing_still_runs`

<storycode:copy id="f-8b67c5b91743-a_document_that_declares_nothing_still_runs">
#[test]
fn a_document_that_declares_nothing_still_runs() {
    // `needs` is optional, and adding it must not have made every existing
    // document without one suspect.
    let Some(result) = run("", |_| "echo hello".to_string()) else {
        return;
    };
    assert!(result.ok, "{}", result.output);
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/needs_preflight.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/needs_preflight.rs"><storycode:paste select="#f-8b67c5b91743-common"/><storycode:paste select="#f-8b67c5b91743-use_declaration"/><storycode:paste select="#f-8b67c5b91743-hick"/><storycode:paste select="#f-8b67c5b91743-document"/><storycode:paste select="#f-8b67c5b91743-Run"/><storycode:paste select="#f-8b67c5b91743-run"/><storycode:paste select="#f-8b67c5b91743-a_missing_program_stops_the_document_before_anything_runs"/><storycode:paste select="#f-8b67c5b91743-a_satisfied_document_really_does_run_its_cell"/><storycode:paste select="#f-8b67c5b91743-every_missing_program_is_named_at_once"/><storycode:paste select="#f-8b67c5b91743-a_program_that_is_present_lets_the_document_run"/><storycode:paste select="#f-8b67c5b91743-the_check_never_appears_in_the_document"/><storycode:paste select="#f-8b67c5b91743-a_document_that_declares_nothing_still_runs"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/needs.rs](contract.md#output-43ab0b0251a2)
- [crates/hickory-executor-sandbox/src/lib.rs](../../../mechanisms/crates/hickory-executor-sandbox/src/lib.rs.md#output-3ed8f409da24)
- [crates/hickory-executor/src/lib.rs](../../../mechanisms/crates/hickory-executor/src/lib.rs.md#output-87ce480a6eab)

### Tests and fixtures

- [crates/hickory-cli/tests/needs_preflight.rs](contract.md#output-8b67c5b91743)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
