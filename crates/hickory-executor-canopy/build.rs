fn main() {
    // Prefer an externally provided protoc; fall back to the vendored binary
    // so a plain `cargo build` needs no system protobuf install.
    if std::env::var_os("PROTOC").is_none() {
        // SAFETY: build scripts are single-threaded at this point; setting
        // PROTOC before invoking tonic-build is the documented mechanism.
        unsafe {
            std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
        }
    }
    tonic_build::compile_protos("proto/canopy.proto").unwrap();
    println!("cargo:rerun-if-changed=proto/canopy.proto");
}
