<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Explicit context updates preserve confirmed history

For: Authors using an agent to change documents while retaining review control.

Given the confined Python agent and its retained branch,
when a user inspects changed context, then the non-modal agent pane compares
source-backed ancestry of confirmed requests against specific current disk or
unsaved-buffer candidates. Captures alone and model-output genealogy do not
advance exposure. Hidden projection segments are not supplied editor references.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/explicit-context-updates.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-361e848bd404"></a>
## `apps/web/src/components/ContextUpdates.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit context updates preserve confirmed history](contract.md).

<a id="f-361e848bd404-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-361e848bd404-expression_statement">
// Guarantee: docs/guarantees/agent/explicit-context-updates.md
import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { ContextUpdates } from "./ContextUpdates";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
</storycode:copy>
<a id="f-361e848bd404-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-361e848bd404-expression_statement-2">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-361e848bd404-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-361e848bd404-expression_statement-3">
it("discloses patches before selection and queues their exact observed revisions through the shared handler", async () => {
  const item = { reference: "old", path: "AGENTS.md", before: "HIDDEN-ORIGINAL-CONTEXT\n", after: "HIDDEN-CANDIDATE-CONTEXT\n", status: "changed_observed_passage", revision: "r2", eligible: true, role: "user", scope: "root", request: "r1" };
  const view = { items: [item], updates: [], coverage: "Confirmed comparison only" };
  vi.mocked(request).mockResolvedValue(view);
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><ContextUpdates doc="workspace" branch="a" /></UiActionContext.Provider>);
  await screen.findByText("AGENTS.md · changed_observed_passage");
  expect(registry.groups().groups).toEqual(["agent-context-disclosures"]);
  let n = 0;
  const invoke = async (parent: string, label: string, args = {}) => {
    const action = registry.observe(parent).actions.find(action => action.label === label)!;
    let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++n) }); }); return result;
  };
  await invoke("agent-context-disclosures", "Set changed agent context visibility", { open: true });
  const parent = registry.groups().groups.find(group => group.startsWith("context-updates:"))!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain("HIDDEN-ORIGINAL-CONTEXT");
  expect(JSON.stringify(registry.observe(parent))).not.toContain("HIDDEN-CANDIDATE-CONTEXT");
  expect(registry.observe(parent).actions[0].revision).toMatch(/^ui-revision:/);
  await waitFor(() => expect(registry.observe(parent).actions.find(action => action.label === "Refresh patches and delivery status")?.enabled).toBe(true));
  await invoke(parent, "Set context patch visibility: AGENTS.md", { open: true });
  const patch = registry.groups().groups.find(group => group.includes("/patch/"))!;
  await invoke(patch, "Select context update: AGENTS.md", { checked: true });
  vi.mocked(request).mockResolvedValue({ ...view, updates: [{ id: "u1", old: "old", state: "queued", request: null, source: { revision: "r2" } }] });
  expect(await invoke(parent, "Queue selected context updates")).toMatchObject({ status: "result", outcome: { value: { updates: [{ id: "u1", state: "queued" }] } } });
  expect(request).toHaveBeenLastCalledWith("POST", "/api/docs/workspace/agent/python/context", { branch: "a", context: { buffers: [] }, selected: ["old"], revisions: { old: "r2" } });
  await invoke("agent-context-disclosures", "Set changed agent context visibility", { open: false });
  expect(registry.observe(patch).actions).toEqual([]);
});
</storycode:copy>
<a id="f-361e848bd404-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-361e848bd404-expression_statement-4">
it("shows the candidate patch and queues explicitly selected unsaved bytes without claiming delivery", async () => {
  const item = { reference: "old", path: "AGENTS.md", before: "old\n", after: "new\n", status: "changed_observed_passage", revision: "r2", eligible: true, role: "user", scope: "root", request: "r1" };
  const view = { items: [item], updates: [], coverage: "Opaque context unavailable" };
  vi.mocked(request).mockResolvedValue(view);
  const getContext = vi.fn(async () => ({ buffers: [{ id: "unsaved", name: "Draft", path: null, content: "draft", focused: true }] }));
  render(<ContextUpdates doc="workspace" branch="a" getContext={getContext} />);
  await screen.findByText("AGENTS.md · changed_observed_passage");
  fireEvent.click(screen.getByLabelText("Select update"));
  vi.mocked(request).mockResolvedValue({ ...view, updates: [{ id: "u1", old: "old", state: "queued", request: null, source: { revision: "r2" } }] });
  fireEvent.click(screen.getByText("Queue selected updates"));
  await waitFor(() => expect(request).toHaveBeenLastCalledWith("POST", "/api/docs/workspace/agent/python/context", expect.objectContaining({ branch: "a", selected: ["old"], revisions: { old: "r2" }, context: expect.objectContaining({ buffers: expect.any(Array) }) })));
  expect(screen.queryByText("Included")).toBeNull();
  expect(document.querySelector(".diff-line--add")?.textContent).toContain("new");
});
</storycode:copy>
<a id="f-361e848bd404-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-361e848bd404-expression_statement-5">
// Guarantee: docs/guarantees/agent/explicit-context-updates.md
it("polls at its interval using the current capture callback without a rerender request flood", async () => {
  vi.useFakeTimers();
  vi.mocked(request).mockClear();
  vi.mocked(request).mockResolvedValue({ items: [], updates: [], coverage: "fixture" });
  const first = vi.fn(async () => ({ buffers: [] }));
  const latest = vi.fn(async () => ({ buffers: [{ name: "fresh", path: null, content: "new", focused: true }] }));
  const ui = render(<ContextUpdates doc="workspace" branch="a" getContext={first} />);
  try {
    await act(async () => {});
    expect(request).toHaveBeenCalledTimes(1);
    for (let i = 0; i < 10; i++) ui.rerender(<ContextUpdates doc="workspace" branch="a" getContext={latest} />);
    await act(async () => {});
    expect(request).toHaveBeenCalledTimes(1);
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
    expect(request).toHaveBeenCalledTimes(2);
    expect(latest).toHaveBeenCalledTimes(1);
    expect(request).toHaveBeenLastCalledWith("POST", "/api/docs/workspace/agent/python/context", { branch: "a", context: await latest() });
  } finally { ui.unmount(); vi.useRealTimers(); }
});

</storycode:copy>
<a id="f-361e848bd404-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-361e848bd404-expression_statement-6">
// Guarantee: docs/guarantees/agent/explicit-context-updates.md
it("does not overlap slow captures or post a retired branch capture", async () => {
  vi.useFakeTimers();
  vi.mocked(request).mockClear();
  vi.mocked(request).mockResolvedValue({ items: [], updates: [], coverage: "fixture" });
  let resolve!: (context: { buffers: [] }) => void;
  const capture = vi.fn(() => new Promise<{ buffers: [] }>(done => { resolve = done; }));
  const ui = render(<ContextUpdates doc="workspace" branch="a" getContext={capture} />);
  try {
    await act(async () => { await vi.advanceTimersByTimeAsync(15000); });
    expect(capture).toHaveBeenCalledTimes(1);
    expect(request).not.toHaveBeenCalled();
    ui.rerender(<ContextUpdates doc="workspace" branch="b" getContext={async () => ({ buffers: [] })} />);
    await act(async () => { resolve({ buffers: [] }); });
    expect(request).toHaveBeenCalledTimes(1);
    expect(request).toHaveBeenCalledWith("POST", "/api/docs/workspace/agent/python/context", { branch: "b", context: { buffers: [] } });
  } finally { ui.unmount(); vi.useRealTimers(); }
});

</storycode:copy>
<a id="f-361e848bd404-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-361e848bd404-expression_statement-7">
// Guarantee: docs/guarantees/agent/explicit-context-updates.md
it("does not submit a manual context capture after its originating branch retires", async () => {
  const view = { items: [], updates: [], coverage: "current fixture" };
  vi.mocked(request).mockResolvedValue(view);
  let release!: (value: { buffers: [] }) => void;
  const capture = vi.fn().mockResolvedValue({ buffers: [] }).mockImplementationOnce(async () => ({ buffers: [] })).mockImplementationOnce(() => new Promise(resolve => { release = resolve; }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const ui = (branch: string) => <UiActionContext.Provider value={registry}><ContextUpdates doc="workspace" branch={branch} getContext={capture} /></UiActionContext.Provider>;
  const rendered = render(ui("a"));
  await waitFor(() => expect(request).toHaveBeenCalledTimes(1));
  const open = registry.observe("agent-context-disclosures").actions[0];
  await act(async () => { await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: open.handle, arguments: { open: true }, request_id: "open" }); });
  await waitFor(() => expect(release).toBeTypeOf("function"));
  rendered.rerender(ui("b"));
  await waitFor(() => expect(vi.mocked(request).mock.calls.some(call => (call[2] as { branch: string }).branch === "b")).toBe(true));
  const calls = vi.mocked(request).mock.calls.length;
  await act(async () => { release({ buffers: [] }); });
  expect(request).toHaveBeenCalledTimes(calls);
  expect(vi.mocked(request).mock.calls.filter(call => (call[2] as { branch: string }).branch === "a")).toHaveLength(1);
});

</storycode:copy>
<a id="f-361e848bd404-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-361e848bd404-expression_statement-8">
// Guarantee: docs/guarantees/agent/explicit-context-updates.md
it("keeps a newer explicit queue result when an older background refresh completes late", async () => {
  vi.useFakeTimers();
  const item = { reference: "old", path: "note.md", before: "before", after: "after", status: "changed_observed_passage", revision: "r2", eligible: true, role: "user", scope: "root", request: "r1" };
  const initial = { items: [item], updates: [], coverage: "initial candidate" };
  let background = false, release!: (value: typeof initial) => void;
  vi.mocked(request).mockImplementation(async (_method, _path, body) => {
    if ((body as { selected?: string[] }).selected?.length) return { ...initial, coverage: "queued candidate", updates: [{ id: "u1", old: "old", state: "queued", request: null, source: { revision: "r2" } }] };
    if (background) return new Promise(resolve => { release = resolve; });
    return initial;
  });
  const rendered = render(<ContextUpdates doc="workspace" branch="a" />);
  try {
    await act(async () => {});
    const outer = document.querySelector("details")!;
    await act(async () => { outer.open = true; fireEvent(outer, new Event("toggle")); });
    const patch = outer.querySelector("details")!;
    await act(async () => { patch.open = true; fireEvent(patch, new Event("toggle")); });
    fireEvent.click(screen.getByLabelText("Select update"));
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    background = true;
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
    expect(release).toBeTypeOf("function");
    await act(async () => fireEvent.click(screen.getByText("Queue selected updates")));
    expect(screen.getByText("queued candidate")).toBeTruthy();
    await act(async () => { release({ ...initial, coverage: "stale background" }); });
    expect(screen.getByText("queued candidate")).toBeTruthy();
    expect(screen.queryByText("stale background")).toBeNull();
  } finally { rendered.unmount(); vi.useRealTimers(); }
});
</storycode:copy>

### Assemble `apps/web/src/components/ContextUpdates.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ContextUpdates.test.tsx"><storycode:paste select="#f-361e848bd404-expression_statement"/><storycode:paste select="#f-361e848bd404-expression_statement-2"/><storycode:paste select="#f-361e848bd404-expression_statement-3"/><storycode:paste select="#f-361e848bd404-expression_statement-4"/><storycode:paste select="#f-361e848bd404-expression_statement-5"/><storycode:paste select="#f-361e848bd404-expression_statement-6"/><storycode:paste select="#f-361e848bd404-expression_statement-7"/><storycode:paste select="#f-361e848bd404-expression_statement-8"/></storycode:file>

<a id="output-a93965a16b0e"></a>
## `crates/hickory-cli/src/context_updates.rs`

Related contracts: [Explicit context updates preserve confirmed history](contract.md).

<a id="f-a93965a16b0e-Exposure"></a>
### `Exposure`

Explicit user updates. Queueing never certifies model delivery.

<storycode:copy id="f-a93965a16b0e-Exposure">
//! Explicit user updates. Queueing never certifies model delivery.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct Exposure {
    pub reference: String,
    #[serde(default)]
    pub unique: bool,
    pub source: hick_context_protocol::Source,
    pub text: String,
    pub start: u64,
    pub role: String,
    pub scope: String,
    pub request: String,
}
</storycode:copy>
<a id="f-a93965a16b0e-Update"></a>
### `Update`

<storycode:copy id="f-a93965a16b0e-Update">
#[derive(Clone, Serialize, Deserialize)]
pub struct Update {
    pub id: String,
    pub old: String,
    pub source: hick_context_protocol::Source,
    pub text: Option<String>,
    pub role: String,
    pub scope: String,
    pub state: String,
    pub request: Option<String>,
}
</storycode:copy>
<a id="f-a93965a16b0e-Branch"></a>
### `Branch`

<storycode:copy id="f-a93965a16b0e-Branch">
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Branch {
    pub exposures: Vec<Exposure>,
    pub updates: Vec<Update>,
}
</storycode:copy>
<a id="f-a93965a16b0e-Inbox"></a>
### `Inbox`

<storycode:copy id="f-a93965a16b0e-Inbox">
#[derive(Default)]
pub struct Inbox(
    pub BTreeMap<String, Branch>,
    BTreeMap<String, std::path::PathBuf>,
    Option<std::path::PathBuf>,
);
</storycode:copy>
<a id="f-a93965a16b0e-impl-Inbox"></a>
### `impl-Inbox`

<storycode:copy id="f-a93965a16b0e-impl-Inbox">
impl Inbox {
    pub fn open(&mut self, session: &std::path::Path) -> anyhow::Result<()> {
        let path = session.with_extension("context-updates.json");
        if self.2.as_ref() == Some(&path) {
            return Ok(());
        }
        let mut branches: BTreeMap<String, Branch> = match std::fs::read(&path) {
            Ok(raw) => serde_json::from_slice(&raw)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(e) => return Err(e.into()),
        };
        for branch in branches.values_mut() {
            for update in &mut branch.updates {
                if matches!(
                    update.state.as_str(),
                    "preparing" | "prepared" | "transport_started"
                ) {
                    update.state = "uncertain".into();
                }
            }
        }
        for (id, branch) in branches {
            if !self.0.contains_key(&id) {
                self.1.insert(id.clone(), path.clone());
                self.0.insert(id, branch);
            }
        }
        self.2 = Some(path);
        Ok(())
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let mut files: BTreeMap<&std::path::PathBuf, BTreeMap<&String, &Branch>> = BTreeMap::new();
        for (id, branch) in &self.0 {
            if let Some(path) = self.1.get(id) {
                files.entry(path).or_default().insert(id, branch);
            }
        }
        for (path, branches) in files {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let temporary = path.with_extension("json.tmp");
            std::fs::write(&temporary, serde_json::to_vec(&branches)?)?;
            std::fs::rename(&temporary, path)?;
        }
        Ok(())
    }
    pub fn fork(&mut self, branch: &str, parent: Option<&str>) {
        let prior = parent
            .and_then(|p| self.0.get(p))
            .cloned()
            .unwrap_or_default();
        self.0.entry(branch.into()).or_insert(prior);
        if let Some(path) = &self.2 {
            self.1.entry(branch.into()).or_insert(path.clone());
        }
    }
    pub fn take(&mut self, branch: &str) -> Vec<Update> {
        self.0
            .entry(branch.into())
            .or_default()
            .updates
            .iter_mut()
            .filter(|u| u.state == "queued")
            .map(|u| {
                u.state = "preparing".into();
                u.clone()
            })
            .collect()
    }
    pub fn delivery(&mut self, branch: &str, request: &str, state: &str, references: &[String]) {
        for update in &mut self.0.entry(branch.into()).or_default().updates {
            if references.contains(&update.id) && update.state != "provider_confirmed" {
                update.state = state.into();
                update.request = Some(request.into());
            }
        }
    }
    pub fn publish(&mut self, branch: &str, exposures: Vec<Exposure>) {
        let current = &mut self.0.entry(branch.into()).or_default().exposures;
        for exposure in exposures {
            // The last supplied passage wins only for that exact original range.
            current.retain(|old| {
                old.source.id != exposure.source.id
                    || !(old.start >= exposure.start
                        && old.start + old.text.len() as u64
                            <= exposure.start + exposure.text.len() as u64)
            });
            current.push(exposure);
        }
    }
}
</storycode:copy>
<a id="f-a93965a16b0e-exposures"></a>
### `exposures`

Preserve request order while walking derived inputs; captures alone are not exposure.

<storycode:copy id="f-a93965a16b0e-exposures">
/// Preserve request order while walking derived inputs; captures alone are not exposure.
pub fn exposures(manifest: &Value) -> Vec<Exposure> {
    fn visit(
        id: &str,
        nodes: &[Value],
        role: &str,
        scope: &str,
        request: &str,
        seen: &mut Vec<String>,
        out: &mut Vec<Exposure>,
    ) {
        if seen.iter().any(|s| s == id) {
            return;
        }
        seen.push(id.into());
        let Some(node) = nodes.iter().find(|n| n["id"] == id) else {
            return;
        };
        if let (Ok(source), Some(text), Some(start)) = (
            serde_json::from_value(node["source"].clone()),
            node["text"].as_str(),
            node["source_span"]["start"].as_u64(),
        ) {
            out.push(Exposure {
                reference: id.into(),
                unique: false,
                source,
                text: text.into(),
                start,
                role: role.into(),
                scope: scope.into(),
                request: request.into(),
            });
        }
        if node["derivation"] == "model_output" {
            return;
        }
        if let Some(parents) = node["parents"].as_array() {
            for parent in parents {
                if let Some(id) = parent.as_str() {
                    visit(id, nodes, role, scope, request, seen, out);
                }
            }
        }
    }
    let mut out = vec![];
    let Some(nodes) = manifest["nodes"].as_array() else {
        return out;
    };
    if let Some(parts) = manifest["request"]["parts"].as_array() {
        for part in parts {
            if let Some(segments) = part["segments"].as_array() {
                for segment in segments {
                    if let Some(id) = segment["node"].as_str() {
                        visit(
                            id,
                            nodes,
                            part["role"].as_str().unwrap_or("user"),
                            part["scope"].as_str().unwrap_or("conversation"),
                            manifest["request"]["id"].as_str().unwrap_or(""),
                            &mut vec![],
                            &mut out,
                        );
                    }
                }
            }
        }
    }
    out
}

</storycode:copy>
<a id="f-a93965a16b0e-compare"></a>
### `compare`

Conservative mapping: never infer which duplicate passage the user meant.

<storycode:copy id="f-a93965a16b0e-compare">
/// Conservative mapping: never infer which duplicate passage the user meant.
pub fn compare(old: &Exposure, current: Option<&str>) -> (&'static str, String) {
    let Some(current) = current else {
        return ("deleted_or_unavailable", String::new());
    };
    let revision = hick_context_protocol::digest(current.as_bytes());
    if revision == old.source.revision {
        return ("unchanged", revision);
    }
    let positions: Vec<_> = current
        .char_indices()
        .filter_map(|(i, _)| current[i..].starts_with(&old.text).then_some(i))
        .take(2)
        .collect();
    let status = if old.text.is_empty() || (!positions.is_empty() && !old.unique) {
        "ambiguous"
    } else {
        match positions.as_slice() {
            [] => "changed_observed_passage",
            [position] if *position as u64 == old.start => "changed_elsewhere",
            [_] => "moved",
            _ => "ambiguous",
        }
    };
    (status, revision)
}

</storycode:copy>
<a id="f-a93965a16b0e-tests"></a>
### `tests`

<storycode:copy id="f-a93965a16b0e-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/explicit-context-updates.md
    #[test]
    fn queue_and_failure_do_not_advance_baseline_or_cross_branches() {
        let mut inbox = Inbox::default();
        inbox.fork("a", None);
        let source = hick_context_protocol::Source {
            id: "file".into(),
            kind: "saved_file".into(),
            locator: "AGENTS.md".into(),
            revision: "old".into(),
        };
        inbox.publish(
            "a",
            vec![Exposure {
                source: source.clone(),
                text: "old".into(),
                reference: "n".into(),
                unique: true,
                start: 0,
                role: "user".into(),
                scope: "child".into(),
                request: "r0".into(),
            }],
        );
        inbox.0.get_mut("a").unwrap().updates.push(Update {
            id: "u".into(),
            old: "n".into(),
            source,
            text: Some("second".into()),
            role: "user".into(),
            scope: "child".into(),
            state: "queued".into(),
            request: None,
        });
        assert_eq!(inbox.take("a").len(), 1);
        assert!(inbox.take("a").is_empty());
        inbox.delivery("a", "r", "failed", &["u".into()]);
        assert_eq!(inbox.0["a"].exposures[0].text, "old");
        inbox.fork("b", Some("a"));
        inbox.delivery("a", "r2", "included", &["u".into()]);
        assert_eq!(inbox.0["b"].updates[0].state, "failed");
    }
    // Guarantee: docs/guarantees/agent/explicit-context-updates.md
    #[test]
    fn comparison_distinguishes_movement_duplicates_and_edits_elsewhere() {
        let old = Exposure {
            reference: "n".into(),
            unique: true,
            source: hick_context_protocol::Source {
                id: "f".into(),
                kind: "saved_file".into(),
                locator: "f".into(),
                revision: hick_context_protocol::digest(b"read\nother"),
            },
            text: "read".into(),
            start: 0,
            role: "user".into(),
            scope: "conversation".into(),
            request: "r".into(),
        };
        assert_eq!(compare(&old, Some("read\nchanged")).0, "changed_elsewhere");
        assert_eq!(compare(&old, Some("prefix\nread")).0, "moved");
        assert_eq!(compare(&old, Some("read\nread")).0, "ambiguous");
        assert_eq!(
            compare(&old, Some("replacement")).0,
            "changed_observed_passage"
        );
        assert_eq!(compare(&old, None).0, "deleted_or_unavailable");
        let mut uncertain = old.clone();
        uncertain.unique = false;
        assert_eq!(compare(&uncertain, Some("prefix\nread")).0, "ambiguous");
    }
    // Guarantee: docs/guarantees/agent/explicit-context-updates.md
    #[test]
    fn two_sessions_persist_separately_and_do_not_clear_active_branches() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.md");
        let b = dir.path().join("b.md");
        let mut inbox = Inbox::default();
        inbox.open(&a).unwrap();
        inbox.fork("a", None);
        inbox.open(&b).unwrap();
        inbox.fork("b", None);
        inbox.save().unwrap();
        assert!(inbox.0.contains_key("a"));
        let mut restarted = Inbox::default();
        restarted.open(&a).unwrap();
        assert!(restarted.0.contains_key("a"));
        assert!(!restarted.0.contains_key("b"));
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/context_updates.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/context_updates.rs"><storycode:paste select="#f-a93965a16b0e-Exposure"/><storycode:paste select="#f-a93965a16b0e-Update"/><storycode:paste select="#f-a93965a16b0e-Branch"/><storycode:paste select="#f-a93965a16b0e-Inbox"/><storycode:paste select="#f-a93965a16b0e-impl-Inbox"/><storycode:paste select="#f-a93965a16b0e-exposures"/><storycode:paste select="#f-a93965a16b0e-compare"/><storycode:paste select="#f-a93965a16b0e-tests"/></storycode:file>

## Test the behavior: `explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_python_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-08f0cd3133cc-explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary"></a>
### `explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary`

<storycode:copy id="f-08f0cd3133cc-explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary">
// Guarantee: docs/guarantees/agent/explicit-context-updates.md
#[tokio::test]
async fn explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary() {
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    std::fs::write(app.root.join("AGENTS.md"), "Original instructions\n").unwrap();
    app.configure(&python).await;
    let model = Model::new([
        json!({"python":"tool('read_file', {'path':'AGENTS.md'})"}),
        json!({"done":"Read source"}),
        json!({"done":"Continue"}),
    ]);
    app.state.agent.set_llm_override(model.clone());
    let first = app.send("Read instructions", None).await;
    let outcome = app.wait(&first).await;
    assert_eq!(outcome["status"], "ok", "{outcome}");
    std::fs::write(
        app.root.join("AGENTS.md"),
        "Changed instructions after pull\n",
    )
    .unwrap();
    let context = json!({"buffers":[{"id":"note-editor","name":"Meeting note","path":"note.md","content":"Unsaved change","focused":true}]});
    let body = json!({"branch":first,"context":context});
    let (status, view) = app
        .request("POST", &app.route("/python/context"), body.clone())
        .await;
    assert_eq!(status, 200, "{view}");
    let items = view["items"].as_array().unwrap();
    assert!(
        items
            .iter()
            .any(|i| i["path"] == "AGENTS.md" && i["status"] == "changed_observed_passage"),
        "{view}"
    );
    assert!(
        items
            .iter()
            .any(|i| i["path"] == "note-editor" && i["after"] == "Unsaved change")
    );
    let selected: Vec<_> = items
        .iter()
        .filter(|i| i["eligible"] == true)
        .map(|i| i["reference"].clone())
        .collect();
    let revisions: serde_json::Map<String, Value> = items
        .iter()
        .map(|i| {
            (
                i["reference"].as_str().unwrap().into(),
                i["revision"].clone(),
            )
        })
        .collect();
    let (status, queued) = app
        .request(
            "POST",
            &app.route("/python/context"),
            json!({"branch":first,"context":context,"selected":selected,"revisions":revisions}),
        )
        .await;
    assert_eq!(status, 200, "{queued}");
    assert!(
        queued["updates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["state"] == "queued")
    );
    assert_eq!(
        std::fs::read_to_string(app.root.join("note.md")).unwrap(),
        "Original human input\n"
    );
    std::fs::write(app.root.join("AGENTS.md"), "Changed again while queued\n").unwrap();
    let child = app.send("Continue", Some(&first)).await;
    let outcome = app.wait(&child).await;
    assert_eq!(outcome["status"], "ok", "{outcome}");
    let (_, delivered) = app
        .request(
            "POST",
            &app.route("/python/context"),
            json!({"branch":child,"context":context}),
        )
        .await;
    assert!(
        delivered["updates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["state"] == "provider_confirmed"),
        "{delivered}"
    );
    let requests = model.requests.lock().unwrap();
    assert!(
        requests
            .last()
            .unwrap()
            .iter()
            .any(|m| m.content.contains("Changed instructions after pull"))
    );
    assert!(
        !requests
            .last()
            .unwrap()
            .iter()
            .any(|m| m.content.contains("Changed again while queued"))
    );
    let (_, parent) = app
        .request("POST", &app.route("/python/context"), body)
        .await;
    assert!(
        parent["updates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["state"] == "queued")
    );
}
</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/ContextUpdates.tsx](../../../mechanisms/apps/web/src/components/ContextUpdates.tsx.md#output-83abe8d9f9e9)
- [crates/hickory-cli/src/context_updates.rs](contract.md#output-a93965a16b0e)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/python_agent.rs.md#output-89d47ddc390d)

### Tests and fixtures

- [apps/web/src/components/ContextUpdates.test.tsx](contract.md#output-361e848bd404)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_python_agent.rs.md#output-08f0cd3133cc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
