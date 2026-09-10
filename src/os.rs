use crate::errors::{Category, ErrorCategory};
use std::cell::RefCell;
use std::collections::VecDeque;
pub mod brew;
pub trait OsAdapter {
    fn name(&self) -> &str;
    fn install_package(&self, package: &str) -> Result<(), OsError>;
    fn command_exists(&self, program: &str) -> Result<bool, OsError>;
}
pub trait CommandRunner {
    fn execute(&self, cmd: &Command) -> Result<(), OsError>;
    fn capture(&self, cmd: &Command) -> Result<Output, OsError>;
}

#[derive(thiserror::Error, Debug)]
pub enum OsError {
    #[error("Command `{command}` failed with exit code {code}")]
    CommandFailed { command: String, code: i32 },
    #[error("Couldn't execute command `{program}`: {source}")]
    ExecutionFailed {
        program: String,
        source: std::io::Error,
    },
    #[error("Command `{command}` was killed by a signal")]
    CommandKilled { command: String },
}

impl ErrorCategory for OsError {
    fn category(&self) -> Category {
        match self {
            Self::CommandFailed { .. } => Category::Tool,
            Self::ExecutionFailed { .. } => Category::Environment,
            Self::CommandKilled { .. } => Category::Tool,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct RealRunner;

pub struct DryRunRunner;

pub struct RecordingRunner {
    commands: RefCell<Vec<Command>>,
    responses: RefCell<VecDeque<Output>>,
}

impl RecordingRunner {
    pub fn new() -> Self {
        Self {
            commands: RefCell::new(Vec::new()),
            responses: RefCell::new(VecDeque::new()),
        }
    }

    pub fn commands(&self) -> Vec<Command> {
        self.commands.borrow().clone()
    }

    pub fn push_response(&self, response: Output) {
        self.responses.borrow_mut().push_back(response);
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(f, "{}", self.program)
        } else {
            write!(f, "{} {}", self.program, self.args.join(" "))
        }
    }
}

impl CommandRunner for RealRunner {
    fn execute(&self, cmd: &Command) -> Result<(), OsError> {
        let status = std::process::Command::new(&cmd.program)
            .args(&cmd.args)
            .status()
            .map_err(|e| OsError::ExecutionFailed {
                program: cmd.program.clone(),
                source: e,
            })?;

        match status.code() {
            Some(0) => Ok(()),
            Some(code) => Err(OsError::CommandFailed {
                command: cmd.to_string(),
                code,
            }),
            None => Err(OsError::CommandKilled {
                command: cmd.to_string(),
            }),
        }
    }

    fn capture(&self, cmd: &Command) -> Result<Output, OsError> {
        let output = std::process::Command::new(&cmd.program)
            .args(&cmd.args)
            .output()
            .map_err(|e| OsError::ExecutionFailed {
                program: cmd.program.clone(),
                source: e,
            })?;

        match output.status.code() {
            Some(code) => Ok(Output {
                code,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            }),
            None => Err(OsError::CommandKilled {
                command: cmd.to_string(),
            }),
        }
    }
}

impl CommandRunner for DryRunRunner {
    fn execute(&self, cmd: &Command) -> Result<(), OsError> {
        log::info!("[dry-run] would run: {}", cmd);
        Ok(())
    }

    fn capture(&self, cmd: &Command) -> Result<Output, OsError> {
        RealRunner.capture(cmd)
    }
}

impl CommandRunner for RecordingRunner {
    fn execute(&self, cmd: &Command) -> Result<(), OsError> {
        self.commands.borrow_mut().push(cmd.clone());
        Ok(())
    }

    fn capture(&self, cmd: &Command) -> Result<Output, OsError> {
        self.commands.borrow_mut().push(cmd.clone());
        match self.responses.borrow_mut().pop_front() {
            Some(response) => Ok(response),
            None => Ok(Output {
                code: 0,
                stdout: String::new(),
                stderr: String::new(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_commands_in_order() {
        let runner = RecordingRunner::new();
        let command_one = Command {
            program: "brew".to_string(),
            args: vec!["install".to_string(), "python".to_string()],
        };
        let command_two = Command {
            program: "apt-get".to_string(),
            args: vec!["install".to_string(), "python3".to_string()],
        };

        runner.execute(&command_one).unwrap();
        runner.execute(&command_two).unwrap();

        let commands = runner.commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], command_one);
        assert_eq!(commands[1], command_two);
    }

    #[test]
    fn dry_run_runner_does_not_spawn_commands() {
        let command = Command {
            program: "holy-moly".to_string(), // purposeful misspelling/nonexistent program
            args: vec![],                     // same here, we're not passing any arguments
        };

        let result = DryRunRunner.execute(&command);

        assert!(result.is_ok()); // should be ok because we're not executing the command
    }

    #[test]
    fn command_displays_program_and_args() {
        let command = Command {
            program: "brew".to_string(),
            args: vec!["install".to_string(), "python".to_string()],
        };

        assert_eq!(command.to_string().as_str(), "brew install python");
    }

    #[test]
    fn command_without_args_displays_without_trailing_space() {
        let command = Command {
            program: "brew".to_string(),
            args: vec![],
        };

        assert_eq!(command.to_string().as_str(), "brew");
    }
}
