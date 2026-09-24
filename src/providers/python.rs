use super::manifest::{self, Manifest, ManifestError};
use super::{Check, InitOpts, InstallContext, Provider, Report, Status};
use anyhow::Result;

const REQUIRED_BINARIES: &[&str] = &["python3", "uv"];

pub struct PythonProvider {
    manifest: Manifest,
}

impl PythonProvider {
    pub fn new() -> Result<Self, ManifestError> {
        let manifest = manifest::parse(include_str!("python/manifest.toml"))?;
        Ok(Self { manifest })
    }
}

impl Provider for PythonProvider {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn install(&self, _ctx: &InstallContext) -> Result<()> {
        anyhow::bail!("`dev install python` is not implemented yet")
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
        InstallContext {
            config: HashMap::new(),
            os_adapter: adapter,
            command_runner: Rc::new(RecordingRunner::new()) as Rc<dyn CommandRunner>,
        }
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
}
