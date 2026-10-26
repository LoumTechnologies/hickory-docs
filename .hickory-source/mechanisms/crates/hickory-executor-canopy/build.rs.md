<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-executor-canopy/build.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a6380aac05ba"></a>
## `crates/hickory-executor-canopy/build.rs`

<a id="f-a6380aac05ba-main"></a>
### `main`

<storycode:copy id="f-a6380aac05ba-main">
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
</storycode:copy>

### Assemble `crates/hickory-executor-canopy/build.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-canopy/build.rs"><storycode:paste select="#f-a6380aac05ba-main"/></storycode:file>

</storycode:doc>
