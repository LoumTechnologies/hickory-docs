fn main() {
    // rust-embed discovers assets during macro expansion. Cargo must also
    // watch the directory: rebuilding the UI can add/remove hashed filenames
    // without changing any Rust source (including after an empty dist build).
    println!("cargo:rerun-if-changed=../../web/dist");
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        assert!(
            std::path::Path::new("../../web/dist/index.html").is_file(),
            "The desktop release requires the built UI. Run `just build-web` or build through `cargo tauri build`."
        );
    }
    tauri_build::build()
}
