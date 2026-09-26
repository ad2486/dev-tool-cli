use super::manifest::{self, Manifest, ManifestError};
use super::{
    Check, InitOpts, InstallContext, Provider, Report, ensure, manifest_command, parse_version,
    shell, status,
};
use crate::os::Command;
use anyhow::Result;
use std::collections::BTreeMap;

pub struct PythonProvider {
    manifest: Manifest,
}

impl PythonProvider {
    pub fn new() -> Result<Self, ManifestError> {
        let manifest = manifest::parse(include_str!("python/manifest.toml"))?;
        Ok(Self { manifest })
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

impl Provider for PythonProvider {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn install(&self, ctx: &InstallContext) -> Result<()> {
        ensure(ctx, &self.manifest, "uv", "install_uv")?;

        if self.has_managed_python(ctx)? {
            log::warn!("a uv-managed Python is already installed, skipping");
            return Ok(());
        }

        log::info!("installing Python");
        ctx.command_runner
            .execute(&shell(manifest_command(&self.manifest, "install_python")?))?;
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

    fn init(&self, ctx: &InstallContext, opts: &InitOpts) -> Result<()> {
        if opts.dir.join("pyproject.toml").exists() {
            log::warn!("a Python project already exists here, skipping `uv init`");
            return Ok(());
        }

        log::info!("initializing a Python project");
        ctx.command_runner.execute(&Command {
            program: "uv".to_string(),
            args: vec!["init".to_string(), opts.dir.display().to_string()],
        })?;
        Ok(())
    }

    fn versions(&self, ctx: &InstallContext) -> Result<BTreeMap<String, String>> {
        let mut versions = BTreeMap::new();

        let uv = ctx.command_runner.capture(&Command {
            program: "uv".to_string(),
            args: vec!["--version".to_string()],
        })?;
        if let Some(version) = parse_version(&uv.stdout) {
            versions.insert("uv".to_string(), version);
        }

        let found = ctx.command_runner.capture(&Command {
            program: "uv".to_string(),
            args: vec![
                "python".to_string(),
                "find".to_string(),
                "--managed-python".to_string(),
            ],
        })?;
        let interpreter = found.stdout.trim();
        if found.code == 0 && !interpreter.is_empty() {
            let python = ctx.command_runner.capture(&Command {
                program: interpreter.to_string(),
                args: vec!["--version".to_string()],
            })?;
            if let Some(version) = parse_version(&python.stdout) {
                versions.insert("python".to_string(), version);
            }
        }

        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os::{CommandRunner, OsAdapter, OsError, Output, RecordingRunner};
    use crate::providers::Status;
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

    fn output(code: i32, stdout: &str) -> Output {
        Output {
            code,
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("dev-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn init_runs_uv_init_in_the_target_directory() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context_with(FakeAdapter::with(&["uv"]), runner.clone());
        let dir = temp_dir("init-fresh");

        provider.init(&ctx, &InitOpts { dir: dir.clone() }).unwrap();

        let commands = runner.commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "uv");
        assert_eq!(
            commands[0].args,
            vec!["init".to_string(), dir.display().to_string()]
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn init_skips_an_existing_python_project() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context_with(FakeAdapter::with(&["uv"]), runner.clone());
        let dir = temp_dir("init-existing");
        std::fs::write(dir.join("pyproject.toml"), "").unwrap();

        provider.init(&ctx, &InitOpts { dir: dir.clone() }).unwrap();

        assert!(runner.commands().is_empty());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn versions_reports_uv_and_the_managed_python() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(output(
            0,
            "uv 0.12.17 (Homebrew 2026-09-18 aarch64-apple-darwin)\n",
        ));
        runner.push_response(output(
            0,
            "/Users/x/.local/share/uv/python/cpython-3.12.1/bin/python3\n",
        ));
        runner.push_response(output(0, "Python 3.12.1\n"));
        let ctx = context_with(FakeAdapter::with(&["uv"]), runner);

        let versions = provider.versions(&ctx).unwrap();

        assert_eq!(versions["uv"], "0.12.17");
        assert_eq!(versions["python"], "3.12.1");
    }

    #[test]
    fn versions_omits_python_when_uv_manages_none() {
        let provider = PythonProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(output(0, "uv 0.12.17\n"));
        runner.push_response(output(2, ""));
        let ctx = context_with(FakeAdapter::with(&["uv"]), runner);

        let versions = provider.versions(&ctx).unwrap();

        assert_eq!(versions.len(), 1);
        assert!(!versions.contains_key("python"));
    }
}
