<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/bootstrap.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f374ffa94b18"></a>
## `crates/hickory-cli/src/serve/bootstrap.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-f374ffa94b18-Prepared"></a>
### `Prepared`

<storycode:copy id="f-f374ffa94b18-Prepared">
use super::*;

/// A prepared session: the router, and the state a caller that wants its own
/// listener needs to talk to it.
pub struct Prepared {
    pub router: Router,
    pub state: LocalState,
}

</storycode:copy>
<a id="f-f374ffa94b18-prepare"></a>
### `prepare`

Build a session without binding a socket.
Split out from [`serve`] so the collaboration guarantees can be driven by a
test over a real socket on an ephemeral port, and so the desktop app can
mount this router on a listener it owns.

<storycode:copy id="f-f374ffa94b18-prepare">
/// Build a session without binding a socket.
///
/// Split out from [`serve`] so the collaboration guarantees can be driven by a
/// test over a real socket on an ephemeral port, and so the desktop app can
/// mount this router on a listener it owns.
pub async fn prepare(opts: ServeOptions) -> Result<Prepared> {
    prepare_inner(opts, None, None).await
}

</storycode:copy>
<a id="f-f374ffa94b18-prepare_without_folder"></a>
### `prepare_without_folder`

An editor-only window: the target is internal session storage, not an open folder.

<storycode:copy id="f-f374ffa94b18-prepare_without_folder">
/// An editor-only window: the target is internal session storage, not an open folder.
pub async fn prepare_without_folder(opts: ServeOptions) -> Result<Prepared> {
    prepare_inner(opts, Some(false), None).await
}

</storycode:copy>
<a id="f-f374ffa94b18-prepare_with_prediction"></a>
### `prepare_with_prediction`

A startup-only, bounded private service. Unsupported ordinary surfaces refuse;
this does not enable a native application flag or attach an unguarded agent.

<storycode:copy id="f-f374ffa94b18-prepare_with_prediction">
/// A startup-only, bounded private service. Unsupported ordinary surfaces refuse;
/// this does not enable a native application flag or attach an unguarded agent.
pub async fn prepare_with_prediction(
    opts: ServeOptions,
    binding: PrivatePredictionBinding,
) -> Result<Prepared> {
    prepare_inner(opts, None, Some(binding)).await
}
</storycode:copy>
<a id="f-f374ffa94b18-prepare_inner"></a>
### `prepare_inner`

<storycode:copy id="f-f374ffa94b18-prepare_inner">
async fn prepare_inner(
    opts: ServeOptions,
    folder_open: Option<bool>,
    prediction: Option<PrivatePredictionBinding>,
) -> Result<Prepared> {
    let target = opts
        .target
        .canonicalize()
        .with_context(|| format!("no such path: {}", opts.target.display()))?;
    let root = if target.is_dir() {
        target.clone()
    } else {
        target
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    // Scan the target to decide whether there is anything to serve, then index
    // the directory that contains it: rooms and path resolution work from the
    // root, so a single-file session can still follow `hick:upstream` beside
    // it.
    // An empty folder is a valid session, not an error: it is the app's
    // first-run state (the default workspace starts with no documents, and
    // the UI lands on a fresh untitled one). A FILE target that does not
    // exist is still refused below by DocIndex::scan.
    let index = Arc::new(DocIndex::scan(&root)?);

    // Provider keys load before the first request so a bad file fails here,
    // with the file named, rather than as a mystery 500 on the first agent
    // turn. No path (the CLI) means an empty store: every store-aware code
    // path then degrades to exactly the env-only behavior.
    let key_store = match &opts.key_store_path {
        Some(path) => hickory_agent::KeyStore::load(path)?,
        None => hickory_agent::KeyStore::default(),
    };

    // Same story for the UI settings: a bad file fails here, named, rather
    // than as a mystery on the first Settings visit.
    let ui_store = match &opts.ui_settings_path {
        Some(path) => UiStore::load(path)?,
        None => UiStore::default(),
    };

    // A bad HICKORY_TERM_SCROLLBACK fails here, naming the variable, rather
    // than when a session happens to overflow its buffer an hour later.
    let term_config = hick_term::TermConfig::from_env()?;

    let store = FileDocStore::new(index.clone());
    let mut state = LocalState {
        private_prediction: None,
        ui_actions: Arc::new(ui_actions::Hub::default()),
        fleet_pairings: Arc::new(fleet_pairings::Pairings::default()),
        opened_files: Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        representations: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        bisects: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        folder_open: folder_open.unwrap_or_else(|| target.is_dir()),
        store: store.clone(),
        index: index.clone(),
        rooms: Arc::new(RoomRegistry::new(store)),
        runs: Arc::new(Mutex::new(HashMap::new())),
        observations: Arc::new(std::sync::Mutex::new(None)),
        params: Arc::new(opts.params),
        executor: opts.executor,
        search: Arc::new(std::sync::Mutex::new(None)),
        agent: Arc::new(agent::AgentHub::default()),
        keys: Arc::new(KeySettings {
            store: std::sync::RwLock::new(key_store),
            path: opts.key_store_path,
        }),
        ui: Arc::new(UiSettings {
            store: std::sync::RwLock::new(ui_store),
            path: opts.ui_settings_path,
        }),
        refactors: Arc::new(Mutex::new(HashMap::new())),
        terminals: Arc::new(hick_term::Terminals::new(term_config)),
        environments: Arc::new(environments::Environments::default()),
        anchors: Arc::new(anchored::Anchors::default()),
        lsp: Arc::new(lsp_bridge::LspHub::new(index.root())),
        held: Arc::new(std::sync::Mutex::new(HashMap::new())),
        scaffolds: Arc::new(Mutex::new(HashMap::new())),
        shell: Arc::new(Mutex::new(None)),
        python_runtime_path: None,
        up_commands: Arc::new(std::sync::Mutex::new(None)),
        writes: Arc::new(tokio::sync::Mutex::new(())),
        window_slot: None,
        engine_woven: None,
    };

    if let Some(binding) = prediction {
        anyhow::ensure!(
            target.is_file(),
            "A private prediction session needs one exact source file"
        );
        state.private_prediction =
            Some(private_predictions::Bound::open(&state, target, binding).await?);
    }
    Ok(Prepared {
        router: router(state.clone()),
        state,
    })
}

</storycode:copy>
<a id="f-f374ffa94b18-serve"></a>
### `serve`

Start the session on loopback and serve until interrupted.
Loopback is not a default that a flag can change. A process that starts
listening on `0.0.0.0` has made a decision about who may reach the user's
files, and this product does not make that decision at all — see
`docs/specs/freeform/local-only.md`.

<storycode:copy id="f-f374ffa94b18-serve">
/// Start the session on loopback and serve until interrupted.
///
/// Loopback is not a default that a flag can change. A process that starts
/// listening on `0.0.0.0` has made a decision about who may reach the user's
/// files, and this product does not make that decision at all — see
/// `docs/specs/freeform/local-only.md`.
pub async fn serve(opts: ServeOptions) -> Result<()> {
    let port = opts.port;
    let prepared = prepare(opts).await?;

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    let bound = listener.local_addr()?;
    log::info!("local session on http://{bound}");

    axum::serve(
        listener,
        prepared
            .router
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/bootstrap.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/bootstrap.rs"><storycode:paste select="#f-f374ffa94b18-Prepared"/><storycode:paste select="#f-f374ffa94b18-prepare"/><storycode:paste select="#f-f374ffa94b18-prepare_without_folder"/><storycode:paste select="#f-f374ffa94b18-prepare_with_prediction"/><storycode:paste select="#f-f374ffa94b18-prepare_inner"/><storycode:paste select="#f-f374ffa94b18-serve"/></storycode:file>

</storycode:doc>
