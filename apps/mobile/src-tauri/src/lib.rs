//! Hickory Docs mobile/desktop shell. All UI comes from apps/web — this crate
//! contributes only the native window; nothing else diverges between targets.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running Hickory Docs");
}
