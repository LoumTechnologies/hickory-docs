use hickory_fleet::{Fleet, Identity, Invitation, Kind};
use hickory_mobile_core::{
    drafts::{Draft, Drafts},
    peer::{Peer, Room, RoomEvent},
};
use hickory_peer::{Phrase, Reach};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{Manager, ipc::Channel};

struct Mobile {
    base: PathBuf,
    drafts: Mutex<Drafts>,
    peer: tokio::sync::Mutex<Option<Arc<Peer>>>,
    room: tokio::sync::Mutex<Option<(String, Room)>>,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Action {
    Init,
    Save {
        draft: Draft,
    },
    Accept {
        invitation: String,
    },
    Pair {
        phrase: String,
        reach: String,
    },
    Connect {
        machine: String,
        address: String,
        reach: String,
    },
    Disconnect,
    Request {
        method: String,
        path: String,
        body: Value,
    },
}

fn trusted(window: &tauri::WebviewWindow) -> Result<(), String> {
    let url = window.url().map_err(|e| e.to_string())?;
    let local = match (url.scheme(), url.host_str()) {
        ("tauri", Some("localhost")) => true,
        ("http" | "https", Some("tauri.localhost")) => true,
        ("http", Some("localhost")) if cfg!(debug_assertions) => url.port() == Some(1420),
        _ => false,
    };
    if window.label() != "main" || !local {
        return Err("Native mobile actions require the local app window".into());
    }
    Ok(())
}
fn reach(value: &str) -> Result<Reach, String> {
    match value {
        "direct" => Ok(Reach::Direct),
        "default" => Ok(Reach::Default),
        url if url.starts_with("https://") => Ok(Reach::Own(url.into())),
        _ => Err("Choose direct, default, or an HTTPS relay you control".into()),
    }
}
fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
#[tauri::command]
async fn mobile(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, Mobile>,
    action: Action,
) -> Result<Value, String> {
    trusted(&window)?;
    match action {
        Action::Save { draft } => {
            let saved = state
                .drafts
                .lock()
                .map_err(error)?
                .save(draft)
                .map_err(error)?;
            Ok(json!(saved))
        }
        Action::Init => {
            let identity = Identity::load_or_create(&state.base, "phone").map_err(error)?;
            let drafts = state.drafts.lock().map_err(error)?.list().map_err(error)?;
            Ok(
                json!({ "drafts":drafts, "invitation":identity.invitation(Kind::Phone).encode(), "machines":Fleet::at(identity.dir()).machines() }),
            )
        }
        Action::Accept { invitation } => {
            let identity = Identity::load_or_create(&state.base, "phone").map_err(error)?;
            let invitation = Invitation::decode(&invitation).map_err(error)?;
            let machine = Fleet::at(identity.dir())
                .add(&invitation, "")
                .map_err(error)?;
            Ok(json!(machine))
        }
        Action::Pair {
            phrase,
            reach: mode,
        } => {
            let identity = Identity::load_or_create(&state.base, "phone").map_err(error)?;
            let fleet = Fleet::at(identity.dir());
            let paired = hickory_peer::join(
                &identity,
                &fleet,
                &Phrase::parse(&phrase).map_err(error)?,
                Kind::Phone,
                &reach(&mode)?,
                "",
            )
            .await
            .map_err(error)?;
            Ok(json!(paired.machine))
        }
        Action::Connect {
            machine,
            address,
            reach: mode,
        } => {
            let identity = Identity::load_or_create(&state.base, "phone").map_err(error)?;
            let fleet = Fleet::at(identity.dir());
            let addr = serde_json::from_str(&address).map_err(error)?;
            let connected = Arc::new(
                Peer::connect(&identity, &fleet, &machine, addr, reach(&mode)?)
                    .await
                    .map_err(error)?,
            );
            state.room.lock().await.take();
            let old = state.peer.lock().await.replace(connected.clone());
            if let Some(old) = old {
                old.close().await;
            }
            Ok(json!({ "machine":machine, "reachability":connected.how().await }))
        }
        Action::Disconnect => {
            state.room.lock().await.take();
            if let Some(peer) = state.peer.lock().await.take() {
                peer.close().await;
            }
            Ok(json!(null))
        }
        Action::Request { method, path, body } => {
            let peer = state
                .peer
                .lock()
                .await
                .clone()
                .ok_or("Connect to a desktop first")?;
            peer.request(&method, &path, body).await.map_err(error)
        }
    }
}
#[tauri::command]
async fn mobile_room_open(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, Mobile>,
    id: String,
    doc: String,
    events: Channel<RoomEvent>,
) -> Result<(), String> {
    trusted(&window)?;
    let mut room = state.room.lock().await;
    room.take();
    let peer = state
        .peer
        .lock()
        .await
        .clone()
        .ok_or("Connect to a desktop first")?;
    let opened = peer
        .room(&doc, move |event| {
            let _ = events.send(event);
        })
        .await
        .map_err(error)?;
    *room = Some((id, opened));
    Ok(())
}
#[tauri::command]
async fn mobile_room_send(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, Mobile>,
    id: String,
    bytes: Vec<u8>,
) -> Result<(), String> {
    trusted(&window)?;
    let room = state.room.lock().await;
    let (current, room) = room.as_ref().ok_or("Desktop room is disconnected")?;
    if current != &id {
        return Err("This editor's desktop room has closed".into());
    }
    room.send(bytes).map_err(error)
}
#[tauri::command]
async fn mobile_room_close(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, Mobile>,
    id: String,
) -> Result<(), String> {
    trusted(&window)?;
    let mut room = state.room.lock().await;
    if room.as_ref().is_some_and(|(current, _)| current == &id) {
        room.take();
    }
    Ok(())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let base = app.path().app_local_data_dir()?;
            let drafts = Drafts::new(&base.join("documents"))?;
            app.manage(Mobile {
                base,
                drafts: Mutex::new(drafts),
                peer: Default::default(),
                room: Default::default(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            mobile,
            mobile_room_open,
            mobile_room_send,
            mobile_room_close
        ])
        .run(tauri::generate_context!())
        .expect("Hickory mobile could not start");
}
