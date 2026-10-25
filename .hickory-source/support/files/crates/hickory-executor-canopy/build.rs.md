<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-executor-canopy/build.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e8dfc632e2cec443652d6d50812f47cfaa7f9f97478c3a73b172c964ce342fd8`.

<book:entry path="crates/hickory-executor-canopy/build.rs">
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
</book:entry>

</book:doc>
