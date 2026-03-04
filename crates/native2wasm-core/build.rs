use std::env;
use std::path::PathBuf;

fn main() {
    // Attempt to build the C++ Remill bridge.
    // If Remill is not installed, we skip the C++ build — the crate will still
    // compile but `lifter::lift()` will fail at link time unless the bridge
    // symbols are provided. This allows `cargo build` to succeed for
    // non-lifting modules (pipeline config, shim management, etc.).
    let remill_prefix =
        env::var("REMILL_PREFIX").unwrap_or_else(|_| "/usr/local".to_string());

    let remill_include = PathBuf::from(&remill_prefix).join("include/remill/Arch/Arch.h");

    if remill_include.exists() {
        // Remill is installed — build the C++ bridge via cmake.
        println!("cargo:warning=Building with Remill from {remill_prefix}");

        let remill_deps_prefix = env::var("REMILL_DEPS_PREFIX").unwrap_or_default();

        let mut cfg = cmake::Config::new("cpp");
        cfg.define("REMILL_PREFIX", &remill_prefix);
        if !remill_deps_prefix.is_empty() {
            cfg.define("REMILL_DEPS_PREFIX", &remill_deps_prefix);
        }
        let dst = cfg.build();

        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        println!("cargo:rustc-link-lib=static=remill_bridge");

        // Link C++ standard library.
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=c++");
        #[cfg(not(target_os = "macos"))]
        println!("cargo:rustc-link-lib=stdc++");

        // Link Remill component static libraries.
        let remill_lib_dir = PathBuf::from(&remill_prefix).join("lib");
        if remill_lib_dir.exists() {
            println!(
                "cargo:rustc-link-search=native={}",
                remill_lib_dir.display()
            );
        }
        for lib in &[
            "remill_arch_x86",
            "remill_arch",
            "remill_arch_sleigh",
            "remill_arch_aarch64",
            "remill_arch_sparc32",
            "remill_arch_sparc64",
            "remill_bc",
            "remill_os",
            "remill_version",
            "sla",
            "slaSupport",
            "decomp",
        ] {
            println!("cargo:rustc-link-lib=static={lib}");
        }

        // Link XED and glog/gflags from the deps prefix.
        if !remill_deps_prefix.is_empty() {
            let deps_lib = PathBuf::from(&remill_deps_prefix).join("lib");
            if deps_lib.exists() {
                println!("cargo:rustc-link-search=native={}", deps_lib.display());
            }
        }
        for lib in &["xed", "xed-ild", "glog", "gflags"] {
            println!("cargo:rustc-link-lib=static={lib}");
        }

        // Link LLVM shared library.
        // Use rustc-link-arg to ensure -L comes before -l in the link line.
        let llvm_prefix = env::var("LLVM_PREFIX")
            .unwrap_or_else(|_| "/usr/local/Cellar/llvm/21.1.8".to_string());
        let llvm_lib_dir = PathBuf::from(&llvm_prefix).join("lib");
        if llvm_lib_dir.exists() {
            println!("cargo:rustc-link-arg=-L{}", llvm_lib_dir.display());
            println!(
                "cargo:rustc-link-arg=-Wl,-rpath,{}",
                llvm_lib_dir.display()
            );
        }
        println!("cargo:rustc-link-arg=-lLLVM");

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

RemillContext *remill_context_create(const char *d, const char *os, const char *arch, const char *sym) {
    (void)d; (void)os; (void)arch; (void)sym; return NULL;
}
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
