//! A packaged-app smoke entry point, so runtime evidence exercises the actual
//! extension without copying/re-signing an installed app or changing its notes.
use super::{Engine, Host, Mount};
use anyhow::{Result, ensure};
use std::io::Write as _;

pub fn smoke_argv(args: &[String]) -> Option<Result<()>> {
    if args.first().map(String::as_str) != Some("--hickory-workspace-smoke") {
        return None;
    }
    Some((|| tokio::runtime::Runtime::new()?.block_on(run()))())
}
async fn run() -> Result<()> {
    let dir = tempfile::Builder::new()
        .prefix("hickory-fskit-smoke-")
        .tempdir()?;
    let source = r##"<hick:copy id="greeting">hello
</hick:copy>
<hick:file path="greeting.txt"><hick:paste select="#greeting" /></hick:file>
"##;
    std::fs::write(dir.path().join("note.md"), source)?;
    let prepared = super::super::prepare(super::super::ServeOptions {
        target: dir.path().into(),
        port: 0,
        params: vec![],
        executor: crate::ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await?;
    let state = prepared.state;
    let id = state.index.sole().unwrap().0;
    let room = state.rooms.get_or_create(&id).await?;
    let engine = Engine::open(state, dir.path().join("access.hick"))?;
    let mount = Mount::start(Host::start(engine).await?).await?;
    let path = mount.path.join("greeting.txt");
    ensure!(
        std::fs::read_to_string(&path)? == "hello\n",
        "mounted output did not reproduce the source"
    );
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(b"welcome\n")?;
        file.sync_all()?;
    }
    ensure!(
        room.text().await == source.replace("hello", "welcome"),
        "mounted save did not reach the live source"
    );
    ensure!(
        std::fs::read_to_string(&path)? == "welcome\n",
        "fresh mounted output did not reflect the save"
    );
    let source_file = std::fs::read_to_string(dir.path().join("note.md"))?;
    ensure!(
        source_file.contains("welcome"),
        "mounted save did not persist the source"
    );
    drop(mount);
    println!(
        "Native FSKit read, reverse save, live editor, source persistence, and fresh output passed in a temporary workspace."
    );
    Ok(())
}
