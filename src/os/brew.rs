use crate::os::{Command, CommandRunner, OsAdapter, OsError};
use std::rc::Rc;
pub struct BrewAdapter {
    runner: Rc<dyn CommandRunner>,
    dry_run: bool,
}

impl BrewAdapter {
    pub fn new(runner: Rc<dyn CommandRunner>, dry_run: bool) -> Self {
        Self { runner, dry_run }
    }
}

impl OsAdapter for BrewAdapter {
    fn name(&self) -> &str {
        "brew"
    }
    fn install_package(&self, package: &str) -> Result<(), OsError> {
        if self.dry_run {
            let command = Command {
                program: "brew".to_string(),
                args: vec!["install".to_string(), "-n".to_string(), package.to_string()],
            };
            let output = self.runner.capture(&command)?;
            log::info!("[dry-run] output: {}", output.stdout.trim_end());
            if output.code == 0 {
                Ok(())
            } else {
                Err(OsError::CommandFailed {
                    command: command.to_string(),
                    code: output.code,
                })
            }
        } else {
            let command = Command {
                program: "brew".to_string(),
                args: vec!["install".to_string(), package.to_string()],
            };
            self.runner.execute(&command)
        }
    }

    fn command_exists(&self, program: &str) -> Result<bool, OsError> {
        super::command_exists(&*self.runner, program)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os::{Output, RecordingRunner};

    #[test]
    fn installs_package() {
        let command = Command {
            program: "brew".to_string(),
            args: vec!["install".to_string(), "python".to_string()],
        };

        let runner = Rc::new(RecordingRunner::new());
        let adapter = BrewAdapter::new(runner.clone(), false);
        adapter.install_package("python").unwrap();
        let commands = runner.commands();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], command);
    }

    #[test]
    fn install_package_simulates_in_dry_run() {
        let command = Command {
            program: "brew".to_string(),
            args: vec![
                "install".to_string(),
                "-n".to_string(),
                "python".to_string(),
            ],
        };
        let runner = Rc::new(RecordingRunner::new());
        let adapter = BrewAdapter::new(runner.clone(), true);
        adapter.install_package("python").unwrap();
        let commands = runner.commands();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], command);
    }

    #[test]
    fn failed_simulation_becomes_an_error() {
        let output = Output {
            code: 1,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(output);
        let adapter = BrewAdapter::new(runner.clone(), true);
        let result = adapter.install_package("python");

        assert!(result.is_err());
    }
}
