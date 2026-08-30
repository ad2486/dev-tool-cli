mod cli;
mod config;
mod core;
mod errors;
mod providers;

use crate::{
    cli::Cli,
    errors::{Category, ErrorCategory},
};
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;

fn main() {
    let cli: Cli = Cli::parse();
    let level: LevelFilter = level_handling(cli.verbose, cli.quiet);
    Builder::new().filter_level(level).init();

    match run(cli) {
        Ok(()) => {}
        Err(e) => {
            let categoria = match e.downcast_ref::<crate::errors::ExempleError>() {
                Some(erro_especifico) => erro_especifico.category(),
                None => crate::errors::Category::Internal,
            };
            std::process::exit(categoria.exit_code())
        }
    };
}

fn level_handling(verbose: u8, quiet: bool) -> LevelFilter {
    match (verbose, quiet) {
        (_, true) => log::LevelFilter::Off,
        (0, false) => log::LevelFilter::Info,
        (1, false) => log::LevelFilter::Debug,
        (2, false) => log::LevelFilter::Trace,
        (_, false) => log::LevelFilter::Trace,
    }
}

fn test_operation() -> Result<(), crate::errors::ExempleError> {
    Err(crate::errors::ExempleError::AbsentDependency)
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let path = match cli.config {
        Some(p) => p,
        None => config::default_path()?,
    };
    let table = config::load(&path)?;
    log::info!("{:?}", table);
    test_operation()?;
    Ok(())
}
