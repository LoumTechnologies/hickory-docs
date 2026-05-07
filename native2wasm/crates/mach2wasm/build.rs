fn main() {
    // Version stamp only. C++ bridge and Remill linking are handled by native2wasm-core.
    println!(
        "cargo:rustc-env=MACH2WASM_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );
}
