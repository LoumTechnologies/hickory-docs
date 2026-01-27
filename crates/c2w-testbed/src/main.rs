use clap::Parser;
use container2wasm::cli::Cli;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    if let Err(e) = container2wasm::run(cli) {
        eprintln!("{:+}", e);
        std::process::exit(1);
    }
}
