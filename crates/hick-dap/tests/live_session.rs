//! A real debug session, against a real adapter, over a real document.
//!
//! Protects docs/specs/freeform/literate-debugging.md
//!
//! Everything here goes through the public session API — the same one the
//! app's debugger, `<hick:capture>` and the MCP tools use — because the claim
//! worth testing is that a breakpoint set on a DOCUMENT line stops the
//! program and answers in document coordinates. That is not observable from
//! a unit test of the protocol.
//!
//! Skipped loudly when no adapter is installed. A machine without `debugpy`
//! cannot run this, and a suite that quietly tests nothing is worse than one
//! that fails.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{BindState, Breakpoint, Launch, Mapping, Session, Step};

/// A document whose generated file has a function worth stopping inside.
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing

Every line is quantity times unit price.

<hick:file path="pricing.py">
LINES = [(2, 9.99), (1, 24.50), (3, 1.25)]


def line_total(quantity, unit_price):
    subtotal = quantity * unit_price
    return subtotal


def cart_total(lines):
    return sum(line_total(q, p) for q, p in lines)


print(f"total {cart_total(LINES):.2f}")
</hick:file>
</hick:doc>
"##;

/// 0-based document line of `    subtotal = quantity * unit_price`.
///
/// Counted from `<?xml` as line 0. Line 10 is the `def` above it, and a
/// breakpoint there binds to the DEFINITION — which executes once, at module
/// level, so the program stops in `<module>` and not in the function. Being
/// one line out is not a rounding error here; it is a different frame.
const SUBTOTAL_LINE: u32 = 11;
/// 0-based document line of `def cart_total(lines):`.
const CART_TOTAL_LINE: u32 = 15;
/// A line of prose, which generates no code at all.
const PROSE_LINE: u32 = 4;

struct Fixture {
    _dir: tempfile::TempDir,
    launch: Launch,
    mapping: Arc<Mapping>,
}

/// Write the document, weave its file by hand, and find an adapter.
fn fixture() -> Option<Fixture> {
    let dir = tempfile::tempdir().ok()?;
    let root = dir.path();
    std::fs::create_dir_all(root.join(".git")).ok()?;
    std::fs::write(root.join("doc.hick"), DOC).ok()?;

    // The generated file, as a run would produce it. Written from the same
    // mapping the session uses, so the file and the coordinates cannot
    // disagree.
    let mapping = Mapping::for_document(Path::new("doc.hick"), DOC, root).ok()?;
    let state = hick_lsp::document::HickDocumentState::from_source(DOC).ok()?;
    for file in &state.virtual_files {
        std::fs::write(root.join(&file.path), file.content()).ok()?;
    }

    borrow_this_repos_adapters(root);
    let adapter = hick_dap::discover("python", root)?;
    eprintln!("using adapter {} ({})", adapter.adapter, adapter.origin);
    Some(Fixture {
        launch: Launch {
            adapter: adapter.command,
            program: root.join("pricing.py"),
            cwd: root.to_path_buf(),
            extra: serde_json::json!({}),
        },
        mapping: Arc::new(mapping),
        _dir: dir,
    })
}

/// Point the scratch project at the adapter this repository installed.
///
/// A developer runs `hick dap install python` once, at the top of this repo;
/// a test project in a temp directory is nowhere near it. Without this the
/// whole file skips on the machine most likely to be running it, which is how
/// a suite ends up green while testing nothing. CI installs debugpy for the
/// machine and never reaches this.
#[cfg(unix)]
fn borrow_this_repos_adapters(into: &Path) {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if cache.join("adapters/python/bin/python3").exists() {
        let _ = std::os::unix::fs::symlink(cache, into.join(".hick-cache"));
    }
}

#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

fn skip(why: &str) {
    eprintln!("SKIPPED: {why} (`hick dap install python`, or install debugpy the usual way)");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_a_document_line_stops_the_program_there() {
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };

    let breakpoints = vec![Breakpoint {
        line: SUBTOTAL_LINE,
        condition: None,
        hit_condition: None,
        log_message: None,
    }];
    let (session, statuses) = Session::start(fixture.launch, fixture.mapping, &breakpoints)
        .await
        .expect("the session starts");

    // Verified, which is the difference between a breakpoint that will work
    // and one the gutter must draw hollow.
    assert_eq!(statuses.len(), 1);
    // debugpy verifies while answering, so a Python breakpoint is bound
    // immediately. netcoredbg does not — see `live_session_csharp.rs`, which
    // is why this is three states rather than a bool.
    assert_eq!(
        statuses[0].state,
        BindState::Bound,
        "the adapter could not bind it: {statuses:?}"
    );
    assert_eq!(statuses[0].line, SUBTOTAL_LINE);

    let stopped = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .expect("waiting works")
        .expect("the program stopped");
    assert_eq!(stopped.reason, "breakpoint");

    // The stack comes back in DOCUMENT coordinates — the whole point.
    let stack = session.stack(stopped.thread_id).await.expect("a stack");
    let top = &stack[0];
    assert_eq!(top.name, "line_total");
    assert_eq!(
        top.line,
        Some(SUBTOTAL_LINE),
        "the frame is on the wrong document line: {top:?}"
    );
    assert!(top.in_document);

    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn evaluating_in_a_frame_sees_that_frames_values() {
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };
    let (session, _) = Session::start(
        fixture.launch,
        fixture.mapping,
        &[Breakpoint {
            line: SUBTOTAL_LINE,
            condition: None,
            hit_condition: None,
            log_message: None,
        }],
    )
    .await
    .expect("starts");

    let stopped = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .unwrap()
        .unwrap();
    let stack = session.stack(stopped.thread_id).await.unwrap();
    let frame = stack[0].id;

    // First iteration: 2 × 9.99.
    let value = session
        .evaluate("quantity * unit_price", Some(frame), "watch")
        .await
        .expect("evaluates");
    assert!(value.value.starts_with("19.98"), "got {}", value.value);

    // The same expression in the CALLER's frame is a different question, and
    // must give a different answer — this is what "in the frame you selected"
    // means, and getting it wrong makes every value subtly wrong.
    let caller = stack
        .iter()
        .find(|f| f.name.contains("cart_total") || f.name.contains("genexpr"));
    if let Some(caller) = caller {
        let there = session.evaluate("quantity", Some(caller.id), "watch").await;
        assert!(
            there.is_err() || there.unwrap().value != value.value,
            "the caller's frame answered with the callee's value"
        );
    }

    // Variables come back with names and values for the inline display.
    let variables = session.variables(frame).await.expect("variables");
    let quantity = variables
        .iter()
        .find(|v| v.name == "quantity")
        .expect("quantity is in scope");
    assert_eq!(quantity.value, "2");

    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn stepping_moves_and_dropping_a_frame_goes_back_to_its_first_line() {
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };
    let (session, _) = Session::start(
        fixture.launch,
        fixture.mapping,
        &[Breakpoint {
            line: SUBTOTAL_LINE,
            condition: None,
            hit_condition: None,
            log_message: None,
        }],
    )
    .await
    .expect("starts");

    let stopped = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .unwrap()
        .unwrap();
    let thread = stopped.thread_id;
    let before = session.stack(thread).await.unwrap()[0].line;
    assert_eq!(before, Some(SUBTOTAL_LINE));

    // Step over: one line further down the same function.
    session
        .step(Step::Over, thread, None)
        .await
        .expect("step over");
    session
        .wait_for_stop(Duration::from_secs(20))
        .await
        .unwrap()
        .expect("stopped after stepping");
    let after = session.stack(thread).await.unwrap()[0].line;
    assert_eq!(
        after,
        Some(SUBTOTAL_LINE + 1),
        "step over did not advance one line"
    );

    // Drop frame — the 80% of stepping backwards. Re-enters `line_total`
    // from its first line, so the position goes BACKWARDS even though
    // execution did not.
    if session.capabilities().restart_frame {
        let frame = session.stack(thread).await.unwrap()[0].id;
        session
            .step(Step::DropFrame, thread, Some(frame))
            .await
            .expect("drop frame");
        session
            .wait_for_stop(Duration::from_secs(20))
            .await
            .unwrap()
            .expect("stopped after drop");
        let dropped = session.stack(thread).await.unwrap()[0].line;
        assert!(
            dropped.is_some_and(|line| line < SUBTOTAL_LINE),
            "dropping the frame did not return to the top of the function: {dropped:?}"
        );
        eprintln!("OK drop frame: {:?} -> {:?}", after, dropped);
    } else {
        eprintln!("NOTE: this adapter cannot drop a frame; the control would be greyed out");
    }

    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn jumping_to_an_earlier_line_runs_it_again() {
    // The backwards move that actually exists for Python. debugpy has no
    // `restartFrame`, but it does have `goto` — so "I stepped one too far,
    // do that again" is answerable, which is what people want from stepping
    // back nine times in ten.
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };
    let (session, _) = Session::start(
        fixture.launch,
        fixture.mapping,
        &[Breakpoint {
            line: SUBTOTAL_LINE,
            condition: None,
            hit_condition: None,
            log_message: None,
        }],
    )
    .await
    .expect("starts");

    let stopped = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .unwrap()
        .unwrap();
    let thread = stopped.thread_id;

    // Step forward twice, then jump back to where we started.
    session.step(Step::Over, thread, None).await.unwrap();
    session
        .wait_for_stop(Duration::from_secs(20))
        .await
        .unwrap()
        .expect("stops");
    let forward = session.stack(thread).await.unwrap()[0].line;
    assert_eq!(forward, Some(SUBTOTAL_LINE + 1));

    if !session.capabilities().goto_targets {
        eprintln!("NOTE: this adapter cannot move the instruction pointer either");
        session.shutdown().await;
        return;
    }

    session
        .jump_to(SUBTOTAL_LINE, thread)
        .await
        .expect("jumps back");
    session
        .wait_for_stop(Duration::from_secs(20))
        .await
        .unwrap()
        .expect("stops after the jump");
    let back = session.stack(thread).await.unwrap()[0].line;
    assert_eq!(
        back,
        Some(SUBTOTAL_LINE),
        "the jump did not land on the line asked for"
    );

    // And the frame is still live: its variables are intact, which is what
    // separates a jump from a restart.
    let frame = session.stack(thread).await.unwrap()[0].id;
    let quantity = session
        .evaluate("quantity", Some(frame), "watch")
        .await
        .unwrap();
    assert_eq!(quantity.value, "2");
    eprintln!("OK jump: {forward:?} -> {back:?}, locals intact");

    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_conditional_breakpoint_stops_only_when_it_should() {
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };
    // The third line item is the only one with quantity 3.
    let (session, statuses) = Session::start(
        fixture.launch,
        fixture.mapping,
        &[Breakpoint {
            line: SUBTOTAL_LINE,
            condition: Some("quantity == 3".into()),
            hit_condition: None,
            log_message: None,
        }],
    )
    .await
    .expect("starts");
    assert_eq!(statuses[0].state, BindState::Bound);

    let stopped = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .unwrap()
        .expect("stops once");
    let frame = session.stack(stopped.thread_id).await.unwrap()[0].id;
    let quantity = session
        .evaluate("quantity", Some(frame), "watch")
        .await
        .unwrap();
    assert_eq!(
        quantity.value, "3",
        "the condition did not hold where it stopped"
    );

    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_breakpoint_on_prose_is_refused_with_a_reason() {
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };
    // Prose generates no code. Reporting it as unverified with an
    // explanation beats sending it to the adapter and relaying a bare
    // `false`.
    let (session, statuses) = Session::start(
        fixture.launch,
        fixture.mapping,
        &[Breakpoint {
            line: PROSE_LINE,
            condition: None,
            hit_condition: None,
            log_message: None,
        }],
    )
    .await
    .expect("starts");

    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].line, PROSE_LINE);
    // Refused, not merely unconfirmed: hick made this call itself, before
    // any adapter was asked, and no `breakpoint` event can overturn it.
    assert_eq!(statuses[0].state, BindState::Refused);
    assert!(
        statuses[0]
            .message
            .as_deref()
            .is_some_and(|m| m.contains("prose")),
        "no explanation: {statuses:?}"
    );

    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_adapters_own_capabilities_decide_what_is_offered() {
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };
    let (session, _) = Session::start(fixture.launch, fixture.mapping, &[])
        .await
        .expect("starts");
    let caps = session.capabilities();
    // debugpy has these; the point is that they are read from the adapter
    // rather than assumed, so a different adapter greys out what it lacks.
    assert!(
        caps.conditional_breakpoints,
        "debugpy should support conditions"
    );
    eprintln!(
        "capabilities: conditional={} logpoints={} restart_frame={} step_back={} set_variable={}",
        caps.conditional_breakpoints,
        caps.log_points,
        caps.restart_frame,
        caps.step_back,
        caps.set_variable
    );
    // Nobody has real reverse execution here, and the UI must not pretend.
    assert!(
        !caps.step_back,
        "if this ever passes, enable the reverse controls"
    );
    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stack_shows_where_the_program_is_beyond_the_document() {
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };
    let (session, _) = Session::start(
        fixture.launch,
        fixture.mapping,
        &[Breakpoint {
            line: CART_TOTAL_LINE + 1,
            condition: None,
            hit_condition: None,
            log_message: None,
        }],
    )
    .await
    .expect("starts");

    let stopped = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .unwrap()
        .expect("stops");
    let stack = session.stack(stopped.thread_id).await.unwrap();
    // Every frame is either placed in the document or explicitly not, and a
    // frame that is not must still name its file so the UI can show it
    // read-only rather than blank.
    for frame in &stack {
        if !frame.in_document {
            assert!(
                frame.source.is_some() || frame.name != "?",
                "an anonymous frame: {frame:?}"
            );
        } else {
            assert!(frame.line.is_some());
        }
    }
    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn running_to_a_line_keeps_the_breakpoints_a_person_set() {
    // `setBreakpoints` replaces every breakpoint in a file, so a one-shot
    // breakpoint sent on its own deletes the ones in the gutter — and leaves
    // its own armed, so a later `continue` stops somewhere nobody asked for.
    // Both were real.
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };
    let (session, statuses) = Session::start(
        fixture.launch,
        fixture.mapping.clone(),
        &[Breakpoint {
            line: SUBTOTAL_LINE,
            condition: None,
            hit_condition: None,
            log_message: None,
        }],
    )
    .await
    .expect("the session starts");
    assert_eq!(statuses[0].state, BindState::Bound);
    let stopped = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .unwrap()
        .expect("stops at the breakpoint");

    // Run to the line after it, in the same frame.
    session
        .run_to(SUBTOTAL_LINE + 1, stopped.thread_id)
        .await
        .expect("runs to the cursor");
    let at = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .unwrap()
        .expect("stops at the cursor");
    let frames = session.stack(at.thread_id).await.unwrap();
    assert_eq!(frames[0].line, Some(SUBTOTAL_LINE + 1));

    // Continue: the next stop must be the breakpoint a person set, on the
    // next call — not the cursor line again.
    session
        .step(Step::Continue, at.thread_id, None)
        .await
        .unwrap();
    let next = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .unwrap()
        .expect("stops again");
    let frames = session.stack(next.thread_id).await.unwrap();
    assert_eq!(
        frames[0].line,
        Some(SUBTOTAL_LINE),
        "the run-to breakpoint was left armed, or the real one was deleted"
    );
    session.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_program_that_runs_to_completion_reports_its_exit() {
    // Protects docs/guarantees — the app reaps a session the moment its
    // program ends, and it can only say HOW it ended if the session recorded
    // the adapter's `exited`/`terminated` rather than merely going quiet.
    let Some(fixture) = fixture() else {
        skip("no Python debug adapter on this machine");
        return;
    };

    // No breakpoints: launched, the program runs straight to the end.
    let (session, statuses) = Session::start(fixture.launch, fixture.mapping, &[])
        .await
        .expect("the session starts");
    assert!(statuses.is_empty());

    let stopped = session
        .wait_for_stop(Duration::from_secs(30))
        .await
        .expect("waiting works");
    assert!(
        stopped.is_none(),
        "with no breakpoints the program should have run to the end: {stopped:?}"
    );

    // The end was recorded, with the code debugpy reported.
    let exit = session.exit().expect("the end of the program was recorded");
    assert_eq!(exit.code, Some(0), "a clean exit reads as code 0: {exit:?}");

    // Shutting down after the program already ended, twice: the automatic
    // reap and an explicit stop legitimately race, and the loser must be a
    // no-op rather than a hang or a panic.
    session.shutdown().await;
    session.shutdown().await;
}
