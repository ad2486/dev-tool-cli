use crate::errors::ExempleError::{AbsentDependency, Bug, CommandFailed, Invalid};

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
pub enum ExempleError {
    #[error("Something invalid {0}")]
    Invalid(String),
    #[error("Absent dependency")]
    AbsentDependency,
    #[error("External command failed")]
    CommandFailed,
    #[error("Impossible state")]
    Bug,
}

impl ErrorCategory for ExempleError {
    fn category(&self) -> Category {
        match self {
            Invalid(_) => Category::User,
            AbsentDependency => Category::Environment,
            CommandFailed => Category::Tool,
            Bug => Category::Internal
        }
    }
}