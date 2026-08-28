mod cli;
mod errors;

use crate::{cli::Cli, errors::{Category, ErrorCategory}};
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;

fn main() {
    let cli: Cli = Cli::parse();
    let level: LevelFilter = level_handling(cli.verbose, cli.quiet);
    Builder::new().filter_level(level).init();
    match run() {
        Ok(()) => {}
        Err(e) => {
            let categoria = match e.downcast_ref::<crate::errors::ExempleError>() {
                Some(erro_especifico) => erro_especifico.category(),
                None => crate::errors::Category::Internal
            };
            std::process::exit(categoria.exit_code())
        }
    };
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

fn test_operation() -> Result<(), crate::errors::ExempleError> {
    Err(crate::errors::ExempleError::AbsentDependency)
}

fn run() -> anyhow::Result<()> {
    test_operation()?;
    Ok(())
}