mod cli;
mod config;
mod core;
mod errors;
mod os;
mod providers;

use crate::{
    cli::{Cli, Commands},
    core::Registry,
    errors::{AppError, ErrorCategory},
    os::{CommandRunner, DryRunRunner, RealRunner},
    providers::python::PythonProvider,
};
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use std::rc::Rc;

fn main() {
    let cli: Cli = Cli::parse();
    let level: LevelFilter = level_handling(cli.verbose, cli.quiet);
    Builder::new().filter_level(level).init();

    if let Err(error) = run(cli) {
        eprintln!("error: {error}");
        std::process::exit(error.category().exit_code());
    }
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

fn run(cli: Cli) -> Result<(), AppError> {
    let path = match cli.config {
        Some(ref path) => path.clone(),
        None => config::default_path()?,
    };
    let config = config::load(&path)?;

    let command_runner: Rc<dyn CommandRunner> = if cli.dry_run {
        Rc::new(DryRunRunner)
    } else {
        Rc::new(RealRunner)
    };
    let os_adapter = os::detect_adapter(command_runner.clone(), cli.dry_run)?;

    let mut registry = Registry::new();
    registry.register(Box::new(PythonProvider::new()?));

    match cli.command {
        Commands::Doctor => cli::doctor::run(&registry, &config, os_adapter, command_runner),
        Commands::Install { .. } => {
            Err(anyhow::anyhow!("`dev install` is not implemented yet").into())
        }
        Commands::Init { .. } => Err(anyhow::anyhow!("`dev init` is not implemented yet").into()),
        Commands::Config { .. } => {
            Err(anyhow::anyhow!("`dev config` is not implemented yet").into())
        }
    }
}
