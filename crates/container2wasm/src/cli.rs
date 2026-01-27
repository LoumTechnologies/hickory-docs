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

    /// Image name (or output path when using --external-bundle or --pack).
    pub image_name: Option<String>,

    /// Output file path.
    pub output: Option<String>,
}
