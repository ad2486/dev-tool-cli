use crate::config::ConfigError;
use crate::os::OsError;
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
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Os(#[from] OsError),
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ErrorCategory for AppError {
    fn category(&self) -> Category {
        match self {
            Self::Config(error) => error.category(),
            Self::Os(error) => error.category(),
            Self::Manifest(error) => error.category(),
            Self::Other(_) => Category::Internal,
        }
    }
}
