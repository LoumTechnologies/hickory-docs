//! A session file read back as a conversation: turns, and within each turn the
//! steps the agent took — reasoning, prose, scripts and their observations,
//! tool calls and their results, the files it was shown, the lines it wrote.
//!
//! This is the one shape both surfaces render: the chat dock (live) and a
//! session document opened in the app (after the fact). A conversation is the
//! file; the file is the conversation. The turn TREE is in it too — each
//! `<hick:user turn=… parent=…>` names its branch point — so a restarted app
//! rebuilds the dock from `sessions/` and a rewind is a parent pointer.

use std::path::Path;

use serde::Serialize;

/// One parsed session file.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SessionView {
    /// RFC 3339 start from the root, if present.
    pub start: Option<String>,
    /// The document the session is about (`doc=` on the root), if recorded.
    pub doc: Option<String>,
    pub turns: Vec<SessionTurn>,
}

/// One user turn and everything the agent did in answer to it.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SessionTurn {
    /// The turn id (`turn=`), falling back to the input id.
    pub id: String,
    /// The parent turn's id — the branch point — when recorded.
    pub parent: Option<String>,
    pub prompt: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub steps: Vec<Step>,
    /// The agent's final words for this turn: the last prose the assistant
    /// wrote before the next turn (or the end).
    pub answer: Option<String>,
    /// Token usage summed over this turn's `<hick:usage turn=…>` elements.
    pub usage: Option<TurnUsage>,
    /// Line in the session file where the turn's `<hick:user>` stands.
    pub session_line: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct TurnUsage {
    pub input: u64,
    pub cache_write: u64,
    pub cache_read: u64,
    pub output: u64,
    pub cost_usd: Option<f64>,
}

/// One thing that happened inside a turn, in session order.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Step {
    /// The model's reasoning, when the provider exposed it. Shown folded.
    Reasoning { text: String, session_line: usize },
    /// Assistant prose.
    Prose { text: String, session_line: usize },
    /// A script the agent ran.
    Action {
        lang: String,
        code: String,
        session_line: usize,
    },
    /// What the script printed.
    Observation {
        id: Option<String>,
        source: Option<String>,
        exit: Option<String>,
        text: String,
        session_line: usize,
    },
    /// A document tool call.
    Tool {
        name: String,
        args: Vec<(String, String)>,
        input: Option<String>,
        session_line: usize,
    },
    /// What the tool answered.
    ToolResult {
        id: Option<String>,
        name: String,
        ok: bool,
        text: String,
        session_line: usize,
    },
    /// A file a tool showed the model (context provenance).
    Read {
        file: String,
        commit: Option<String>,
        sha256: String,
        lines: String,
        session_line: usize,
    },
    /// Lines an edit left in a file.
    Wrote {
        file: String,
        lines: String,
        session_line: usize,
    },
}

/// Parse a session source into its conversation view. A file that is not a
/// session, or does not parse, is an empty view rather than an error — the
/// caller decides what to say about that.
pub fn session_view(source: &str) -> SessionView {
    let Ok(doc) = hick_lang::parse(source) else {
        return SessionView {
            start: None,
            doc: None,
            turns: Vec::new(),
        };
    };
    let root = doc.tags().find(|t| t.name == "session");
    let (start, doc_attr, nodes): (Option<String>, Option<String>, Vec<&hick_lang::HickTag>) =
        match root {
            Some(root) => (
                root.get_attribute("start").map(str::to_string),
                root.get_attribute("doc").map(str::to_string),
                root.children
                    .iter()
                    .filter_map(|n| match n {
                        hick_lang::HickNode::Tag(t) => Some(t),
                        _ => None,
                    })
                    .collect(),
            ),
            None => (None, None, doc.tags().collect()),
        };

    let mut turns: Vec<SessionTurn> = Vec::new();
    let push_step = |turns: &mut Vec<SessionTurn>, step: Step| {
        if let Some(t) = turns.last_mut() {
            t.steps.push(step);
        }
    };
    for tag in nodes {
        match tag.name.as_str() {
            "user" => {
                let id = tag
                    .get_attribute("turn")
                    .or_else(|| tag.get_attribute("id"))
                    .unwrap_or("turn")
                    .to_string();
                turns.push(SessionTurn {
                    id,
                    parent: tag.get_attribute("parent").map(str::to_string),
                    prompt: tag.text_content().trim().to_string(),
                    provider: tag.get_attribute("provider").map(str::to_string),
                    model: tag.get_attribute("model").map(str::to_string),
                    steps: Vec::new(),
                    answer: None,
                    usage: None,
                    session_line: tag.source_line,
                });
            }
            "assistant" => {
                let mut prose = String::new();
                for child in &tag.children {
                    match child {
                        hick_lang::HickNode::Text(t, _) => prose.push_str(t),
                        hick_lang::HickNode::Tag(c) => match c.name.as_str() {
                            "reasoning" => push_step(
                                &mut turns,
                                Step::Reasoning {
                                    text: c.text_content().trim().to_string(),
                                    session_line: c.source_line,
                                },
                            ),
                            "action" => {
                                flush_prose(&mut turns, &mut prose, tag.source_line);
                                push_step(
                                    &mut turns,
                                    Step::Action {
                                        lang: c.get_attribute("lang").unwrap_or("sh").to_string(),
                                        code: c.text_content().trim_matches('\n').to_string(),
                                        session_line: c.source_line,
                                    },
                                );
                            }
                            "tool" => {
                                flush_prose(&mut turns, &mut prose, tag.source_line);
                                let mut args = Vec::new();
                                let mut input = None;
                                for gc in &c.children {
                                    if let hick_lang::HickNode::Tag(a) = gc {
                                        if a.name == "arg" {
                                            args.push((
                                                a.get_attribute("name").unwrap_or("").to_string(),
                                                a.text_content().trim().to_string(),
                                            ));
                                        } else if a.name == "input" {
                                            input = Some(
                                                a.text_content().trim_matches('\n').to_string(),
                                            );
                                        }
                                    }
                                }
                                push_step(
                                    &mut turns,
                                    Step::Tool {
                                        name: c.get_attribute("name").unwrap_or("").to_string(),
                                        args,
                                        input,
                                        session_line: c.source_line,
                                    },
                                );
                            }
                            // Anything else inside an assistant turn is part
                            // of what it said.
                            _ => prose.push_str(&c.text_content()),
                        },
                    }
                }
                flush_prose(&mut turns, &mut prose, tag.source_line);
            }
            "observation" => push_step(
                &mut turns,
                Step::Observation {
                    id: tag.get_attribute("id").map(str::to_string),
                    source: tag.get_attribute("source").map(str::to_string),
                    exit: tag.get_attribute("exit").map(str::to_string),
                    text: tag.text_content().trim_matches('\n').to_string(),
                    session_line: tag.source_line,
                },
            ),
            "tool-result" => push_step(
                &mut turns,
                Step::ToolResult {
                    id: tag.get_attribute("id").map(str::to_string),
                    name: tag.get_attribute("name").unwrap_or("").to_string(),
                    ok: tag.get_attribute("ok") != Some("false"),
                    text: tag.text_content().trim_matches('\n').to_string(),
                    session_line: tag.source_line,
                },
            ),
            "read" => push_step(
                &mut turns,
                Step::Read {
                    file: tag.get_attribute("file").unwrap_or("").to_string(),
                    commit: tag.get_attribute("commit").map(str::to_string),
                    sha256: tag.get_attribute("sha256").unwrap_or("").to_string(),
                    lines: tag.get_attribute("lines").unwrap_or("").to_string(),
                    session_line: tag.source_line,
                },
            ),
            "wrote" => push_step(
                &mut turns,
                Step::Wrote {
                    file: tag.get_attribute("file").unwrap_or("").to_string(),
                    lines: tag.get_attribute("lines").unwrap_or("").to_string(),
                    session_line: tag.source_line,
                },
            ),
            "usage" => {
                // Per-turn usage rows add up; the session total is skipped.
                if tag.get_attribute("turn").is_some()
                    && let Some(t) = turns.last_mut()
                {
                    let u = t.usage.get_or_insert_with(TurnUsage::default);
                    let num = |k: &str| {
                        tag.get_attribute(k)
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(0)
                    };
                    u.input += num("input");
                    u.cache_write += num("cache-write");
                    u.cache_read += num("cache-read");
                    u.output += num("output");
                    if let Some(c) = tag
                        .get_attribute("cost-usd")
                        .and_then(|v| v.parse::<f64>().ok())
                    {
                        u.cost_usd = Some(u.cost_usd.unwrap_or(0.0) + c);
                    }
                }
            }
            _ => {}
        }
    }
    // The answer is the last prose of each turn.
    for t in &mut turns {
        t.answer = t.steps.iter().rev().find_map(|s| match s {
            Step::Prose { text, .. } if !text.trim().is_empty() => Some(text.clone()),
            _ => None,
        });
    }
    SessionView {
        start,
        doc: doc_attr,
        turns,
    }
}

fn flush_prose(turns: &mut [SessionTurn], prose: &mut String, line: usize) {
    let text = prose.trim().to_string();
    prose.clear();
    if text.is_empty() {
        return;
    }
    if let Some(t) = turns.last_mut() {
        t.steps.push(Step::Prose {
            text,
            session_line: line,
        });
    }
}

/// The conversations recorded for `doc_path` under `sessions_dir`: every
/// session whose root names the document, oldest first, with the path each
/// came from. This is what a restarted app rebuilds the dock from.
pub fn conversations_for(
    sessions_dir: &Path,
    doc_path: &Path,
) -> Vec<(std::path::PathBuf, SessionView)> {
    let Ok(entries) = std::fs::read_dir(sessions_dir) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "hick"))
        .collect();
    files.sort();
    let want = doc_path.canonicalize().ok();
    let mut out = Vec::new();
    for path in files {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !hick_lang::is_session_source(&source) {
            continue;
        }
        let view = session_view(&source);
        let Some(doc) = &view.doc else { continue };
        let same = match &want {
            Some(w) => {
                let d = Path::new(doc);
                d.canonicalize().ok().as_ref() == Some(w)
                    || sessions_dir
                        .parent()
                        .map(|root| root.join(doc))
                        .and_then(|p| p.canonicalize().ok())
                        .as_ref()
                        == Some(w)
            }
            None => false,
        } || same_suffix(doc, &doc_path.display().to_string());
        if same {
            out.push((path, view));
        }
    }
    out
}

fn same_suffix(a: &str, b: &str) -> bool {
    let a = a.replace('\\', "/");
    let b = b.replace('\\', "/");
    a == b || a.ends_with(&format!("/{b}")) || b.ends_with(&format!("/{a}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-08-22T10:00:00Z" doc="notes/plan.hick">
<hick:user id="in0" turn="t1" provider="anthropic" model="claude-sonnet-5">Make it friendlier.</hick:user>
<hick:usage turn="0" input="10" cache-write="0" cache-read="0" output="5" cost-usd="0.001"/>
<hick:assistant>
<hick:reasoning>
Let me look first.
</hick:reasoning>
<hick:tool name="read_doc"></hick:tool>
</hick:assistant>
<hick:tool-result id="in1" name="read_doc" ok="true">
doc: plan.hick
aaaa|hello
</hick:tool-result>
<hick:read file="plan.hick" sha256="ff" lines="1-1"/>
<hick:assistant>
<hick:action lang="sh">echo hi</hick:action>
</hick:assistant>
<hick:observation id="in2" source="action-0" exit="0">hi</hick:observation>
<hick:assistant>Done — friendlier now.</hick:assistant>
<hick:usage turn="1" input="20" cache-write="0" cache-read="0" output="7" cost-usd="0.002"/>
<hick:user id="in3" turn="t2" parent="t1" provider="anthropic" model="claude-sonnet-5">And shorter.</hick:user>
<hick:assistant>Shorter.</hick:assistant>
</hick:session>
"#;

    /// Guarantee: docs/guarantees/agent/a-session-is-the-conversation.md
    #[test]
    fn a_session_reads_back_as_turns_with_steps_answers_and_the_tree() {
        let v = session_view(SESSION);
        assert_eq!(v.doc.as_deref(), Some("notes/plan.hick"));
        assert_eq!(v.turns.len(), 2);
        let t1 = &v.turns[0];
        assert_eq!(t1.id, "t1");
        assert_eq!(t1.parent, None);
        assert_eq!(t1.prompt, "Make it friendlier.");
        assert_eq!(t1.answer.as_deref(), Some("Done — friendlier now."));
        let kinds: Vec<&str> = t1
            .steps
            .iter()
            .map(|s| match s {
                Step::Reasoning { .. } => "reasoning",
                Step::Prose { .. } => "prose",
                Step::Action { .. } => "action",
                Step::Observation { .. } => "observation",
                Step::Tool { .. } => "tool",
                Step::ToolResult { .. } => "tool-result",
                Step::Read { .. } => "read",
                Step::Wrote { .. } => "wrote",
            })
            .collect();
        assert_eq!(
            kinds,
            [
                "reasoning",
                "tool",
                "tool-result",
                "read",
                "action",
                "observation",
                "prose"
            ]
        );
        let u = t1.usage.as_ref().unwrap();
        assert_eq!((u.input, u.output), (30, 12));
        let t2 = &v.turns[1];
        assert_eq!(t2.parent.as_deref(), Some("t1"));
        assert_eq!(t2.answer.as_deref(), Some("Shorter."));
    }
}
