use crate::config::{self, ConfigTable};
use crate::core::Registry;
use crate::errors::AppError;
use crate::os::{CommandRunner, OsAdapter};
use crate::project::{self, Project, ProjectError};
use crate::providers::{InitOpts, InstallContext};
use std::path::Path;
use std::rc::Rc;

pub fn run(
    registry: &Registry,
    config: &ConfigTable,
    os_adapter: Rc<dyn OsAdapter>,
    command_runner: Rc<dyn CommandRunner>,
    language: &str,
    dry_run: bool,
) -> Result<(), AppError> {
    let provider = registry.find(language)?;

    let defaults = provider.manifest().tools.clone();
    let resolved = config::resolve(config, provider.name(), &defaults)?;
    let ctx = InstallContext {
        config: resolved,
        os_adapter,
        command_runner,
    };

    let dir = std::env::current_dir().map_err(ProjectError::Io)?;

    provider.install(&ctx)?;
    provider.init(&ctx, &InitOpts { dir: dir.clone() })?;

    if dry_run {
        log::info!(
            "[dry-run] would record `{}` in {}",
            provider.name(),
            project::FILE_NAME
        );
        return Ok(());
    }

    let versions = provider.versions(&ctx)?;
    let mut state = load_or_new(&dir)?;
    state.record(provider.name(), versions);
    project::save(&dir, &state)?;

    println!("Initialized a {} project", provider.name());
    Ok(())
}

fn load_or_new(dir: &Path) -> Result<Project, ProjectError> {
    match project::load(dir) {
        Ok(project) => Ok(project),
        Err(ProjectError::NotAProject) => Ok(Project::new()),
        Err(error) => Err(error),
    }
}
