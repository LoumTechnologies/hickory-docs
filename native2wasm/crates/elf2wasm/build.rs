fn main() {
    println!(
        "cargo:rustc-env=ELF2WASM_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );
}
