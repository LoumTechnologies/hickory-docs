//! Where a terminal's typed commands land in a document.
//!
//! `docs/specs/freeform/a-terminal-that-writes-the-document.md` settles the
//! two questions this answers, and both answers are "nothing new":
//!
//! **The anchor is the container name.** A document already models a
//! sequence of commands that share state — several `<hick:exec
//! container="sdk">` blocks naming the same container — and the DAG already
//! treats that state as an edge. So a terminal bound to `sdk` in a document
//! *is* that shell. Say "anchored to a container", never "attached to a
//! document", which does not say the thing that makes it work.
//!
//! **One cell that grows, not one cell per line.** A shell session of forty
//! exploratory commands would otherwise become forty cells, and a document
//! is something a person reads. `commands: Vec<String>` on
//! `ExecTranscriptEntry` is plural and always has been. Starting a new cell
//! is a deliberate act — stop typing and write a sentence — and it is what
//! resuming after a suspension always does.
//!
//! This module edits document TEXT rather than a parse tree, and that is the
//! point: everything outside the one cell being appended to must come back
//! byte for byte. The hick parser's no-escaping invariant means a cell body
//! is raw text, so appending a line is inserting a line — there is nothing
//! to escape and nothing that may be re-serialised.

/// Where in `source` the anchored cell's body ends, for a given container.
///
/// The **last** matching cell, because the anchor grows the newest one: a
/// document may hold several `hick:exec` blocks naming one container, and
/// they are a sequence, so a terminal typing now belongs at the end of it.
fn last_body_end(source: &str, container: &str) -> Option<usize> {
    let needle = format!("container=\"{container}\"");
    let mut found = None;
    let mut at = 0;
    while let Some(open_rel) = source[at..].find("<hick:exec") {
        let open = at + open_rel;
        let Some(gt_rel) = source[open..].find('>') else {
            break;
        };
        let tag_end = open + gt_rel + 1;
        // A self-closing cell has no body to append to; it is a declaration,
        // not a transcript, and growing it would change what it means.
        let self_closing = source[open..tag_end].trim_end().ends_with("/>");
        let names_it = source[open..tag_end].contains(&needle);
        if let Some(close_rel) = source[tag_end..].find("</hick:exec>") {
            let close = tag_end + close_rel;
            if names_it && !self_closing {
                found = Some(close);
            }
            at = close + "</hick:exec>".len();
        } else {
            at = tag_end;
        }
    }
    found
}

/// Append `command` to the cell anchored to `container`, creating one when
/// there is none to grow — or when `new_cell` says this is a fresh start.
///
/// `new_cell` is what a resume passes. After a suspension the shell holds
/// state the document does not describe, so continuing the previous cell
/// would claim that state came from the lines above it.
pub fn append_command(source: &str, container: &str, command: &str, new_cell: bool) -> String {
    if !new_cell && let Some(end) = last_body_end(source, container) {
        let mut out = String::with_capacity(source.len() + command.len() + 1);
        out.push_str(&source[..end]);
        // The body ends with a newline in every document a weave produces;
        // one is added when it does not, so a command can never be glued to
        // the end of the line above it.
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(command);
        out.push('\n');
        out.push_str(&source[end..]);
        return out;
    }
    insert_cell(source, container, command)
}

/// A new `hick:exec` for this container, placed where it will still be inside
/// the document.
fn insert_cell(source: &str, container: &str, command: &str) -> String {
    let cell = format!("\n<hick:exec container=\"{container}\">\n{command}\n</hick:exec>\n");
    // A document with the optional wrapper (`bare-documents.md` made it
    // optional, not obsolete) must have the cell go INSIDE it — appending
    // after `</hick:doc>` writes text no weave will ever see.
    if let Some(close) = source.rfind("</hick:doc>") {
        let mut out = String::with_capacity(source.len() + cell.len());
        out.push_str(&source[..close]);
        out.push_str(&cell);
        out.push_str(&source[close..]);
        return out;
    }
    let mut out = source.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&cell);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
        # Scaffolding\n\
        \n\
        <hick:exec container=\"sdk\">\n\
        dotnet new console -o app\n\
        </hick:exec>\n\
        \n\
        </hick:doc>\n";

    #[test]
    fn a_command_grows_the_cell_that_is_already_there() {
        let out = append_command(DOC, "sdk", "ls app", false);
        assert!(
            out.contains("dotnet new console -o app\nls app\n</hick:exec>"),
            "{out}"
        );
        // One cell, not two: a shell session of forty commands must not
        // become forty cells.
        assert_eq!(out.matches("<hick:exec").count(), 1);
    }

    #[test]
    fn everything_outside_the_cell_comes_back_byte_for_byte() {
        let out = append_command(DOC, "sdk", "ls app", false);
        // The only difference anywhere is the inserted line.
        assert_eq!(out.replace("ls app\n", ""), DOC);
    }

    #[test]
    fn a_container_with_no_cell_yet_gets_one_inside_the_document() {
        let out = append_command(DOC, "other", "echo hi", false);
        let cell = out
            .find("container=\"other\"")
            .expect("the cell was created");
        let close = out.find("</hick:doc>").expect("the wrapper survived");
        assert!(cell < close, "the cell landed outside the document:\n{out}");
        assert!(
            out.contains("<hick:exec container=\"other\">\necho hi\n</hick:exec>"),
            "{out}"
        );
    }

    #[test]
    fn a_new_cell_is_what_resuming_after_a_suspension_writes() {
        // Continuing the old cell would claim the shell state a suspension
        // hid came from the lines above it.
        let out = append_command(DOC, "sdk", "make", true);
        assert_eq!(out.matches("<hick:exec").count(), 2);
        assert!(
            out.contains("dotnet new console -o app\n</hick:exec>"),
            "{out}"
        );
        assert!(
            out.contains("<hick:exec container=\"sdk\">\nmake\n</hick:exec>"),
            "{out}"
        );
    }

    #[test]
    fn the_last_cell_grows_when_a_container_has_several() {
        let two = "<hick:exec container=\"sdk\">\nfirst\n</hick:exec>\n\
                   <hick:exec container=\"sdk\">\nsecond\n</hick:exec>\n";
        let out = append_command(two, "sdk", "third", false);
        assert!(out.contains("second\nthird\n</hick:exec>"), "{out}");
        assert!(out.contains("first\n</hick:exec>"), "{out}");
    }

    #[test]
    fn another_containers_cell_is_left_alone() {
        let mixed = "<hick:exec container=\"a\">\nin a\n</hick:exec>\n\
                     <hick:exec container=\"b\">\nin b\n</hick:exec>\n";
        let out = append_command(mixed, "a", "also a", false);
        assert!(out.contains("in a\nalso a\n</hick:exec>"), "{out}");
        assert!(out.contains("in b\n</hick:exec>"), "{out}");
    }

    #[test]
    fn a_self_closing_cell_is_a_declaration_and_is_not_grown() {
        // `<hick:exec … />` has no body. Appending to it would change what
        // the element means rather than add to a transcript.
        let declared = "<hick:exec container=\"sdk\" cmd=\"true\" />\n";
        let out = append_command(declared, "sdk", "ls", false);
        assert!(
            out.contains("<hick:exec container=\"sdk\" cmd=\"true\" />"),
            "{out}"
        );
        assert!(
            out.contains("<hick:exec container=\"sdk\">\nls\n</hick:exec>"),
            "{out}"
        );
    }

    #[test]
    fn a_bare_document_with_no_wrapper_still_gets_its_cell() {
        // `bare-documents.md`: the wrapper is optional, and a note that never
        // had one must not grow one here.
        let bare = "# Just a note\n\nSome prose.\n";
        let out = append_command(bare, "sdk", "ls", false);
        assert!(out.starts_with("# Just a note\n\nSome prose.\n"), "{out}");
        assert!(
            out.contains("<hick:exec container=\"sdk\">\nls\n</hick:exec>"),
            "{out}"
        );
        assert!(!out.contains("<hick:doc"), "a wrapper was invented: {out}");
    }

    #[test]
    fn a_multi_line_command_lands_as_the_lines_it_was() {
        // A heredoc is one typed line to the shell and several lines in the
        // cell, and the cell body is raw text — there is nothing to escape.
        let out = append_command(DOC, "sdk", "cat <<'EOT'\nhi\nEOT", false);
        assert!(
            out.contains("dotnet new console -o app\ncat <<'EOT'\nhi\nEOT\n</hick:exec>"),
            "{out}"
        );
    }

    #[test]
    fn what_is_written_still_parses() {
        let mut source = DOC.to_string();
        for command in ["ls app", "dotnet build app", "cat <<'EOT'\nx\nEOT"] {
            source = append_command(&source, "sdk", command, false);
        }
        hick_lang::parse(&source).expect("an anchored terminal must not break its document");
    }
}
