fn main() {
    println!(
        "cargo:rustc-env=PE2WASM_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );
}
