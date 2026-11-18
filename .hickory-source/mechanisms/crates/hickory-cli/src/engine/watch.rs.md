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
use super::{Attach, GenerationBinding};
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
    // Each admitted headless watch keeps its own role policy. Ordinary
    // editor windows never add a generation binding.
    generation_bindings: Vec<GenerationBinding>,
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
                            let view = views.entry(root.clone()).or_insert(View {
                                state: reg.state, generation_bindings: Vec::new(), generated_revision: None,
                            });
                            if let Some(binding) = reg.attach.generation_binding()? {
                                let active = subscriptions.read().unwrap().values().cloned().collect::<Vec<_>>();
                                view.state.resource_root.check(&root).map_err(anyhow::Error::msg)?;
                                let held = entry_lease(&root, &binding)?;
                                let mut retained = view.generation_bindings.clone();
                                retained.retain(|old| generation_is_active(&root, old, &active)
                                    && old.implicit.as_ref().is_none_or(|stamp| held.matches_current(stamp)));
                                anyhow::ensure!(
                                    retained.iter().all(|old| old.entry == binding.entry),
                                    "This root already has a different live generation entry"
                                );
                                let revision = publish_generation(&root, &binding, &held, None)?;
                                if !retained.contains(&binding) { retained.push(binding); }
                                view.generation_bindings = retained;
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
                        // Only a live headless subscription may publish. A
                        // stale implicit stamp cannot become current again,
                        // even if the next role later returns to its address.
                        for (root, view) in &mut views {
                            view.generation_bindings.retain(|binding| generation_is_active(root, binding, &active));
                            if view.generation_bindings.is_empty() {
                                view.generated_revision = None;
                                continue;
                            }
                            if !batch.iter().any(|path| path.starts_with(root) || (root.starts_with(path) && path.is_dir())) {
                                continue;
                            }
                            let mut last_failure = None;
                            for binding in &view.generation_bindings {
                                let result = (|| {
                                    view.state.resource_root.check(root).map_err(anyhow::Error::msg)?;
                                    let held = entry_lease(root, binding)?;
                                    publish_generation(root, binding, &held, view.generated_revision.as_deref())
                                })();
                                match result {
                                    Ok(revision) => {
                                        view.generated_revision = Some(revision);
                                        last_failure = None;
                                        break;
                                    }
                                    Err(error) => last_failure = Some(error),
                                }
                            }
                            if let Some(error) = last_failure { eprintln!("generation held: {error:#}"); }
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

## Hold one generation binding through capture and publication

The watcher compares the complete subscription, including any entry
generation. An explicit watch for the same address has its own authority.
The held read lease blocks a competing entry transition while capture and
publication run. Generated output snapshots still go through their existing
revision and physical publication checks.

<storycode:copy id="engine-generation-entry-lease">
fn generation_is_active(root: &std::path::Path, binding: &GenerationBinding, active: &[Attach]) -> bool {
    active.iter().any(|subscription| {
        subscription.generation_entry.as_ref() == Some(&binding.entry) &&
            subscription.implicit_entry == binding.implicit &&
            (subscription.target == root ||
                binding.implicit.is_none() && subscription.target.parent() == Some(root))
    })
}

fn entry_lease(
    root: &std::path::Path,
    binding: &GenerationBinding,
) -> Result<crate::serve::workspace_entry_authority::EntryReadLease> {
    crate::serve::workspace_entry_authority::acquire_entry_read_lease(root, binding.implicit.as_ref())
        .map_err(anyhow::Error::msg)
}

fn publish_generation(
    root: &std::path::Path,
    binding: &GenerationBinding,
    held: &crate::serve::workspace_entry_authority::EntryReadLease,
    previous: Option<&str>,
) -> Result<String> {
    held.check().map_err(anyhow::Error::msg)?;
    let context = hick_project_env::generation::GenerationContext::capture(root, &binding.entry)
        .map_err(anyhow::Error::msg)?;
    let revision = context.plan().revision;
    held.check().map_err(anyhow::Error::msg)?;
    if previous != Some(revision.as_str()) {
        context.publish(&revision).map_err(anyhow::Error::msg)?;
    }
    Ok(revision)
}

#[cfg(test)]
mod workspace_entry_watch_tests {
    use super::*;
    use crate::serve::workspace_entry_authority::EntryStamp;

    #[test]
    fn an_explicit_watch_and_a_new_role_do_not_keep_an_old_implicit_binding_active() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let entry = "project.md".to_owned();
        let mut implicit = Attach::new(root.to_path_buf(), crate::ExecutorChoice::Local);
        implicit.generation_entry = Some(entry.clone());
        implicit.implicit_entry = Some(EntryStamp {
            version: 1, identity: "a".repeat(32), generation: 1, path: Some(entry.clone()),
        });
        let original = implicit.generation_binding().unwrap().unwrap();
        let mut restored = implicit.clone();
        restored.implicit_entry.as_mut().unwrap().generation = 3;
        let restored_binding = restored.generation_binding().unwrap().unwrap();

        let mut explicit = Attach::new(root.join(&entry), crate::ExecutorChoice::Local);
        explicit.generation_entry = Some(entry.clone());
        let explicit_binding = explicit.generation_binding().unwrap().unwrap();
        let living = [restored, explicit];
        assert!(!generation_is_active(root, &original, &living));
        assert!(generation_is_active(root, &restored_binding, &living));
        assert!(generation_is_active(root, &explicit_binding, &living));

        implicit.background = true;
        assert!(implicit.generation_binding().is_err());
        implicit.background = false;
        implicit.generation_entry = Some("foreign.md".into());
        assert!(implicit.generation_binding().is_err());
    }
}
</storycode:copy>

<storycode:file path="crates/hickory-cli/src/engine/watch.rs"><storycode:paste select="#f-dc5bee2ae567-Coordinator"/><storycode:paste select="#f-dc5bee2ae567-Registration"/><storycode:paste select="#f-dc5bee2ae567-View"/><storycode:paste select="#f-dc5bee2ae567-impl-Coordinator"/><storycode:paste select="#f-dc5bee2ae567-publish"/><storycode:paste select="#engine-generation-entry-lease"/></storycode:file>

</storycode:doc>
