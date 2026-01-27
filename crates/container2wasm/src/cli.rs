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

    /// Image name (or output path when using --external-bundle or --pack).
    pub image_name: Option<String>,

    /// Output file path.
    pub output: Option<String>,
}
