use super::manifest::{self, Manifest, ManifestError};
use super::{
    Check, InitOpts, InstallContext, Provider, Report, args, ensure, parse_version, status,
};
use crate::os::Command;
use anyhow::Result;
use std::collections::BTreeMap;

pub struct RustProvider {
    manifest: Manifest,
}

impl RustProvider {
    pub fn new() -> Result<Self, ManifestError> {
        let manifest = manifest::parse(include_str!("rust/manifest.toml"))?;
        Ok(Self { manifest })
    }

    fn active_toolchain(&self, ctx: &InstallContext) -> Result<Option<String>> {
        let output = ctx.command_runner.capture(&Command {
            program: "rustup".to_string(),
            args: args(&["show", "active-toolchain"]),
        })?;
        if output.code != 0 {
            return Ok(None);
        }
        Ok(output.stdout.split_whitespace().next().map(str::to_string))
    }

    fn rustup_run(toolchain: &str, words: &[&str]) -> Command {
        let mut all = vec!["run", toolchain];
        all.extend_from_slice(words);
        Command {
            program: "rustup".to_string(),
            args: args(&all),
        }
    }
}

impl Provider for RustProvider {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn install(&self, ctx: &InstallContext) -> Result<()> {
        ensure(ctx, &self.manifest, "rustup", "install_rustup")?;

        if self.active_toolchain(ctx)?.is_some() {
            log::warn!("a Rust toolchain is already active, skipping");
            return Ok(());
        }

        log::info!("installing the stable Rust toolchain");
        ctx.command_runner.execute(&Command {
            program: "rustup".to_string(),
            args: args(&["default", "stable"]),
        })?;
        Ok(())
    }

    fn doctor(&self, ctx: &InstallContext) -> Result<Report> {
        let rustup = ctx.os_adapter.command_exists("rustup")?;
        let rust = rustup && self.active_toolchain(ctx)?.is_some();

        Ok(Report::new(
            self.name(),
            vec![
                Check::new("rustup", status(rustup)),
                Check::new("rust", status(rust)),
            ],
        ))
    }

    fn init(&self, ctx: &InstallContext, opts: &InitOpts) -> Result<()> {
        if opts.dir.join("Cargo.toml").exists() {
            log::warn!("a Rust project already exists here, skipping `cargo init`");
            return Ok(());
        }

        let toolchain = self
            .active_toolchain(ctx)?
            .unwrap_or_else(|| "stable".to_string());
        let dir = opts.dir.display().to_string();
        let mut words = vec!["cargo", "init"];
        if let Some(edition) = ctx.config.get("edition") {
            words.extend(["--edition", edition]);
        }
        words.push(&dir);

        log::info!("initializing a Rust project");
        ctx.command_runner
            .execute(&Self::rustup_run(&toolchain, &words))?;
        Ok(())
    }

    fn versions(&self, ctx: &InstallContext) -> Result<BTreeMap<String, String>> {
        let mut versions = BTreeMap::new();

        let rustup = ctx.command_runner.capture(&Command {
            program: "rustup".to_string(),
            args: args(&["--version"]),
        })?;
        if let Some(version) = parse_version(&rustup.stdout) {
            versions.insert("rustup".to_string(), version);
        }

        if let Some(toolchain) = self.active_toolchain(ctx)? {
            let rustc = ctx
                .command_runner
                .capture(&Self::rustup_run(&toolchain, &["rustc", "--version"]))?;
            if let Some(version) = parse_version(&rustc.stdout) {
                versions.insert("rust".to_string(), version);
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

    fn context(present: &[&str], runner: Rc<RecordingRunner>) -> InstallContext {
        InstallContext {
            config: HashMap::new(),
            os_adapter: Rc::new(FakeAdapter {
                present: present.iter().map(|name| name.to_string()).collect(),
            }),
            command_runner: runner as Rc<dyn CommandRunner>,
        }
    }

    fn output(code: i32, stdout: &str) -> Output {
        Output {
            code,
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    fn no_toolchain() -> Output {
        output(1, "")
    }

    fn stable() -> Output {
        output(0, "stable-aarch64-apple-darwin (default)\n")
    }

    fn command_lines(runner: &RecordingRunner) -> Vec<String> {
        runner
            .commands()
            .iter()
            .map(|command| command.to_string())
            .collect()
    }

    #[test]
    fn loads_its_embedded_manifest() {
        let provider = RustProvider::new().unwrap();

        assert_eq!(provider.name(), "rust");
        assert_eq!(provider.manifest().bin_dirs, vec!["~/.cargo/bin"]);
    }

    #[test]
    fn install_runs_the_rustup_script_on_a_bare_machine() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(stable());
        let ctx = context(&[], runner.clone());

        provider.install(&ctx).unwrap();

        let commands = runner.commands();
        assert_eq!(commands[0].program, "sh");
        assert!(commands[0].args[1].contains("sh.rustup.rs"));
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn install_picks_stable_when_rustup_has_no_toolchain() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(no_toolchain());
        let ctx = context(&["rustup"], runner.clone());

        provider.install(&ctx).unwrap();

        assert_eq!(
            command_lines(&runner).last().unwrap(),
            "rustup default stable"
        );
    }

    #[test]
    fn install_is_idempotent_when_a_toolchain_is_active() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(stable());
        let ctx = context(&["rustup"], runner.clone());

        provider.install(&ctx).unwrap();

        assert_eq!(command_lines(&runner), vec!["rustup show active-toolchain"]);
    }

    #[test]
    fn doctor_reports_a_rustc_outside_rustup_as_missing() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context(&["rustc", "cargo"], runner.clone());

        let report = provider.doctor(&ctx).unwrap();

        assert_eq!(
            report.checks,
            vec![
                Check::new("rustup", Status::Missing),
                Check::new("rust", Status::Missing),
            ]
        );
        assert!(runner.commands().is_empty());
    }

    #[test]
    fn doctor_is_healthy_with_an_active_toolchain() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(stable());
        let ctx = context(&["rustup"], runner);

        assert!(provider.doctor(&ctx).unwrap().is_healthy());
    }

    #[test]
    fn init_runs_cargo_through_the_active_toolchain() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(stable());
        let mut ctx = context(&["rustup"], runner.clone());
        ctx.config = HashMap::from([("edition".to_string(), "2021".to_string())]);
        let dir = std::env::temp_dir().join(format!("dev-rust-init-{}", std::process::id()));

        provider.init(&ctx, &InitOpts { dir: dir.clone() }).unwrap();

        assert_eq!(
            command_lines(&runner).last().unwrap(),
            &format!(
                "rustup run stable-aarch64-apple-darwin cargo init --edition 2021 {}",
                dir.display()
            )
        );
    }

    #[test]
    fn init_skips_an_existing_rust_project() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        let ctx = context(&["rustup"], runner.clone());
        let dir = std::env::temp_dir().join(format!("dev-rust-existing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "").unwrap();

        provider.init(&ctx, &InitOpts { dir: dir.clone() }).unwrap();

        assert!(runner.commands().is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn versions_reports_rustup_and_the_active_rustc() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(output(0, "rustup 1.29.0 (28d1352db 2026-03-05)\n"));
        runner.push_response(stable());
        runner.push_response(output(0, "rustc 1.98.0 (abc 2026-08-01)\n"));
        let ctx = context(&["rustup"], runner);

        let versions = provider.versions(&ctx).unwrap();

        assert_eq!(versions["rustup"], "1.29.0");
        assert_eq!(versions["rust"], "1.98.0");
    }

    #[test]
    fn versions_omits_rust_without_a_toolchain() {
        let provider = RustProvider::new().unwrap();
        let runner = Rc::new(RecordingRunner::new());
        runner.push_response(output(0, "rustup 1.29.0\n"));
        runner.push_response(no_toolchain());
        let ctx = context(&["rustup"], runner);

        let versions = provider.versions(&ctx).unwrap();

        assert!(!versions.contains_key("rust"));
    }
}
