use super::manifest::{self, Manifest, ManifestError};
use super::{Check, InitOpts, InstallContext, Provider, Report, Status};
use crate::os::Command;
use anyhow::{Context, Result};

const REQUIRED_BINARIES: &[&str] = &["python3", "uv"];

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
        self.ensure(ctx, "python3", "install_python")?;
        Ok(())
    }

    fn doctor(&self, ctx: &InstallContext) -> Result<Report> {
        let mut checks = Vec::new();

        for binary in REQUIRED_BINARIES {
            let status = if ctx.os_adapter.command_exists(binary)? {
                Status::Ok
            } else {
                Status::Missing
            };
            checks.push(Check::new(*binary, status));
        }

        Ok(Report::new(self.name(), checks))
    }

    fn init(&self, _ctx: &InstallContext, _opts: InitOpts) -> Result<()> {
        anyhow::bail!("`dev init python` is not implemented yet")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os::{CommandRunner, OsAdapter, OsError, RecordingRunner};
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
            .map(|command| command.args.last().cloned().unwrap_or_default())
            .collect()
    }

    #[test]
    fn loads_its_embedded_manifest() {
        let provider = PythonProvider::new().unwrap();

        assert_eq!(provider.name(), "python");
        assert_eq!(provider.manifest().schema, 1);
    }

    #[test]
    fn doctor_reports_every_required_binary_as_ok_when_present() {
        let provider = PythonProvider::new().unwrap();
        let ctx = context(FakeAdapter::with(&["python3", "uv"]));

        let report = provider.doctor(&ctx).unwrap();

        assert_eq!(report.component, "python");
        assert!(report.is_healthy());
        assert_eq!(report.checks.len(), REQUIRED_BINARIES.len());
    }

    #[test]
    fn doctor_marks_an_absent_binary_as_missing() {
        let provider = PythonProvider::new().unwrap();
        let ctx = context(FakeAdapter::with(&["python3"]));

        let report = provider.doctor(&ctx).unwrap();

        assert!(!report.is_healthy());
        assert_eq!(
            report.checks,
            vec![
                Check::new("python3", Status::Ok),
                Check::new("uv", Status::Missing),
            ]
        );
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
    fn install_skips_what_is_already_there() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context_with(FakeAdapter::with(&["uv"]), runner.clone());

        provider.install(&ctx).unwrap();

        assert_eq!(scripts(&runner), vec!["uv python install"]);
    }

    #[test]
    fn install_is_idempotent_when_everything_is_installed() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context_with(FakeAdapter::with(&["uv", "python3"]), runner.clone());

        provider.install(&ctx).unwrap();

        assert!(runner.commands().is_empty());
    }
}
