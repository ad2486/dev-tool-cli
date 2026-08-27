use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

/// Configure development environments consistently and reproducibly
#[derive(Parser, Debug)]
#[command(name = "dev", version, about)]
pub struct Cli {
    /// Show commands that would run, without executing them
    #[arg(long, global = true)]
    dry_run: bool,
    /// Skip interactive confirmation (for scripts)
    #[arg(short, long, global = true)]
    yes: bool,
    /// Use an alternate configuration file
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    /// Decrease log verbosity
    #[arg(short, long, global = true)]
    pub quiet: bool,
    /// Increase log verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install a language and its standard tools
    Install {
        /// Language identifier (e.g. python, rust)
        language: String,
    },
    /// Scaffold a project for a language
    Init {
        /// Language identifier (e.g. python, rust)
        language: String,
    },
    /// Diagnose the environment and standard tools
    Doctor,
    /// Read or write user configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Read a configuration value
    Get {
        /// Key to read; omit to list all
        key: Option<String>,
    },
    /// Write a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Value to store
        value: String,
    },
}
