use crate::os::{Command, CommandRunner, OsAdapter, OsError};
use std::rc::Rc;
pub struct BrewAdapter {
    runner: Rc<dyn CommandRunner>,
}

impl BrewAdapter {
    pub fn new(runner: Rc<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

impl OsAdapter for BrewAdapter {
    fn name(&self) -> &str {
        "brew"
    }
    fn install_package(&self, package: &str) -> Result<(), OsError> {
        let command = Command {
            program: "brew".to_string(),
            args: vec!["install".to_string(), package.to_string()],
        };
        self.runner.execute(&command)
    }

    fn command_exists(&self, program: &str) -> Result<bool, OsError> {
        let command = Command {
            program: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                r#"command -v "$1""#.to_string(),
                "sh".to_string(),
                program.to_string(),
            ],
        };
        Ok(self.runner.capture(&command)?.code == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os::{Output, RealRunner, RecordingRunner};

    #[test]
    fn installs_package() {
        let command = Command {
            program: "brew".to_string(),
            args: vec!["install".to_string(), "python".to_string()],
        };

        let runner = Rc::new(RecordingRunner::new());
        let adapter = BrewAdapter::new(runner.clone());
        adapter.install_package("python").unwrap();
        let commands = runner.commands();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], command);
    }

    #[test]
    fn command_exists_is_true_when_lookup_succeeds() {
        let output = Output {
            code: 0,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let runner = Rc::new(RecordingRunner::new());
        let adapter = BrewAdapter::new(runner.clone());
        runner.push_response(output);
        let result = adapter.command_exists("python").unwrap();
        assert!(result);
    }

    #[test]
    fn command_exists_is_false_when_lookup_fails() {
        let output = Output {
            code: 1,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let runner = Rc::new(RecordingRunner::new());
        let adapter = BrewAdapter::new(runner.clone());
        runner.push_response(output);
        let result = adapter.command_exists("python").unwrap();
        assert!(!result);
    }

    #[test]
    fn command_exists_builds_posix_lookup() {
        let command = Command {
            program: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                r#"command -v "$1""#.to_string(),
                "sh".to_string(),
                "python".to_string(),
            ],
        };
        let runner = Rc::new(RecordingRunner::new());
        let adapter = BrewAdapter::new(runner.clone());
        adapter.command_exists("python").unwrap();
        let commands = runner.commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], command);
    }

    #[test]
    #[cfg(unix)]
    fn posix_lookup_finds_an_installed_program() {
        let adapter = BrewAdapter::new(Rc::new(RealRunner)); // building adapter with a real runner
        let result = adapter.command_exists("sh").unwrap();
        assert!(result);
    }

    #[test]
    #[cfg(unix)]
    fn posix_lookup_rejects_a_missing_program() {
        let adapter = BrewAdapter::new(Rc::new(RealRunner));
        let result = adapter.command_exists("holy-moly").unwrap(); // purposeful misspelling/nonexistent program
        assert!(!result);
    }
}
