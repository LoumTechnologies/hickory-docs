//! The agent works in the DOCUMENT's container, not a side room.
//!
//! The point of an executable document is that its cells declare an
//! environment. An agent that runs its scripts somewhere else is reasoning
//! about a machine it cannot touch: it writes a file, the cell that should
//! consume the file does not see it, and the two surfaces silently disagree.
//!
//! This drives a real agent session against a canned LLM: the agent's script
//! writes a file, then `verify` executes the document — and the document's own
//! `hick:exec` cell reads that file back. The expectation is pinned exactly, so
//! the cell only passes if it ran in the same room the script did.
//!
//! This runs on Windows now, and that is the point of it. It could not
//! before: `run_script` wrote every block with `mkdir -p … && cat > …` and ran
//! it under `if command -v timeout …; else set -m; … kill -TERM -$pid …; fi` —
//! POSIX shell handed to `cmd.exe /C` — so an agent could execute nothing at
//! all there (#19). The block is now WRITTEN rather than composed, and the
//! limit is the executor's own.
//!
//! It asserts a side effect rather than output: the agent leaves a file and a
//! CELL reads it back, with the expectation pinned exactly. A block that
//! silently ran nothing would still produce a plausible empty observation, but
//! it cannot leave a file for something else to find.

use std::sync::Arc;

use hickory_agent::{AgentConfig, AgentEvent, ScriptedLlmClient, run_agent};
use hickory_executor::{Executor, LocalExecutor};

/// The cell reads a file the document never creates — only the agent does.
///
/// `type` on Windows, `cat` elsewhere: the cell gets that platform's shell.
/// The expectation is LF on both because captured output is recorded with LF
/// line endings everywhere (#18), which is what lets one exact expectation
/// mean the same thing on either.
fn doc() -> String {
    let read = if cfg!(windows) {
        "type handoff.txt"
    } else {
        "cat handoff.txt"
    };
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Shared room

<hick:container name="workshop" image="host" />
<hick:exec container="workshop">
{read}
<hick:expect match="exact">written by the agent
</hick:expect>
</hick:exec>
</hick:doc>
"##
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn the_agent_and_the_document_cells_share_a_container() {
    let project = tempfile::tempdir().unwrap();
    let doc_path = project.path().join("doc.hick");
    std::fs::write(&doc_path, doc()).unwrap();

    let llm = ScriptedLlmClient::new(vec![
        // (a) write the file the document's cell expects to find.
        format!(
            "<hick:next>code</hick:next>\nLeaving the handoff file for the cell.\n\
             ```bash\n{}\n```",
            // No space before `>`: cmd would write it into the file.
            if cfg!(windows) {
                "echo written by the agent>handoff.txt"
            } else {
                "printf 'written by the agent\\n' > handoff.txt"
            }
        ),
        // (b) run the document. The cell must see the file.
        "<hick:next>tool</hick:next>\nNow verify.\n\
         <hick:tool name=\"verify\">\n</hick:tool>"
            .to_string(),
        "<hick:next>done</hick:next>\nThe cell read the file I wrote.".to_string(),
    ]);

    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new().unwrap());
    let mut config = AgentConfig::new("Leave a handoff file for the cell.", project.path());
    config.doc_path = Some(doc_path.clone());
    // Exactly what the server derives from the document's first declaration.
    config.container = "workshop".to_string();
    config.image = "host".to_string();

    let mut events = Vec::new();
    let mut on_event = |e: AgentEvent| events.push(e);
    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event)
        .await
        .expect("agent session");
    executor.shutdown().await.ok();

    // `verify` runs the pipeline through the SAME executor, so the cell's
    // pinned expectation is the assertion: it passes only if `cat handoff.txt`
    // found the file the agent's script wrote.
    let session = std::fs::read_to_string(&outcome.session_path).unwrap();
    assert!(
        session.contains("PASS") && session.contains("1 expectation(s) met"),
        "the cell never saw the agent's file — they ran in different rooms.\n\
         In separate rooms `cat handoff.txt` fails and the pinned expectation \
         cannot match.\n{session}"
    );

    // And the transcript proves WHERE the script ran: the document's container,
    // not the standalone agent room.
    let transcripts = executor.transcripts();
    assert!(
        transcripts.contains_key("workshop"),
        "the agent ran nothing in the document's container: {:?}",
        transcripts.keys().collect::<Vec<_>>()
    );
    assert!(
        !transcripts.contains_key(hickory_agent::AGENT_CONTAINER),
        "the agent still opened its own side room: {:?}",
        transcripts.keys().collect::<Vec<_>>()
    );
}
