use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = mach2wasm::cli::Cli::parse();

    env_logger::Builder::new()
        .filter_level(if cli.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    mach2wasm::run(cli)
}
