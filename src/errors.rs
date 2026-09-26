use crate::config::ConfigError;
use crate::os::OsError;
use crate::project::ProjectError;
use crate::providers::ManifestError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    User,
    Environment,
    Tool,
    Internal,
}

impl Category {
    pub fn exit_code(&self) -> i32 {
        match self {
            Category::User => 2,
            Category::Environment => 3,
            Category::Tool => 4,
            Category::Internal => 70,
        }
    }
}

pub trait ErrorCategory {
    fn category(&self) -> Category;
}

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("Unknown language `{0}`. Supported: {1}")]
    UnknownLanguage(String, String),
}

impl ErrorCategory for CliError {
    fn category(&self) -> Category {
        match self {
            Self::UnknownLanguage(..) => Category::User,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Cli(#[from] CliError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Os(#[from] OsError),
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ErrorCategory for AppError {
    fn category(&self) -> Category {
        match self {
            Self::Cli(error) => error.category(),
            Self::Config(error) => error.category(),
            Self::Os(error) => error.category(),
            Self::Manifest(error) => error.category(),
            Self::Project(error) => error.category(),
            Self::Other(_) => Category::Internal,
        }
    }
}
