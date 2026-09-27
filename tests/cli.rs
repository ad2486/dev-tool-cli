use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Sandbox {
    dir: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("dev-it-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("project")).unwrap();
        Self { dir }
    }

    fn project(&self) -> PathBuf {
        self.dir.join("project")
    }

    fn config(&self) -> PathBuf {
        self.dir.join("config.toml")
    }

    fn dev(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dev"))
            .arg("--config")
            .arg(self.config())
            .args(args)
            .current_dir(self.project())
            .output()
            .unwrap()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn entries(dir: &Path) -> usize {
    std::fs::read_dir(dir).unwrap().count()
}

#[test]
fn doctor_outside_a_project_suggests_init() {
    let sandbox = Sandbox::new("doctor-no-project");

    let output = sandbox.dev(&["doctor"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("dev init"));
}

#[test]
fn doctor_rejects_an_unknown_component() {
    let sandbox = Sandbox::new("doctor-unknown");
    std::fs::write(
        sandbox.project().join("dev.toml"),
        "schema = 1\ncomponents = [\"cobol\"]\n\n[dependencies]\n",
    )
    .unwrap();

    let output = sandbox.dev(&["doctor"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("cobol"));
}

#[test]
fn install_rejects_an_unknown_language() {
    let sandbox = Sandbox::new("install-unknown");

    let output = sandbox.dev(&["install", "cobol"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("python, rust"));
}

#[test]
fn config_get_lists_the_manifest_defaults() {
    let sandbox = Sandbox::new("config-defaults");

    let output = sandbox.dev(&["config", "get"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("python.formatter = ruff (default)"));
    assert!(stdout(&output).contains("rust.edition = 2024 (default)"));
}

#[test]
fn config_set_is_read_back_and_keeps_comments() {
    let sandbox = Sandbox::new("config-set");
    std::fs::write(sandbox.config(), "# mine\n[rust]\nedition = \"2021\"\n").unwrap();

    let set = sandbox.dev(&["config", "set", "python.formatter", "black"]);
    let get = sandbox.dev(&["config", "get", "python.formatter"]);

    assert_eq!(set.status.code(), Some(0));
    assert_eq!(stdout(&get), "python.formatter = black\n");
    assert_eq!(
        std::fs::read_to_string(sandbox.config()).unwrap(),
        "# mine\n[rust]\nedition = \"2021\"\n\n[python]\nformatter = \"black\"\n"
    );
}

#[test]
fn config_set_rejects_an_unknown_key_without_writing() {
    let sandbox = Sandbox::new("config-unknown");

    let output = sandbox.dev(&["config", "set", "python.formattr", "black"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(!sandbox.config().exists());
}

#[test]
fn an_invalid_config_is_a_user_error() {
    let sandbox = Sandbox::new("config-invalid");
    std::fs::write(sandbox.config(), "[python\n").unwrap();

    let output = sandbox.dev(&["config", "get"]);

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn init_dry_run_leaves_the_directory_untouched() {
    let sandbox = Sandbox::new("init-dry-run");

    for language in ["python", "rust"] {
        let output = sandbox.dev(&["init", language, "--dry-run"]);

        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    }
    assert_eq!(entries(&sandbox.project()), 0);
}
