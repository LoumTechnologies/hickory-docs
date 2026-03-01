use std::env;
use std::path::PathBuf;

fn main() {
    // Emit version stamp.
    println!(
        "cargo:rustc-env=PE2WASM_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );

    // Attempt to build the C++ Remill bridge.
    // If Remill is not installed, we skip the C++ build — the crate will still
    // compile but `lifter::lift()` will fail at link time unless the bridge
    // symbols are provided. This allows `cargo build` to succeed for
    // non-lifting modules (PE parser, shim management, etc.).
    let remill_prefix = env::var("REMILL_PREFIX").unwrap_or_else(|_| "/usr/local".to_string());

    let remill_include = PathBuf::from(&remill_prefix).join("include/remill/Arch/Arch.h");

    if remill_include.exists() {
        // Remill is installed — build the C++ bridge via cmake.
        println!("cargo:warning=Building with Remill from {remill_prefix}");

        let dst = cmake::Config::new("cpp")
            .define("REMILL_PREFIX", &remill_prefix)
            .build();

        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        println!("cargo:rustc-link-lib=static=remill_bridge");

        // Link C++ standard library.
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=c++");
        #[cfg(not(target_os = "macos"))]
        println!("cargo:rustc-link-lib=stdc++");

        // Link LLVM and Remill libraries discovered by cmake.
        // The cmake build handles most of this, but we need to tell
        // cargo about the link search path.
        let llvm_lib_dir = PathBuf::from(&remill_prefix).join("lib");
        if llvm_lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", llvm_lib_dir.display());
        }

        println!("cargo:rustc-cfg=has_remill");
    } else {
        println!(
            "cargo:warning=Remill not found at {remill_prefix}. \
             Building without lifting support. Set REMILL_PREFIX to enable."
        );

        // Provide stub symbols so the crate compiles without Remill.
        // The lifter module will produce link errors only if actually called.
        let out_dir = env::var("OUT_DIR").unwrap();
        let stub_path = PathBuf::from(&out_dir).join("remill_stubs.c");

        std::fs::write(
            &stub_path,
            r#"
#include <stdint.h>
#include <stddef.h>

typedef void RemillContext;
typedef void RemillModule;

RemillContext *remill_context_create(const char *d) { (void)d; return NULL; }
void remill_context_destroy(RemillContext *c) { (void)c; }
void remill_add_code_region(RemillContext *c, const char *n, uint64_t v,
                            const uint8_t *b, uint64_t s) {
    (void)c; (void)n; (void)v; (void)b; (void)s;
}
void remill_add_data_section(RemillContext *c, const char *n, uint64_t v,
                             const uint8_t *b, uint64_t s, int w) {
    (void)c; (void)n; (void)v; (void)b; (void)s; (void)w;
}
void remill_set_entry_point(RemillContext *c, uint64_t v) { (void)c; (void)v; }
RemillModule *remill_lift(RemillContext *c) { (void)c; return NULL; }
void remill_module_destroy(RemillModule *m) { (void)m; }
int remill_write_bc(RemillModule *m, const char *p) { (void)m; (void)p; return 1; }
int remill_write_ll(RemillModule *m, const char *p) { (void)m; (void)p; return 1; }
const char *remill_get_error(RemillContext *c) { (void)c; return "remill not available"; }
"#,
        )
        .expect("failed to write remill stubs");

        cc::Build::new().file(&stub_path).compile("remill_stubs");
    }
}
