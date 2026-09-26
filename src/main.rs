mod cli;
mod config;
mod core;
mod errors;
mod os;
mod project;
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
use std::path::PathBuf;
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

    let mut registry = Registry::new();
    registry.register(Box::new(PythonProvider::new()?));

    let real_runner = RealRunner::with_path(tool_bin_dirs(&registry));
    let command_runner: Rc<dyn CommandRunner> = if cli.dry_run {
        Rc::new(DryRunRunner::new(real_runner))
    } else {
        Rc::new(real_runner)
    };
    let os_adapter = os::detect_adapter(command_runner.clone(), cli.dry_run)?;

    match cli.command {
        Commands::Doctor => cli::doctor::run(&registry, &config, os_adapter, command_runner),
        Commands::Install { ref language } => {
            cli::install::run(&registry, &config, os_adapter, command_runner, language)
        }
        Commands::Init { ref language } => cli::init::run(
            &registry,
            &config,
            os_adapter,
            command_runner,
            language,
            cli.dry_run,
        ),
        Commands::Config { .. } => {
            Err(anyhow::anyhow!("`dev config` is not implemented yet").into())
        }
    }
}

fn tool_bin_dirs(registry: &Registry) -> Vec<PathBuf> {
    let Some(home) = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()) else {
        return Vec::new();
    };
    registry
        .all()
        .iter()
        .flat_map(|provider| provider.manifest().bin_dirs.iter())
        .map(|dir| os::expand_home(dir, &home))
        .collect()
}
