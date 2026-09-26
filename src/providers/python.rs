use super::manifest::{self, Manifest, ManifestError};
use super::{Check, InitOpts, InstallContext, Provider, Report, Status};
use crate::os::Command;
use anyhow::{Context, Result};

pub struct PythonProvider {
    manifest: Manifest,
}

impl PythonProvider {
    pub fn new() -> Result<Self, ManifestError> {
        let manifest = manifest::parse(include_str!("python/manifest.toml"))?;
        Ok(Self { manifest })
    }

    fn command(&self, key: &str) -> Result<&str> {
        self.manifest
            .commands
            .get(key)
            .map(|command| command.as_str())
            .with_context(|| format!("manifest is missing the `{key}` command"))
    }

    fn ensure(&self, ctx: &InstallContext, binary: &str, command_key: &str) -> Result<()> {
        if ctx.os_adapter.command_exists(binary)? {
            log::warn!("{binary} is already installed, skipping");
            return Ok(());
        }

        let script = self.command(command_key)?;
        log::info!("installing {binary}");
        ctx.command_runner.execute(&shell(script))?;
        Ok(())
    }

    fn has_managed_python(&self, ctx: &InstallContext) -> Result<bool> {
        let command = Command {
            program: "uv".to_string(),
            args: vec![
                "python".to_string(),
                "list".to_string(),
                "--managed-python".to_string(),
                "--only-installed".to_string(),
            ],
        };

        let output = ctx.command_runner.capture(&command)?;
        Ok(!output.stdout.trim().is_empty())
    }
}

fn status(present: bool) -> Status {
    if present { Status::Ok } else { Status::Missing }
}

fn shell(script: &str) -> Command {
    Command {
        program: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
    }
}

impl Provider for PythonProvider {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn install(&self, ctx: &InstallContext) -> Result<()> {
        self.ensure(ctx, "uv", "install_uv")?;

        if self.has_managed_python(ctx)? {
            log::warn!("a uv-managed Python is already installed, skipping");
            return Ok(());
        }

        log::info!("installing Python");
        ctx.command_runner
            .execute(&shell(self.command("install_python")?))?;
        Ok(())
    }

    fn doctor(&self, ctx: &InstallContext) -> Result<Report> {
        let uv = ctx.os_adapter.command_exists("uv")?;
        let python = uv && self.has_managed_python(ctx)?;

        Ok(Report::new(
            self.name(),
            vec![
                Check::new("uv", status(uv)),
                Check::new("python", status(python)),
            ],
        ))
    }

    fn init(&self, _ctx: &InstallContext, _opts: InitOpts) -> Result<()> {
        anyhow::bail!("`dev init python` is not implemented yet")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os::{CommandRunner, OsAdapter, OsError, Output, RecordingRunner};
    use std::collections::HashMap;
    use std::rc::Rc;

    struct FakeAdapter {
        present: Vec<String>,
    }

    impl FakeAdapter {
        fn with(present: &[&str]) -> Rc<Self> {
            Rc::new(Self {
                present: present.iter().map(|name| name.to_string()).collect(),
            })
        }
    }

    impl OsAdapter for FakeAdapter {
        fn name(&self) -> &str {
            "fake"
        }

        fn install_package(&self, _package: &str) -> Result<(), OsError> {
            Ok(())
        }

        fn command_exists(&self, program: &str) -> Result<bool, OsError> {
            Ok(self.present.iter().any(|name| name == program))
        }
    }

    fn context(adapter: Rc<dyn OsAdapter>) -> InstallContext {
        context_with(adapter, Rc::new(RecordingRunner::new()))
    }

    fn context_with(adapter: Rc<dyn OsAdapter>, runner: Rc<RecordingRunner>) -> InstallContext {
        InstallContext {
            config: HashMap::new(),
            os_adapter: adapter,
            command_runner: runner as Rc<dyn CommandRunner>,
        }
    }

    fn scripts(runner: &RecordingRunner) -> Vec<String> {
        runner
            .commands()
            .iter()
            .filter(|command| command.program == "sh")
            .map(|command| command.args.last().cloned().unwrap_or_default())
            .collect()
    }

    fn installed_python() -> Output {
        Output {
            code: 0,
            stdout: "cpython-3.12.1-macos-aarch64-none    /Users/x/.local/share/uv/python\n"
                .to_string(),
            stderr: String::new(),
        }
    }

    #[test]
    fn loads_its_embedded_manifest() {
        let provider = PythonProvider::new().unwrap();

        assert_eq!(provider.name(), "python");
        assert_eq!(provider.manifest().schema, 1);
    }

    #[test]
    fn doctor_is_healthy_when_uv_manages_a_python() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(installed_python());
        let ctx = context_with(FakeAdapter::with(&["uv"]), runner);

        let report = provider.doctor(&ctx).unwrap();

        assert_eq!(report.component, "python");
        assert!(report.is_healthy());
    }

    #[test]
    fn doctor_reports_a_system_python_as_missing() {
        let provider = PythonProvider::new().unwrap();
        let ctx = context(FakeAdapter::with(&["uv", "python3"]));

        let report = provider.doctor(&ctx).unwrap();

        assert_eq!(
            report.checks,
            vec![
                Check::new("uv", Status::Ok),
                Check::new("python", Status::Missing),
            ]
        );
    }

    #[test]
    fn doctor_does_not_query_uv_when_uv_is_missing() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context_with(FakeAdapter::with(&[]), runner.clone());

        let report = provider.doctor(&ctx).unwrap();

        assert!(!report.is_healthy());
        assert!(runner.commands().is_empty());
    }

    #[test]
    fn install_runs_both_steps_on_a_bare_machine() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context_with(FakeAdapter::with(&[]), runner.clone());

        provider.install(&ctx).unwrap();

        assert_eq!(
            scripts(&runner),
            vec![
                "curl -LsSf https://astral.sh/uv/install.sh | sh",
                "uv python install",
            ]
        );
    }

    #[test]
    fn install_skips_uv_when_it_is_already_there() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context_with(FakeAdapter::with(&["uv"]), runner.clone());

        provider.install(&ctx).unwrap();

        assert_eq!(scripts(&runner), vec!["uv python install"]);
    }

    #[test]
    fn install_is_idempotent_when_uv_already_manages_a_python() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(installed_python());
        let ctx = context_with(FakeAdapter::with(&["uv"]), runner.clone());

        provider.install(&ctx).unwrap();

        assert!(scripts(&runner).is_empty());
    }

    #[test]
    fn a_system_python_does_not_count_as_installed() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context_with(FakeAdapter::with(&["uv", "python3"]), runner.clone());

        provider.install(&ctx).unwrap();

        assert_eq!(scripts(&runner), vec!["uv python install"]);
    }
}
