use crate::errors::{Category, ErrorCategory};
use apt::AptAdapter;
use brew::BrewAdapter;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub mod apt;
pub mod brew;

const SUPPORTED_PACKAGE_MANAGERS: &str = "apt-get, brew";
pub trait OsAdapter {
    fn name(&self) -> &str;
    fn install_package(&self, package: &str) -> Result<(), OsError>;
    fn command_exists(&self, program: &str) -> Result<bool, OsError>;
}
pub trait CommandRunner {
    fn execute(&self, cmd: &Command) -> Result<(), OsError>;
    fn capture(&self, cmd: &Command) -> Result<Output, OsError>;
}

pub fn command_exists(runner: &dyn CommandRunner, program: &str) -> Result<bool, OsError> {
    let command = Command {
        program: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            r#"command -v "$1""#.to_string(),
            "sh".to_string(),
            program.to_string(),
        ],
    };
    Ok(runner.capture(&command)?.code == 0)
}

pub fn detect_adapter(
    runner: Rc<dyn CommandRunner>,
    dry_run: bool,
) -> Result<Rc<dyn OsAdapter>, OsError> {
    if command_exists(&*runner, "apt-get")? {
        return Ok(Rc::new(AptAdapter::new(runner, dry_run)));
    }
    if command_exists(&*runner, "brew")? {
        return Ok(Rc::new(BrewAdapter::new(runner, dry_run)));
    }
    Err(OsError::NoPackageManager)
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
    #[error("No supported package manager found (looked for{SUPPORTED_PACKAGE_MANAGERS})")]
    NoPackageManager,
}

impl ErrorCategory for OsError {
    fn category(&self) -> Category {
        match self {
            Self::CommandFailed { .. } => Category::Tool,
            Self::ExecutionFailed { .. } => Category::Environment,
            Self::CommandKilled { .. } => Category::Tool,
            Self::NoPackageManager => Category::Environment,
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

#[derive(Debug, Clone, Default)]
pub struct RealRunner {
    extra_path: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct DryRunRunner {
    real: RealRunner,
}

impl RealRunner {
    pub fn with_path(extra_path: Vec<PathBuf>) -> Self {
        Self { extra_path }
    }

    fn command(&self, cmd: &Command) -> std::process::Command {
        let mut command = std::process::Command::new(&cmd.program);
        command.args(&cmd.args);
        if !self.extra_path.is_empty() {
            command.env(
                "PATH",
                augmented_path(std::env::var_os("PATH"), &self.extra_path),
            );
        }
        command
    }
}

impl DryRunRunner {
    pub fn new(real: RealRunner) -> Self {
        Self { real }
    }
}

pub fn augmented_path(current: Option<OsString>, extra: &[PathBuf]) -> OsString {
    let mut dirs: Vec<PathBuf> = current
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default();
    for dir in extra {
        if !dirs.contains(dir) {
            dirs.push(dir.clone());
        }
    }
    std::env::join_paths(dirs).unwrap_or_default()
}

pub fn expand_home(path: &str, home: &Path) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None => PathBuf::from(path),
    }
}

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
        let status = self
            .command(cmd)
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
        let output = self
            .command(cmd)
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
        self.real.capture(cmd)
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

        let result = DryRunRunner::default().execute(&command);

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

    #[test]
    fn command_exists_is_true_when_lookup_succeeds() {
        let output = Output {
            code: 0,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let runner = RecordingRunner::new();
        runner.push_response(output);
        let result = command_exists(&runner, "python").unwrap();
        assert!(result);
    }

    #[test]
    fn command_exists_is_false_when_lookup_fails() {
        let output = Output {
            code: 1,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let runner = RecordingRunner::new();
        runner.push_response(output);
        let result = command_exists(&runner, "python").unwrap();
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
        let runner = RecordingRunner::new();
        command_exists(&runner, "python").unwrap();
        let commands = runner.commands();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], command);
    }

    #[test]
    #[cfg(unix)]
    fn posix_lookup_finds_an_installed_program() {
        let result = command_exists(&RealRunner::default(), "sh").unwrap();
        assert!(result);
    }

    #[test]
    #[cfg(unix)]
    fn posix_lookup_rejects_a_missing_program() {
        let result = command_exists(&RealRunner::default(), "holy-moly").unwrap(); // purposeful misspelling/nonexistent program
        assert!(!result);
    }

    #[test]
    fn detect_adapter_picks_apt_when_available() {
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(Output {
            code: 0,
            stdout: "".to_string(),
            stderr: "".to_string(),
        });
        let adapter = detect_adapter(runner.clone(), false).unwrap();
        let commands = runner.commands();
        assert_eq!(commands[0].args.last().unwrap(), "apt-get");
        assert_eq!(adapter.name(), "apt");
    }

    #[test]
    fn detect_adapter_falls_back_to_brew_without_apt() {
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(Output {
            code: 1,
            stdout: "".to_string(),
            stderr: "".to_string(),
        });
        runner.push_response(Output {
            code: 0,
            stdout: "".to_string(),
            stderr: "".to_string(),
        });
        let adapter = detect_adapter(runner.clone(), false).unwrap();
        let commands = runner.commands();
        assert_eq!(adapter.name(), "brew");
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn detect_adapter_errors_without_any_package_manager() {
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(Output {
            code: 1,
            stdout: "".to_string(),
            stderr: "".to_string(),
        });
        runner.push_response(Output {
            code: 1,
            stdout: "".to_string(),
            stderr: "".to_string(),
        });
        let result = detect_adapter(runner.clone(), false);

        assert!(result.is_err());
    }

    #[test]
    fn augmented_path_appends_missing_dirs_after_the_existing_ones() {
        let current = std::env::join_paths(["/usr/bin", "/bin"]).unwrap();

        let path = augmented_path(Some(current), &[PathBuf::from("/home/x/.cargo/bin")]);

        let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
        assert_eq!(
            dirs,
            vec![
                PathBuf::from("/usr/bin"),
                PathBuf::from("/bin"),
                PathBuf::from("/home/x/.cargo/bin"),
            ]
        );
    }

    #[test]
    fn augmented_path_does_not_repeat_a_dir_already_there() {
        let current = std::env::join_paths(["/usr/bin", "/home/x/.cargo/bin"]).unwrap();

        let path = augmented_path(Some(current), &[PathBuf::from("/home/x/.cargo/bin")]);

        assert_eq!(std::env::split_paths(&path).count(), 2);
    }

    #[test]
    fn expand_home_replaces_a_leading_tilde() {
        let home = Path::new("/home/x");

        assert_eq!(
            expand_home("~/.local/bin", home),
            PathBuf::from("/home/x/.local/bin")
        );
        assert_eq!(expand_home("/opt/bin", home), PathBuf::from("/opt/bin"));
    }

    #[test]
    #[cfg(unix)]
    fn real_runner_finds_a_program_in_an_extra_dir() {
        let dir = std::env::temp_dir().join(format!("dev-extra-path-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let program = dir.join("dev-fake-tool");
        std::fs::write(&program, "#!/bin/sh\necho found\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();

        let runner = RealRunner::with_path(vec![dir.clone()]);
        let output = runner
            .capture(&Command {
                program: "dev-fake-tool".to_string(),
                args: vec![],
            })
            .unwrap();

        assert_eq!(output.stdout.trim(), "found");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
