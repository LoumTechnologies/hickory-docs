//! The carry: what moves from one attempt at a session to the next.
//!
//! See `docs/specs/freeform/sessions-you-run-again.md`. You run an agent
//! session, get the result you want, and along the way change your mind twice.
//! So you run it again with a better opening prompt and different answers, and
//! get there without the wandering.
//!
//! **A re-run is not a reenactment.** Every byte of the second session was
//! written by the harness, the tools really ran, the result really came out.
//! It stands without the first ever having existed — which the product already
//! treats correctly by accident, because `hick init` gitignores `sessions/`,
//! so discarding the first attempt is the default rather than a decision.
//!
//! What moves is not the transcript. It is what you learned, distilled into
//! inputs:
//!
//! - **a better opening prompt**;
//! - **the tests you kept** — the common case, because a session's most
//!   durable output is often the discovery of what "done" meant;
//! - **an approach you ruled out**, so the next attempt does not spend a turn
//!   rediscovering it.
//!
//! Tests are one instance and not the mechanism: building "carry the tests" as
//! a primitive would be building the example rather than the thing.
//!
//! ## Two things this deliberately does not do
//!
//! **It does not distil for you.** The carried tests are the ones **you read
//! and kept**, not everything the first attempt emitted — because they were
//! written by an agent that had already seen one implementation, so they tend
//! to pin that implementation's incidental choices rather than the behaviour
//! you wanted. Session 2 would then be bound to session 1's arbitrary
//! decisions while appearing to have chosen freely. So this writes a carry
//! with the first attempt's opening prompt and **empty, marked slots** for the
//! rest; filling them is the person's job, and the guard is not mechanical.
//!
//! **It invents no fourth artifact.** The carry has no obvious home — it is
//! not the session, which is discardable, and not obviously the document
//! either. Rather than add a kind, a carry is **an ordinary `.md`
//! document**: no new file type, no new store, no new gitignore rule, and it
//! is committed by default, which it must be — a carry whose only support is
//! a gitignored session dangles by construction.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

/// What a carry was written from, and where it landed.
#[derive(Debug)]
pub struct CarryOutcome {
    pub path: PathBuf,
    /// The session it distils, as a repository-relative-ish path string.
    pub from: String,
    /// The opening prompt it carried across.
    pub prompt: String,
}

/// The opening prompt of a session: its first user turn.
///
/// The FIRST, not the best: a re-run starts where the first attempt started,
/// and the point of the carry is that you improve it by hand.
pub fn opening_prompt(session_source: &str) -> Option<String> {
    let doc = hick_lang::parse_session(session_source).ok()?;
    doc.nodes.iter().find_map(|node| match node {
        hick_lang::SessionNode::User { text } => {
            let trimmed = text.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }
        _ => None,
    })
}

/// A short slug for a carry's filename, from its prompt.
fn slug(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.extend(ch.to_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
        if out.len() >= 48 {
            break;
        }
    }
    out.trim_matches('-').to_string()
}

/// The carry document's source.
///
/// The `cites=` on the root is a DECLARED claim pointing at the session it was
/// distilled from — an assertion, not a derivation, drawn and reported as one.
/// It is also what makes self-containment checkable: a carried requirement
/// whose only support is a session nobody has is a dangling citation, of
/// exactly the shape `hick cites` already reports.
fn carry_source(from: &str, date: &str, prompt: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
---
carried-from: {from}
date: {date}
---

# Carry

What moves to the next attempt at this session. Not the transcript — what you
learned, distilled into inputs. The first attempt is a draft you may discard;
this is what you keep.

## The opening prompt

Edit this. The whole reason to re-run is that you can now say in one prompt
what took the first attempt several turns to discover — but keep it a prompt
that invites questions rather than one that preempts every misunderstanding.
A prescient monolith is worse for a later reader precisely because it is
prescient: it concentrates everything you learned into one unexplained block.

<hick:copy id="prompt">
{prompt}
</hick:copy>

## The tests you kept

The ones you READ and kept — not everything the first attempt emitted. They
were written by an agent that had already seen one implementation, so they tend
to pin that implementation's incidental choices (call order, internal names, an
error string) rather than the behaviour you wanted. Carrying all of them binds
the next attempt to the first one's arbitrary decisions while appearing to have
chosen freely.

<hick:copy id="tests">
</hick:copy>

## Approaches ruled out

So the next attempt does not spend a turn rediscovering them. Say why, not just
what: "we tried X and it lost the provenance" carries; "not X" does not.

<hick:copy id="ruled-out">
</hick:copy>
</hick:doc>
"#
    )
}

/// Whether a carry still has empty slots — the person has not finished it.
///
/// Reported rather than enforced: an empty slot is a normal state for a carry
/// you wrote a minute ago, and a tool that refused one would be refusing the
/// thing it just created.
pub fn unfilled_slots(source: &str) -> Vec<String> {
    let Ok(doc) = hick_lang::parse(source) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for tag in doc.tags() {
        if tag.name == "copy"
            && let Some(id) = tag.get_attribute("id")
            && hick_lang::tag_text(tag).trim().is_empty()
        {
            out.push(id.to_string());
        }
    }
    out.sort();
    out
}

/// Write a carry distilled from `session`.
pub fn carry_from_session(session: &Path, out: Option<&Path>, today: &str) -> Result<CarryOutcome> {
    let source = std::fs::read_to_string(session)
        .with_context(|| format!("could not read {}", session.display()))?;
    if !hick_lang::is_session_source(&source) {
        bail!(
            "{} is not a session document — a carry is distilled from a \
             conversation, and this file's root is not <hick:session>.\n  \
             Next step: point this at a file under `sessions/`, which is where \
             the dock writes one conversation per file.",
            session.display()
        );
    }
    let prompt = opening_prompt(&source).with_context(|| {
        format!(
            "{} records no user turn, so there is no opening prompt to carry. \
             A session that nobody spoke in has nothing to distil.",
            session.display()
        )
    })?;

    let from = session
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("the session")
        .to_string();

    let path = match out {
        Some(p) => p.to_path_buf(),
        None => {
            // Beside the sessions folder, not inside it: `sessions/` is
            // gitignored, and a carry that vanished with the session it
            // distils would be pointless.
            let base = session
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(Path::new("."))
                .join("carries");
            std::fs::create_dir_all(&base)
                .with_context(|| format!("could not create {}", base.display()))?;
            let stem = slug(&prompt);
            base.join(format!(
                "{today}-{}.md",
                if stem.is_empty() { "carry" } else { &stem }
            ))
        }
    };
    if path.exists() {
        bail!(
            "{} already exists. A carry is written once and then edited by \
             hand — the whole point is what YOU keep, so this will not \
             overwrite yours.\n  \
             Next step: edit it, or pass --out to write a second one.",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let body = carry_source(&from, today, &prompt);
    std::fs::write(&path, &body).with_context(|| format!("could not write {}", path.display()))?;
    Ok(CarryOutcome { path, from, prompt })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-08-22T13:01:02Z">
<hick:user>Add a retry policy to the client.</hick:user>
<hick:assistant>Sure.</hick:assistant>
<hick:user>Also handle 429.</hick:user>
</hick:session>
"#;

    #[test]
    fn the_carry_takes_the_first_prompt_not_the_last() {
        // A re-run starts where the first attempt started; improving the
        // prompt is the person's job, and the point of the artifact.
        assert_eq!(
            opening_prompt(SESSION).as_deref(),
            Some("Add a retry policy to the client.")
        );
    }

    #[test]
    fn a_fresh_carry_names_its_unfilled_slots() {
        // Reported, never enforced: an empty slot is the normal state of a
        // carry a second old, and refusing one would refuse what was just
        // created.
        let dir = tempfile::tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let session = sessions.join("s.hick");
        std::fs::write(&session, SESSION).unwrap();

        let out = carry_from_session(&session, None, "2026-08-23").unwrap();
        let body = std::fs::read_to_string(&out.path).unwrap();
        assert!(body.contains("Add a retry policy to the client."));
        // Written OUTSIDE `sessions/`, which is gitignored — a carry that
        // vanished with the session it distils would be pointless.
        assert!(
            out.path.starts_with(dir.path().join("carries")),
            "{:?}",
            out.path
        );
        assert_eq!(unfilled_slots(&body), vec!["ruled-out", "tests"]);
    }

    #[test]
    fn a_filled_slot_stops_being_reported() {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:copy id="prompt">do the thing</hick:copy>
<hick:copy id="tests">the retry test</hick:copy>
<hick:copy id="ruled-out"></hick:copy>
</hick:doc>
"#;
        assert_eq!(unfilled_slots(body), vec!["ruled-out"]);
    }

    #[test]
    fn a_document_that_is_not_a_session_is_refused_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("notes.hick");
        std::fs::write(
            &doc,
            "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">hi</hick:doc>",
        )
        .unwrap();
        let err = carry_from_session(&doc, None, "2026-08-23").unwrap_err();
        assert!(format!("{err:#}").contains("not a session document"));
    }

    #[test]
    fn an_existing_carry_is_never_overwritten() {
        // The whole point is what YOU keep.
        let dir = tempfile::tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let session = sessions.join("s.hick");
        std::fs::write(&session, SESSION).unwrap();
        let first = carry_from_session(&session, None, "2026-08-23").unwrap();
        std::fs::write(&first.path, "mine, edited").unwrap();
        let err = carry_from_session(&session, None, "2026-08-23").unwrap_err();
        assert!(format!("{err:#}").contains("already exists"));
        assert_eq!(
            std::fs::read_to_string(&first.path).unwrap(),
            "mine, edited"
        );
    }
}
