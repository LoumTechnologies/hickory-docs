<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Conversation edits use the client's review policy

For: Authors using an agent to change documents while retaining review control.

Given a built-in or ACP conversation, when the user chooses **Review** or **Auto-accept**
under **Document edits**, then that choice belongs to this conversation and is
restored when its saved conversation is reopened. New conversations start with
Review. Changing it during a turn is refused.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-82b78a4e5237"></a>
## `apps/web/src/lib/diff.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Conversation edits use the client's review policy](contract.md).

<a id="f-82b78a4e5237-roundTrip"></a>
### `roundTrip`

<storycode:copy id="f-82b78a4e5237-roundTrip">
import { describe, expect, it } from "vitest";
import { applyEdits, computeEdits, toByteEdits } from "./diff";
import { byteLength } from "./offsets";

function roundTrip(oldText: string, newText: string) {
  const edits = computeEdits(oldText, newText);
  expect(applyEdits(oldText, edits)).toBe(newText);
  return edits;
}

</storycode:copy>
<a id="f-82b78a4e5237-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-82b78a4e5237-expression_statement">
describe("computeEdits", () => {
  it("returns no edits for identical text", () => {
    expect(computeEdits("a\nb\n", "a\nb\n")).toEqual([]);
  });

  it("produces one tight edit for a single-line change", () => {
    const edits = roundTrip("a\nhello world\nb\n", "a\nhello there\nb\n");
    expect(edits).toHaveLength(1);
    expect(edits[0]).toEqual({ start: 8, end: 13, text: "there" });
  });

  it("produces separate edits for changes in distant lines", () => {
    const oldText = "one\ntwo\nthree\nfour\nfive\n";
    const newText = "ONE\ntwo\nthree\nfour\nFIVE\n";
    const edits = roundTrip(oldText, newText);
    expect(edits).toHaveLength(2);
    expect(edits[0].text).toBe("ONE");
    expect(edits[1].text).toBe("FIVE");
  });

  it("handles pure insertion and pure deletion", () => {
    const ins = roundTrip("a\nc\n", "a\nb\nc\n");
    expect(ins).toHaveLength(1);
    expect(ins[0].start).toBe(ins[0].end);

    const del = roundTrip("a\nb\nc\n", "a\nc\n");
    expect(del).toHaveLength(1);
    expect(del[0].text).toBe("");
  });

  it("handles append at end without trailing newline", () => {
    roundTrip("a\nb", "a\nb\nc");
    roundTrip("", "content\n");
    roundTrip("content\n", "");
  });

  it("keeps unchanged interior lines out of the edits", () => {
    const oldText = "h1\nsame\nsame\nold\nsame\n";
    const newText = "h1\nsame\nsame\nnew\nsame\n";
    const edits = roundTrip(oldText, newText);
    expect(edits).toHaveLength(1);
    expect(oldText.slice(edits[0].start, edits[0].end)).toBe("old");
  });

  it("toByteEdits converts char offsets to UTF-8 byte offsets", () => {
    const oldText = "naïve\nline\n"; // ï is 2 bytes in UTF-8
    const newText = "naïve\nLine\n";
    const edits = computeEdits(oldText, newText);
    expect(edits).toHaveLength(1);
    const bytes = toByteEdits(oldText, edits);
    // char offset of "l" is 6; byte offset is 7 (ï = 2 bytes).
    expect(edits[0].start).toBe(6);
    expect(bytes[0].start).toBe(7);
    expect(byteLength(oldText)).toBe(oldText.length + 1);
  });

  it("survives emoji (surrogate pairs) in the buffer", () => {
    const oldText = "x 🎉 y\n";
    const newText = "x 🎉 z\n";
    const edits = roundTrip(oldText, newText);
    const bytes = toByteEdits(oldText, edits);
    expect(bytes[0].start).toBe(byteLength("x 🎉 "));
  });
});

</storycode:copy>
<a id="f-82b78a4e5237-import_statement"></a>
### `import_statement`

<storycode:copy id="f-82b78a4e5237-import_statement">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
import { unifiedDiff } from "./diff";
</storycode:copy>
<a id="f-82b78a4e5237-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-82b78a4e5237-expression_statement-2">
it("shows a final-newline-only edit and an added empty line", () => {
  expect(unifiedDiff("note.md", "note", "note\n")).toContain("-note\n\\ No newline at end of file\n+note");
  expect(unifiedDiff("note.md", "", "\n")).toContain("@@ -0,0 +1,1 @@\n+\n");
});
</storycode:copy>

### Assemble `apps/web/src/lib/diff.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/diff.test.ts"><storycode:paste select="#f-82b78a4e5237-roundTrip"/><storycode:paste select="#f-82b78a4e5237-expression_statement"/><storycode:paste select="#f-82b78a4e5237-import_statement"/><storycode:paste select="#f-82b78a4e5237-expression_statement-2"/></storycode:file>

## Test the behavior: `pending_change`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_acp.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2fce65a349e6-pending_change"></a>
### `pending_change`

<storycode:copy id="f-2fce65a349e6-pending_change">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
async fn pending_change(app: &App) -> Value {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let (_, state) = app.request("GET", &app.route("/acp"), json!({})).await;
            if let Some(change) = state["edits"]["changes"].as_array().and_then(|cs| {
                cs.iter()
                    .find(|c| c["status"] == "pending" || c["status"] == "applying")
            }) {
                return change.clone();
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap()
}

</storycode:copy>

## Test the behavior: `acp_untitled_edits_use_tools_and_wait_for_client_review`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_acp.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2fce65a349e6-acp_untitled_edits_use_tools_and_wait_for_client_review"></a>
### `acp_untitled_edits_use_tools_and_wait_for_client_review`

<storycode:copy id="f-2fce65a349e6-acp_untitled_edits_use_tools_and_wait_for_client_review">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
async fn acp_untitled_edits_use_tools_and_wait_for_client_review() {
    let (_dir, mut app) = fixture(vec![]).await;
    app.doc = "workspace".into();
    let ready = app.connect(None).await;
    assert_eq!(ready["edits"]["mode"], "review");
    for (accepted, parent) in [(false, None), (true, Some("previous"))] {
        let parent = if parent.is_some() {
            let (_, turns) = app.request("GET", &app.route("/turns"), json!({})).await;
            Some(turns["turns"][0]["id"].as_str().unwrap().to_string())
        } else {
            None
        };
        let (status, response) = app.request("POST", &app.route(""), json!({
            "backend":"fixture", "prompt":"buffer-edit", "parent_id":parent,
            "context":{"buffers":[{"id":"untitled-tab","kind":"untitled","name":"Untitled 1","path":null,"content":"# Original 📝\n","focused":true}]}
        })).await;
        assert_eq!(status, 202, "{response}");
        let change = pending_change(&app).await;
        // A remounted Agent pane can recover its active connection without
        // blocking the review decision that lets this turn finish.
        let resumed = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            app.connect(ready["session"].as_str()),
        )
        .await
        .expect("reconnecting during review must not wait for the turn");
        assert_eq!(
            resumed["edits"]["changes"]
                .as_array()
                .unwrap()
                .last()
                .unwrap()["id"],
            change["id"]
        );
        assert_eq!(change["oldText"], "# Original 📝\n");
        assert_eq!(change["newText"], "# Changed by ACP 📝\n");
        assert_eq!(change["name"], "Untitled 1");
        assert_eq!(change["buffer"], "untitled-tab");
        assert!(change["path"].is_null());
        assert!(!app.root.join("Untitled 1").exists());
        assert_eq!(
            std::fs::read_to_string(app.root.join("demo.md")).unwrap(),
            DOC
        );
        let (status, result) = app
            .request(
                "POST",
                &app.route("/acp/edits"),
                json!({"id":change["id"],"accepted":accepted}),
            )
            .await;
        assert_eq!(status, 200, "{result}");
        let turn = app.wait(response["session_id"].as_str().unwrap()).await;
        assert_eq!(turn["status"], "ok", "{turn}");
        assert!(turn["answer"].as_str().unwrap().contains(if accepted {
            "Edited the live buffer"
        } else {
            "rejected this edit"
        }));
        let (status, _) = app
            .request(
                "POST",
                &app.route("/acp/edits"),
                json!({"id":change["id"],"accepted":true}),
            )
            .await;
        assert_eq!(status, 422); // Decisions are consumed once.
    }
}

</storycode:copy>

## Test the behavior: `acp_auto_accept_is_conversation_scoped_and_restored`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_acp.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2fce65a349e6-acp_auto_accept_is_conversation_scoped_and_restored"></a>
### `acp_auto_accept_is_conversation_scoped_and_restored`

<storycode:copy id="f-2fce65a349e6-acp_auto_accept_is_conversation_scoped_and_restored">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
async fn acp_auto_accept_is_conversation_scoped_and_restored() {
    let (_dir, mut app) = fixture(vec![]).await;
    app.doc = "workspace".into();
    app.connect(None).await;
    let (status, state) = app
        .request(
            "POST",
            &app.route("/acp/edits"),
            json!({"mode":"auto-accept"}),
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(state["edits"]["mode"], "auto-accept");
    let (_, response) = app.request("POST", &app.route(""), json!({"backend":"fixture","prompt":"buffer-edit","context":{"buffers":[{"id":"tab","name":"Untitled","path":null,"content":"original\n","focused":true}]}})).await;
    let change = pending_change(&app).await;
    assert_eq!(change["status"], "applying");
    app.request(
        "POST",
        &app.route("/acp/edits"),
        json!({"id":change["id"],"accepted":true}),
    )
    .await;
    let turn = app.wait(response["session_id"].as_str().unwrap()).await;
    let session = turn["session"].as_str().unwrap().to_string();
    let root = app.root.clone();
    drop(app);
    let mut app = start(&root).await;
    app.doc = "workspace".into();
    assert_eq!(
        app.connect(Some(&session)).await["edits"]["mode"],
        "auto-accept"
    );
    // An independent conversation gets its own default.
    app.doc = app.state.index.sole().unwrap().0;
    assert_eq!(app.connect(None).await["edits"]["mode"], "review");
}

</storycode:copy>

## Test the behavior: `acp_existing_output_tool_waits_for_review_before_writing`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_acp.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2fce65a349e6-acp_existing_output_tool_waits_for_review_before_writing"></a>
### `acp_existing_output_tool_waits_for_review_before_writing`

<storycode:copy id="f-2fce65a349e6-acp_existing_output_tool_waits_for_review_before_writing">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
async fn acp_existing_output_tool_waits_for_review_before_writing() {
    let (_dir, app) = fixture(vec![]).await;
    app.state.rooms.get_or_create(&app.doc).await.unwrap();
    app.connect(None).await;
    let id = app.send("edit", None).await;
    let change = pending_change(&app).await;
    assert_eq!(change["editor"], false);
    assert!(change["newText"].as_str().unwrap().contains("ACP"));
    assert_eq!(
        std::fs::read_to_string(app.root.join("demo.md")).unwrap(),
        DOC
    );
    app.request(
        "POST",
        &app.route("/acp/edits"),
        json!({"id":change["id"],"accepted":true}),
    )
    .await;
    assert_eq!(app.wait(&id).await["status"], "ok");
    assert!(
        std::fs::read_to_string(app.root.join("demo.md"))
            .unwrap()
            .contains("println!(\"ACP\")")
    );
}

</storycode:copy>

## Test the behavior: `stopping_a_review_cancels_the_edit`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_acp.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2fce65a349e6-stopping_a_review_cancels_the_edit"></a>
### `stopping_a_review_cancels_the_edit`

<storycode:copy id="f-2fce65a349e6-stopping_a_review_cancels_the_edit">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
async fn stopping_a_review_cancels_the_edit() {
    let (_dir, mut app) = fixture(vec![]).await;
    app.doc = "workspace".into();
    app.connect(None).await;
    let (_, response) = app.request("POST", &app.route(""), json!({"backend":"fixture","prompt":"buffer-edit","context":{"buffers":[{"id":"tab","name":"Untitled","path":null,"content":"original\n","focused":true}]}})).await;
    let change = pending_change(&app).await;
    app.request("POST", &app.route("/stop"), json!({})).await;
    assert_eq!(
        app.wait(response["session_id"].as_str().unwrap()).await["status"],
        "stopped"
    );
    let (_, state) = app.request("GET", &app.route("/acp"), json!({})).await;
    assert_eq!(state["edits"]["changes"][0]["status"], "rejected");
    assert_eq!(
        std::fs::read_to_string(app.root.join("demo.md")).unwrap(),
        DOC
    );
    assert_eq!(
        app.request(
            "POST",
            &app.route("/acp/edits"),
            json!({"id":change["id"],"accepted":true})
        )
        .await
        .0,
        422
    );
}

</storycode:copy>

## Test the behavior: `live_codex_edits_current_note_through_review`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_acp.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2fce65a349e6-live_codex_edits_current_note_through_review"></a>
### `live_codex_edits_current_note_through_review`

<storycode:copy id="f-2fce65a349e6-live_codex_edits_current_note_through_review">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
#[ignore = "uses the installed authenticated Codex ACP adapter and a real model turn"]
async fn live_codex_edits_current_note_through_review() {
    let command = std::env::var("HICKORY_ACP_LIVE_COMMAND").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let before = "# Original heading\n\nKeep this paragraph unchanged.\n";
    std::fs::write(root.join("demo.md"), before).unwrap();
    let mut app = start(&root).await;
    app.doc = "workspace".into();
    app.request(
        "PUT",
        "/api/agents",
        json!([{"id":"fixture","name":"Codex live","command":command,"args":[]}]),
    )
    .await;
    let connected = app.connect(None).await;
    assert_eq!(connected["ready"], true, "{connected}");
    let (_, response) = app.request("POST", &app.route(""), json!({"backend":"fixture","prompt":"Change the heading in the current document to Meeting notes. Keep its paragraph unchanged.","context":{"buffers":[{"id":"note-tab","kind":"document","name":"demo.md","path":"demo.md","content":before,"focused":true}]}})).await;
    let change = tokio::time::timeout(Duration::from_secs(90), async {
        loop {
            let (_, state) = app.request("GET", &app.route("/acp"), json!({})).await;
            if let Some(change) = state["edits"]["changes"]
                .as_array()
                .and_then(|cs| cs.iter().find(|c| c["status"] == "pending"))
            {
                break change.clone();
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    })
    .await
    .expect("Codex should submit a tool edit rather than printing the replacement document");
    assert_eq!(change["oldText"], before);
    let after = "# Meeting notes\n\nKeep this paragraph unchanged.\n";
    assert_eq!(change["newText"], after, "{change}");
    assert_eq!(
        std::fs::read_to_string(root.join("demo.md")).unwrap(),
        before
    );
    // The real UI uses its editor transaction. Here the HTTP client applies
    // the approved bytes through the public document route before acknowledging.
    let doc = app.state.index.sole().unwrap().0;
    app.request("PUT", &format!("/api/docs/{doc}"), json!({"source":after}))
        .await;
    app.request(
        "POST",
        &app.route("/acp/edits"),
        json!({"id":change["id"],"accepted":true}),
    )
    .await;
    let turn = app.wait(response["session_id"].as_str().unwrap()).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    assert_eq!(
        std::fs::read_to_string(root.join("demo.md")).unwrap(),
        after
    );
    println!(
        "Live Codex current-note tool edit reviewed and accepted: {}",
        turn["answer"]
    );
}
</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-agent/src/tools/preview.rs](../../../mechanisms/crates/hickory-agent/src/tools/preview.rs.md#output-256c54fefd54)
- [crates/hickory-cli/src/mcp_preview.rs](../../../mechanisms/crates/hickory-cli/src/mcp_preview.rs.md#output-4382c2bba873)
- [crates/hickory-cli/src/serve/acp/edits.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/edits.rs.md#output-076ee922e67a)
- [crates/hickory-cli/src/serve/agent_review.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent_review.rs.md#output-ee38c84c9207)

### Tests and fixtures

- [apps/web/e2e/python-agent.spec.ts](../../../mechanisms/apps/web/e2e/python-agent.spec.ts.md#output-c40e3ff7e23b)
- [apps/web/src/components/AgentChanges.test.tsx](../../../mechanisms/apps/web/src/components/AgentChanges.tsx.md#output-7ab5cdb69360)
- [apps/web/src/lib/agentEdit.test.ts](../../../mechanisms/apps/web/src/lib/agentEdit.ts.md#output-ebba6809ee8c)
- [apps/web/src/lib/diff.test.ts](contract.md#output-82b78a4e5237)
- [crates/hickory-cli/tests/serve_acp.rs](../../../mechanisms/crates/hickory-cli/tests/serve_acp.rs.md#output-2fce65a349e6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
