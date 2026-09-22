use crate::os::{Command, CommandRunner, OsAdapter, OsError};
use std::rc::Rc;

pub struct AptAdapter {
    runner: Rc<dyn CommandRunner>,
    dry_run: bool,
}

impl AptAdapter {
    pub fn new(runner: Rc<dyn CommandRunner>, dry_run: bool) -> Self {
        Self { runner, dry_run }
    }
}

impl OsAdapter for AptAdapter {
    fn name(&self) -> &str {
        "apt"
    }

    fn install_package(&self, package: &str) -> Result<(), OsError> {
        if self.dry_run {
            let command = Command {
                program: "apt-get".to_string(),
                args: vec!["install".to_string(), "-s".to_string(), package.to_string()],
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
                program: "sudo".to_string(),
                args: vec![
                    "apt-get".to_string(),
                    "install".to_string(),
                    "-y".to_string(),
                    package.to_string(),
                ],
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
    use crate::os::{Output, RealRunner, RecordingRunner};

    #[test]
    fn installs_package() {
        let command = Command {
            program: "sudo".to_string(),
            args: vec![
                "apt-get".to_string(),
                "install".to_string(),
                "-y".to_string(),
                "python3".to_string(),
            ],
        };
        let runner = Rc::new(RecordingRunner::new());
        let adapter = AptAdapter::new(runner.clone(), false);
        adapter.install_package("python3").unwrap();
        let commands = runner.commands();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], command);
    }

    #[test]
    fn install_package_simulates_in_dry_run() {
        let command = Command {
            program: "apt-get".to_string(),
            args: vec![
                "install".to_string(),
                "-s".to_string(),
                "python3".to_string(),
            ],
        };
        let runner = Rc::new(RecordingRunner::new());
        let adapter = AptAdapter::new(runner.clone(), true);
        adapter.install_package("python3").unwrap();
        let commands = runner.commands();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], command);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn real_apt_accepts_the_simulated_install() {
        let adapter = AptAdapter::new(Rc::new(RealRunner), true);
        assert!(adapter.install_package("python3").is_ok());
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
        let adapter = AptAdapter::new(runner.clone(), true);
        let result = adapter.install_package("python3");

        assert!(result.is_err());
    }
}
