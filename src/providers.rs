mod manifest;
pub mod python;
use crate::os::{CommandRunner, OsAdapter};
use anyhow::Result;
pub use manifest::Manifest;
use std::{collections::HashMap, rc::Rc};
pub struct InstallContext {
    pub config: HashMap<String, String>,
    pub os_adapter: Rc<dyn OsAdapter>,
    pub command_runner: Rc<dyn CommandRunner>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Ok,
    Missing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub name: String,
    pub status: Status,
}

impl Check {
    pub fn new(name: impl Into<String>, status: Status) -> Self {
        Self {
            name: name.into(),
            status,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub component: String,
    pub checks: Vec<Check>,
}

impl Report {
    pub fn new(component: impl Into<String>, checks: Vec<Check>) -> Self {
        Self {
            component: component.into(),
            checks,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.checks.iter().all(|check| check.status == Status::Ok)
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}:", self.component)?;
        for check in &self.checks {
            let mark = match check.status {
                Status::Ok => "ok",
                Status::Missing => "missing",
            };
            writeln!(f, "  {:<12} {}", check.name, mark)?;
        }
        Ok(())
    }
}

pub struct InitOpts;
pub trait Provider {
    fn name(&self) -> &str;
    fn manifest(&self) -> &Manifest;
    fn install(&self, ctx: &InstallContext) -> Result<()>;
    fn doctor(&self, ctx: &InstallContext) -> Result<Report>;
    fn init(&self, ctx: &InstallContext, opts: InitOpts) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_is_healthy_when_every_check_passed() {
        let report = Report::new(
            "python",
            vec![
                Check::new("python3", Status::Ok),
                Check::new("uv", Status::Ok),
            ],
        );

        assert!(report.is_healthy());
    }

    #[test]
    fn report_is_not_healthy_when_a_check_is_missing() {
        let report = Report::new(
            "python",
            vec![
                Check::new("python3", Status::Ok),
                Check::new("uv", Status::Missing),
            ],
        );

        assert!(!report.is_healthy());
    }

    #[test]
    fn report_without_checks_is_healthy() {
        let report = Report::new("python", vec![]);

        assert!(report.is_healthy());
    }

    #[test]
    fn report_displays_one_line_per_check() {
        let report = Report::new(
            "python",
            vec![
                Check::new("python3", Status::Ok),
                Check::new("uv", Status::Missing),
            ],
        );

        assert_eq!(
            report.to_string(),
            "python:\n  python3      ok\n  uv           missing\n"
        );
    }
}
