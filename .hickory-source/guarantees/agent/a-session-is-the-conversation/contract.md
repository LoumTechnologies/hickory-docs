<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Read the retained branch of an agent conversation

For: Authors using an agent to change documents while retaining review control.

Reopen a conversation after changing branches. The recorded selected branch must remain available; it must not be reconstructed from a model summary.

## What must happen

The session retains prompts, responses, actions, observations and branch relationships as document evidence. The selected branch determines the conversation reading. Python owns current conversation/context construction; Rust provides supervision, tool boundaries, transport and recording. Abandoned work remains historical rather than silently becoming current context.

## Limits of the evidence

Opaque provider delivery and hidden model state are not reconstructed by a retained session.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-session-is-the-conversation.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Apply the Python-runtime amendment. Retain the existing session and branch vocabulary; retire references to the removed Rust ReAct loop.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-d6c33a544e5e"></a>
## `apps/web/src/components/ChatTree.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Read the retained branch of an agent conversation](contract.md).

<a id="f-d6c33a544e5e-ROW"></a>
### `ROW`

The conversation zoomed out: every turn as a node, each joined to the turn
it continued from, newest at the top — the same lane layout the git pane
uses for commits, because a conversation with rewinds IS a small DAG. The
highlighted path is the branch the dock is showing; clicking a node makes
it the tip (a rewind), which is how you go back to where you branched off.

<storycode:copy id="f-d6c33a544e5e-ROW">
// The conversation zoomed out: every turn as a node, each joined to the turn
// it continued from, newest at the top — the same lane layout the git pane
// uses for commits, because a conversation with rewinds IS a small DAG. The
// highlighted path is the branch the dock is showing; clicking a node makes
// it the tip (a rewind), which is how you go back to where you branched off.

import { useMemo } from "react";
import type { AgentTurn } from "../api/types";
import { graphWidth, laneColor, layout } from "../lib/gitGraph";

const ROW = 30;
</storycode:copy>
<a id="f-d6c33a544e5e-LANE"></a>
### `LANE`

<storycode:copy id="f-d6c33a544e5e-LANE">
const LANE = 14;
</storycode:copy>
<a id="f-d6c33a544e5e-x"></a>
### `x`

<storycode:copy id="f-d6c33a544e5e-x">
const x = (lane: number) => LANE * (lane + 0.5);
</storycode:copy>
<a id="f-d6c33a544e5e-edgePath"></a>
### `edgePath`

<storycode:copy id="f-d6c33a544e5e-edgePath">
function edgePath(from: number, to: number): string {
  const x0 = x(from);
  const x1 = x(to);
  if (from === to) return `M ${x0} 0 L ${x0} ${ROW}`;
  const mid = ROW / 2;
  return `M ${x0} 0 C ${x0} ${mid}, ${x1} ${mid}, ${x1} ${ROW}`;
}

</storycode:copy>
<a id="f-d6c33a544e5e-pathTo"></a>
### `pathTo`

The ids on the path from the root to `tip`, as a set.

<storycode:copy id="f-d6c33a544e5e-pathTo">
/** The ids on the path from the root to `tip`, as a set. */
export function pathTo(
  turns: readonly AgentTurn[],
  tip: string | null,
): Set<string> {
  const out = new Set<string>();
  let cursor = tip;
  const byId = new Map(turns.map((t) => [t.id, t]));
  while (cursor && byId.has(cursor) && !out.has(cursor)) {
    out.add(cursor);
    cursor = byId.get(cursor)!.parent_id;
  }
  return out;
}

</storycode:copy>
<a id="f-d6c33a544e5e-ChatTree"></a>
### `ChatTree`

<storycode:copy id="f-d6c33a544e5e-ChatTree">
export function ChatTree({
  turns,
  tip,
  onSelect,
}: {
  turns: readonly AgentTurn[];
  tip: string | null;
  onSelect: (id: string) => void;
}) {
  // gitGraph wants newest first, each node naming its parents.
  const ordered = useMemo(() => [...turns].reverse(), [turns]);
  const rows = useMemo(
    () =>
      layout(
        ordered.map((t) => ({
          sha: t.id,
          parents: t.parent_id ? [t.parent_id] : [],
        })),
      ),
    [ordered],
  );
  const onPath = useMemo(() => pathTo(turns, tip), [turns, tip]);
  const graphPx = (graphWidth(rows) + 1) * LANE;
  if (turns.length === 0)
    return (
      <p className="muted chat-empty">No turns yet — nothing to zoom out of.</p>
    );
  return (
    <ol
      className="chat-tree"
      style={{ ["--chat-graph" as string]: `${graphPx}px` }}
      aria-label="Conversation tree"
    >
      {ordered.map((turn, i) => {
        const row = rows[i];
        const here = turn.id === tip;
        const lit = onPath.has(turn.id);
        return (
          <li
            key={turn.id}
            className={`chat-tree__row${here ? " chat-tree__row--tip" : ""}${lit ? " chat-tree__row--on" : ""}`}
          >
            <svg
              className="chat-tree__graph"
              width={graphPx}
              height={ROW}
              aria-hidden="true"
            >
              {row.through.map((line, j) => (
                <path
                  key={j}
                  className={`git-edge git-edge--c${laneColor(line.from)}`}
                  d={edgePath(line.from, line.to)}
                />
              ))}
              <circle
                className={`git-node git-node--c${laneColor(row.lane)}${here ? " chat-tree__node--tip" : ""}`}
                cx={x(row.lane)}
                cy={ROW / 2}
                r={here ? 5.5 : 4}
              />
            </svg>
            <button
              type="button"
              className="chat-tree__turn"
              onClick={() => onSelect(turn.id)}
              data-tip={
                here
                  ? "The current tip"
                  : "Make this the tip — continue from here"
              }
            >
              <span className="chat-tree__prompt">
                {turn.prompt.length > 80
                  ? `${turn.prompt.slice(0, 80)}…`
                  : turn.prompt}
              </span>
              <span className="chat-tree__meta muted">
                {turn.status === "running"
                  ? "running"
                  : turn.status === "error"
                    ? "failed"
                    : turn.model}
              </span>
            </button>
          </li>
        );
      })}
    </ol>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/ChatTree.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ChatTree.tsx"><storycode:paste select="#f-d6c33a544e5e-ROW"/><storycode:paste select="#f-d6c33a544e5e-LANE"/><storycode:paste select="#f-d6c33a544e5e-x"/><storycode:paste select="#f-d6c33a544e5e-edgePath"/><storycode:paste select="#f-d6c33a544e5e-pathTo"/><storycode:paste select="#f-d6c33a544e5e-ChatTree"/></storycode:file>

<a id="output-ed3dc6749f3b"></a>
## `apps/web/src/editor/session.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Read the retained branch of an agent conversation](contract.md).

<a id="f-ed3dc6749f3b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ed3dc6749f3b-expression_statement">
// @vitest-environment jsdom
import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { DocumentEditor } from "./DocumentEditor";
import { LocalRealtime } from "../api/realtime";
import { parseHickDoc, languageForBlock } from "./hickDoc";

afterEach(cleanup);

</storycode:copy>
<a id="f-ed3dc6749f3b-SESSION"></a>
### `SESSION`

A hick:session document: the conversation, as the file the agent wrote.

<storycode:copy id="f-ed3dc6749f3b-SESSION">
// A hick:session document: the conversation, as the file the agent wrote.
const SESSION = `<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:user>Make the greeting friendlier.</hick:user>
<hick:assistant>Writing the file now.
<hick:action lang="bash">
printf 'hello' > out.txt
</hick:action>
</hick:assistant>
<hick:observation source="action-0" exit="0">done
</hick:observation>
</hick:session>
`;

</storycode:copy>
<a id="f-ed3dc6749f3b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ed3dc6749f3b-expression_statement-2">
describe("a session document renders as a conversation", () => {
  it("frames each turn and labels who is speaking", async () => {
    const realtime = new LocalRealtime();
    const { container } = render(
      <DocumentEditor
        docId="s1"
        initialSource={SESSION}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() =>
      expect(container.querySelector(".cm-content")?.textContent).toContain(
        "Make the greeting friendlier.",
      ),
    );

    const chips = [...container.querySelectorAll(".cm-hick-banner-label")].map(
      (e) => e.textContent,
    );
    expect(chips).toEqual(expect.arrayContaining(["you", "agent", "ran", "output"]));

    // The turn bodies are framed, so the conversation reads as one.
    expect(container.querySelector(".cm-turn-user")).toBeTruthy();
    expect(container.querySelector(".cm-turn-assistant")).toBeTruthy();
    expect(container.querySelector(".cm-turn-action")).toBeTruthy();
    expect(container.querySelector(".cm-turn-observation")).toBeTruthy();

    // The exit status is on the chip: a failed action must be visible without
    // reading the raw attribute.
    const details = [...container.querySelectorAll(".cm-hick-banner-detail")].map(
      (e) => e.textContent,
    );
    expect(details).toContain("exit 0");

    // Every byte is still the document's own text — nothing was replaced.
    expect(container.querySelector(".cm-content")?.textContent).toContain("<hick:action");

    // Top-level turns are bubbles — both voices on the left — and a nested action is framed inside
    // its bubble, not bubbled.
    // docs/guarantees/agent/a-session-is-the-conversation.md
    const you = container.querySelector(".cm-turn-user.cm-bubble-you");
    expect(you?.classList.contains("cm-bubble-first")).toBe(true);
    expect(container.querySelector(".cm-turn-assistant.cm-bubble-agent")).toBeTruthy();
    expect(container.querySelector(".cm-turn-observation.cm-bubble-agent")).toBeTruthy();
    expect(container.querySelector(".cm-turn-action.cm-bubble-agent")).toBeTruthy();
    expect(container.querySelector(".cm-turn-action.cm-bubble-you")).toBeNull();
    // The opening tag line of a bubble is its first line; the closing tag
    // line its last — the tags are inside the bubble, not around it.
    const lines = [...container.querySelectorAll(".cm-line")];
    // (The chip leads the line now, so the tag is inside the text, not at
    // its start.)
    const userOpen = lines.find((l) => l.textContent?.includes("<hick:user>"));
    expect(userOpen?.classList.contains("cm-bubble-first")).toBe(true);
    expect(userOpen?.classList.contains("cm-bubble-last")).toBe(true);
    const obsClose = lines.find((l) => l.textContent?.includes("</hick:observation>"));
    expect(obsClose?.classList.contains("cm-bubble-last")).toBe(true);
    // The chip is the bubble's caption: it leads the opening line.
    expect(userOpen?.textContent?.indexOf("you")).toBeLessThan(
      userOpen?.textContent?.indexOf("<hick:user>") ?? -1,
    );

    // The work opens folded: the action's script is hidden behind a
    // placeholder, its opening and closing tag lines still on screen.
    await waitFor(() =>
      expect(container.querySelector(".cm-hick-fold-placeholder")).toBeTruthy(),
    );
    const shown = container.querySelector(".cm-content")?.textContent ?? "";
    expect(shown).toContain("<hick:action");
    expect(shown).toContain("</hick:action>");
    expect(shown).not.toContain("printf 'hello'");
    // What was SAID is not folded.
    expect(shown).toContain("Writing the file now.");
    realtime.close();
  });

  it("treats an agent's action as code, not transcript", () => {
    const structure = parseHickDoc(SESSION);
    const action = structure.blocks.find((b) => b.name === "action")!;
    expect(languageForBlock(structure, action)).toBe("shellscript");
  });

  it("shows a failed observation as failed", () => {
    const failed = SESSION.replace('exit="0"', 'exit="1"');
    const structure = parseHickDoc(failed);
    const obs = structure.blocks.find((b) => b.name === "observation")!;
    expect(obs.attrs.exit).toBe("1");
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/session.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/session.test.tsx"><storycode:paste select="#f-ed3dc6749f3b-expression_statement"/><storycode:paste select="#f-ed3dc6749f3b-SESSION"/><storycode:paste select="#f-ed3dc6749f3b-expression_statement-2"/></storycode:file>

<a id="output-3ec8f02a5c70"></a>
## `crates/hick-lsp/src/session_lint.rs`

Related contracts: [Read the retained branch of an agent conversation](contract.md).

<a id="f-3ec8f02a5c70-SOURCE"></a>
### `SOURCE`

Problems in a `hick:session` document, as LSP diagnostics.
A session is a record of a conversation, and a record can be incomplete or
inconsistent in ways a person reading it would want marked: a turn nobody
answered, a tool call whose result never came back, a `parent=` that names
a turn the file does not hold. Those are problems WITH THE RECORD and they
count — errors and warnings, in the status bar with everything else.
What the agent did is not a problem with the record. A refused tool, a
command that exited 1, a compaction the harness performed — those are
facts the conversation contains, and the reader wants them findable
without the status bar saying the document is broken. They are published
as *information*: a marker in the gutter, a line in the problems list,
never a count. (The web client counts errors and warnings only — see
`apps/web/src/lib/problems.ts`.)
Only a document whose root is `hick:session` is linted here; a note is
somebody else's business.

<storycode:copy id="f-3ec8f02a5c70-SOURCE">
//! Problems in a `hick:session` document, as LSP diagnostics.
//!
//! A session is a record of a conversation, and a record can be incomplete or
//! inconsistent in ways a person reading it would want marked: a turn nobody
//! answered, a tool call whose result never came back, a `parent=` that names
//! a turn the file does not hold. Those are problems WITH THE RECORD and they
//! count — errors and warnings, in the status bar with everything else.
//!
//! What the agent did is not a problem with the record. A refused tool, a
//! command that exited 1, a compaction the harness performed — those are
//! facts the conversation contains, and the reader wants them findable
//! without the status bar saying the document is broken. They are published
//! as *information*: a marker in the gutter, a line in the problems list,
//! never a count. (The web client counts errors and warnings only — see
//! `apps/web/src/lib/problems.ts`.)
//!
//! Only a document whose root is `hick:session` is linted here; a note is
//! somebody else's business.

use hick_lang::{HickDocument, HickNode, HickTag};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::structural::byte_to_position;

const SOURCE: &str = "hick-session";

</storycode:copy>
<a id="f-3ec8f02a5c70-session_diagnostics"></a>
### `session_diagnostics`

Diagnostics for `doc`, or none when it is not a session.

<storycode:copy id="f-3ec8f02a5c70-session_diagnostics">
/// Diagnostics for `doc`, or none when it is not a session.
pub fn session_diagnostics(source: &str, doc: &HickDocument) -> Vec<Diagnostic> {
    let Some(root) = doc.tags().find(|t| t.name == "session") else {
        return Vec::new();
    };
    let turns: Vec<&HickTag> = root
        .children
        .iter()
        .filter_map(|n| match n {
            HickNode::Tag(t) => Some(t),
            HickNode::Text(..) => None,
        })
        .collect();

    let mut out = Vec::new();
    lint_turns(source, &turns, &mut out);
    lint_tool_calls(source, &turns, &mut out);
    lint_outcomes(source, &turns, &mut out);
    lint_broken_close_tags(source, &mut out);
    out
}

</storycode:copy>
<a id="f-3ec8f02a5c70-range_of"></a>
### `range_of`

The range of a tag's opening marker — the line a reader's eye lands on.

<storycode:copy id="f-3ec8f02a5c70-range_of">
/// The range of a tag's opening marker — the line a reader's eye lands on.
fn range_of(source: &str, tag: &HickTag) -> Range {
    match &tag.source_span {
        Some(span) => {
            let (sl, sc) = byte_to_position(source, span.start);
            let (el, ec) = byte_to_position(source, span.end);
            Range {
                start: Position {
                    line: sl,
                    character: sc,
                },
                end: Position {
                    line: el,
                    character: ec,
                },
            }
        }
        None => {
            let line = tag.source_line.saturating_sub(1) as u32;
            Range {
                start: Position { line, character: 0 },
                end: Position {
                    line,
                    character: u32::MAX,
                },
            }
        }
    }
}

</storycode:copy>
<a id="f-3ec8f02a5c70-diagnostic"></a>
### `diagnostic`

<storycode:copy id="f-3ec8f02a5c70-diagnostic">
fn diagnostic(
    range: Range,
    severity: DiagnosticSeverity,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(severity),
        source: Some(SOURCE.to_string()),
        message: message.into(),
        ..Default::default()
    }
}

</storycode:copy>
<a id="f-3ec8f02a5c70-lint_turns"></a>
### `lint_turns`

Turns: each `hick:user` needs a reply, a `parent=` must name a turn in
this file, and two turns must not share an id.

<storycode:copy id="f-3ec8f02a5c70-lint_turns">
/// Turns: each `hick:user` needs a reply, a `parent=` must name a turn in
/// this file, and two turns must not share an id.
fn lint_turns(source: &str, turns: &[&HickTag], out: &mut Vec<Diagnostic>) {
    let ids: Vec<Option<&str>> = turns
        .iter()
        .filter(|t| t.name == "user")
        .map(|t| t.get_attribute("turn"))
        .collect();
    let mut seen: Vec<&str> = Vec::new();
    for (i, t) in turns.iter().enumerate() {
        if t.name != "user" {
            continue;
        }
        // Answered: an assistant element before the next user element.
        let answered = turns[i + 1..]
            .iter()
            .take_while(|n| n.name != "user")
            .any(|n| n.name == "assistant");
        if !answered {
            out.push(diagnostic(
                range_of(source, t),
                DiagnosticSeverity::WARNING,
                "This turn has no reply recorded — the conversation ended (or was interrupted) \
                 before the agent answered.",
            ));
        }
        if let Some(turn) = t.get_attribute("turn") {
            if seen.contains(&turn) {
                out.push(diagnostic(
                    range_of(source, t),
                    DiagnosticSeverity::ERROR,
                    format!("Two turns share the id `{turn}`; the tree cannot tell them apart."),
                ));
            }
            seen.push(turn);
        }
        if let Some(parent) = t.get_attribute("parent")
            && !ids.contains(&Some(parent))
        {
            out.push(diagnostic(
                range_of(source, t),
                DiagnosticSeverity::WARNING,
                format!(
                    "`parent=\"{parent}\"` names a turn this file does not hold — the branch \
                     point is missing, so the tree view will root this turn."
                ),
            ));
        }
    }
}

</storycode:copy>
<a id="f-3ec8f02a5c70-lint_tool_calls"></a>
### `lint_tool_calls`

Tool calls: every `hick:tool` inside an assistant turn should have a
`hick:tool-result` before the next assistant or user element — matched by
`call=` when both sides carry it, by order otherwise.

<storycode:copy id="f-3ec8f02a5c70-lint_tool_calls">
/// Tool calls: every `hick:tool` inside an assistant turn should have a
/// `hick:tool-result` before the next assistant or user element — matched by
/// `call=` when both sides carry it, by order otherwise.
fn lint_tool_calls(source: &str, turns: &[&HickTag], out: &mut Vec<Diagnostic>) {
    for (i, t) in turns.iter().enumerate() {
        if t.name != "assistant" {
            continue;
        }
        let calls: Vec<&HickTag> = t
            .children
            .iter()
            .filter_map(|n| match n {
                HickNode::Tag(c) if c.name == "tool" => Some(c),
                _ => None,
            })
            .collect();
        if calls.is_empty() {
            continue;
        }
        let results: Vec<&HickTag> = turns[i + 1..]
            .iter()
            .take_while(|n| n.name != "assistant" && n.name != "user")
            .filter(|n| n.name == "tool-result")
            .copied()
            .collect();
        for (k, call) in calls.iter().enumerate() {
            let matched = match call.get_attribute("call") {
                Some(id) => results
                    .iter()
                    .any(|r| r.get_attribute("call").is_none_or(|rid| rid == id)),
                None => results.len() > k,
            };
            if !matched {
                let name = call.get_attribute("name").unwrap_or("?");
                out.push(diagnostic(
                    range_of(source, call),
                    DiagnosticSeverity::WARNING,
                    format!(
                        "The `{name}` call has no result recorded — the session ended, or was cut, \
                         before the tool answered."
                    ),
                ));
            }
        }
    }
}

</storycode:copy>
<a id="f-3ec8f02a5c70-lint_outcomes"></a>
### `lint_outcomes`

Outcomes the reader wants to find: a refused or failed tool, a command
that did not exit 0, a compaction. Information, never a count.

<storycode:copy id="f-3ec8f02a5c70-lint_outcomes">
/// Outcomes the reader wants to find: a refused or failed tool, a command
/// that did not exit 0, a compaction. Information, never a count.
fn lint_outcomes(source: &str, turns: &[&HickTag], out: &mut Vec<Diagnostic>) {
    for t in turns {
        match t.name.as_str() {
            "tool-result" if t.get_attribute("ok") == Some("false") => {
                let name = t.get_attribute("name").unwrap_or("tool");
                let first = first_line(&t.text_content());
                out.push(diagnostic(
                    range_of(source, t),
                    DiagnosticSeverity::INFORMATION,
                    format!("`{name}` refused or failed: {first}"),
                ));
            }
            "observation" => {
                if let Some(exit) = t.get_attribute("exit")
                    && exit != "0"
                {
                    out.push(diagnostic(
                        range_of(source, t),
                        DiagnosticSeverity::INFORMATION,
                        format!("The command exited {exit}."),
                    ));
                }
            }
            "context" if t.get_attribute("kind") == Some("compact") => {
                out.push(diagnostic(
                    range_of(source, t),
                    DiagnosticSeverity::INFORMATION,
                    "The conversation was compacted here: what the model saw after this point \
                     is a summary of what came before, not the turns themselves.",
                ));
            }
            _ => {}
        }
    }
}

</storycode:copy>
<a id="f-3ec8f02a5c70-lint_broken_close_tags"></a>
### `lint_broken_close_tags`

`</hick:input >` and friends: a verbatim body that quoted its own close
tag, which `hick ingest --from claude-code` broke with a space so the element would not end
early. The one place an imported byte differs from the transcript.

<storycode:copy id="f-3ec8f02a5c70-lint_broken_close_tags">
/// `</hick:input >` and friends: a verbatim body that quoted its own close
/// tag, which `hick ingest --from claude-code` broke with a space so the element would not end
/// early. The one place an imported byte differs from the transcript.
fn lint_broken_close_tags(source: &str, out: &mut Vec<Diagnostic>) {
    for element in ["input", "tool-result", "reasoning", "context", "transcript"] {
        let token = format!("</hick:{element} >");
        let mut from = 0;
        while let Some(at) = source[from..].find(&token) {
            let start = from + at;
            let (sl, sc) = byte_to_position(source, start);
            let (el, ec) = byte_to_position(source, start + token.len());
            out.push(diagnostic(
                Range {
                    start: Position {
                        line: sl,
                        character: sc,
                    },
                    end: Position {
                        line: el,
                        character: ec,
                    },
                },
                DiagnosticSeverity::INFORMATION,
                format!(
                    "This body quoted its own close tag; the import broke `</hick:{element}>` \
                     with a space so the element would not end here. The original had no space."
                ),
            ));
            from = start + token.len();
        }
    }
}

</storycode:copy>
<a id="f-3ec8f02a5c70-first_line"></a>
### `first_line`

<storycode:copy id="f-3ec8f02a5c70-first_line">
fn first_line(text: &str) -> String {
    let line = text.trim().lines().next().unwrap_or("").trim();
    if line.chars().count() > 80 {
        let cut: String = line.chars().take(77).collect();
        format!("{cut}…")
    } else {
        line.to_string()
    }
}

</storycode:copy>
<a id="f-3ec8f02a5c70-tests"></a>
### `tests`

<storycode:copy id="f-3ec8f02a5c70-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn lint(body: &str) -> Vec<Diagnostic> {
        let src = format!(
            "<hick:session xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:session>\n"
        );
        let doc = hick_lang::parse(&src).unwrap();
        session_diagnostics(&src, &doc)
    }

    fn messages(d: &[Diagnostic]) -> Vec<String> {
        d.iter().map(|d| d.message.clone()).collect()
    }

    #[test]
    fn a_note_is_not_linted() {
        let src = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n<hick:user>x</hick:user>\n</hick:doc>\n";
        let doc = hick_lang::parse(src).unwrap();
        assert!(session_diagnostics(src, &doc).is_empty());
    }

    #[test]
    fn an_unanswered_turn_is_a_warning_on_its_line() {
        // docs/guarantees/agent/a-session-is-the-conversation.md
        let d = lint(
            "<hick:user turn=\"a\">hello</hick:user>\n<hick:assistant>hi</hick:assistant>\n<hick:user turn=\"b\" parent=\"a\">and?</hick:user>",
        );
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, Some(DiagnosticSeverity::WARNING));
        assert!(d[0].message.contains("no reply"));
        assert_eq!(d[0].range.start.line, 3);
    }

    #[test]
    fn a_parent_that_names_no_turn_and_a_duplicate_id_are_problems() {
        let d = lint(
            "<hick:user turn=\"a\" parent=\"zzz\">x</hick:user>\n<hick:assistant>y</hick:assistant>\n<hick:user turn=\"a\">x</hick:user>\n<hick:assistant>y</hick:assistant>",
        );
        let m = messages(&d);
        assert!(m.iter().any(|s| s.contains("parent=\"zzz\"")));
        assert!(
            d.iter()
                .any(|d| d.severity == Some(DiagnosticSeverity::ERROR)
                    && d.message.contains("share the id"))
        );
    }

    #[test]
    fn a_tool_call_without_a_result_is_a_warning_by_id_or_by_order() {
        let d = lint(
            "<hick:user>x</hick:user>\n<hick:assistant>\n<hick:tool name=\"Read\" call=\"t1\"></hick:tool>\n<hick:tool name=\"Bash\" call=\"t2\"></hick:tool>\n</hick:assistant>\n<hick:tool-result name=\"Read\" call=\"t1\" ok=\"true\">ok</hick:tool-result>\n<hick:assistant>done</hick:assistant>",
        );
        let m = messages(&d);
        assert_eq!(m.len(), 1, "{m:?}");
        assert!(m[0].contains("`Bash` call has no result"));
        let d = lint(
            "<hick:user>x</hick:user>\n<hick:assistant>\n<hick:tool name=\"read_doc\"></hick:tool>\n</hick:assistant>\n<hick:assistant>done</hick:assistant>",
        );
        assert!(messages(&d)[0].contains("`read_doc` call has no result"));
    }

    #[test]
    fn outcomes_are_information_not_problems() {
        let d = lint(
            "<hick:user>x</hick:user>\n<hick:assistant>\n<hick:action lang=\"sh\">false</hick:action>\n</hick:assistant>\n<hick:observation source=\"action-0\" exit=\"1\"></hick:observation>\n<hick:tool-result name=\"write_doc\" ok=\"false\">\ndeclined — fixture\n</hick:tool-result>\n<hick:context kind=\"compact\">Conversation compacted</hick:context>\n<hick:assistant>done</hick:assistant>",
        );
        assert_eq!(d.len(), 3, "{:?}", messages(&d));
        assert!(
            d.iter()
                .all(|d| d.severity == Some(DiagnosticSeverity::INFORMATION))
        );
        let m = messages(&d);
        assert!(m.iter().any(|s| s == "The command exited 1."));
        assert!(
            m.iter()
                .any(|s| s == "`write_doc` refused or failed: declined — fixture")
        );
        assert!(m.iter().any(|s| s.contains("compacted")));
    }

    #[test]
    fn a_broken_close_tag_is_pointed_at_exactly() {
        let d = lint(
            "<hick:user>x</hick:user>\n<hick:assistant>y</hick:assistant>\n<hick:tool-result name=\"Read\" ok=\"true\">\nquoted: </hick:tool-result >\n</hick:tool-result>",
        );
        let broken = d.iter().find(|d| d.message.contains("broke")).unwrap();
        assert_eq!(broken.range.start.line, 4);
        assert_eq!(broken.range.start.character, 8);
        assert_eq!(broken.severity, Some(DiagnosticSeverity::INFORMATION));
    }
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/session_lint.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/session_lint.rs"><storycode:paste select="#f-3ec8f02a5c70-SOURCE"/><storycode:paste select="#f-3ec8f02a5c70-session_diagnostics"/><storycode:paste select="#f-3ec8f02a5c70-range_of"/><storycode:paste select="#f-3ec8f02a5c70-diagnostic"/><storycode:paste select="#f-3ec8f02a5c70-lint_turns"/><storycode:paste select="#f-3ec8f02a5c70-lint_tool_calls"/><storycode:paste select="#f-3ec8f02a5c70-lint_outcomes"/><storycode:paste select="#f-3ec8f02a5c70-lint_broken_close_tags"/><storycode:paste select="#f-3ec8f02a5c70-first_line"/><storycode:paste select="#f-3ec8f02a5c70-tests"/></storycode:file>

## Test the behavior: `a_sessions_problems_reach_the_window_as_diagnostics`

This is the maintained test fragment from `crates/hickory-cli/tests/lsp_desktop.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c14f2fec35ca-a_sessions_problems_reach_the_window_as_diagnostics"></a>
### `a_sessions_problems_reach_the_window_as_diagnostics`

Protects docs/guarantees/agent/a-session-is-the-conversation.md — the
problem markers a session carries reach the app's window through the same
bridge as every other diagnostic, with no language server installed.

<storycode:copy id="f-c14f2fec35ca-a_sessions_problems_reach_the_window_as_diagnostics">
/// Protects docs/guarantees/agent/a-session-is-the-conversation.md — the
/// problem markers a session carries reach the app's window through the same
/// bridge as every other diagnostic, with no language server installed.
#[tokio::test(flavor = "multi_thread")]
async fn a_sessions_problems_reach_the_window_as_diagnostics() {
    // The room is any open document's; the text the window sends is what
    // gets linted, exactly as when a session tab opens in the app.
    let app = open_app().await;
    std::fs::write(app.root.join("session.md"), SESSION).unwrap();
    let url = format!("{}/api/ws?doc=doc:{}", app.base, app.doc_id);
    let (mut socket, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("the app's window connects");
    let uri = "hick:///session.md";
    send(&mut socket, json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri, "languageId": "hick", "version": 1, "text": SESSION}},
    }))
    .await;

    // The first publish may be empty (the parse succeeded, nothing from a
    // child yet); wait for the one that carries the session's own lints.
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    let mut diagnostics: Vec<Value> = Vec::new();
    while std::time::Instant::now() < deadline {
        let Some(message) = wait_for_method(
            &mut socket,
            "textDocument/publishDiagnostics",
            Duration::from_secs(10),
        )
        .await
        else {
            break;
        };
        let list = message["params"]["diagnostics"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if list.iter().any(|d| d["source"] == "hick-session") {
            diagnostics = list;
            break;
        }
    }
    assert!(
        !diagnostics.is_empty(),
        "no session diagnostics reached the window"
    );
    let refused = diagnostics
        .iter()
        .find(|d| d["message"].as_str().unwrap_or("").contains("refused"))
        .expect("the refused tool is marked");
    assert_eq!(
        refused["severity"], 3,
        "a refused tool is information, not a count"
    );
    // Line 8 (0-based) is the `<hick:tool-result …>` opening tag.
    assert_eq!(refused["range"]["start"]["line"], 8);
    let unanswered = diagnostics
        .iter()
        .find(|d| d["message"].as_str().unwrap_or("").contains("no reply"))
        .expect("the unanswered turn is marked");
    assert_eq!(unanswered["severity"], 2);
    assert_eq!(unanswered["range"]["start"]["line"], 12);
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [apps/web/src/components/ChatTree.tsx](contract.md#output-d6c33a544e5e)
- [apps/web/src/components/SessionTurns.tsx](../../../mechanisms/apps/web/src/components/SessionTurns.tsx.md#output-b05fd3f7cf29)
- [apps/web/src/editor/folding.ts](../../../mechanisms/apps/web/src/editor/folding.ts.md#output-eee9d1963829)
- [apps/web/src/editor/wysiwyg.ts](../../../shared/files/apps/web/src/editor/wysiwyg.ts.md)
- [apps/web/src/elements/session/view.tsx](../../../mechanisms/apps/web/src/elements/session/view.tsx.md#output-d096a8a10df6)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)
- [crates/hick-lsp/src/session_lint.rs](contract.md#output-3ec8f02a5c70)
- [crates/hickory-agent/src/session.rs](../../../mechanisms/crates/hickory-agent/src/session.rs.md#output-9c7ae0f28c14)
- [crates/hickory-agent/src/session_view.rs](../../../mechanisms/crates/hickory-agent/src/session_view.rs.md#output-009403b22bd0)
- [crates/hickory-cli/src/serve/agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent.rs.md#output-2ce16640df92)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- [apps/web/src/editor/folding.test.ts](../../../mechanisms/apps/web/src/editor/folding.ts.md#output-92a9cd526370)
- [apps/web/src/editor/session.test.tsx](contract.md#output-ed3dc6749f3b)
- [apps/web/src/views/ConversationEditor.test.tsx](../../../mechanisms/apps/web/src/views/ConversationEditor.tsx.md#output-edc92cd336d7)
- [crates/hickory-cli/tests/lsp_desktop.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_desktop.rs.md#output-c14f2fec35ca)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_python_agent.rs.md#output-08f0cd3133cc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
