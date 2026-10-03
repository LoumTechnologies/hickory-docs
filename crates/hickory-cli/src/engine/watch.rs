//! A single event queue and write history, including overlapping folder views.
use super::Attach;
use crate::{
    serve::{
        LocalState,
        watch::{notify_files_changed, publish_held, reconcile_rooms},
    },
    up::{UpCommand, UpConfig, state::WovenState},
};
use anyhow::{Context, Result};
use notify::{RecursiveMode, Watcher};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::{Mutex, mpsc, oneshot};

pub struct Coordinator {
    add: mpsc::UnboundedSender<Registration>,
    pub woven: Arc<Mutex<WovenState>>,
    events: mpsc::UnboundedSender<PathBuf>,
}
struct Registration {
    state: LocalState,
    attach: Attach,
    ready: oneshot::Sender<Result<()>>,
}
struct View {
    state: LocalState,
    config: UpConfig,
}

impl Coordinator {
    pub fn start(
        writes: Arc<Mutex<()>>,
        subscriptions: Arc<std::sync::RwLock<HashMap<String, Attach>>>,
    ) -> Result<Self> {
        let (events, mut rx) = mpsc::unbounded_channel();
        let notify = events.clone();
        let queued = events.clone();
        let mut watcher =
            notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                if let Ok(event) = result {
                    for path in event.paths {
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
        tokio::spawn(async move {
            let mut views: HashMap<PathBuf, View> = HashMap::new();
            let mut watched: Vec<PathBuf> = Vec::new();
            let mut marks = HashMap::new();
            loop {
                tokio::select! {
                    Some(reg) = registrations.recv() => {
                        let _write = writes.lock().await;
                        let mut woven = history.lock().await;
                        let root = reg.state.index.root().to_path_buf();
                        let result = async {
                            if !watched.iter().any(|p| root.starts_with(p)) {
                                watcher.watch(&root, RecursiveMode::Recursive).map_err(crate::up::watch_error)
                                    .with_context(|| format!("watching {}", root.display()))?;
                                let descendants: Vec<_> = watched.iter().filter(|p| p.starts_with(&root)).cloned().collect();
                                for path in descendants { let _ = watcher.unwatch(&path); watched.retain(|p| p != &path); }
                                watched.push(root.clone());
                            }
                            *reg.state.up_commands.lock().unwrap() = Some(commands.clone());
                            crate::up::drain_inbox(&root, &crate::ingest::InboxConfig::from_env()?);
                            let config = UpConfig { root: reg.attach.target.clone(), params: reg.attach.params,
                                executor: reg.attach.executor, run: reg.attach.run };
                            let docs: Vec<_> = reg.state.index.entries().iter()
                                .filter_map(|(id, _)| reg.state.index.absolute(id))
                                .filter(|path| woven.doc_source(path).is_none()).collect();
                            if config.run { crate::up::establish_baselines(&docs, &config, &mut woven).await; }
                            for doc in docs {
                                if let Err(error) = crate::up::weave_document(&doc, &config, &mut woven).await {
                                    eprintln!("error: {error:#}");
                                }
                            }
                            // --run is a live client subscription, set separately by daemon leases.
                            views.entry(root.clone()).or_insert(View { state: reg.state.clone(), config });
                            reconcile_rooms(&reg.state, &root, &woven).await;
                            publish_held(&reg.state, &root, &woven);
                            Ok(())
                        }.await;
                        let _ = reg.ready.send(result);
                    }
                    Some(command) = command_rx.recv() => {
                        let _write = writes.lock().await;
                        let mut woven = history.lock().await;
                        let result = match command {
                            UpCommand::Regenerate(path) => woven.restore_output(&path),
                            UpCommand::Resolve(path, content) => woven.resolve_output(&path, &content),
                            UpCommand::Reweave(path) => { let _ = queued.send(path); Ok(()) },
                        };
                        if let Err(error) = result { eprintln!("error: {error:#}"); }
                        publish(&views, &woven, &mut marks).await;
                    }
                    Some(first) = rx.recv() => {
                        let mut batch = HashSet::from([first]);
                        while let Ok(Some(path)) = tokio::time::timeout(std::time::Duration::from_millis(120), rx.recv()).await { batch.insert(path); }
                        while !crate::up::settle_batch(&batch).await {
                            while let Ok(path) = rx.try_recv() { batch.insert(path); }
                        }
                        let _write = writes.lock().await;
                        let mut woven = history.lock().await;
                        // A physical path is processed once, choosing the narrowest view's config.
                        let mut groups: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
                        for path in batch {
                            if let Some(root) = views.keys().filter(|p| path.starts_with(p)).max_by_key(|p| p.components().count()) {
                                groups.entry(root.clone()).or_default().insert(path);
                            }
                        }
                        for (root, mut batch) in groups {
                            let view = &views[&root];
                            if let Ok(inbox) = crate::ingest::InboxConfig::from_env()
                                && batch.iter().any(|p| p.starts_with(inbox.inbox(&root)))
                            {
                                batch.extend(crate::up::drain_inbox(&root, &inbox));
                            }
                            for path in &batch {
                                if path.is_file() && path.extension().is_some_and(|e| e == "hick" || e == "md") {
                                    for (root, view) in &views {
                                        if let Ok(rel) = path.strip_prefix(root) { view.state.index.add(&rel.to_string_lossy().replace('\\', "/")); }
                                        view.state.store.register(&view.state.index);
                                    }
                                }
                            }
                            let active: Vec<_> = subscriptions.read().unwrap().values().cloned().collect();
                            let config_for = |doc: &std::path::Path| {
                                let mut config = UpConfig { root: view.config.root.clone(), params: view.config.params.clone(), executor: view.config.executor, run: false };
                                if let Some(sub) = active.iter().find(|sub| sub.run && (doc == sub.target || doc.starts_with(&sub.target))) {
                                    config.run = true; config.params = sub.params.clone(); config.executor = sub.executor;
                                }
                                config
                            };
                            if let Err(error) = crate::up::handle_batch_with(batch, &view.config, &mut woven, Some(&config_for)).await { eprintln!("error: {error:#}"); }
                        }
                        publish(&views, &woven, &mut marks).await;
                    }
                    else => break,
                }
            }
            history.lock().await.release_read_only();
        });
        Ok(Self { add, woven, events })
    }

    pub fn refresh(&self, docs: Vec<PathBuf>) {
        for doc in docs {
            let _ = self.events.send(doc);
        }
    }

    pub async fn register(&self, state: LocalState, attach: Attach) -> Result<()> {
        let (ready, rx) = oneshot::channel();
        self.add
            .send(Registration {
                state,
                attach,
                ready,
            })
            .map_err(|_| anyhow::anyhow!("engine watcher stopped"))?;
        rx.await?
    }
}

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
