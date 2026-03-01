use std::env;
use std::path::PathBuf;

fn main() {
    // Emit version stamp.
    println!(
        "cargo:rustc-env=MACH2WASM_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );

    // Attempt to build the C++ Remill bridge.
    // If Remill is not installed, we skip the C++ build — the crate will still
    // compile but `lifter::lift()` will fail at link time unless the bridge
    // symbols are provided. This allows `cargo build` to succeed for
    // non-lifting modules (macho parser, shim management, etc.).
    let remill_prefix = env::var("REMILL_PREFIX")
        .unwrap_or_else(|_| "/usr/local".to_string());

    let remill_include = PathBuf::from(&remill_prefix).join("include/remill/Arch/Arch.h");

    if remill_include.exists() {
        // Remill is installed — build the C++ bridge via cmake.
        println!("cargo:warning=Building with Remill from {remill_prefix}");

        // Resolve LLVM prefix early — needed both for cmake and for runtime linking.
        // Remill was built against a specific LLVM major version. We must match it
        // to avoid ABI mismatches that cause crashes in the C++ bridge.
        //
        // Priority: LLVM_PREFIX env > Remill's bundled LLVM cmake > versioned Cellar > opt/llvm
        let llvm_prefix = resolve_llvm_prefix(&remill_prefix);
        println!("cargo:warning=Using LLVM from {llvm_prefix}");

        let llvm_cmake_dir = format!("{}/lib/cmake/llvm", llvm_prefix);

        let dst = cmake::Config::new("cpp")
            .define("REMILL_PREFIX", &remill_prefix)
            .define("LLVM_DIR", &llvm_cmake_dir)
            .build();

        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        println!("cargo:rustc-link-lib=static=remill_bridge");

        // Link C++ standard library.
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=c++");
        #[cfg(not(target_os = "macos"))]
        println!("cargo:rustc-link-lib=stdc++");

        // Link Remill static libraries.
        let remill_lib_dir = PathBuf::from(&remill_prefix).join("lib");
        if remill_lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", remill_lib_dir.display());
        }
        for lib in &[
            "remill_bc", "remill_os", "remill_arch", "remill_arch_x86",
            "remill_arch_aarch64", "remill_arch_sleigh",
            "remill_arch_sparc32", "remill_arch_sparc64",
            "remill_version",
        ] {
            println!("cargo:rustc-link-lib=static={lib}");
        }

        // Link sleigh/decomp libraries bundled with Remill.
        for lib in &["sla", "slaSupport", "decomp"] {
            println!("cargo:rustc-link-lib=static={lib}");
        }

        // Link Intel XED (x86 Encoder Decoder) — required by remill_arch_x86.
        // XED_PREFIX env overrides; otherwise look in REMILL_PREFIX, then /usr/local.
        let xed_prefix = env::var("XED_PREFIX").ok();
        let xed_search_dirs: Vec<PathBuf> = match &xed_prefix {
            Some(p) => vec![PathBuf::from(p).join("lib")],
            None => vec![
                PathBuf::from(&remill_prefix).join("lib"),
                PathBuf::from("/usr/local/lib"),
            ],
        };
        let mut xed_found = false;
        for dir in &xed_search_dirs {
            if dir.join("libxed.a").exists() {
                println!("cargo:warning=Found XED in {}", dir.display());
                println!("cargo:rustc-link-search=native={}", dir.display());
                xed_found = true;
                break;
            }
        }
        if !xed_found {
            println!(
                "cargo:warning=libxed.a not found! Searched: {:?}. \
                 Run scripts/install-xed.sh or set XED_PREFIX.",
                xed_search_dirs,
            );
        }
        println!("cargo:rustc-link-lib=static=xed");
        println!("cargo:rustc-link-lib=static=xed-ild");

        // Link LLVM shared library.
        // On macOS, libLLVM.dylib's install name may point to a symlink
        // (e.g., /usr/local/opt/llvm/lib) that could change to a newer version.
        // We add an rpath to the specific LLVM prefix we're linking against so
        // dyld finds the correct version at runtime.
        let llvm_lib_dir = PathBuf::from(&llvm_prefix).join("lib");
        if llvm_lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", llvm_lib_dir.display());
            // Add rpath so the correct LLVM is found at runtime even if
            // /usr/local/opt/llvm symlink changes to a different version.
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", llvm_lib_dir.display());
        }
        println!("cargo:rustc-link-lib=dylib=LLVM");

        // Link glog and gflags (Remill dependencies).
        // Search in REMILL_PREFIX/lib, /usr/local/lib, and /opt/homebrew/lib.
        println!("cargo:rustc-link-search=native=/usr/local/lib");
        if PathBuf::from("/opt/homebrew/lib").exists() {
            println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
        }
        println!("cargo:rustc-link-lib=dylib=glog");
        println!("cargo:rustc-link-lib=dylib=gflags");

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

        cc::Build::new()
            .file(&stub_path)
            .compile("remill_stubs");
    }
}

/// Find the LLVM installation that matches the version Remill was built against.
///
/// Remill's static libraries contain LLVM symbols compiled against a specific
/// LLVM major version. If we link against a different version, we get ABI
/// mismatches that manifest as crashes in the C++ bridge. This function tries
/// to find the correct version by:
///   1. Checking the LLVM_PREFIX environment variable (explicit override)
///   2. Looking for LLVM cmake config in the Remill prefix
///   3. Querying the Remill-bundled LLVM version and finding it in Homebrew's Cellar
///   4. Falling back to the default brew llvm
fn resolve_llvm_prefix(remill_prefix: &str) -> String {
    // 1. Explicit override.
    if let Ok(prefix) = env::var("LLVM_PREFIX") {
        return prefix;
    }

    // 2. Check if Remill bundles its own LLVM cmake config.
    let remill_llvm_cmake = PathBuf::from(remill_prefix).join("lib/cmake/llvm");
    if remill_llvm_cmake.exists() {
        return remill_prefix.to_string();
    }

    // 3. Try to determine what LLVM version Remill was built against by looking
    //    at the Remill cmake config or by checking version-specific Cellar paths.
    //    Remill 21.x was built against LLVM 21; check if that exact version exists.
    let cellar_base = PathBuf::from("/usr/local/Cellar/llvm");
    if cellar_base.exists() {
        // Scan for installed LLVM versions and prefer the one matching Remill's
        // expected version (21.x). We can detect this by checking if the Remill
        // libs link against symbols from a specific LLVM version.
        if let Ok(entries) = std::fs::read_dir(&cellar_base) {
            let mut versions: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().join("lib/cmake/llvm").exists())
                .collect();
            // Sort by version string descending.
            versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

            // Prefer LLVM 21.x (the version Remill was built against).
            // If multiple 21.x exist, take the latest.
            for entry in &versions {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("21.") {
                    let path = entry.path().to_string_lossy().into_owned();
                    println!("cargo:warning=Found LLVM 21 at {path} (matching Remill)");
                    return path;
                }
            }

            // No 21.x found; fall through to default.
        }
    }

    // Also check /opt/homebrew for Apple Silicon.
    let brew_llvm21 = PathBuf::from("/opt/homebrew/opt/llvm@21");
    if brew_llvm21.exists() {
        return brew_llvm21.to_string_lossy().into_owned();
    }
    let brew_llvm = PathBuf::from("/opt/homebrew/opt/llvm");
    if brew_llvm.exists() {
        return brew_llvm.to_string_lossy().into_owned();
    }

    // 4. Default fallback.
    "/usr/local/opt/llvm".to_string()
}
