mod manifest;
pub mod python;
pub mod rust;
use crate::os::{Command, CommandRunner, OsAdapter};
use anyhow::{Context, Result};
pub use manifest::{Manifest, ManifestError};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    rc::Rc,
};
pub struct InstallContext {
    pub config: HashMap<String, String>,
    pub os_adapter: Rc<dyn OsAdapter>,
    pub command_runner: Rc<dyn CommandRunner>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Ok,
    Missing,
    Mismatch { expected: String, found: String },
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
            let mark = match &check.status {
                Status::Ok => "ok".to_string(),
                Status::Missing => "missing".to_string(),
                Status::Mismatch { expected, found } => {
                    format!("{found} (project uses {expected})")
                }
            };
            writeln!(f, "  {:<12} {}", check.name, mark)?;
        }
        Ok(())
    }
}

pub struct InitOpts {
    pub dir: PathBuf,
}

pub trait Provider {
    fn name(&self) -> &str;
    fn manifest(&self) -> &Manifest;
    fn install(&self, ctx: &InstallContext) -> Result<()>;
    fn doctor(&self, ctx: &InstallContext) -> Result<Report>;
    fn init(&self, ctx: &InstallContext, opts: &InitOpts) -> Result<()>;
    fn versions(&self, ctx: &InstallContext) -> Result<BTreeMap<String, String>>;
}

pub(crate) fn manifest_command<'a>(manifest: &'a Manifest, key: &str) -> Result<&'a str> {
    manifest
        .commands
        .get(key)
        .map(|command| command.as_str())
        .with_context(|| format!("manifest is missing the `{key}` command"))
}

pub(crate) fn ensure(
    ctx: &InstallContext,
    manifest: &Manifest,
    binary: &str,
    command_key: &str,
) -> Result<()> {
    if ctx.os_adapter.command_exists(binary)? {
        log::warn!("{binary} is already installed, skipping");
        return Ok(());
    }

    let script = manifest_command(manifest, command_key)?;
    log::info!("installing {binary}");
    ctx.command_runner.execute(&shell(script))?;
    Ok(())
}

pub(crate) fn shell(script: &str) -> Command {
    Command {
        program: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
    }
}

pub(crate) fn status(present: bool) -> Status {
    if present { Status::Ok } else { Status::Missing }
}

pub(crate) fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

pub fn same_minor(a: &str, b: &str) -> bool {
    a.split('.').take(2).eq(b.split('.').take(2))
}

pub fn parse_version(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|word| word.starts_with(|c: char| c.is_ascii_digit()))
        .map(|word| word.to_string())
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

    #[test]
    fn parse_version_takes_the_first_numeric_word() {
        assert_eq!(
            parse_version("uv 0.12.17 (Homebrew 2026-09-18 aarch64-apple-darwin)"),
            Some("0.12.17".to_string())
        );
        assert_eq!(parse_version("Python 3.12.1\n"), Some("3.12.1".to_string()));
        assert_eq!(
            parse_version("rustc 1.83.0 (90b35a623 2024-11-26)"),
            Some("1.83.0".to_string())
        );
    }

    #[test]
    fn parse_version_is_none_without_a_number() {
        assert_eq!(parse_version("command not found"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn same_minor_ignores_the_patch_level() {
        assert!(same_minor("3.12.1", "3.12.9"));
        assert!(!same_minor("3.12.1", "3.13.0"));
        assert!(!same_minor("3.12.1", "4.12.1"));
    }

    #[test]
    fn a_mismatch_is_not_healthy_and_shows_both_versions() {
        let report = Report::new(
            "python",
            vec![Check::new(
                "python",
                Status::Mismatch {
                    expected: "3.12.1".to_string(),
                    found: "3.13.0".to_string(),
                },
            )],
        );

        assert!(!report.is_healthy());
        assert_eq!(
            report.to_string(),
            "python:\n  python       3.13.0 (project uses 3.12.1)\n"
        );
    }
}
