<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/engine/watch.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-dc5bee2ae567"></a>
## `crates/hickory-cli/src/engine/watch.rs`

Related contracts: [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../../guarantees/agent/acp-agents-are-first-class/contract.md), [The daemon exits with its last client](../../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md), [The Editor Does Not Wait For Workspace Preparation](../../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md).

<a id="f-dc5bee2ae567-Coordinator"></a>
### `Coordinator`

A single event queue and write history, including overlapping folder views.

<storycode:copy id="f-dc5bee2ae567-Coordinator">
//! A single event queue and write history, including overlapping folder views.
use super::Attach;
use super::startup::Startup;
use crate::{
    serve::{
        LocalState,
        watch::{notify_files_changed, publish_held, reconcile_rooms},
    },
    up::{UpCommand, state::WovenState},
};
use anyhow::{Context, Result};
use notify::RecursiveMode;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::{Mutex, mpsc};

pub struct Coordinator {
    task: tokio::task::JoinHandle<()>,
    add: mpsc::UnboundedSender<Registration>,
    pub woven: Arc<Mutex<WovenState>>,
    events: mpsc::UnboundedSender<PathBuf>,
}
</storycode:copy>
<a id="f-dc5bee2ae567-Registration"></a>
### `Registration`

<storycode:copy id="f-dc5bee2ae567-Registration">
struct Registration {
    state: LocalState,
    attach: Attach,
    startup: Startup,
    completed: tokio::sync::watch::Sender<Option<Result<(), String>>>,
}
</storycode:copy>
<a id="f-dc5bee2ae567-View"></a>
### `View`

<storycode:copy id="f-dc5bee2ae567-View">
struct View {
    state: LocalState,
    // Only `hick up` supplies this explicit publication binding. GUI opens do not.
    generation_entry: Option<String>,
    generated_revision: Option<String>,
}

</storycode:copy>
<a id="f-dc5bee2ae567-impl-Coordinator"></a>
### `impl-Coordinator`

Opening subscriptions install observation only. They can index new Markdown and
reconcile saved text into editor rooms, but cannot weave, ingest an inbox or
publish outputs. A separate explicit `hick up` subscription supplies a declared
entry; only that binding can call the shared generation capability. Source changes
regenerate its whole closure once per revision, never each discovered page.

<storycode:copy id="f-dc5bee2ae567-impl-Coordinator">
impl Coordinator {
    pub fn start(
        writes: Arc<Mutex<()>>,
        subscriptions: Arc<std::sync::RwLock<HashMap<String, Attach>>>,
    ) -> Result<Self> {
        let (events, mut rx) = mpsc::unbounded_channel();
        let notify = events.clone();
        let mut watcher = crate::up::file_watcher(move |result: notify::Result<notify::Event>| {
            if let Ok(event) = result {
                if matches!(event.kind, notify::EventKind::Access(_)) { return; }
                for path in event.paths {
                    // Native events may use a filesystem alias (for example
                    // /var versus /private/var) unlike the canonical context.
                    let path = path.canonicalize().unwrap_or(path);
                    if !crate::up::is_noise(&path) {
                        let _ = notify.send(path);
                    }
                }
            }
        })
        .map_err(crate::up::watch_error)?;
        let (add, mut registrations) = mpsc::unbounded_channel::<Registration>();
        let (commands, mut command_rx) = mpsc::unbounded_channel();
        let woven = Arc::new(Mutex::new(WovenState::default()));
        let history = woven.clone();
        let task = tokio::spawn(async move {
            let mut views: HashMap<PathBuf, View> = HashMap::new();
            let mut watched: Vec<PathBuf> = Vec::new();
            let mut shallow = HashSet::new();
            let mut marks = HashMap::new();
            loop {
                tokio::select! {
                    Some(reg) = registrations.recv() => {
                        let _write = writes.lock().await;
                        let root = reg.state.index.root().to_path_buf();
                        reg.startup.progress("watching", 0, 0, None);
                        let result = (|| -> Result<()> {
                            if !watched.iter().any(|p| root.starts_with(p))
                                && (reg.attach.target.is_dir() || !shallow.contains(&root)) {
                                if shallow.remove(&root) { let _ = watcher.unwatch(&root); }
                                watcher.watch(&root, if reg.attach.target.is_dir() { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive })
                                    .map_err(crate::up::watch_error).with_context(|| format!("watching {}", root.display()))?;
                                if reg.attach.target.is_dir() {
                                    let descendants: Vec<_> = watched.iter().filter(|p| p.starts_with(&root)).cloned().collect();
                                    for path in descendants { let _ = watcher.unwatch(&path); watched.retain(|p| p != &path); }
                                    watched.push(root.clone());
                                } else { shallow.insert(root.clone()); }
                            }
                            if reg.attach.target.is_dir() {
                                crate::serve::watch::discover_changed_path(&reg.state, &root, true);
                            }
                            *reg.state.up_commands.lock().unwrap() = Some(commands.clone());
                            let view = views.entry(root.clone()).or_insert(View { state: reg.state, generation_entry: None, generated_revision: None });
                            if let Some(entry) = reg.attach.generation_entry {
                                anyhow::ensure!(view.generation_entry.as_ref().is_none_or(|bound| bound == &entry), "This root already has a different explicit generation entry");
                                let context = hick_project_env::generation::GenerationContext::capture(&root, &entry).map_err(anyhow::Error::msg)?;
                                let revision = context.plan().revision;
                                context.publish(&revision).map_err(anyhow::Error::msg)?;
                                view.generation_entry = Some(entry);
                                view.generated_revision = Some(revision);
                            }
                            Ok(())
                        })();
                        // A normal open grants observation only. The separate
                        // generation_entry opt-in is the CLI's explicit action.
                        let _ = reg.completed.send(Some(result.map_err(|e| format!("{e:#}"))));
                    }
                    Some(command) = command_rx.recv() => {
                        match command {
                            UpCommand::Resolve { request, reply } => {
                                let result = history.lock().await.resolve_output(&request).map_err(|e| format!("{e:#}"));
                                let _ = reply.send(result);
                            }
                            // A run may request a refresh, but it cannot turn this
                            // discovery loop into a generator after startup either.
                            UpCommand::Reweave(_) => {}
                        }
                    }
                    Some(first) = rx.recv() => {
                        let mut batch = HashSet::from([first]);
                        while let Ok(Some(path)) = tokio::time::timeout(std::time::Duration::from_millis(120), rx.recv()).await { batch.insert(path); }
                        while !crate::up::settle_batch(&batch).await {
                            while let Ok(path) = rx.try_recv() { batch.insert(path); }
                        }
                        let _write = writes.lock().await;
                        let active: Vec<_> = subscriptions.read().unwrap().values().cloned().collect();
                        for path in &batch {
                            for (root, view) in &views {
                                let scope = if path.starts_with(root) { path.as_path() }
                                    else if root.starts_with(path) && path.is_dir() { root.as_path() }
                                    else { continue; };
                                if active.iter().any(|sub| sub.target.is_dir() && scope.starts_with(&sub.target)) {
                                    crate::serve::watch::discover_changed_path(&view.state, scope, true);
                                }
                            }
                        }
                        // This branch is reachable only for a separate explicit
                        // headless subscription, never because a scan found a page.
                        for (root, view) in &mut views {
                            let Some(entry) = &view.generation_entry else { continue; };
                            if !active.iter().any(|sub| sub.generation_entry.as_ref() == Some(entry)
                                && (sub.target == *root || sub.target.parent() == Some(root.as_path()))) {
                                view.generation_entry = None;
                                view.generated_revision = None;
                                continue;
                            }
                            if !batch.iter().any(|path| path.starts_with(root) || (root.starts_with(path) && path.is_dir())) { continue; }
                            let result = (|| -> Result<()> {
                                let context = hick_project_env::generation::GenerationContext::capture(root, entry).map_err(anyhow::Error::msg)?;
                                let revision = context.plan().revision;
                                if view.generated_revision.as_ref() != Some(&revision) {
                                    context.publish(&revision).map_err(anyhow::Error::msg)?;
                                    view.generated_revision = Some(revision);
                                }
                                Ok(())
                            })();
                            if let Err(error) = result { eprintln!("generation held: {error:#}"); }
                        }
                        let woven = history.lock().await;
                        publish(&views, &woven, &mut marks).await;
                    }
                    else => break,
                }
            }
            history.lock().await.release_read_only();
        });
        Ok(Self {
            task,
            add,
            woven,
            events,
        })
    }

    pub async fn shutdown(&self) {
        self.task.abort();
        self.woven.lock().await.release_read_only();
    }

    pub fn refresh(&self, docs: Vec<PathBuf>) {
        for doc in docs {
            let _ = self.events.send(doc);
        }
    }

    pub async fn register(&self, state: LocalState, attach: Attach) -> Result<Startup> {
        let (completed, startup) = Startup::channel();
        self.add
            .send(Registration {
                state,
                attach,
                completed,
                startup: startup.clone(),
            })
            .map_err(|_| anyhow::anyhow!("engine watcher stopped"))?;
        Ok(startup)
    }
}
</storycode:copy>
<a id="f-dc5bee2ae567-publish"></a>
### `publish`

<storycode:copy id="f-dc5bee2ae567-publish">
async fn publish(
    views: &HashMap<PathBuf, View>,
    woven: &WovenState,
    marks: &mut HashMap<PathBuf, u64>,
) {
    for (root, view) in views {
        view.state.store.register(&view.state.index);
        reconcile_rooms(&view.state, root, woven).await;
        publish_held(&view.state, root, woven);
        // Marks are per view: sharing one map would suppress the second view's notification.
        let mut view_marks = marks.clone();
        notify_files_changed(&view.state, root, woven, &mut view_marks).await;
    }
    if let Some((root, view)) = views.iter().next() {
        notify_files_changed(&view.state, root, woven, marks).await;
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/engine/watch.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/engine/watch.rs"><storycode:paste select="#f-dc5bee2ae567-Coordinator"/><storycode:paste select="#f-dc5bee2ae567-Registration"/><storycode:paste select="#f-dc5bee2ae567-View"/><storycode:paste select="#f-dc5bee2ae567-impl-Coordinator"/><storycode:paste select="#f-dc5bee2ae567-publish"/></storycode:file>

</storycode:doc>
