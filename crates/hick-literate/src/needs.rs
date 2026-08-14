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

/// The shell command that answers "is this on PATH?".
///
/// `command -v` rather than `which`: it is a POSIX shell builtin, so it
/// exists wherever `sh` does, and it does not depend on a `which` binary
/// being installed — which would make the check's own dependency the first
/// thing to go missing.
pub fn probe_command(bin: &str) -> String {
    // Single-quoted, with the one escape sh needs, so a tool name containing
    // shell syntax cannot become shell syntax.
    let quoted = bin.replace('\'', r"'\''");
    format!("command -v '{quoted}' > /dev/null 2>&1")
}

/// What a preflight found.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Missing {
    /// Container name -> the tools it declared and could not find.
    pub by_container: BTreeMap<String, Vec<Need>>,
}

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
             If one IS installed, the cell cannot see it: cells run confined by default, with \n\
             an empty $HOME apart from the usual toolchain directories. A binary somewhere \n\
             unusual can be reached with HICKORY_EXECUTOR=local, which runs unconfined.\n\
             \n\
             If a declaration is simply wrong, the fix is in the document: <hick:needs bin=\"…\" />",
        );
        out
    }
}

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
            if !executor.probe(container, &probe_command(&need.bin)).await? {
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

/// Fail with the report, or return.
pub fn require(missing: &Missing, document: &str) -> Result<()> {
    if missing.is_empty() {
        return Ok(());
    }
    bail!("{}", missing.report(document))
}

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
    fn the_probe_cannot_be_turned_into_shell_syntax() {
        // A tool name is author-supplied text, and the probe runs in a shell.
        let command = probe_command("evil'; rm -rf /; echo '");
        assert!(command.starts_with("command -v '"), "{command}");
        assert!(!command.contains("; rm -rf /; echo ;"), "{command}");
        // The dangerous text survives INSIDE the quotes, which is the point:
        // it is looked up as a (very odd) program name, not executed.
        assert!(command.contains(r"'\''"), "{command}");
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
        let report = missing.report("report.hick");
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
        let report = missing.report("d.hick");
        assert!(
            report.contains("1 program that is not installed"),
            "{report}"
        );
    }

    #[test]
    fn nothing_missing_is_not_an_error() {
        assert!(require(&Missing::default(), "d.hick").is_ok());
    }
}
