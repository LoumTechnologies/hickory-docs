use c2w::cli::Cli;
use clap::Parser;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    if let Err(e) = c2w::run(cli) {
        eprintln!("{:+}", e);
        std::process::exit(1);
    }
}
