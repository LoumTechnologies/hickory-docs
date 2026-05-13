use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "c2w",
    about = "container to wasm converter",
    version = env!("C2W_VERSION"),
    override_usage = "c2w [OPTIONS] <IMAGE_NAME> [OUTPUT]",
)]
pub struct Cli {
    /// Custom location of build assets.
    #[arg(long)]
    pub assets: Option<String>,

    /// Custom location of Dockerfile (default: embedded to this command).
    #[arg(long)]
    pub dockerfile: Option<String>,

    /// Builder command to use.
    #[arg(long, default_value = "docker")]
    pub builder: String,

    /// Target architecture of the source image to use.
    #[arg(long, default_value = "amd64")]
    pub target_arch: String,

    /// Additional build arguments.
    #[arg(long)]
    pub build_arg: Vec<String>,

    /// Output JS files runnable on the browsers using emscripten.
    #[arg(long)]
    pub to_js: bool,

    /// Enable debug print in the output image.
    #[arg(long)]
    pub debug_image: bool,

    /// Show default Dockerfile.
    #[arg(long)]
    pub show_dockerfile: bool,

    /// Use "docker build" instead of buildx (no support for assets flag).
    #[arg(long)]
    pub legacy: bool,

    /// Do not embed container image to the Wasm image but mount it during runtime.
    #[arg(long)]
    pub external_bundle: bool,

    /// Extra flag for builders.
    #[arg(long)]
    pub extra_flag: Vec<String>,

    /// Target stage of the build.
    #[arg(long)]
    pub target_stage: Option<String>,

    /// Overwrite directory to pack with the emulator (valid only for aarch64 QEMU on emscripten).
    #[arg(long)]
    pub pack: Option<String>,

    /// Enable Wizer pre-initialization for instant startup.
    ///
    /// After the Docker build produces a .wasm file, runs Wizer to snapshot
    /// the boot process (CPU emulator init, kernel boot, etc.) so the module
    /// starts near-instantly at a responsive shell. Passes SKIP_WIZER=true to
    /// the Docker build so the Dockerfile can skip its own Wizer step.
    #[arg(long)]
    pub wizer: bool,

    /// Run only Wizer pre-initialization on an existing .wasm file, skipping
    /// the Docker build entirely. The image_name argument is treated as the
    /// input .wasm file path.
    #[arg(long)]
    pub wizer_only: bool,

    /// Directory mapping for Wizer WASI filesystem (format: guest_dir::host_dir).
    ///
    /// Makes host directories available to the WASI environment during the
    /// pre-initialization phase. For container2wasm, you typically need to map
    /// the /pack directory containing the kernel and container filesystem.
    /// Can be specified multiple times.
    #[arg(long)]
    pub wizer_map_dir: Vec<String>,

    /// Name of the Wizer initialization function export.
    #[arg(long, default_value = "wizer.initialize")]
    pub wizer_init_func: String,

    // -- No-Docker pipeline flags --
    /// Skip Docker and use the pure Rust OCI pipeline instead.
    ///
    /// Pulls the container image directly from the registry via HTTP,
    /// then assembles the WASM module using local tools (wasi-vfs, wizer).
    /// Requires a pre-built template directory with emulator + kernel artifacts.
    #[arg(long)]
    pub no_docker: bool,

    /// Path to a pre-built template directory containing emulator WASM,
    /// kernel, BIOS, and guest binaries. Required for --no-docker unless
    /// the template is already cached.
    #[arg(long)]
    pub template: Option<PathBuf>,

    /// Template version to use (default: matches upstream container2wasm release).
    #[arg(long)]
    pub template_version: Option<String>,

    /// Override the template cache directory (default: ~/.cache/c2w/templates/).
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,

    /// Shell command to run inside the extracted container rootfs before building
    /// the WASM image (--no-docker mode only). Commands run in order inside a
    /// chroot using a user namespace (requires Linux with unprivileged user
    /// namespaces enabled). Can be specified multiple times.
    ///
    /// Example: `--setup-cmd "apk add --no-cache nodejs npm" --setup-cmd "npm install -g tsx"`
    #[arg(long)]
    pub setup_cmd: Vec<String>,

    /// Registry username for OCI image pull (--no-docker mode).
    #[arg(long)]
    pub registry_username: Option<String>,

    /// Registry password for OCI image pull (--no-docker mode).
    #[arg(long)]
    pub registry_password: Option<String>,

    /// Image name (or output path when using --external-bundle or --pack).
    pub image_name: Option<String>,

    /// Output file path.
    pub output: Option<String>,
}
