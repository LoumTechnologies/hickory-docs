<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/desktop/src-tauri/build.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Desktop Build Carries The UI](../../../../contract.md)

Captured SHA-256: `d3bf1bd64a31b7a5c80d17add4570d4b62c6a1df36c6e7372c8eca8b3f4b1246`.

<book:entry path="apps/desktop/src-tauri/build.rs">
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
</book:entry>

</book:doc>
