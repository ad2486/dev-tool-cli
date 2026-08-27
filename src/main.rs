mod cli;
use crate::cli::Cli;
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;

fn main() {
    let cli: Cli = Cli::parse();
    let level: LevelFilter = level_handling(cli.verbose, cli.quiet);
    Builder::new().filter_level(level).init();
    log::error!("{:?}", cli)
}

fn level_handling(verbose: u8, quiet: bool) -> LevelFilter {
    match (verbose, quiet) {
        (_, true) => log::LevelFilter::Off,
        (0, false) => log::LevelFilter::Error,
        (1, false) => log::LevelFilter::Warn,
        (2, false) => log::LevelFilter::Info,
        (3, false) => log::LevelFilter::Debug,
        (_, false) => log::LevelFilter::Debug,
    }
}
